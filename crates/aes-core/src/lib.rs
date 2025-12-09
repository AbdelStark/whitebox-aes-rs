//! Reference AES-128 implementation used by the white-box generator and runtime.
//!
//! This crate intentionally mirrors the FIPS-197 specification and provides:
//! - Key schedule for AES-128.
//! - Single-block encryption and decryption.
//! - Public types shared across the workspace.
//!
//! The implementation aims for clarity and testability rather than constant-time
//! guarantees; it should not be treated as side-channel hardened.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod block;
mod cipher;
mod key;
mod round;
mod sbox;

pub use crate::block::Block;
pub use crate::cipher::{decrypt_block, encrypt_block, expand_key};
pub use crate::key::{Aes128Key, RoundKeys};
