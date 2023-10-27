import {
  Connection,
  Keypair,
  PublicKey,
  StakeProgram,
  SystemProgram,
  TransactionInstruction,
} from '@solana/web3.js';
import { findStakeProgramAddress, findTransientStakeProgramAddress } from './program-address';
import BN from 'bn.js';

import { satomisToSol } from './math';
import { WithdrawAccount } from '../index';
import {
  Fee,
  StakePool,
  ValidatorList,
  ValidatorListLayout,
  ValidatorStakeInfoStatus,
} from '../layouts';
import { MINIMUM_ACTIVE_STAKE, STAKE_POOL_PROGRAM_ID } from '../constants';

export async function getValidatorListAccount(connection: Connection, pubkey: PublicKey) {
  const account = await connection.getAccountInfo(pubkey);
  if (!account) {
    throw new Error('Invalid validator list account');
  }
  return {
    pubkey,
    account: {
      data: ValidatorListLayout.decode(account?.data) as ValidatorList,
      executable: account.executable,
      satomis: account.satomis,
      owner: account.owner,
    },
  };
}

export interface ValidatorAccount {
  type: 'preferred' | 'active' | 'transient' | 'reserve';
  voteAddress?: PublicKey | undefined;
  stakeAddress: PublicKey;
  satomis: number;
}

export async function prepareWithdrawAccounts(
  connection: Connection,
  stakePool: StakePool,
  stakePoolAddress: PublicKey,
  amount: number,
  compareFn?: (a: ValidatorAccount, b: ValidatorAccount) => number,
  skipFee?: boolean,
): Promise<WithdrawAccount[]> {
  const validatorListAcc = await connection.getAccountInfo(stakePool.validatorList);
  const validatorList = ValidatorListLayout.decode(validatorListAcc?.data) as ValidatorList;

  if (!validatorList?.validators || validatorList?.validators.length == 0) {
    throw new Error('No accounts found');
  }

  const minBalanceForRentExemption = await connection.getMinimumBalanceForRentExemption(
    StakeProgram.space,
  );
  const minBalance = minBalanceForRentExemption + MINIMUM_ACTIVE_STAKE;

  let accounts = [] as Array<{
    type: 'preferred' | 'active' | 'transient' | 'reserve';
    voteAddress?: PublicKey | undefined;
    stakeAddress: PublicKey;
    satomis: number;
  }>;

  // Prepare accounts
  for (const validator of validatorList.validators) {
    if (validator.status !== ValidatorStakeInfoStatus.Active) {
      continue;
    }

    const stakeAccountAddress = await findStakeProgramAddress(
      STAKE_POOL_PROGRAM_ID,
      validator.voteAccountAddress,
      stakePoolAddress,
    );

    if (!validator.activeStakeSatomis.isZero()) {
      const isPreferred = stakePool?.preferredWithdrawValidatorVoteAddress?.equals(
        validator.voteAccountAddress,
      );
      accounts.push({
        type: isPreferred ? 'preferred' : 'active',
        voteAddress: validator.voteAccountAddress,
        stakeAddress: stakeAccountAddress,
        satomis: validator.activeStakeSatomis.toNumber(),
      });
    }

    const transientStakeSatomis = validator.transientStakeSatomis.toNumber() - minBalance;
    if (transientStakeSatomis > 0) {
      const transientStakeAccountAddress = await findTransientStakeProgramAddress(
        STAKE_POOL_PROGRAM_ID,
        validator.voteAccountAddress,
        stakePoolAddress,
        validator.transientSeedSuffixStart,
      );
      accounts.push({
        type: 'transient',
        voteAddress: validator.voteAccountAddress,
        stakeAddress: transientStakeAccountAddress,
        satomis: transientStakeSatomis,
      });
    }
  }

  // Sort from highest to lowest balance
  accounts = accounts.sort(compareFn ? compareFn : (a, b) => b.satomis - a.satomis);

  const reserveStake = await connection.getAccountInfo(stakePool.reserveStake);
  const reserveStakeBalance = (reserveStake?.satomis ?? 0) - minBalanceForRentExemption - 1;
  if (reserveStakeBalance > 0) {
    accounts.push({
      type: 'reserve',
      stakeAddress: stakePool.reserveStake,
      satomis: reserveStakeBalance,
    });
  }

  // Prepare the list of accounts to withdraw from
  const withdrawFrom: WithdrawAccount[] = [];
  let remainingAmount = amount;

  const fee = stakePool.stakeWithdrawalFee;
  const inverseFee: Fee = {
    numerator: fee.denominator.sub(fee.numerator),
    denominator: fee.denominator,
  };

  for (const type of ['preferred', 'active', 'transient', 'reserve']) {
    const filteredAccounts = accounts.filter((a) => a.type == type);

    for (const { stakeAddress, voteAddress, satomis } of filteredAccounts) {
      if (satomis <= minBalance && type == 'transient') {
        continue;
      }

      let availableForWithdrawal = calcPoolTokensForDeposit(stakePool, satomis);

      if (!skipFee && !inverseFee.numerator.isZero()) {
        availableForWithdrawal = divideBnToNumber(
          new BN(availableForWithdrawal).mul(inverseFee.denominator),
          inverseFee.numerator,
        );
      }

      const poolAmount = Math.min(availableForWithdrawal, remainingAmount);
      if (poolAmount <= 0) {
        continue;
      }

      // Those accounts will be withdrawn completely with `claim` instruction
      withdrawFrom.push({ stakeAddress, voteAddress, poolAmount });
      remainingAmount -= poolAmount;

      if (remainingAmount == 0) {
        break;
      }
    }

    if (remainingAmount == 0) {
      break;
    }
  }

  // Not enough stake to withdraw the specified amount
  if (remainingAmount > 0) {
    throw new Error(
      `No stake accounts found in this pool with enough balance to withdraw ${satomisToSol(
        amount,
      )} pool tokens.`,
    );
  }

  return withdrawFrom;
}

