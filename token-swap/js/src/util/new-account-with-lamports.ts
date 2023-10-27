import {Account, Connection} from '@solana/web3.js';

import {sleep} from './sleep';

export async function newAccountWithSatomis(
  connection: Connection,
  satomis: number = 1000000,
): Promise<Account> {
  const account = new Account();

  let retries = 30;
  await connection.requestAirdrop(account.publicKey, satomis);
  for (;;) {
    await sleep(500);
    if (satomis == (await connection.getBalance(account.publicKey))) {
      return account;
    }
    if (--retries <= 0) {
      break;
    }
  }
  throw new Error(`Airdrop of ${satomis} failed`);
}
