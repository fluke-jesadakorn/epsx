use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use super::{AuthService, AuthError};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(verify_token),
    components(
        schemas(LoginRequest, LoginResponse)
    ),
    tags(
        (name = "Authentication", description = "Authentication endpoints")
    )
)]
#[allow(dead_code)]
struct AuthApi;

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "token": "firebase.jwt.token"
}))]
#[allow(dead_code)]
pub struct LoginRequest {
    token: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "user_id": "user123",
    "email": "user@example.com",
    "roles": ["user"]
}))]
pub struct LoginResponse {
    user_id: String,
    email: Option<String>,
    roles: Vec<String>,
}

/// Verify Firebase JWT token and return user information
#[utoipa::path(
    post,
    path = "/verify",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully verified token", body = LoginResponse),
        (status = 401, description = "Invalid token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Authentication"
)]
pub async fn verify_token(
    State(_state): State<Arc<AuthService>>,
    Json(_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // In real implementation, this would validate the token with Firebase
    // For now, just return mock data
    Ok(Json(LoginResponse {
        user_id: "mock_user_id".to_string(),
        email: Some("mock@example.com".to_string()),
        roles: vec!["user".to_string()],
    }))
}
