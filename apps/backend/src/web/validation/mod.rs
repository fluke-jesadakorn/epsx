use std::collections::HashMap;// Comprehensive input validation for all HTTP handlers
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use validator::{Validate, ValidationErrors};

pub mod validators;
pub mod middleware;
pub mod request_dtos;

// Integration tests removed - using external E2E test suite

pub use validators::*;
pub use middleware::*;
pub use request_dtos::*;

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

/// Validate a request payload and return validation errors if any
pub fn validate_request<T: Validate>(payload: &T) -> ValidationResult<()> {
    match payload.validate() {
        Ok(_) => Ok(()),
        Err(validation_errors) => {
            tracing::warn!("Validation failed: {:?}", validation_errors);
            Err(ValidationErrorResponse::from_validation_errors(validation_errors))
        }
    }
}

/// Validate multiple request payloads
pub fn validate_multiple<T: Validate>(payloads: &[T]) -> ValidationResult<()> {
    for (index, payload) in payloads.iter().enumerate() {
        if let Err(mut validation_error) = validate_request(payload) {
            // Add index prefix to field names for batch validation
            let mut new_fields = HashMap::new();
            for (field, messages) in validation_error.fields {
                let indexed_field = format!("[{}].{}", index, field);
                new_fields.insert(indexed_field, messages);
            }
            validation_error.fields = new_fields;
            validation_error.message = format!("Validation failed for item at index {}", index);
            return Err(validation_error);
        }
    }
    Ok(())
}

/// Sanitize input string to prevent XSS and other injection attacks
pub fn sanitize_string(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
        .collect()
}

/// Sanitize HTML input more aggressively
pub fn sanitize_html(input: &str) -> String {
    // Remove all HTML tags and sanitize special characters
    let no_tags = regex::Regex::new(r"<[^>]*>").unwrap().replace_all(input, "");
    sanitize_string(&no_tags)
}

/// Validate and sanitize text input
pub fn validate_and_sanitize_text(input: &str, max_length: usize) -> ValidationResult<String> {
    if input.is_empty() {
        return Err(ValidationErrorResponse::single_field(
            "text".to_string(),
            "Text cannot be empty".to_string(),
        ));
    }

    if input.len() > max_length {
        return Err(ValidationErrorResponse::single_field(
            "text".to_string(),
            format!("Text cannot exceed {} characters", max_length),
        ));
    }

    // Check for suspicious patterns including SQL injection
    let suspicious_patterns = [
        r"<script",
        r"javascript:",
        r"vbscript:",
        r"onload=",
        r"onerror=",
        r"onclick=",
        r"onmouseover=",
        r"expression\(",
        r"eval\(",
        r"document\.cookie",
        r"document\.write",
        r"window\.location",
        r"drop\s+table",
        r"delete\s+from",
        r"insert\s+into",
        r"update\s+.*\s+set",
        r"union\s+select",
        r"'.*or\s+.*=",
        r"'.*;\s*--",
        r"'.*;\s*/\*",
    ];

    let lower_input = input.to_lowercase();
    for pattern in &suspicious_patterns {
        if regex::Regex::new(pattern).unwrap().is_match(&lower_input) {
            return Err(ValidationErrorResponse::single_field(
                "text".to_string(),
                "Text contains potentially harmful content".to_string(),
            ));
        }
    }

    Ok(sanitize_string(input))
}

/// Validate file upload
pub fn validate_file_upload(
    content_type: &str,
    file_size: usize,
    max_size: usize,
    allowed_types: &[&str],
) -> ValidationResult<()> {
    if file_size > max_size {
        return Err(ValidationErrorResponse::single_field(
            "file".to_string(),
            format!("File size cannot exceed {} bytes", max_size),
        ));
    }

    if !allowed_types.contains(&content_type) {
        return Err(ValidationErrorResponse::single_field(
            "file".to_string(),
            format!("File type '{}' is not allowed", content_type),
        ));
    }

    Ok(())
}

