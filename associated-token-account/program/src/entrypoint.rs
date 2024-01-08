//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use domichain_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    domichain_program::msg!("[{file}:{line}] Starting associated-token-account", file=file!(), line=line!());
    crate::processor::process_instruction(program_id, accounts, instruction_data)
}
