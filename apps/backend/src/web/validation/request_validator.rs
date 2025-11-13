// Comprehensive request validator
use super::types::{ValidationErrorResponse, ValidationResult};
use super::sanitizers::validate_and_sanitize_text;
use super::constants::limits;

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
