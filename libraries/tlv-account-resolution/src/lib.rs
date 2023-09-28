//! Crate defining a state interface for offchain account resolution. If a
//! program writes the proper state information into one of their accounts, any
//! offchain and onchain client can fetch any additional required accounts for
//! an instruction.

#![allow(clippy::integer_arithmetic)]
#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod error;
pub mod pod;
pub mod seeds;
pub mod state;

// Export current sdk types for downstream users building with a different sdk
// version
pub use domichain_program;
