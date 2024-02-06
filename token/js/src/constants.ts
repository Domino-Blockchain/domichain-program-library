import { PublicKey } from '@solana/web3.js';

/** Address of the SPL Token program */
export const TOKEN_PROGRAM_ID = new PublicKey('E6MGqpUar31wjvconpQxzdBts7Z3pvBookQgAgeH4VtS');

/** Address of the SPL Token 2022 program */
export const TOKEN_2022_PROGRAM_ID = new PublicKey('FCgt5eHwR6GvSDTTKcFjpZdYB5rLMumaTK3Z5jk89egj');

/** Address of the SPL Associated Token Account program */
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey('FCG3wAYg9gNLEDmJauNopceZo9tUx1FqH7Ysjm7jheLm');

/** Address of the special mint for wrapped native SOL in spl-token */
export const NATIVE_MINT = new PublicKey('So11111111111111111111111111111111111111112');

/** Address of the special mint for wrapped native SOL in spl-token-2022 */
export const NATIVE_MINT_2022 = new PublicKey('9pan9bMn5HatX4EJdBwg9VgCa7Uz5HL8N1m5D3NdXejP');

/** Check that the token program provided is not `Tokenkeg...`, useful when using extensions */
export function programSupportsExtensions(programId: PublicKey): boolean {
    if (programId === TOKEN_PROGRAM_ID) {
        return false;
    } else {
        return true;
    }
}
