//! Materials-Simulato-R Core Library
//!
//! This crate provides the core types, traits, and utilities used throughout
//! the Materials-Simulato-R platform.

pub mod error;
pub mod material;
pub mod property;
pub mod config;
pub mod auto_optimizer;
pub mod feature_flags;
pub mod lirs;

// 🧠 Advanced Intelligence Modules
pub mod embeddings;
pub mod ml_predictor;
pub mod knowledge_graph;
pub mod discovery;
pub mod recommendations;

// 💊 Drug Discovery Module (re-exports from drugs-core and drugs-molecular)
pub mod drugs {
    //! Drug discovery and pharmaceutical materials
    //!
    //! This module integrates Drugs-Simulato-R for AI-powered drug discovery.
    //! Provides 100-1000x performance improvement over Python RDKit.

    pub use drugs_core::{Molecule, Compound, Target};
    pub use drugs_molecular::{
        MolecularDescriptors,
        Fingerprint,
        FingerprintType,
        tanimoto_similarity,
        dice_similarity,
    };
}

// 🌀 Chaotic Attractor Compression (Memory Optimization)
pub mod compression {
    //! Chaotic attractor-based compression for vector embeddings
    //!
    //! Achieves 100-1000x compression ratios for high-dimensional vectors.
    //! Based on PP25-CHAOTIC_ATTRACTOR_COMPRESSION research.

    pub use materials_chaos_compression::attractor_compress;
    pub use materials_chaos_compression::attractor_decompress;
}

// 🧬 Graph Neural Networks
pub mod gnn;

// 🗣️ Natural Language Interface
pub mod nli;

// 🎨 3D Visualization Engine
pub mod viz3d;

// ⚛️ Quantum Chemistry DFT Integration
pub mod quantum;
pub mod quantum_lirs;

// 🖥️ REPL - Interactive Shell
pub mod repl;

// 🔬 High-Throughput Screening
pub mod hts;

// 🔷 Advanced Crystallography
pub mod crystallography;

pub use error::{Error, Result};
pub use material::Material;
pub use property::Property;
pub use config::Config;

/// Version of the Materials-Simulato-R platform
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum Supported Rust Version
pub const MSRV: &str = "1.75.0";
