import {Account, Connection, Keypair, SystemProgram, Transaction} from '@solana/web3.js';

import {sleep} from './sleep';

let i = 1;

export async function newAccountWithSatomis(
  connection: Connection,
  satomis: number = 1000000,
): Promise<Account> {
  // 1thX6LZfHDZZGkq4tt1q2yRAPVfCTpX99XN4RHFssv
  // 1thX6LZfHDZZGkq4tt1q2yRAPVfCTpX99XN4RHFssv

  // const account = new Account();

  // EZz2d58fQjd6cstzo5XHHTHRC5pjqWD3KTLccw53RsEG
  // EeyAYrbZXnjqB49Ze3aNjsNcXFG34h2M8Fo7xpeqSixi
  // 2UJ5yWPgmcHqqhiwUhj1S54wMWWSCoCQd5Fv1XcBUd8Z
  // 5hsGsnSzsS6V1mGmg9HmEftVwJxPC9BCAt3BKT83nYJY
  // C3WzX4zD7Juqh2FXfZzkNsoHR8RoiR6pbPJzZkoZSo46
  // 3ShqsZWEi5Rz87Anirxb4BkqK78U4NMrq9S9qipN1HS1
  let kp = Keypair.fromSeed(new Uint8Array(
    [0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,i]
  ));
  const account = new Account(kp.secretKey);
  i += 1;
  console.log("Using pre-generated key:", account.publicKey.toString());
  const balance = await connection.getBalance(account.publicKey);
  console.log("With satomi balance:", balance);

  return account;

  // let retries = 30;
  // await connection.requestAirdrop(account.publicKey, satomis);
  // for (;;) {
  //   await sleep(500);
  //   if (satomis == (await connection.getBalance(account.publicKey))) {
  //     return account;
  //   }
  //   if (--retries <= 0) {
  //     break;
  //   }
  // }
  // throw new Error(`Airdrop of ${satomis} failed`);
}
