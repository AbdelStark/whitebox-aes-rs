# whitebox-aes-rs

Experimental Rust workspace implementing Baek–Cheon–Hong’s “White-Box AES Implementation Revisited” scheme. The project will expose a clean AES-128 core, a white-box instance generator, a runtime evaluator, and a CLI for experimentation.

## Status

Scaffolding is in place; implementation follows the detailed roadmap in `.ai/plan.md`.

## Design source

The binding design document is `.ai/plan.md`. All crate structure, APIs, data types, and testing guidance come from that plan. Additional design notes will live in `docs/` as the implementation progresses.

## Minimum Rust version

Targeting stable Rust 1.75+ (edition 2021). CI will enforce MSRV once the code matures.

## Security disclaimer

This is research and educational code. Classic CEJO/Chow-style white-box AES schemes, including the revisited variant implemented here, are vulnerable to generic algebraic and DCA-style attacks. Do not use this as a secure key-protection mechanism.
