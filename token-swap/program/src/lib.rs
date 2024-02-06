#![allow(clippy::integer_arithmetic)]
#![deny(missing_docs)]

//! An Uniswap-like program for the Solana blockchain.

pub mod constraints;
pub mod curve;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use domichain_program;

domichain_program::declare_id!("4ZH2DZxr5irLBnfEYtZBVKLWeWnu1uAv9NJkWJyhJvHv");
