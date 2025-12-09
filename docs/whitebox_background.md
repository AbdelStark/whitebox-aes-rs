# White-box AES background (brief)

This project implements the Baek–Cheon–Hong “White-Box AES Implementation Revisited” scheme, building on the CEJO/Chow framework. Key points:

## CEJO / Chow-style designs (SAC 2002)
- AES is rewritten as key-dependent lookup tables (T-boxes, Ty tables) with per-table input/output encodings and mixing bijections.
- External encodings conceal the plaintext/ciphertext domains but keep functional equivalence.

## Attack landscape
- **BGE algebraic attack** (Billet–Gilbert–Ech-Chatbi): strips encodings and recovers keys for classic Chow constructions.
- **Michiels–Gorissen–Hollmann CEJO analysis**: generic approach to affine equivalence in CEJO designs.
- **DCA-style attacks**: differential computation analysis exploits side channels in software execution traces.
- **Baek–Cheon–Hong toolbox**: bounds attack complexity based on block size `n`, S-box size `m`, and encoding block size `m_A`; Chow-style with 8/16-bit encodings is weak, prompting the 256-bit revisited scheme.

## Revisited scheme (JCN 2016)
- Processes two AES-128 blocks together (256-bit state).
- Uses **unsplit 256-bit affine encodings** with a sparse banded structure (diagonal, super-diagonal, wrap).
- Each round is the XOR of 32 tables `T_i: (u8,u8)→256-bit` with random masks `h_i`, increasing generic attack cost while remaining table-evaluable.

## Threat model and caveats
- White-box attacker: full visibility of code and tables, ability to instrument execution.
- No side-channel hardening; DCA remains applicable.
- External encodings are optional and default off to simplify correctness testing; when enabled, they only raise effort modestly.
- This code is for research/education only; do not deploy for real-world key protection.

## References
- S. Chow et al., “White-Box Cryptography and an AES Implementation,” SAC 2002.
- J. A. Muir, “A Tutorial on White-box AES,” 2013.
- C. H. Baek, J. H. Cheon, H. Hong, “White-Box AES Implementation Revisited,” JCN 2016 (ePrint 2014/688).
