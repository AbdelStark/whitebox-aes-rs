//! GF(2) matrix utilities for 8×8 and 256×256 dimensions.

use core::convert::TryInto;

use rand::{CryptoRng, RngCore};

/// 8×8 binary matrix over GF(2), stored row-major with each row packed into a `u8`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matrix8 {
    rows: [u8; 8],
}

impl Matrix8 {
    /// Returns the zero matrix.
    pub fn zero() -> Self {
        Self { rows: [0u8; 8] }
    }

    /// Returns the identity matrix.
    pub fn identity() -> Self {
        let mut rows = [0u8; 8];
        for (i, row) in rows.iter_mut().enumerate() {
            *row = 1u8 << i;
        }
        Self { rows }
    }

    /// Generates a uniformly random 8×8 matrix (not necessarily invertible).
    fn random<R: RngCore>(rng: &mut R) -> Self {
        let mut rows = [0u8; 8];
        for row in rows.iter_mut() {
            *row = rng.next_u32() as u8;
        }
        Self { rows }
    }

    /// Generates a uniformly random invertible matrix, retrying until one is found.
    pub fn random_invertible<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        loop {
            let candidate = Self::random(rng);
            if candidate.is_invertible() {
                return candidate;
            }
        }
    }

    /// Applies the matrix to an 8-bit value, treating bits as a column vector.
    pub fn apply(&self, value: u8) -> u8 {
        let mut out = 0u8;
        for (row_idx, row) in self.rows.iter().enumerate() {
            let parity = (row & value).count_ones() as u8 & 1;
            out |= parity << row_idx;
        }
        out
    }

    /// Multiplies two matrices (`self * rhs`).
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut result = Self::zero();
        for (row_idx, row_bits) in self.rows.iter().enumerate() {
            let mut acc = 0u8;
            let mut bits = *row_bits;
            while bits != 0 {
                let bit = bits.trailing_zeros() as usize;
                acc ^= rhs.rows[bit];
                bits &= bits - 1;
            }
            result.rows[row_idx] = acc;
        }
        result
    }

    /// Attempts to invert the matrix via Gaussian elimination.
    pub fn invert(&self) -> Option<Self> {
        let mut left = self.rows;
        let mut right = Self::identity().rows;

        for col in 0..8 {
            let mut pivot = None;
            for (row_idx, row_bits) in left.iter().enumerate().skip(col) {
                if (row_bits >> col) & 1 == 1 {
                    pivot = Some(row_idx);
                    break;
                }
            }
            let pivot = pivot?;
            if pivot != col {
                left.swap(pivot, col);
                right.swap(pivot, col);
            }
            for row in 0..8 {
                if row != col && ((left[row] >> col) & 1 == 1) {
                    left[row] ^= left[col];
                    right[row] ^= right[col];
                }
            }
        }

        Some(Self { rows: right })
    }

    /// Returns true if the matrix is invertible.
    pub fn is_invertible(&self) -> bool {
        self.invert().is_some()
    }

    /// Exposes the underlying rows (little-endian bit order within each byte).
    pub fn rows(&self) -> &[u8; 8] {
        &self.rows
    }
}

/// 256×256 binary matrix over GF(2), stored row-major, four `u64` segments per row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix256 {
    rows: [[u64; 4]; 256],
}

impl Matrix256 {
    /// Returns the zero matrix.
    pub fn zero() -> Self {
        Self {
            rows: [[0u64; 4]; 256],
        }
    }

    /// Returns the identity matrix.
    pub fn identity() -> Self {
        let mut rows = [[0u64; 4]; 256];
        for (i, row) in rows.iter_mut().enumerate() {
            let segment = i / 64;
            let offset = i % 64;
            row[segment] |= 1u64 << offset;
        }
        Self { rows }
    }

    /// Sets a single bit at (row, col).
    fn set_bit(&mut self, row: usize, col: usize, value: bool) {
        let segment = col / 64;
        let offset = col % 64;
        let mask = 1u64 << offset;
        if value {
            self.rows[row][segment] |= mask;
        } else {
            self.rows[row][segment] &= !mask;
        }
    }

    /// Reads a bit at (row, col).
    fn bit(&self, row: usize, col: usize) -> bool {
        let segment = col / 64;
        let offset = col % 64;
        (self.rows[row][segment] >> offset) & 1 == 1
    }

    /// Clears an 8×8 block.
    fn clear_block(&mut self, row_block: usize, col_block: usize) {
        for row in 0..8 {
            for col in 0..8 {
                self.set_bit(row_block * 8 + row, col_block * 8 + col, false);
            }
        }
    }

