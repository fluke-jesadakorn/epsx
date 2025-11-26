// Core validation functions
use std::collections::HashMap;
use validator::Validate;
use super::types::{ValidationErrorResponse, ValidationResult};

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

/// Rate limiting validation helper (DEPRECATED)
///
/// Note: Actual rate limiting is handled by UnifiedRateLimiter in web/middleware/rate_limiter.rs
/// This function is kept for legacy compatibility and logs rate limit parameters.
/// Use the middleware-based rate limiting for production workloads.
pub fn validate_rate_limit(
    wallet_address: &str,
    endpoint: &str,
    max_requests: u32,
    window_seconds: u64,
) -> ValidationResult<()> {
    // Actual rate limiting is implemented in UnifiedRateLimiter (web/middleware/rate_limiter.rs)
    // with full Redis + in-memory cache support
    tracing::debug!(
        "Rate limit check for user {} on endpoint {}: max {} requests per {} seconds",
        wallet_address, endpoint, max_requests, window_seconds
    );

    Ok(())
}
