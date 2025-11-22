//! Error types for Drugs-Simulato-R

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid SMILES: {0}")]
    InvalidSmiles(String),

    #[error("Invalid molecule: {0}")]
    InvalidMolecule(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
