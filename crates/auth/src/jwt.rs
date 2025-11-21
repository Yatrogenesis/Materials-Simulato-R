//! JWT token handling

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,  // User ID
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

pub struct JWTService {
    secret: String,
}

impl JWTService {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    pub fn create_token(&self, _user_id: Uuid) -> Result<String> {
        // TODO: Implement with jsonwebtoken
        Err(Error::Other("Not yet implemented".to_string()))
    }

    pub fn verify_token(&self, _token: &str) -> Result<Claims> {
        // TODO: Implement with jsonwebtoken
        Err(Error::Other("Not yet implemented".to_string()))
    }
}
