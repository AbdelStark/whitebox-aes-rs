//! Table representations for white-box AES rounds.

use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

const ENTRY_BYTES: usize = 32;
const ENTRIES: usize = 1 << 16;

/// A 16→256-bit table `(x, y) ∈ u8 × u8 → 256-bit value`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table16x256 {
    data: Box<[u8]>,
}

impl Table16x256 {
    /// Allocates a zeroed table.
    pub fn new_zeroed() -> Self {
        Self {
            data: vec![0u8; ENTRIES * ENTRY_BYTES].into_boxed_slice(),
        }
    }

    /// Writes the entry for `(x, y)`.
    pub fn set(&mut self, x: u8, y: u8, value: &[u8; 32]) {
        let idx = entry_index(x, y);
        let start = idx * ENTRY_BYTES;
        self.data[start..start + ENTRY_BYTES].copy_from_slice(value);
    }

    /// Reads the entry for `(x, y)`.
    pub fn get(&self, x: u8, y: u8) -> [u8; 32] {
        let idx = entry_index(x, y);
        let start = idx * ENTRY_BYTES;
        let mut out = [0u8; 32];
        out.copy_from_slice(&self.data[start..start + ENTRY_BYTES]);
        out
    }
}

/// Collection of 32 tables for one round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoundTables {
    /// The 32 tables for the round, indexed by byte position.
    pub tables: [Table16x256; 32],
}

impl RoundTables {
    /// Allocates zeroed tables for the round.
    pub fn new_zeroed() -> Self {
        Self {
            tables: std::array::from_fn(|_| Table16x256::new_zeroed()),
        }
    }
}

impl Default for RoundTables {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

/// Random mask table `h_i: u8 -> 256-bit`.
#[derive(Clone, Debug)]
pub struct HTable {
    data: [[u8; 32]; 256],
}

impl HTable {
    /// Generates a random mask table.
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut data = [[0u8; 32]; 256];
        for entry in data.iter_mut() {
            rng.fill_bytes(entry);
        }
        Self { data }
    }

    /// Returns the mask for input `x`.
    #[inline]
    pub fn get(&self, x: u8) -> &[u8; 32] {
        &self.data[x as usize]
    }
}

#[inline]
const fn entry_index(x: u8, y: u8) -> usize {
    ((x as usize) << 8) | y as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_roundtrip() {
        let mut table = Table16x256::new_zeroed();
        let mut value = [0u8; 32];
        value[0] = 0xaa;
        value[31] = 0x55;
        table.set(1, 2, &value);
        assert_eq!(table.get(1, 2), value);
        assert_eq!(table.get(0, 0), [0u8; 32]);
    }

    #[test]
    fn roundtables_initially_zero() {
        let round = RoundTables::new_zeroed();
        assert_eq!(round.tables[0].get(0, 0), [0u8; 32]);
        assert_eq!(round.tables[31].get(255, 255), [0u8; 32]);
    }
}
