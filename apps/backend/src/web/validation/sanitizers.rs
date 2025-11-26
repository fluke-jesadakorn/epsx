// Input sanitization functions
use super::types::{ValidationErrorResponse, ValidationResult};

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