    /// Sets an 8×8 block at `(row_block, col_block)` to `block`.
    pub fn set_block(&mut self, row_block: usize, col_block: usize, block: &Matrix8) {
        self.clear_block(row_block, col_block);
        for row in 0..8 {
            let row_bits = block.rows[row];
            for bit in 0..8 {
                let value = (row_bits >> bit) & 1 == 1;
                if value {
                    self.set_bit(row_block * 8 + row, col_block * 8 + bit, true);
                }
            }
        }
    }

    /// Returns the 8×8 block at `(row_block, col_block)`.
    pub fn block(&self, row_block: usize, col_block: usize) -> Matrix8 {
        let mut rows = [0u8; 8];
        for (row_offset, row_slot) in rows.iter_mut().enumerate() {
            let mut row_bits = 0u8;
            for bit in 0..8 {
                if self.bit(row_block * 8 + row_offset, col_block * 8 + bit) {
                    row_bits |= 1u8 << bit;
                }
            }
            *row_slot = row_bits;
        }
        Matrix8 { rows }
    }

    /// Generates a sparse unsplit matrix with the banded structure described in the revisited scheme.
    ///
    /// Non-zero blocks appear only on the diagonal, first super-diagonal, and the wrap-around block
    /// from the last row block to the first column block. Diagonal blocks are guaranteed invertible;
    /// generation retries until the full 256×256 matrix is invertible.
    pub fn random_sparse_unsplit<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        loop {
            let mut mat = Self::zero();
            for block in 0..32 {
                let diag = Matrix8::random_invertible(rng);
                mat.set_block(block, block, &diag);
            }
            for block in 0..31 {
                let super_block = Matrix8::random(rng);
                mat.set_block(block, block + 1, &super_block);
            }
            let wrap_block = Matrix8::random(rng);
            mat.set_block(31, 0, &wrap_block);

            if mat.is_invertible() {
                return mat;
            }
        }
    }

    /// Multiplies two matrices (`self * rhs`).
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut result = Self::zero();
        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut accum = [0u64; 4];
            for (segment_idx, segment) in row.iter().enumerate() {
                let mut bits = *segment;
                while bits != 0 {
                    let bit = bits.trailing_zeros() as usize;
                    let source_row = segment_idx * 64 + bit;
                    for (seg_idx, accum_seg) in accum.iter_mut().enumerate() {
                        *accum_seg ^= rhs.rows[source_row][seg_idx];
                    }
                    bits &= bits - 1;
                }
            }
            result.rows[row_idx] = accum;
        }
        result
    }

    /// Attempts to invert the matrix via bit-sliced Gaussian elimination.
    pub fn invert(&self) -> Option<Self> {
        let mut left = self.rows;
        let mut right = Self::identity().rows;

        for col in 0..256 {
            let mut pivot = None;
            for (row_idx, row_bits) in left.iter().enumerate().skip(col) {
                if (row_bits[col / 64] >> (col % 64)) & 1 == 1 {
                    pivot = Some(row_idx);
                    break;
                }
            }
            let pivot = pivot?;
            if pivot != col {
                left.swap(pivot, col);
                right.swap(pivot, col);
            }
            for row in 0..256 {
                if row == col {
                    continue;
                }
                if (left[row][col / 64] >> (col % 64)) & 1 == 1 {
                    for seg in 0..4 {
                        left[row][seg] ^= left[col][seg];
                        right[row][seg] ^= right[col][seg];
                    }
                }
            }
        }

        Some(Self { rows: right })
    }

    /// Returns true if the matrix is invertible.
    pub fn is_invertible(&self) -> bool {
        self.invert().is_some()
    }

    /// Applies the matrix to a 256-bit vector represented as 32 bytes.
    pub fn apply_to_bytes(&self, input: &[u8; 32]) -> [u8; 32] {
        let input_segments = bytes_to_segments(input);
        let mut output_segments = [0u64; 4];

        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut acc = 0u32;
            for seg in 0..4 {
                acc ^= (row[seg] & input_segments[seg]).count_ones();
            }
            if acc & 1 == 1 {
                let segment = row_idx / 64;
                let offset = row_idx % 64;
                output_segments[segment] |= 1u64 << offset;
            }
        }

        segments_to_bytes(&output_segments)
    }

    /// Applies the matrix to a 256-bit vector in place.
    pub fn apply_in_place(&self, input: &mut [u8; 32]) {
        *input = self.apply_to_bytes(input);
    }

    /// Returns the map `u8 -> 256-bit` for the given byte position, using the current linear map.
    pub fn submatrix_byte_map(&self, byte_index: usize) -> [[u8; 32]; 256] {
        assert!(byte_index < 32, "byte index out of range");

        let mut basis_outputs = [[0u8; 32]; 8];
        for (bit, output) in basis_outputs.iter_mut().enumerate() {
            let mut input = [0u8; 32];
            input[byte_index] = 1u8 << bit;
            *output = self.apply_to_bytes(&input);
        }

        let mut map = [[0u8; 32]; 256];
        for value in 1u16..=255 {
            let mut acc = [0u8; 32];
            let mut v = value as u8;
            let mut bit = 0;
            while v != 0 {
                if v & 1 == 1 {
                    xor_bytes(&mut acc, &basis_outputs[bit]);
                }
                v >>= 1;
                bit += 1;
            }
            map[value as usize] = acc;
        }
        map
    }
}

