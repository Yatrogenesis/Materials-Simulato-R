//! Drugs-Simulato-R Core Library
//!
//! Core types, traits, and utilities for drug discovery simulation
//!
//! Author: Francisco Molina Burgos

pub mod molecule;
pub mod compound;
pub mod target;
pub mod error;

pub use error::{Error, Result};
pub use molecule::Molecule;
pub use compound::Compound;
pub use target::Target;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
