//! AES round transformations.

use crate::block::{xor_in_place, Block};
use crate::sbox::{inv_sbox, sbox};

/// Applies SubBytes to the state in place.
#[inline]
pub fn sub_bytes(state: &mut Block) {
    for byte in state.iter_mut() {
        *byte = sbox(*byte);
    }
}

/// Applies the inverse SubBytes transformation.
#[inline]
pub fn inv_sub_bytes(state: &mut Block) {
    for byte in state.iter_mut() {
        *byte = inv_sbox(*byte);
    }
}

/// Performs ShiftRows in place.
#[inline]
pub fn shift_rows(state: &mut Block) {
    let mut tmp = [0u8; 16];
    tmp[0] = state[0];
    tmp[1] = state[5];
    tmp[2] = state[10];
    tmp[3] = state[15];

    tmp[4] = state[4];
    tmp[5] = state[9];
    tmp[6] = state[14];
    tmp[7] = state[3];

    tmp[8] = state[8];
    tmp[9] = state[13];
    tmp[10] = state[2];
    tmp[11] = state[7];

    tmp[12] = state[12];
    tmp[13] = state[1];
    tmp[14] = state[6];
    tmp[15] = state[11];

    *state = tmp;
}

/// Performs the inverse of ShiftRows in place.
#[inline]
pub fn inv_shift_rows(state: &mut Block) {
    let mut tmp = [0u8; 16];
    tmp[0] = state[0];
    tmp[1] = state[13];
    tmp[2] = state[10];
    tmp[3] = state[7];

    tmp[4] = state[4];
    tmp[5] = state[1];
    tmp[6] = state[14];
    tmp[7] = state[11];

    tmp[8] = state[8];
    tmp[9] = state[5];
    tmp[10] = state[2];
    tmp[11] = state[15];

    tmp[12] = state[12];
    tmp[13] = state[9];
    tmp[14] = state[6];
    tmp[15] = state[3];

    *state = tmp;
}

fn xtime(byte: u8) -> u8 {
    let shifted = byte << 1;
    if byte & 0x80 != 0 {
        shifted ^ 0x1b
    } else {
        shifted
    }
}

fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut product = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            product ^= a;
        }
        let hi_bit_set = a & 0x80;
        a <<= 1;
        if hi_bit_set != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    product
}

fn mix_single_column(col: &mut [u8; 4]) {
    let [a0, a1, a2, a3] = *col;
    col[0] = xtime(a0) ^ (xtime(a1) ^ a1) ^ a2 ^ a3;
    col[1] = a0 ^ xtime(a1) ^ (xtime(a2) ^ a2) ^ a3;
    col[2] = a0 ^ a1 ^ xtime(a2) ^ (xtime(a3) ^ a3);
    col[3] = (xtime(a0) ^ a0) ^ a1 ^ a2 ^ xtime(a3);
}

fn inv_mix_single_column(col: &mut [u8; 4]) {
    let [a0, a1, a2, a3] = *col;
    col[0] = gmul(a0, 0x0e) ^ gmul(a1, 0x0b) ^ gmul(a2, 0x0d) ^ gmul(a3, 0x09);
    col[1] = gmul(a0, 0x09) ^ gmul(a1, 0x0e) ^ gmul(a2, 0x0b) ^ gmul(a3, 0x0d);
    col[2] = gmul(a0, 0x0d) ^ gmul(a1, 0x09) ^ gmul(a2, 0x0e) ^ gmul(a3, 0x0b);
    col[3] = gmul(a0, 0x0b) ^ gmul(a1, 0x0d) ^ gmul(a2, 0x09) ^ gmul(a3, 0x0e);
}

/// MixColumns over all four columns.
#[inline]
pub fn mix_columns(state: &mut Block) {
    for col in 0..4 {
        let idx = col * 4;
        let mut column = [state[idx], state[idx + 1], state[idx + 2], state[idx + 3]];
        mix_single_column(&mut column);
        state[idx] = column[0];
        state[idx + 1] = column[1];
        state[idx + 2] = column[2];
        state[idx + 3] = column[3];
    }
}

/// Inverse MixColumns over all four columns.
#[inline]
pub fn inv_mix_columns(state: &mut Block) {
    for col in 0..4 {
        let idx = col * 4;
        let mut column = [state[idx], state[idx + 1], state[idx + 2], state[idx + 3]];
        inv_mix_single_column(&mut column);
        state[idx] = column[0];
        state[idx + 1] = column[1];
        state[idx + 2] = column[2];
        state[idx + 3] = column[3];
    }
}

/// Adds (XORs) a round key into the state.
#[inline]
pub fn add_round_key(state: &mut Block, round_key: &Block) {
    xor_in_place(state, round_key);
}
