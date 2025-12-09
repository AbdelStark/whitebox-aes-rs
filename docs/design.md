# whitebox-aes-rs design notes

This document summarizes how the Baek–Cheon–Hong revisited white-box AES scheme maps to the Rust implementation in this repository. The authoritative implementation roadmap remains `.ai/plan.md`.

## Goals

- Provide a clean AES-128 core as ground truth.
- Generate 256-bit white-box instances (two AES blocks) using sparse unsplit affine encodings and per-round 16→256-bit tables.
- Evaluate instances efficiently via table lookups and XORs.
- Keep APIs explicit and testable; no implicit randomness.

## Data types

- AES core: `Block = [u8; 16]`, `Aes128Key([u8; 16])`, `RoundKeys([Block; 11])`.
- Linear algebra (`wbaes-gen`):
  - `Matrix8`, `Matrix128`, `Matrix256` over GF(2) with inversion and application to byte slices.
  - `Affine8`, `Affine256` with `apply`, `invert`, `compose`. `Affine256::random_sparse_unsplit` builds the banded structure from the revisited scheme (non-zero blocks on diagonal, super-diagonal, wrap).
- Tables:
  - `Table16x256` holds 2^16 entries × 32 bytes as a contiguous `Box<[u8]>`.
  - `RoundTables` is an array of 32 tables.
  - `HTable` provides random masks `h_i: u8 → 256-bit`.
- Instance:
  - `WbInstance256` includes `[RoundTables; 10]`, `ExternalEncodings` (input + optional output), and `InstanceParams` metadata. Serialized via `serde` + `bincode`.

## Round construction (generator)

For each round `r`:

- Precompute `A^(r)` sparse unsplit affine encodings.
- Derive `B_lin^(r) = (A_lin^(r+1))^{-1} * L_r`, where `L_r` is `MC∘SR` for rounds 1–9 and `SR` for round 10. The affine bias of `B` is chosen so that, after applying `(A^(r+1))^{-1}`, it incorporates the next affine bias and the round key contribution.
- Split the bias into per-byte slices `b_i` whose XOR equals the target bias.
- Extract per-byte maps `B_i: u8 → 256-bit` from `B_lin^(r)`.
- Generate random `h_i` masks.
- For each table index `i` and inputs `(x, y)`, compute:

  ```
  z = A_{i,i} * x ⊕ A_{i,i+1 or wrap} * y ⊕ a_i
  t = S(z)                      // key is already folded into bias via duplication
  u = B_i(t) ⊕ b_i              // 256-bit
  v = u ⊕ h_i(x) ⊕ h_{i+1}(y)
  T_i(x, y) = v
  ```

- Initial key whitening: the AES round-0 key is duplicated across both 16-byte halves and folded into the input encoding (`Min ∘ ARK ∘ (A^(1))^{-1}`).

## Runtime evaluation

- `WbCipher256` applies input external encoding, then iterates over 10 rounds:

  ```
  acc = 0
  for i in 0..32:
      entry = T_i[state[i], state[i+1 mod 32]]
      acc ^= entry
  state = acc
  ```

- Optional output encoding is applied if present (default instances fold it into round 10 tables).
- Convenience `encrypt_pair` packs two 16-byte blocks.

## CLI behavior

- `gen`: produce instance from key (hex), optional seed, optional external encodings (off by default to simplify checks/decrypt).
- `enc`: encrypt 32-byte-block multiples with a serialized instance.
- `dec`: AES-core decryption assuming no external output encoding (debug/demo only).
- `check`: compares runtime encryption to two AES encryptions for random samples.

## Testing strategy

- AES core: NIST vectors and random round trips.
- `wbaes-gen`: matrix/affine inversion and composition, sparsity checks, linear layer equivalence, table/instance serialization.
- Runtime: equality to AES for random inputs when external encodings are neutral/absorbed.
- CLI: smoke tested indirectly via library tests; integration harness can be added later with `assert_cmd`.

## Threat model and limitations

- White-box attacker with full table visibility and ability to instrument execution.
- Scheme is subject to known analytic attacks (BGE, Baek–Cheon–Hong toolbox) and DCA. No side-channel hardening is attempted.
- External encodings are optional and primarily for experimentation; defaults favor easier correctness checks.

## Performance notes

- Tables are contiguous for cache-friendly lookups.
- Generation is single-threaded and deterministic under a seeded RNG; a future `parallel-gen` feature could parallelize per-table builds while keeping reproducibility via derived seeds.
- Runtime is allocation-free after instance load.
