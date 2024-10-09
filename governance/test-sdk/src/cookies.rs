use domichain_program::pubkey::Pubkey;
use domichain_sdk::account::Account;

#[derive(Debug)]
pub struct TokenAccountCookie {
    pub address: Pubkey,
}

#[derive(Debug)]
pub struct WalletCookie {
    pub address: Pubkey,
    pub account: Account,
}
