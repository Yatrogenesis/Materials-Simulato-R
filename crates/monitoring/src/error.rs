//! Monitoring error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Metrics error: {0}")]
    Metrics(String),

    #[error("Tracing error: {0}")]
    Tracing(String),

    #[error("{0}")]
    Other(String),
}
