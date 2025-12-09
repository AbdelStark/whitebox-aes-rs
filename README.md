# whitebox-aes-rs

Rust workspace implementing Baek–Cheon–Hong’s “White-Box AES Implementation Revisited” scheme (two AES-128 blocks with sparse unsplit 256-bit encodings). Built for study and experimentation—not for production key protection.

## Highlights
- **Clean AES-128 core (`aes-core`)**: key expansion, encrypt/decrypt, NIST vectors.
- **White-box generator (`wbaes-gen`)**: sparse unsplit affine encodings, per-round 32×16→256-bit tables, mask gadgets, external encodings (optional).
- **Runtime evaluator (`wbaes-runtime`)**: table execution for 32-byte blocks with external encodings.
- **CLI (`wbaes-cli`)**: generate instances, encrypt/decrypt, correctness check, and a self-contained demo.
- **Docs & tooling**: design/background docs, example, Criterion benches, CI (fmt/clippy/test).

## Repository layout
- `crates/` — `aes-core`, `wbaes-gen`, `wbaes-runtime`, `wbaes-cli`.
- `docs/` — `design.md` (mapping scheme→code), `whitebox_background.md` (threat model/context).
- `examples/basic.rs` — minimal generation/encryption roundtrip.
- `benches/wbaes_bench.rs` — generation/runtime benchmarks.
- `.github/workflows/ci.yml` — fmt/clippy/test on stable.

## Quick start (CLI)
```bash
# Generate an instance (no external output encoding by default)
cargo run -p wbaes-cli -- gen \
  --key-hex 000102030405060708090a0b0c0d0e0f \
  --out wb.bin

# Encrypt a multiple of 32 bytes
cargo run -p wbaes-cli -- enc --instance wb.bin --input plain.bin --output ct.bin

# Decrypt (only when external output encoding is disabled)
cargo run -p wbaes-cli -- dec \
  --instance wb.bin \
  --key-hex 000102030405060708090a0b0c0d0e0f \
  --in ct.bin --out pt.bin

# Compare white-box vs AES for random samples
cargo run -p wbaes-cli -- check \
  --instance wb.bin \
  --key-hex 000102030405060708090a0b0c0d0e0f

# Quick demo: generate key/instance, encrypt random 32B, decrypt back
cargo run -p wbaes-cli -- demo
```

## Library sketch
```rust
use aes_core::{Aes128Key, expand_key, encrypt_block};
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use wbaes_gen::{Generator, GeneratorConfig};
use wbaes_runtime::WbCipher256;

let key = Aes128Key::from([0u8; 16]);
let mut gen = Generator::with_config(
    ChaCha20Rng::from_seed([1u8; 32]),
    GeneratorConfig { external_encodings: false },
);
let instance = gen.generate_instance(&key);
let cipher = WbCipher256::new(instance);

let mut block = [0u8; 32];
cipher.encrypt_block(&mut block);
```

See `examples/basic.rs` for a full AES-consistency check.

## Design & background
- Implementation mapping and data flow: `docs/design.md`
- Threat model and CEJO/Chow context: `docs/whitebox_background.md`

### White-box AES flow (mermaid)
```mermaid
flowchart LR
    P[Plaintext (32B)] --> F0[External input encoding F^(0)]
    F0 --> R1[Round 1: 32 tables (T_i^(1))]
    R1 --> R2[Round 2]
    R2 --> R3[Round 3]
    R3 --> R4[Round 4]
    R4 --> R5[Round 5]
    R5 --> R6[Round 6]
    R6 --> R7[Round 7]
    R7 --> R8[Round 8]
    R8 --> R9[Round 9]
    R9 --> R10[Round 10 (SR only, Mout folded)]
    R10 --> OUT[Ciphertext (32B)]
    classDef tbl fill:#dfe7fd,stroke:#5c6bc0,stroke-width:1px,color:#1b1b1b;
    class R1,R2,R3,R4,R5,R6,R7,R8,R9,R10 tbl;
```

Key references:
- S. Chow et al., “White-Box Cryptography and an AES Implementation,” SAC 2002.
- J. A. Muir, “A Tutorial on White-box AES,” 2013.
- C. H. Baek, J. H. Cheon, H. Hong, “White-Box AES Implementation Revisited,” JCN 2016 (ePrint 2014/688).

## Security model (please read)
- **Research/educational only.** Not a secure key-protection mechanism.
- Vulnerable to known analytic/DCA-style attacks (BGE, Baek–Cheon–Hong toolbox, etc.).
- No side-channel hardening. External encodings are optional and default off for testability.
- Treat all keys and tables as sensitive; avoid logging or exposing them.

## Build, test, bench
- MSRV: stable Rust 1.75+ (edition 2021).
- CI: fmt, clippy (`-D warnings`), test on stable.
- Local:
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features --workspace -- -D warnings`
  - `cargo test --workspace`
  - `cargo bench` (Criterion; generation is heavyweight)
