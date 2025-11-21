//! Authentication middleware for Axum

use crate::{jwt::{Claims, JWTService}, Error, Result, Role};
use axum::{
    extract::{Request, Extension},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::sync::Arc;

/// Extract JWT claims from request
pub async fn auth_middleware(
    Extension(jwt_service): Extension<Arc<JWTService>>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, AuthError> {
    let token = extract_token(&request)?;

    let claims = jwt_service
        .verify_token(token)
        .map_err(|_| AuthError::InvalidToken)?;

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Middleware that requires a specific role
pub fn require_role(required_role: Role) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, AuthError>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let role = required_role;
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or(AuthError::MissingCredentials)?;

            if !role_has_permission(&claims.role, &role) {
                return Err(AuthError::InsufficientPermissions);
            }

            Ok(next.run(request).await)
        })
    }
}

/// Check if a role has permission for another role
fn role_has_permission(user_role: &Role, required_role: &Role) -> bool {
    match (user_role, required_role) {
        (Role::Admin, _) => true, // Admin can access everything
        (Role::Researcher, Role::Researcher) => true,
        (Role::Researcher, Role::ReadOnly) => true,
        (Role::ReadOnly, Role::ReadOnly) => true,
        _ => false,
    }
}

/// Extract Bearer token from Authorization header
fn extract_token(request: &Request) -> std::result::Result<&str, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingCredentials)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidToken);
    }

    Ok(&auth_header[7..])
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    InvalidToken,
    InsufficientPermissions,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingCredentials => (StatusCode::UNAUTHORIZED, "Missing credentials"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InsufficientPermissions => (StatusCode::FORBIDDEN, "Insufficient permissions"),
        };

        (status, message).into_response()
    }
}
