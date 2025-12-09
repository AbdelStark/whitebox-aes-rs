//! Block representation helpers.

/// AES block of 16 bytes.
pub type Block = [u8; 16];

/// XORs two blocks, writing the result into `dst`.
#[inline]
pub fn xor_in_place(dst: &mut Block, rhs: &Block) {
    for (d, r) in dst.iter_mut().zip(rhs.iter()) {
        *d ^= *r;
    }
}
