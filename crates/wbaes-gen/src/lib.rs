//! White-box AES instance generation primitives.
//!
//! This crate provides the linear algebra and affine encoding building blocks
//! required to construct Baek–Cheon–Hong’s revisited white-box AES scheme. It
//! focuses on GF(2) matrices, affine maps, and helpers that will later be
//! composed into full round encodings and lookup tables. The goal is fidelity
//! to the published scheme for research and education; it is not hardened
//! against side-channel or DCA-style attacks.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod affine;
mod generator;
mod instance;
mod linear;
mod matrix;
mod tables;

pub use affine::{Affine256, Affine8};
pub use generator::{Generator, GeneratorConfig};
pub use instance::{ExternalEncodings, InstanceParams, SchemeId, WbInstance256};
pub use linear::{mc_sr_matrix_128, mc_sr_matrix_256, sr_matrix_128, sr_matrix_256};
pub use matrix::{Matrix128, Matrix256, Matrix8};
pub use tables::{RoundTables, Table16x256};
