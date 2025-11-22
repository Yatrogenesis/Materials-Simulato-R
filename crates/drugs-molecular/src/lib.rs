//! Molecular descriptors and properties
//!
//! 200+ molecular descriptors for QSAR/QSPR modeling
//! Based on RDKit functionality but optimized in Rust

pub mod descriptors;
pub mod fingerprints;
pub mod similarity;

pub use descriptors::MolecularDescriptors;
pub use fingerprints::{Fingerprint, FingerprintType};
pub use similarity::{tanimoto_similarity, dice_similarity};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
