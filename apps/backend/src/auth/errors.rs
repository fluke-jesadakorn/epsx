use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Email sign-up failed: {message}")]
    EmailSignUpFailed {
        message: String,
        details: Option<Value>,
    },

    #[error("Email sign-in failed: {message}")]
    EmailSignInFailed {
        message: String,
        details: Option<Value>,
    },
}

#[allow(dead_code)]
impl AuthError {
    pub fn internal<T: ToString>(message: T) -> Self {
        Self::Internal(message.to_string())
    }

    pub fn unauthorized(message: &str) -> Self {
        Self::Unauthorized(message.to_string())
    }

    pub fn bad_request(message: &str) -> Self {
        Self::BadRequest(message.to_string())
    }

    pub fn session_expired() -> Self {
        Self::SessionExpired
    }

    pub fn email_sign_up_failed(message: &str, details: Option<Value>) -> Self {
        Self::EmailSignUpFailed {
            message: message.to_string(),
            details,
        }
    }

    pub fn email_sign_in_failed(message: &str, details: Option<Value>) -> Self {
        Self::EmailSignInFailed {
            message: message.to_string(),
            details,
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::SessionExpired => StatusCode::UNAUTHORIZED,
            Self::EmailSignUpFailed { .. } => StatusCode::BAD_REQUEST,
            Self::EmailSignInFailed { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> Value {
        match self {
            Self::Internal(message) => json!({
                "code": "INTERNAL_ERROR",
                "message": message
            }),
            Self::Unauthorized(message) => json!({
                "code": "UNAUTHORIZED",
                "message": message
            }),
            Self::BadRequest(message) => json!({
                "code": "BAD_REQUEST",
                "message": message
            }),
            Self::SessionExpired => json!({
                "code": "SESSION_EXPIRED",
                "message": "Session has expired"
            }),
            Self::EmailSignUpFailed { message, details } => json!({
                "code": "EMAIL_SIGN_UP_FAILED",
                "message": message,
                "details": details
            }),
            Self::EmailSignInFailed { message, details } => json!({
                "code": "EMAIL_SIGN_IN_FAILED",
                "message": message,
                "details": details
            }),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.error_response());
        
        (status, body).into_response()
    }
}
