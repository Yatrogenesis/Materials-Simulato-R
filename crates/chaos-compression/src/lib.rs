//! Compression Experiment Library
//!
//! Métodos de compresión y datasets para evaluar compresión de vectores ML

pub mod methods;
pub mod datasets;
pub mod attractor_analysis;

// Re-export for convenience
pub use methods::*;
pub use datasets::*;
pub use attractor_analysis::*;
