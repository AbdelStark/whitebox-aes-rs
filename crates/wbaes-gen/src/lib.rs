//! White-box AES instance generation primitives.
//!
//! This crate provides the linear algebra and affine encoding building blocks
//! required to construct Baek–Cheon–Hong’s revisited white-box AES scheme. It
//! focuses on GF(2) matrices, affine maps, and helpers that will later be
//! composed into full round encodings and lookup tables.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod affine;
mod matrix;

pub use affine::{Affine256, Affine8};
pub use matrix::{Matrix256, Matrix8};