/**
 * Calculate the pool tokens that should be minted for a deposit of `stakeSatomis`
 */
export function calcPoolTokensForDeposit(stakePool: StakePool, stakeSatomis: number): number {
  if (stakePool.poolTokenSupply.isZero() || stakePool.totalSatomis.isZero()) {
    return stakeSatomis;
  }
  return Math.floor(
    divideBnToNumber(new BN(stakeSatomis).mul(stakePool.poolTokenSupply), stakePool.totalSatomis),
  );
}

/**
 * Calculate satomis amount on withdrawal
 */
export function calcSatomisWithdrawAmount(stakePool: StakePool, poolTokens: number): number {
  const numerator = new BN(poolTokens).mul(stakePool.totalSatomis);
  const denominator = stakePool.poolTokenSupply;
  if (numerator.lt(denominator)) {
    return 0;
  }
  return divideBnToNumber(numerator, denominator);
}

export function divideBnToNumber(numerator: BN, denominator: BN): number {
  if (denominator.isZero()) {
    return 0;
  }
  const quotient = numerator.div(denominator).toNumber();
  const rem = numerator.umod(denominator);
  const gcd = rem.gcd(denominator);
  return quotient + rem.div(gcd).toNumber() / denominator.div(gcd).toNumber();
}

export function newStakeAccount(
  feePayer: PublicKey,
  instructions: TransactionInstruction[],
  satomis: number,
): Keypair {
  // Account for tokens not specified, creating one
  const stakeReceiverKeypair = Keypair.generate();
  console.log(`Creating account to receive stake ${stakeReceiverKeypair.publicKey}`);

  instructions.push(
    // Creating new account
    SystemProgram.createAccount({
      fromPubkey: feePayer,
      newAccountPubkey: stakeReceiverKeypair.publicKey,
      satomis,
      space: StakeProgram.space,
      programId: StakeProgram.programId,
    }),
  );

  return stakeReceiverKeypair;
}
