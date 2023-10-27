import {Account, Connection} from '@solana/web3.js';

/**
 * Create a new system account and airdrop it some satomis
 *
 * @private
 */
export async function newSystemAccountWithAirdrop(
  connection: Connection,
  satomis: number = 1,
): Promise<Account> {
  const account = new Account();
  await connection.requestAirdrop(account.publicKey, satomis);
  return account;
}
