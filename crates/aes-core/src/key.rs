//! Key types for AES-128.

use crate::block::Block;

/// AES-128 key wrapper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aes128Key(pub [u8; 16]);

impl From<[u8; 16]> for Aes128Key {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

/// Expanded round keys for AES-128.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundKeys(pub [Block; 11]);

impl RoundKeys {
    /// Returns the round key at the requested index (0..=10).
    #[inline]
    pub fn get(&self, round: usize) -> &Block {
        &self.0[round]
    }
}
