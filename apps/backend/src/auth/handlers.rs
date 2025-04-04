use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::auth::{AuthService};

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "token": "firebase.auth.token"
}))]
pub struct SignInRequest {
    token: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SignInResponse {
    roles: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "success": true,
    "message": "Authentication successful",
    "roles": ["user"]
}))]
pub struct AuthResponse {
    success: bool,
    message: String,
    roles: Vec<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "message": "Protected endpoint response"
}))]
pub struct ProtectedResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/auth/session/validate",
    responses(
        (status = 200, description = "Session validated successfully", body = SignInResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn session_validate(
    State(_state): State<AuthService>,
) -> Result<Json<SignInResponse>, StatusCode> {
    Ok(Json(SignInResponse {
        roles: vec!["user".to_string()],
    }))
}

#[utoipa::path(
    get,
    path = "/auth/protected",
    responses(
        (status = 200, description = "Protected endpoint accessed", body = ProtectedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn protected_example() -> Result<Json<ProtectedResponse>, StatusCode> {
    Ok(Json(ProtectedResponse {
        message: "This is a protected endpoint".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/auth/admin-only",
    responses(
        (status = 200, description = "Admin endpoint accessed", body = ProtectedResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn admin_only_example() -> Result<Json<ProtectedResponse>, StatusCode> {
    Ok(Json(ProtectedResponse {
        message: "This is an admin-only endpoint".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/auth/sign-in",
    request_body = SignInRequest,
    responses(
        (status = 200, description = "Sign in successful", body = SignInResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn sign_in(
    State(auth_service): State<AuthService>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, StatusCode> {
    auth_service
        .validate_session(&payload.token)
        .await
        .map(|auth_user| Json(SignInResponse {
            roles: auth_user.roles.iter().map(|r| r.to_string()).collect(),
        }))
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

#[utoipa::path(
    get,
    path = "/auth/sign-out",
    responses(
        (status = 200, description = "Sign out successful", body = Object),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn sign_out() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({ "success": true })))
}
