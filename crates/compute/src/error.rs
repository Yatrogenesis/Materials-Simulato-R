//! Computation error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ML error: {0}")]
    ML(String),

    #[error("MD error: {0}")]
    MD(String),

    #[error("DFT error: {0}")]
    DFT(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Computation failed: {0}")]
    ComputationFailed(String),

    #[error("Core error: {0}")]
    Core(#[from] materials_core::Error),

    #[error("{0}")]
    Other(String),
}
