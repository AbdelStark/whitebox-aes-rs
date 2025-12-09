//! AES linear layer matrices.

use core::convert::TryInto;

use aes_core::round::{mix_columns, shift_rows};
use aes_core::Block;

use crate::matrix::{Matrix128, Matrix256};

/// Returns the matrix for `MC ∘ SR` on a single 128-bit AES state.
pub fn mc_sr_matrix_128() -> Matrix128 {
    Matrix128::from_linear_transform(|state: &mut [u8; 16]| {
        shift_rows(state);
        mix_columns(state);
    })
}

/// Returns the block-diagonal matrix for `MC ∘ SR` on two concatenated AES states (256 bits).
pub fn mc_sr_matrix_256() -> Matrix256 {
    Matrix256::from_linear_transform(|state: &mut [u8; 32]| {
        let (first, second) = state.split_at_mut(16);
        apply_mc_sr(first);
        apply_mc_sr(second);
    })
}

fn apply_mc_sr(state: &mut [u8]) {
    let block: &mut Block = state
        .try_into()
        .expect("apply_mc_sr expects a 16-byte AES state");
    shift_rows(block);
    mix_columns(block);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn mc_sr_128_matches_aes_round_linear_layer() {
        let matrix = mc_sr_matrix_128();
        let mut rng = ChaCha20Rng::from_seed([20u8; 32]);
        for _ in 0..32 {
            let mut state = [0u8; 16];
            rng.fill_bytes(&mut state);
            let mut expected = state;
            shift_rows(&mut expected);
            mix_columns(&mut expected);
            let actual = matrix.apply_to_bytes(&state);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn mc_sr_256_matches_two_block_application() {
        let matrix = mc_sr_matrix_256();
        let mut rng = ChaCha20Rng::from_seed([21u8; 32]);
        for _ in 0..32 {
            let mut state = [0u8; 32];
            rng.fill_bytes(&mut state);
            let mut expected = state;
            {
                let (first, second) = expected.split_at_mut(16);
                apply_mc_sr(first);
                apply_mc_sr(second);
            }
            let actual = matrix.apply_to_bytes(&state);
            assert_eq!(actual, expected);
        }
    }
}
