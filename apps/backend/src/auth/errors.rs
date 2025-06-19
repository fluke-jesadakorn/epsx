use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Database error: {0}")]
    DatabaseError(#[from] mongodb::error::Error),
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidToken | Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::DatabaseError(_) | Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        tracing::error!("Auth error: {}", self);
        (status, self.to_string()).into_response()
    }
}
