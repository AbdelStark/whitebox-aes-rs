# White-box AES background (brief)

This project implements the Baek–Cheon–Hong “White-Box AES Implementation Revisited” scheme, building on the CEJO/Chow framework for white-box AES. Key background points:

- **CEJO / Chow-style designs** (SAC 2002): rewrite AES as key-dependent lookup tables (T-boxes, Ty tables) with per-table input/output encodings and mixing bijections. External encodings hide plaintext/ciphertext domains.
- **Analytic breaks**: Billet–Gilbert–Ech-Chatbi’s algebraic attack (BGE), Michiels–Gorissen–Hollmann’s CEJO analysis, and later differential computation analysis (DCA) show that classic designs are weak. Baek–Cheon–Hong quantify generic attack complexity in terms of block size and encoding sizes.
- **Revisited scheme (JCN 2016)**: processes two AES-128 blocks together (256-bit state) and uses unsplit 256-bit affine encodings with a sparse banded structure, raising generic attack cost while keeping table-based evaluation (32 tables per round).

Threat model and caveats:

- White-box attacker with full visibility of code, tables, and ability to instrument execution.
- No side-channel hardening is provided; DCA-style attacks remain applicable.
- External encodings are optional and for experimentation; defaults favor correctness testing.

References:
- S. Chow et al., “White-Box Cryptography and an AES Implementation,” SAC 2002.
- J. A. Muir, “A Tutorial on White-box AES,” 2013.
- C. H. Baek, J. H. Cheon, H. Hong, “White-Box AES Implementation Revisited,” JCN 2016 (ePrint 2014/688).
