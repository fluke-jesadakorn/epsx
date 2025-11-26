// Comprehensive input validation for all HTTP handlers
// Split into focused modules for better organization

pub mod validators;
pub mod middleware;
pub mod request_dtos;
pub mod types;
pub mod sanitizers;
pub mod core_validators;
pub mod request_validator;
pub mod constants;

// Re-exports for backward compatibility
pub use validators::*;
pub use middleware::*;
pub use request_dtos::*;
pub use types::*;
pub use sanitizers::*;
pub use core_validators::*;
pub use request_validator::RequestValidator;
pub use constants::{patterns, limits};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
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