fn bytes_to_segments(bytes: &[u8; 32]) -> [u64; 4] {
    [
        u64::from_le_bytes(bytes[0..8].try_into().expect("slice length 8")),
        u64::from_le_bytes(bytes[8..16].try_into().expect("slice length 8")),
        u64::from_le_bytes(bytes[16..24].try_into().expect("slice length 8")),
        u64::from_le_bytes(bytes[24..32].try_into().expect("slice length 8")),
    ]
}

fn segments_to_bytes(segments: &[u64; 4]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (idx, segment) in segments.iter().enumerate() {
        let bytes = segment.to_le_bytes();
        let start = idx * 8;
        out[start..start + 8].copy_from_slice(&bytes);
    }
    out
}

fn xor_bytes(dst: &mut [u8; 32], src: &[u8; 32]) {
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
    fn matrix8_inversion_roundtrip() {
        let mut rng = ChaCha20Rng::from_seed([1u8; 32]);
        for _ in 0..32 {
            let m = Matrix8::random_invertible(&mut rng);
            let inv = m.invert().expect("invertible");
            let prod = m.mul(&inv);
            assert_eq!(prod, Matrix8::identity());
        }
    }

    #[test]
    fn matrix8_apply_inverse_recovers_input() {
        let mut rng = ChaCha20Rng::from_seed([2u8; 32]);
        for _ in 0..32 {
            let m = Matrix8::random_invertible(&mut rng);
            let inv = m.invert().unwrap();
            let value = rng.next_u32() as u8;
            let out = m.apply(value);
            let recovered = inv.apply(out);
            assert_eq!(recovered, value);
        }
    }

    #[test]
    fn matrix256_sparse_structure() {
        let mut rng = ChaCha20Rng::from_seed([3u8; 32]);
        let m = Matrix256::random_sparse_unsplit(&mut rng);
        for row_block in 0..32 {
            for col_block in 0..32 {
                let block = m.block(row_block, col_block);
                let is_allowed = col_block == row_block
                    || col_block == row_block + 1
                    || (row_block == 31 && col_block == 0);
                if is_allowed {
                    continue;
                }
                assert_eq!(
                    block,
                    Matrix8::zero(),
                    "block ({}, {}) should be zero",
                    row_block,
                    col_block
                );
            }
        }
    }

    #[test]
    fn matrix256_inversion_roundtrip() {
        let mut rng = ChaCha20Rng::from_seed([4u8; 32]);
        let m = Matrix256::random_sparse_unsplit(&mut rng);
        let inv = m.invert().expect("matrix must be invertible");
        let prod = m.mul(&inv);
        assert_eq!(prod, Matrix256::identity());
    }

    #[test]
    fn matrix256_apply_inverse_recovers_input() {
        let mut rng = ChaCha20Rng::from_seed([5u8; 32]);
        let m = Matrix256::random_sparse_unsplit(&mut rng);
        let inv = m.invert().unwrap();

        let mut input = [0u8; 32];
        for chunk in input.iter_mut() {
            *chunk = rng.next_u32() as u8;
        }
        let out = m.apply_to_bytes(&input);
        let recovered = inv.apply_to_bytes(&out);
        assert_eq!(recovered, input);
    }

    #[test]
    fn submatrix_byte_map_matches_direct_application() {
        let mut rng = ChaCha20Rng::from_seed([6u8; 32]);
        let m = Matrix256::random_sparse_unsplit(&mut rng);
        let byte_index = 7usize;
        let map = m.submatrix_byte_map(byte_index);
        for value in [0u8, 1, 0x5a, 0xff] {
            let mut input = [0u8; 32];
            input[byte_index] = value;
            let direct = m.apply_to_bytes(&input);
            assert_eq!(map[value as usize], direct);
        }
    }
}
