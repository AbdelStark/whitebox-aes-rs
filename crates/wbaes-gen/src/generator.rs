//! Instance generator for the revisited white-box AES scheme.

use std::convert::TryInto;

use aes_core::{expand_key, sbox, Aes128Key};
use rand::{CryptoRng, RngCore};

use crate::affine::Affine256;
use crate::instance::{ExternalEncodings, WbInstance256};
use crate::linear::{mc_sr_matrix_256, sr_matrix_256};
use crate::matrix::Matrix256;
use crate::tables::{HTable, RoundTables};

/// Configuration for the generator.
#[derive(Clone, Debug, Default)]
pub struct GeneratorConfig {
    /// Whether to include random external encodings (`Min`, `Mout`).
    pub external_encodings: bool,
}

/// White-box instance generator parametrized by an RNG.
pub struct Generator<R: RngCore + CryptoRng> {
    rng: R,
    config: GeneratorConfig,
}

impl<R: RngCore + CryptoRng> Generator<R> {
    /// Creates a new generator with default configuration.
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            config: GeneratorConfig::default(),
        }
    }

    /// Creates a generator with explicit configuration.
    pub fn with_config(rng: R, config: GeneratorConfig) -> Self {
        Self { rng, config }
    }

    /// Returns a mutable reference to the configuration.
    pub fn config_mut(&mut self) -> &mut GeneratorConfig {
        &mut self.config
    }

    /// Generates a white-box instance for the provided AES-128 key.
    pub fn generate_instance(&mut self, key: &Aes128Key) -> WbInstance256 {
        let round_keys = expand_key(key);
        let mc_sr = mc_sr_matrix_256();
        let sr_only = sr_matrix_256();

        let key0_block = duplicate_round_key(round_keys.get(0));
        let key0_affine = Affine256::new(Matrix256::identity(), key0_block);

        let mut a_encodings = Vec::with_capacity(10);
        for _ in 0..10 {
            a_encodings.push(Affine256::random_sparse_unsplit(&mut self.rng));
        }

        let (min_encoding, mout_encoding) = if self.config.external_encodings {
            (
                Affine256::random_sparse_unsplit(&mut self.rng),
                Some(Affine256::random_sparse_unsplit(&mut self.rng)),
            )
        } else {
            (Affine256::identity(), None)
        };

        let a1_inv = a_encodings[0].invert().expect("A^(1) should be invertible");
        let min_total = min_encoding.compose(&key0_affine);
        let input_encoding = a1_inv.compose(&min_total);

        let mut rounds: Vec<RoundTables> = Vec::with_capacity(10);
        for r in 0..10 {
            let a_curr = &a_encodings[r];
            let identity_output = Affine256::identity();
            let next_affine = if r == 9 {
                mout_encoding.as_ref().unwrap_or(&identity_output)
            } else {
                &a_encodings[r + 1]
            };
            let linear_layer = if r == 9 { &sr_only } else { &mc_sr };
            let round_key_block = duplicate_round_key(round_keys.get(r + 1));
            let round_tables = build_round(
                &mut self.rng,
                a_curr,
                next_affine,
                linear_layer,
                &round_key_block,
            );
            rounds.push(round_tables);
        }

        let rounds: [RoundTables; 10] = rounds
            .try_into()
            .expect("round vector should have length 10");

        WbInstance256 {
            rounds,
            encodings: ExternalEncodings {
                input: input_encoding,
                output: None, // output encoding is folded into round 10
            },
            params: Default::default(),
        }
    }
}

fn build_round<R: RngCore + CryptoRng>(
    rng: &mut R,
    a_curr: &Affine256,
    next_affine: &Affine256,
    linear_layer: &Matrix256,
    round_key_block: &[u8; 32],
) -> RoundTables {
    let next_inv = next_affine
        .lin
        .invert()
        .expect("next affine must be invertible");
    let b_lin = next_inv.mul(linear_layer);
    let mut b_bias_target = next_inv.apply_to_bytes(&next_affine.bias);
    let key_contribution = next_inv.apply_to_bytes(round_key_block);
    xor_in_place(&mut b_bias_target, &key_contribution);
    let b_biases = split_biases(rng, &b_bias_target);
    let b_maps: [Vec<[u8; 32]>; 32] = std::array::from_fn(|i| {
        let map = b_lin.submatrix_byte_map(i);
        map.into_iter().collect()
    });

    let h_tables: [HTable; 32] = std::array::from_fn(|_| HTable::random(rng));

    let mut round_tables = RoundTables::new_zeroed();

    for i in 0..32 {
        let block_left = a_curr.lin.block(i, i);
        let block_right = if i == 31 {
            a_curr.lin.block(i, 0)
        } else {
            a_curr.lin.block(i, i + 1)
        };
        let a_bias = a_curr.bias[i];
        let b_bias = &b_biases[i];
        let h_i = &h_tables[i];
        let h_next = &h_tables[(i + 1) % 32];
        let b_map = &b_maps[i];

        for x in 0u16..=255 {
            for y in 0u16..=255 {
                let z = block_left.apply(x as u8) ^ block_right.apply(y as u8) ^ a_bias;
                let t = sbox(z);
                let mut value = b_map[t as usize];
                xor_in_place(&mut value, b_bias);
                xor_in_place(&mut value, h_i.get(x as u8));
                xor_in_place(&mut value, h_next.get(y as u8));
                round_tables.tables[i].set(x as u8, y as u8, &value);
            }
        }
    }

    round_tables
}

fn split_biases<R: RngCore + CryptoRng>(rng: &mut R, target: &[u8; 32]) -> [[u8; 32]; 32] {
    let mut biases = [[0u8; 32]; 32];
    let mut accum = [0u8; 32];
    for bias in biases.iter_mut().take(31) {
        rng.fill_bytes(bias);
        xor_in_place(&mut accum, bias);
    }
    let last = &mut biases[31];
    for (dst, (&t, &acc)) in last.iter_mut().zip(target.iter().zip(accum.iter())) {
        *dst = t ^ acc;
    }
    biases
}

fn xor_in_place(dst: &mut [u8; 32], src: &[u8; 32]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d ^= *s;
    }
}

fn duplicate_round_key(round_key: &[u8; 16]) -> [u8; 32] {
    let mut block = [0u8; 32];
    block[..16].copy_from_slice(round_key);
    block[16..].copy_from_slice(round_key);
    block
}