/// Validate JSON structure depth to prevent DoS attacks
pub fn validate_json_depth(json_str: &str, max_depth: usize) -> ValidationResult<()> {
    let mut depth = 0;
    let mut max_found_depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for ch in json_str.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => {
                depth += 1;
                max_found_depth = max_found_depth.max(depth);
                if depth > max_depth {
                    return Err(ValidationErrorResponse::single_field(
                        "json".to_string(),
                        format!("JSON nesting depth cannot exceed {}", max_depth),
                    ));
                }
            }
            '}' | ']' if !in_string => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    Ok(())
}

/// Rate limiting validation
pub fn validate_rate_limit(
    wallet_address: &str,
    endpoint: &str,
    max_requests: u32,
    window_seconds: u64,
) -> ValidationResult<()> {
    // This would integrate with Redis or in-memory rate limiting
    // For now, we'll implement a basic check
    let _cache_key = format!("rate_limit:{}:{}", wallet_address, endpoint);
    
    // TODO: Implement actual rate limiting with Redis
    // For now, just log the rate limit check
    tracing::debug!(
        "Rate limit check for user {} on endpoint {}: max {} requests per {} seconds",
        wallet_address, endpoint, max_requests, window_seconds
    );
    
    Ok(())
}

/// Request validator for comprehensive input validation
pub struct RequestValidator {
    _max_json_depth: usize,
    max_string_length: usize,
    max_array_size: usize,
}

impl RequestValidator {
    pub fn new() -> Self {
        Self {
            _max_json_depth: limits::MAX_JSON_DEPTH,
            max_string_length: limits::MAX_DESCRIPTION_LENGTH,
            max_array_size: limits::MAX_ARRAY_SIZE,
        }
    }

    /// Validate user input from JSON
    pub fn validate_user_input(&self, input: &serde_json::Value) -> ValidationResult<()> {
        match input {
            serde_json::Value::String(s) => {
                validate_and_sanitize_text(s, self.max_string_length)?;
            }
            serde_json::Value::Object(obj) => {
                for (key, value) in obj {
                    // Validate key
                    validate_and_sanitize_text(key, 100)?;
                    // Recursively validate value
                    self.validate_user_input(value)?;
                }
            }
            serde_json::Value::Array(arr) => {
                if arr.len() > self.max_array_size {
                    return Err(ValidationErrorResponse::single_field(
                        "array".to_string(),
                        format!("Array size cannot exceed {}", self.max_array_size),
                    ));
                }
                for item in arr {
                    self.validate_user_input(item)?;
                }
            }
            _ => {} // Numbers, booleans, null are generally safe
        }
        Ok(())
    }

