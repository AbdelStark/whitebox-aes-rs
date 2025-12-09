# Dev Notes

## 2025-12-09 11:12 UTC
- Read `.ai/plan.md` completely to internalize the architecture and phased roadmap.
- Key understanding: Cargo workspace with `aes-core`, `wbaes-gen`, `wbaes-runtime`, `wbaes-cli`; AES-128 reference core feeds generator; generator builds 10-round 2×AES white-box instance using sparse unsplit 256-bit affine encodings, GF(2) matrices, and 32 per-round 16→256-bit tables; runtime evaluates tables with optional external encodings; CLI drives generation/encryption/decryption/checking; documentation and CI to follow.
- Testing expectations: AES vectors and property tests in `aes-core`, matrix/encoding/table correctness and WB-vs-AES equivalence in `wbaes-gen`/`wbaes-runtime`, CLI smoke and serialization round-trips, benches later.
- Security posture: research/educational only; CEJO/BGE/DCA class attacks apply; no side-channel hardening beyond reasonable constant-time structure.
- Next: Phase 1 — set up workspace scaffolding and crate skeletons per the plan.
