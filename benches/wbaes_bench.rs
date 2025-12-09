use criterion::{criterion_group, criterion_main, Criterion};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

use aes_core::{encrypt_block, expand_key, Aes128Key};
use wbaes_gen::{Generator, GeneratorConfig};
use wbaes_runtime::WbCipher256;

fn bench_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation");
    group.sample_size(10);
    group.bench_function("generate_instance", |b| {
        b.iter(|| {
            let key = Aes128Key::from([0u8; 16]);
            let mut gen = Generator::with_config(
                ChaCha20Rng::from_seed([1u8; 32]),
                GeneratorConfig {
                    external_encodings: false,
                },
            );
            gen.generate_instance(&key);
        });
    });
    group.finish();
}

fn bench_runtime(c: &mut Criterion) {
    let key = Aes128Key::from([0u8; 16]);
    let mut gen = Generator::with_config(
        ChaCha20Rng::from_seed([2u8; 32]),
        GeneratorConfig {
            external_encodings: false,
        },
    );
    let instance = gen.generate_instance(&key);
    let cipher = WbCipher256::new(instance);

    let round_keys = expand_key(&key);
    let mut rng = ChaCha20Rng::from_seed([3u8; 32]);

    let mut group = c.benchmark_group("runtime");
    group.sample_size(20);
    group.bench_function("wbaes_encrypt_block", |b| {
        let mut block = [0u8; 32];
        rng.fill_bytes(&mut block);
        b.iter(|| {
            let mut data = block;
            cipher.encrypt_block(&mut data);
        });
    });
    group.bench_function("aes_core_encrypt_pair", |b| {
        let mut block1 = [0u8; 16];
        let mut block2 = [0u8; 16];
        rng.fill_bytes(&mut block1);
        rng.fill_bytes(&mut block2);
        b.iter(|| {
            let _ = encrypt_block(&block1, &round_keys);
            let _ = encrypt_block(&block2, &round_keys);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_generation, bench_runtime);
criterion_main!(benches);
