use axum::{ response::{ IntoResponse, Response }, http::StatusCode };

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("Invalid token: {0}")] InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Initialization failed: {0}")] InitializationFailed(String),
    #[error("Database error: {0}")] DatabaseError(#[from] mongodb::error::Error),
    #[error("Internal server error: {0}")] InternalError(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidToken(..) | Self::TokenExpired => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::FORBIDDEN,
            Self::DatabaseError(_) | Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InitializationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        tracing::error!("Auth error: {}", self);
        (status, self.to_string()).into_response()
    }
}
