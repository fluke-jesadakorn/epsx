// Validation types and error responses
use std::collections::HashMap;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use validator::ValidationErrors;

/// Validation error response
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub error: String,
    pub message: String,
    pub fields: HashMap<String, Vec<String>>,
    pub code: u16,
}

impl ValidationErrorResponse {
    pub fn new(message: String, fields: HashMap<String, Vec<String>>) -> Self {
        Self {
            error: "validation_error".to_string(),
            message,
            fields,
            code: 400,
        }
    }

    pub fn single_field(field: String, message: String) -> Self {
        let mut fields = HashMap::new();
        fields.insert(field, vec![message]);
        Self::new("Input validation failed".to_string(), fields)
    }

    pub fn from_validation_errors(errors: ValidationErrors) -> Self {
        let mut fields = HashMap::new();

        for (field, field_errors) in errors.field_errors() {
            let messages: Vec<String> = field_errors
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("Invalid {}", field))
                })
                .collect();
            fields.insert(field.to_string(), messages);
        }

        Self::new("Input validation failed".to_string(), fields)
    }
}

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> Response {
        let status = StatusCode::BAD_REQUEST;
        (status, Json(self)).into_response()
    }
}

/// Custom validation result type
pub type ValidationResult<T> = Result<T, ValidationErrorResponse>;
