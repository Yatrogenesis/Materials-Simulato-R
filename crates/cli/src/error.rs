//! CLI error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Command error: {0}")]
    Command(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}
