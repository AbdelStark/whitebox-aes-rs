//! AES-128 key schedule and block encryption/decryption.

use core::convert::TryInto;

use crate::block::Block;
use crate::key::{Aes128Key, RoundKeys};
use crate::round::{
    add_round_key, inv_mix_columns, inv_shift_rows, inv_sub_bytes, mix_columns, shift_rows,
    sub_bytes,
};
use crate::sbox::sbox;

const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

fn rot_word(word: u32) -> u32 {
    word.rotate_left(8)
}

fn sub_word(word: u32) -> u32 {
    let b0 = sbox((word >> 24) as u8) as u32;
    let b1 = sbox((word >> 16) as u8) as u32;
    let b2 = sbox((word >> 8) as u8) as u32;
    let b3 = sbox(word as u8) as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

fn u32_from_be(bytes: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*bytes)
}

fn be_from_u32(word: u32) -> [u8; 4] {
    word.to_be_bytes()
}

/// Expands a 128-bit key into 11 round keys.
pub fn expand_key(key: &Aes128Key) -> RoundKeys {
    let mut w = [0u32; 44];
    for (i, chunk) in key.0.chunks_exact(4).enumerate() {
        let bytes: [u8; 4] = chunk.try_into().expect("chunk length is four");
        w[i] = u32_from_be(&bytes);
    }

    for i in 4..44 {
        let mut temp = w[i - 1];
        if i % 4 == 0 {
            temp = sub_word(rot_word(temp)) ^ (u32::from(RCON[(i / 4) - 1]) << 24);
        }
        w[i] = w[i - 4] ^ temp;
    }

    let mut round_keys = [[0u8; 16]; 11];
    for round in 0..11 {
        for word_idx in 0..4 {
            let word = w[round * 4 + word_idx];
            let bytes = be_from_u32(word);
            let offset = word_idx * 4;
            round_keys[round][offset] = bytes[0];
            round_keys[round][offset + 1] = bytes[1];
            round_keys[round][offset + 2] = bytes[2];
            round_keys[round][offset + 3] = bytes[3];
        }
    }

    RoundKeys(round_keys)
}

/// Encrypts a single 16-byte block with pre-expanded round keys.
pub fn encrypt_block(block: &Block, round_keys: &RoundKeys) -> Block {
    let mut state = *block;

    add_round_key(&mut state, round_keys.get(0));

    for round in 1..10 {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, round_keys.get(round));
    }

    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, round_keys.get(10));

    state
}

/// Decrypts a single 16-byte block with pre-expanded round keys.
pub fn decrypt_block(block: &Block, round_keys: &RoundKeys) -> Block {
    let mut state = *block;

    add_round_key(&mut state, round_keys.get(10));
    for round in (1..10).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, round_keys.get(round));
        inv_mix_columns(&mut state);
    }
    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, round_keys.get(0));

    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::Aes128Key;
    use rand::RngCore;

    const NIST_KEY: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f,
    ];
    const NIST_PLAIN: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];
    const NIST_CIPHER: [u8; 16] = [
        0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4, 0xc5,
        0x5a,
    ];

    #[test]
    fn encrypt_matches_nist_vector() {
        let key = Aes128Key::from(NIST_KEY);
        let round_keys = expand_key(&key);
        let ct = encrypt_block(&NIST_PLAIN, &round_keys);
        assert_eq!(ct, NIST_CIPHER);
    }

    #[test]
    fn decrypt_matches_nist_vector() {
        let key = Aes128Key::from(NIST_KEY);
        let round_keys = expand_key(&key);
        let pt = decrypt_block(&NIST_CIPHER, &round_keys);
        assert_eq!(pt, NIST_PLAIN);
    }

    #[test]
    fn encrypt_decrypt_round_trip_random() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let mut key_bytes = [0u8; 16];
            let mut block = [0u8; 16];
            rng.fill_bytes(&mut key_bytes);
            rng.fill_bytes(&mut block);
            let key = Aes128Key::from(key_bytes);
            let rks = expand_key(&key);
            let ct = encrypt_block(&block, &rks);
            let pt = decrypt_block(&ct, &rks);
            assert_eq!(pt, block);
        }
    }
}
