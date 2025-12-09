//! AES-128 reference implementation used by the white-box generator and runtime.
//! The initial version is a scaffold; full cipher logic will be added in later phases.

#![forbid(unsafe_code)]

/// Marker module for the upcoming AES-128 implementation.
pub mod placeholder {
    /// Placeholder function to confirm the crate builds; real cipher code will replace this.
    pub fn hello() -> &'static str {
        "aes-core scaffold"
    }
}
