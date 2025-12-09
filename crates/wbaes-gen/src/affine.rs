//! Affine encodings over GF(2).

use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::matrix::{Matrix256, Matrix8};

/// 8-bit affine map `x -> lin * x ⊕ bias`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affine8 {
    /// Linear component.
    pub lin: Matrix8,
    /// Additive bias.
    pub bias: u8,
}

impl Affine8 {
    /// Constructs a new affine map from components.
    pub const fn new(lin: Matrix8, bias: u8) -> Self {
        Self { lin, bias }
    }

    /// Identity affine map.
    pub fn identity() -> Self {
        Self::new(Matrix8::identity(), 0)
    }

    /// Generates a random invertible affine map.
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let lin = Matrix8::random_invertible(rng);
        let bias = rng.next_u32() as u8;
        Self::new(lin, bias)
    }

    /// Applies the affine map.
    pub fn apply(&self, value: u8) -> u8 {
        self.lin.apply(value) ^ self.bias
    }

    /// Applies the inverse map, if the linear part is invertible.
    pub fn invert(&self) -> Option<Self> {
        let lin_inv = self.lin.invert()?;
        let bias = lin_inv.apply(self.bias);
        Some(Self::new(lin_inv, bias))
    }

    /// Composes `self` after `other` (i.e., `self ∘ other`).
    pub fn compose(&self, other: &Self) -> Self {
        let lin = self.lin.mul(&other.lin);
        let bias = self.lin.apply(other.bias) ^ self.bias;
        Self::new(lin, bias)
    }
}

/// 256-bit affine map `x -> lin * x ⊕ bias`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affine256 {
    /// Linear component.
    pub lin: Matrix256,
    /// Additive bias.
    pub bias: [u8; 32],
}

impl Affine256 {
    /// Constructs a new affine map from components.
    pub const fn new(lin: Matrix256, bias: [u8; 32]) -> Self {
        Self { lin, bias }
    }

    /// Identity affine map.
    pub fn identity() -> Self {
        Self::new(Matrix256::identity(), [0u8; 32])
    }

    /// Generates a random affine map using a sparse unsplit invertible linear part.
    pub fn random_sparse_unsplit<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let lin = Matrix256::random_sparse_unsplit(rng);
        let mut bias = [0u8; 32];
        rng.fill_bytes(&mut bias);
        Self::new(lin, bias)
    }

    /// Applies the affine map.
    pub fn apply(&self, value: &[u8; 32]) -> [u8; 32] {
        let mut out = self.lin.apply_to_bytes(value);
        xor_in_place(&mut out, &self.bias);
        out
    }

    /// Applies the affine map in place.
    pub fn apply_in_place(&self, value: &mut [u8; 32]) {
        *value = self.apply(value);
    }

    /// Applies the inverse map, if the linear part is invertible.
    pub fn invert(&self) -> Option<Self> {
        let lin_inv = self.lin.invert()?;
        let bias = lin_inv.apply_to_bytes(&self.bias);
        Some(Self::new(lin_inv, bias))
    }

    /// Composes `self` after `other` (i.e., `self ∘ other`).
    pub fn compose(&self, other: &Self) -> Self {
        let lin = self.lin.mul(&other.lin);
        let bias_from_other = self.lin.apply_to_bytes(&other.bias);
        let mut bias = self.bias;
        xor_in_place(&mut bias, &bias_from_other);
        Self::new(lin, bias)
    }
}

fn xor_in_place(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d ^= *s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn affine8_roundtrip() {
        let mut rng = ChaCha20Rng::from_seed([10u8; 32]);
        for _ in 0..32 {
            let aff = Affine8::random(&mut rng);
            let inv = aff.invert().expect("invertible");
            let value = rng.next_u32() as u8;
            let enc = aff.apply(value);
            let dec = inv.apply(enc);
            assert_eq!(dec, value);
        }
    }

    #[test]
    fn affine8_composition_matches_manual() {
        let mut rng = ChaCha20Rng::from_seed([11u8; 32]);
        let a = Affine8::random(&mut rng);
        let b = Affine8::random(&mut rng);
        let composed = a.compose(&b);
        let value = rng.next_u32() as u8;
        let direct = a.apply(b.apply(value));
        let via_comp = composed.apply(value);
        assert_eq!(direct, via_comp);
    }

    #[test]
    fn affine256_roundtrip() {
        let mut rng = ChaCha20Rng::from_seed([12u8; 32]);
        let aff = Affine256::random_sparse_unsplit(&mut rng);
        let inv = aff.invert().expect("invertible");
        let mut value = [0u8; 32];
        rng.fill_bytes(&mut value);
        let enc = aff.apply(&value);
        let dec = inv.apply(&enc);
        assert_eq!(dec, value);
    }

    #[test]
    fn affine256_composition_matches_manual() {
        let mut rng = ChaCha20Rng::from_seed([13u8; 32]);
        let a = Affine256::random_sparse_unsplit(&mut rng);
        let b = Affine256::random_sparse_unsplit(&mut rng);
        let composed = a.compose(&b);
        let mut value = [0u8; 32];
        rng.fill_bytes(&mut value);
        let direct = a.apply(&b.apply(&value));
        let via_comp = composed.apply(&value);
        assert_eq!(direct, via_comp);
    }
}
