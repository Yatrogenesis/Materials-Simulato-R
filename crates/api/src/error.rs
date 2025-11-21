//! API error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Database error: {0}")]
    Database(#[from] materials_database::Error),

    #[error("Auth error: {0}")]
    Auth(#[from] materials_auth::Error),

    #[error("{0}")]
    Other(String),
}
