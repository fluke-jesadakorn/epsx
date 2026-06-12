// EPS-Specific Error Handling
// Focused module handling error conversion and EPS-specific error responses

use axum::{http::StatusCode, response::Json};
use tracing::{warn, error};

use crate::core::errors::AppError as KernelAppError;

/// Local wrapper around the kernel `AppError` so we can implement
/// `From<EPSError>` and the `IntoResponse` shape without tripping the
/// orphan rule (both `EPSError` and the kernel `AppError` are foreign
/// to each other now that `AppError` lives in `epsx-contracts`).
///
/// All public helpers in this module return `AppError` (the local
/// newtype) — `.0` and `Deref` give ergonomic access to the inner
/// kernel value, and the free function `to_axum_response` produces
/// the `(StatusCode, Json<Value>)` pair the old orphan impl produced.
pub struct AppError(pub KernelAppError);

impl std::ops::Deref for AppError {
    type Target = KernelAppError;
    fn deref(&self) -> &KernelAppError { &self.0 }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<KernelAppError> for AppError {
    fn from(inner: KernelAppError) -> Self {
        Self(inner)
    }
}

/// Convert an `AppError` to the `(StatusCode, Json<Value>)` pair the
/// EPS handlers used to get via the orphan `From` impl. Lives here
/// (not as an impl) because the target type is a tuple of foreign
/// types and can't host an impl block.
pub fn to_axum_response(error: &AppError) -> (StatusCode, Json<serde_json::Value>) {
    use crate::core::errors::ErrorKind;

    match error.kind {
        ErrorKind::ValidationError => {
            warn!("Validation error in EPS API: {}", error.message);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "validation_error",
                    "message": error.message,
                    "code": "EPS_VALIDATION_FAILED"
                }))
            )
        }
        ErrorKind::DatabaseError => {
            error!("Database error in EPS API: {}", error.message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "database_error",
                    "message": "Internal server error",
                    "code": "EPS_DATABASE_ERROR"
                }))
            )
        }
        ErrorKind::ExternalServiceError => {
            warn!("External service error in EPS API: {}", error.message);
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "external_service_error",
                    "message": "External service temporarily unavailable",
                    "code": "EPS_EXTERNAL_SERVICE_ERROR"
                }))
            )
        }
        ErrorKind::ConfigurationError => {
            error!("Configuration error in EPS API: {}", error.message);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "configuration_error",
                    "message": "Service configuration error",
                    "code": "EPS_CONFIGURATION_ERROR"
                }))
            )
        }
        _ => {
            error!("Unexpected error in EPS API: {:?}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "internal_error",
                    "message": "An unexpected error occurred",
                    "code": "EPS_INTERNAL_ERROR"
                }))
            )
        }
    }
}

/// EPS-specific error types
#[derive(Debug, thiserror::Error)]
pub enum EPSError {
    #[error("Invalid EPS value: {0}")]
    InvalidEPSValue(f64),

    #[error("Country not supported: {0}")]
    CountryNotSupported(String),

    #[error("Sector not found for country: {country}")]
    SectorNotFound { country: String },

    #[error("WebSocket enhancement failed: {0}")]
    WebSocketEnhancementFailed(String),

    #[error("Cache operation failed: {0}")]
    CacheOperationFailed(String),

    #[error("Data transformation failed: {0}")]
    DataTransformationFailed(String),
}

impl From<EPSError> for AppError {
    fn from(error: EPSError) -> Self {
        use crate::core::errors::ErrorKind;

        let kind = match error {
            EPSError::InvalidEPSValue(_) => ErrorKind::ValidationError,
            EPSError::CountryNotSupported(_) => ErrorKind::ValidationError,
            EPSError::SectorNotFound { .. } => ErrorKind::ValidationError,
            EPSError::WebSocketEnhancementFailed(_) => ErrorKind::ExternalServiceError,
            EPSError::CacheOperationFailed(_) => ErrorKind::DatabaseError,
            EPSError::DataTransformationFailed(_) => ErrorKind::ValidationError,
        };
        AppError(crate::core::errors::AppError::new(kind, error.to_string()))
    }
}

/// Helper functions for error creation
pub fn invalid_eps_error(eps: f64) -> AppError {
    EPSError::InvalidEPSValue(eps).into()
}

pub fn country_not_supported_error(country: String) -> AppError {
    EPSError::CountryNotSupported(country).into()
}

pub fn websocket_enhancement_error(message: String) -> AppError {
    EPSError::WebSocketEnhancementFailed(message).into()
}

pub fn cache_operation_error(message: String) -> AppError {
    EPSError::CacheOperationFailed(message).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::ErrorKind;

    #[test]
    fn test_eps_error_conversion() {
        let eps_error = EPSError::InvalidEPSValue(-1.0);
        let app_error: AppError = eps_error.into();

        assert!(matches!(app_error.0.kind, ErrorKind::ValidationError));
        assert!(app_error.0.message.contains("Invalid EPS value"));
    }

    #[test]
    fn test_helper_functions() {
        let error = invalid_eps_error(0.0);
        assert!(matches!(error.0.kind, ErrorKind::ValidationError));

        let error = country_not_supported_error("invalid".to_string());
        assert!(matches!(error.0.kind, ErrorKind::ValidationError));

        let error = websocket_enhancement_error("connection failed".to_string());
        assert!(matches!(error.0.kind, ErrorKind::ExternalServiceError));
    }

    #[test]
    fn test_error_response_conversion() {
        let app_error = AppError(crate::core::errors::AppError::new(
            ErrorKind::ValidationError,
            "Test validation error".to_string(),
        ));
        let (status, json_response) = to_axum_response(&app_error);

        assert_eq!(status, StatusCode::BAD_REQUEST);

        let response_value = json_response.0;
        assert_eq!(response_value["error"], "validation_error");
        assert_eq!(response_value["code"], "EPS_VALIDATION_FAILED");
    }
}