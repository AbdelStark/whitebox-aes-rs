//! Instance representation and serialization helpers.

use serde::{Deserialize, Serialize};

use crate::affine::Affine256;
use crate::tables::RoundTables;

/// Scheme identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemeId {
    /// Baek–Cheon–Hong revisited white-box AES (JCN 2016).
    BaekCheonHong2016,
}

/// Static parameters describing the instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceParams {
    /// Number of rounds (10 for AES-128).
    pub rounds: usize,
    /// Block size in bytes (32 for two AES blocks).
    pub block_bytes: usize,
    /// Input bits per table (16 for `u8 × u8`).
    pub table_input_bits: u32,
    /// Output bits per table entry (256).
    pub table_output_bits: u32,
    /// Encoding size `m_A` (256 bits for unsplit sparse encodings).
    pub ma_bits: u32,
    /// Scheme identifier.
    pub scheme: SchemeId,
    /// Version tag for future compatibility changes.
    pub version: u32,
}

impl Default for InstanceParams {
    fn default() -> Self {
        Self {
            rounds: 10,
            block_bytes: 32,
            table_input_bits: 16,
            table_output_bits: 256,
            ma_bits: 256,
            scheme: SchemeId::BaekCheonHong2016,
            version: 1,
        }
    }
}

/// External encodings applied before and after the table network.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalEncodings {
    /// Input encoding `F^(0)`.
    pub input: Affine256,
    /// Optional output encoding to apply after the final round (if not already folded).
    pub output: Option<Affine256>,
}

/// Complete white-box AES-256-bit instance (two AES-128 blocks).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WbInstance256 {
    /// Round tables for 10 rounds.
    pub rounds: [RoundTables; 10],
    /// External encodings.
    pub encodings: ExternalEncodings,
    /// Static parameters.
    pub params: InstanceParams,
}

impl WbInstance256 {
    /// Serializes the instance with `bincode`.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes an instance with `bincode`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tables::RoundTables;

    #[test]
    fn serialize_roundtrip() {
        let instance = WbInstance256 {
            rounds: std::array::from_fn(|_| RoundTables::new_zeroed()),
            encodings: ExternalEncodings {
                input: Affine256::identity(),
                output: None,
            },
            params: InstanceParams::default(),
        };
        let bytes = instance.to_bytes().expect("serialize");
        let decoded = WbInstance256::from_bytes(&bytes).expect("deserialize");
        assert_eq!(decoded.params.rounds, 10);
        assert_eq!(decoded.encodings.output, None);
        assert_eq!(decoded.rounds[0].tables[0].get(0, 0), [0u8; 32]);
    }
}
