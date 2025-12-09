//! Demonstrates generating a white-box instance and encrypting two blocks.

use aes_core::{encrypt_block, expand_key, Aes128Key};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use wbaes_gen::{Generator, GeneratorConfig};
use wbaes_runtime::WbCipher256;

fn main() {
    // Deterministic seed for reproducibility in the example.
    let key = Aes128Key::from([0u8; 16]);
    let mut gen = Generator::with_config(
        ChaCha20Rng::from_seed([1u8; 32]),
        GeneratorConfig {
            external_encodings: false,
        },
    );
    let instance = gen.generate_instance(&key);
    let cipher = WbCipher256::new(instance);

    let mut block = [0u8; 32];
    block[..16].copy_from_slice(b"first block here");
    block[16..].copy_from_slice(b"second blockhere");

    let round_keys = expand_key(&key);
    let expected_first = encrypt_block(&block[..16].try_into().unwrap(), &round_keys);
    let expected_second = encrypt_block(&block[16..].try_into().unwrap(), &round_keys);

    cipher.encrypt_block(&mut block);
    assert_eq!(&block[..16], &expected_first);
    assert_eq!(&block[16..], &expected_second);

    println!("example succeeded; ciphertext matches AES reference");
}
