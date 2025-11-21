//! JWT token handling

use crate::{Error, Result, Role};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,    // User ID (as string for JWT compatibility)
    pub email: String,  // Email
    pub role: Role,     // User role
    pub exp: usize,     // Expiration time
    pub iat: usize,     // Issued at
    pub nbf: usize,     // Not before
}

impl Claims {
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub)
            .map_err(|e| Error::Other(format!("Invalid user ID in token: {}", e)))
    }
}

pub struct JWTService {
    secret: Vec<u8>,
    expiration_seconds: u64,
}

impl JWTService {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into().into_bytes(),
            expiration_seconds: 3600 * 24, // 24 hours default
        }
    }

    pub fn with_expiration(mut self, seconds: u64) -> Self {
        self.expiration_seconds = seconds;
        self
    }

    pub fn create_token(&self, user_id: Uuid, email: impl Into<String>, role: Role) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Other(format!("System time error: {}", e)))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.into(),
            role,
            exp: now + self.expiration_seconds as usize,
            iat: now,
            nbf: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret)
        )
        .map_err(|e| Error::Other(format!("Failed to encode JWT: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);

        decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &validation
        )
        .map(|data| data.claims)
        .map_err(|e| Error::Other(format!("Failed to decode JWT: {}", e)))
    }

    pub fn refresh_token(&self, old_token: &str) -> Result<String> {
        let claims = self.verify_token(old_token)?;
        self.create_token(claims.user_id()?, claims.email, claims.role)
    }
}

/// Password hashing utilities
pub mod password {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };

    use crate::{Error, Result};

    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| Error::Other(format!("Failed to hash password: {}", e)))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| Error::Other(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
