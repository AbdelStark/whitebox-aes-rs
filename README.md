# whitebox-aes-rs

Experimental Rust implementation of Baek–Cheon–Hong’s “White-Box AES Implementation Revisited” scheme. It provides:

- `aes-core`: baseline AES-128 with key expansion, encrypt/decrypt, and NIST vector tests.
- `wbaes-gen`: generator for 2×AES white-box instances using sparse unsplit 256-bit affine encodings and per-round 16→256-bit tables.
- `wbaes-runtime`: evaluator that executes the generated tables with external encodings.
- `wbaes-cli`: command-line tool to generate instances, encrypt/decrypt blocks, and verify correctness.

The binding design document is `docs/design.md` and `docs/whitebox_background.md` for narrative context.

## Quick start

```bash
cargo run -p wbaes-cli -- gen \
  --key-hex 000102030405060708090a0b0c0d0e0f \
  --out wb.bin

cargo run -p wbaes-cli -- enc --instance wb.bin --in plain.bin --out ct.bin

# Quick demo: generate random key/instance, encrypt random 32B block, and decrypt back
cargo run -p wbaes-cli -- demo
```

`dec` expects instances with no external output encoding (the default): it decrypts with AES-core. `check` compares white-box outputs against two AES encryptions for random samples.

## Minimum Rust version

Stable Rust 1.75+ (edition 2021). CI enforces fmt/clippy/test on stable.

## Security disclaimer

Research and educational code only. Classic CEJO/Chow-style and revisited white-box AES constructions are vulnerable to algebraic and DCA-style attacks (e.g., BGE, Baek–Cheon–Hong toolbox). No side-channel hardening is provided. Do not use for production key protection.
