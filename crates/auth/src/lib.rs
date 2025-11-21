//! Materials-Simulato-R Authentication & Authorization
//!
//! Provides:
//! - JWT-based authentication
//! - RBAC (Role-Based Access Control)
//! - Multi-tenant support

#![allow(dead_code, unused_imports)]

pub mod jwt;
pub mod rbac;
pub mod multi_tenant;
pub mod error;

pub use error::{Error, Result};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    Admin,
    Researcher,
    ReadOnly,
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
