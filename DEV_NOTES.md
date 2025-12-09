# Dev Notes

## 2025-12-09 11:12 UTC
- Read `.ai/plan.md` completely to internalize the architecture and phased roadmap.
- Key understanding: Cargo workspace with `aes-core`, `wbaes-gen`, `wbaes-runtime`, `wbaes-cli`; AES-128 reference core feeds generator; generator builds 10-round 2×AES white-box instance using sparse unsplit 256-bit affine encodings, GF(2) matrices, and 32 per-round 16→256-bit tables; runtime evaluates tables with optional external encodings; CLI drives generation/encryption/decryption/checking; documentation and CI to follow.
- Testing expectations: AES vectors and property tests in `aes-core`, matrix/encoding/table correctness and WB-vs-AES equivalence in `wbaes-gen`/`wbaes-runtime`, CLI smoke and serialization round-trips, benches later.
- Security posture: research/educational only; CEJO/BGE/DCA class attacks apply; no side-channel hardening beyond reasonable constant-time structure.
- Next: Phase 1 — set up workspace scaffolding and crate skeletons per the plan.

## 2025-12-09 11:30 UTC
- Created Cargo workspace with four crates (`aes-core`, `wbaes-gen`, `wbaes-runtime`, `wbaes-cli`) and added scaffold lib/main files with unsafe forbidden.
- Added root README stub referencing `.ai/plan.md` and MSRV intent.
- Ran `cargo fmt`, `cargo clippy --all-targets --all-features --workspace -- -D warnings`, and `cargo test --workspace` (all passing).
- Next: implement AES-128 core (key schedule and encrypt) with NIST vectors and property tests.

## 2025-12-09 11:34 UTC
- Implemented full AES-128 core: key expansion, encrypt, decrypt, round transforms, and S-box/inverse tables with public types (`Block`, `Aes128Key`, `RoundKeys`).
- Added NIST FIPS-197 test vector coverage and random encrypt/decrypt roundtrip tests.
- Clippy/tests passing across workspace.
- Next: build GF(2) matrix and affine encoding infrastructure in `wbaes-gen` (Phase 3).

## 2025-12-09 11:42 UTC
- Built GF(2) linear algebra for 8×8 and 256×256 matrices with inversion, multiplication, application, sparse unsplit generation, and byte-to-256 maps.
- Added affine maps (8-bit and 256-bit) with compose/invert/apply helpers and random sparse-unsplit generation.
- Comprehensive tests for invertibility, roundtrip correctness, sparsity structure, and composition; workspace fmt/clippy/test all green.
- Next: implement AES linear layers and round encoding scaffolding in `wbaes-gen` (Phase 4).

## 2025-12-09 11:50 UTC
- Implemented AES linear layer matrices: `MC ∘ SR` for 128-bit and 256-bit states, built via matrix builders from AES core round functions.
- Added Matrix128 type plus from-linear-transform helpers for 128/256; verified matrix application matches direct AES shift+mix on random states.
- `wbaes-gen` now depends on `aes-core`; fmt/clippy/tests passing.
- Next: white-box table generation pipeline, instance struct, and serialization (Phases 5–6).

## 2025-12-09 12:09 UTC
- Added table representations (`Table16x256`, `RoundTables`, random `HTable`) and full generator pipeline to build 10 rounds from AES keys, sparse encodings, and masking tables.
- Implemented instance structs with parameters, external encodings, and bincode serialization; added serde support to affine/matrix types.
- Generator produces instances with external output encoding folded into round 10; configuration flag controls external encodings (defaults off). Workspace fmt/clippy/tests pass.
- Next: runtime evaluator and CLI for end-to-end encryption/decryption (Phases 7–8).

## 2025-12-09 12:38 UTC
- Implemented runtime evaluator executing round tables with external input encoding and validated outputs against AES for random samples.
- Built CLI with `gen`, `enc`, `dec` (AES-core decrypt, no external output encoding), and `check` commands; supports deterministic seeds.
- Added CLI/runtime dependencies and passing workspace fmt/clippy/tests (generation-heavy tests included).
- Next: documentation, examples, CI, and benches per later phases.

## 2025-12-09 12:49 UTC
- Expanded documentation: README quick start and disclaimers, design overview in `docs/design.md`, background in `docs/whitebox_background.md`.
- Added example (`examples/basic.rs`), Criterion benches scaffold, and GitHub Actions CI (fmt/clippy/test on stable).
- Benchmarks wired via workspace dependency; fmt/clippy/tests all green post-additions.
- Next: future work could add parallel generation feature, richer CLI integration tests, and publish docs.