    /// Validate file path to prevent path traversal
    pub fn validate_file_path(&self, input: &serde_json::Value) -> ValidationResult<()> {
        if let Some(path_str) = input.get("file_path").and_then(|v| v.as_str()) {
            // Check for path traversal patterns
            let dangerous_patterns = ["../", "..\\", "/etc/", "\\windows\\", "~", "$HOME"];
            
            for pattern in &dangerous_patterns {
                if path_str.contains(pattern) {
                    return Err(ValidationErrorResponse::single_field(
                        "file_path".to_string(),
                        "File path contains dangerous patterns".to_string(),
                    ));
                }
            }

            // Ensure path doesn't start with sensitive directories
            let sensitive_dirs = ["/etc", "/usr", "/var", "/boot", "/sys", "/proc"];
            for dir in &sensitive_dirs {
                if path_str.starts_with(dir) {
                    return Err(ValidationErrorResponse::single_field(
                        "file_path".to_string(),
                        "Access to system directories is not allowed".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate API endpoint access
    pub fn validate_api_access(&self, endpoint: &str, method: &str) -> ValidationResult<()> {
        // Check for suspicious endpoint patterns
        let blocked_patterns = [
            "/admin/",
            "/internal/",
            "/debug/",
            "/test/",
            "/../",
            "/./",
        ];

        for pattern in &blocked_patterns {
            if endpoint.contains(pattern) {
                return Err(ValidationErrorResponse::single_field(
                    "endpoint".to_string(),
                    format!("Access to endpoint pattern '{}' is restricted", pattern),
                ));
            }
        }

        // Validate HTTP method
        let allowed_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !allowed_methods.contains(&method.to_uppercase().as_str()) {
            return Err(ValidationErrorResponse::single_field(
                "method".to_string(),
                format!("HTTP method '{}' is not allowed", method),
            ));
        }

        Ok(())
    }
}

impl Default for RequestValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common validation patterns
pub mod patterns {
    pub const EMAIL_REGEX: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
    pub const PASSWORD_REGEX: &str = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";
    pub const PHONE_REGEX: &str = r"^\+?[1-9]\d{1,14}$";
    pub const URL_REGEX: &str = r"^https?://[^\s/$.?#].[^\s]*$";
    pub const UUID_REGEX: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    pub const ALPHANUMERIC_REGEX: &str = r"^[a-zA-Z0-9]+$";
    pub const SLUG_REGEX: &str = r"^[a-z0-9-]+$";
    pub const SAFE_TEXT_REGEX: &str = r"^[a-zA-Z0-9\s\-_.,!?()]+$";
}

/// Validation limits
pub mod limits {
    pub const MAX_EMAIL_LENGTH: usize = 254;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_NAME_LENGTH: usize = 100;
    pub const MAX_DESCRIPTION_LENGTH: usize = 1000;
    pub const MAX_TITLE_LENGTH: usize = 200;
    pub const MAX_JSON_DEPTH: usize = 10;
    pub const MAX_ARRAY_SIZE: usize = 1000;
    pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
    pub const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct TestPayload {
        #[validate(email)]
        email: String,
        #[validate(length(min = 8, max = 128))]
        password: String,
    }

    #[test]
    fn test_validate_request_success() {
        let payload = TestPayload {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(validate_request(&payload).is_ok());
    }

    #[test]
    fn test_validate_request_failure() {
        let payload = TestPayload {
            email: "invalid-email".to_string(),
            password: "short".to_string(),
        };

        let result = validate_request(&payload);
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(error.fields.contains_key("email"));
            assert!(error.fields.contains_key("password"));
        }
    }

    #[test]
    fn test_sanitize_string() {
        let input = "<script>alert('xss')</script>";
        let sanitized = sanitize_string(input);
        assert_eq!(sanitized, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
    }

    #[test]
    fn test_validate_and_sanitize_text() {
        // Valid text
        let result = validate_and_sanitize_text("Hello, world!", 100);
        assert!(result.is_ok());

        // Too long
        let long_text = "a".repeat(1001);
        let result = validate_and_sanitize_text(&long_text, 1000);
        assert!(result.is_err());

        // Suspicious content
        let malicious = "<script>alert('xss')</script>";
        let result = validate_and_sanitize_text(malicious, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_depth() {
        // Valid JSON
        let json = r#"{"a": {"b": {"c": 1}}}"#;
        assert!(validate_json_depth(json, 5).is_ok());

        // Too deep JSON
        let deep_json = r#"{"a": {"b": {"c": {"d": {"e": {"f": 1}}}}}}"#;
        assert!(validate_json_depth(deep_json, 4).is_err());
    }

    #[test]
    fn test_validation_error_response() {
        let mut fields = HashMap::new();
        fields.insert("email".to_string(), vec!["Invalid email format".to_string()]);
        
        let error = ValidationErrorResponse::new("Validation failed".to_string(), fields);
        assert_eq!(error.error, "validation_error");
        assert_eq!(error.code, 400);
        assert!(error.fields.contains_key("email"));
    }
}