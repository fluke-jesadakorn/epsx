// Custom validators for application-specific validation rules
use regex::Regex;
use validator::ValidationError;
use once_cell::sync::Lazy;

/// Custom validator for strong passwords
pub fn validate_strong_password(password: &str) -> Result<(), ValidationError> {
    // Check length
    if password.len() < 8 {
        let mut error = ValidationError::new("invalid_password");
        error.message = Some("Password must be at least 8 characters long".into());
        return Err(error);
    }

    // Check for required character types
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "@$!%*?&".contains(c));

    if !has_lowercase || !has_uppercase || !has_digit || !has_special {
        let mut error = ValidationError::new("invalid_password");
        error.message = Some("Password must contain at least one lowercase letter, one uppercase letter, one digit, and one special character (@$!%*?&)".into());
        return Err(error);
    }

    // Additional checks for common weak passwords
    let weak_patterns = [
        "password", "123456", "qwerty", "admin", "letmein", 
        "welcome", "monkey", "dragon", "master", "trustno1"
    ];

    let lower_password = password.to_lowercase();
    for pattern in &weak_patterns {
        if lower_password.contains(pattern) {
            let mut error = ValidationError::new("weak_password");
            error.message = Some("Password contains commonly used patterns and is too weak".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Custom validator for email addresses with additional security checks
pub fn validate_secure_email(email: &str) -> Result<(), ValidationError> {
    static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });

    if !EMAIL_REGEX.is_match(email) {
        let mut error = ValidationError::new("invalid_email");
        error.message = Some("Invalid email format".into());
        return Err(error);
    }

    // Check for suspicious patterns
    let suspicious_patterns = ["+script", "+alert", "+javascript", "%", "script"];
    let lower_email = email.to_lowercase();
    
    for pattern in &suspicious_patterns {
        if lower_email.contains(pattern) {
            let mut error = ValidationError::new("suspicious_email");
            error.message = Some("Email contains potentially harmful content".into());
            return Err(error);
        }
    }

    // Check email length
    if email.len() > 254 {
        let mut error = ValidationError::new("email_too_long");
        error.message = Some("Email address cannot exceed 254 characters".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for user display names
pub fn validate_display_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        let mut error = ValidationError::new("empty_name");
        error.message = Some("Display name cannot be empty".into());
        return Err(error);
    }

    if name.len() > 100 {
        let mut error = ValidationError::new("name_too_long");
        error.message = Some("Display name cannot exceed 100 characters".into());
        return Err(error);
    }

    // Check for only safe characters
    static SAFE_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9\s\-_.,'()]+$").unwrap()
    });

    if !SAFE_NAME_REGEX.is_match(name) {
        let mut error = ValidationError::new("invalid_name_characters");
        error.message = Some("Display name contains invalid characters. Only letters, numbers, spaces, and basic punctuation are allowed".into());
        return Err(error);
    }

    // Check for suspicious patterns
    let suspicious_patterns = ["script", "javascript", "vbscript", "onload", "onerror"];
    let lower_name = name.to_lowercase();
    
    for pattern in &suspicious_patterns {
        if lower_name.contains(pattern) {
            let mut error = ValidationError::new("suspicious_name");
            error.message = Some("Display name contains potentially harmful content".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Custom validator for URLs with security checks
pub fn validate_secure_url(url: &str) -> Result<(), ValidationError> {
    static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
    });

    if !URL_REGEX.is_match(url) {
        let mut error = ValidationError::new("invalid_url");
        error.message = Some("Invalid URL format. Only HTTP and HTTPS URLs are allowed".into());
        return Err(error);
    }

    // Check for suspicious patterns
    let suspicious_patterns = ["javascript:", "vbscript:", "data:", "file:", "ftp:"];
    let lower_url = url.to_lowercase();
    
    for pattern in &suspicious_patterns {
        if lower_url.starts_with(pattern) {
            let mut error = ValidationError::new("suspicious_url");
            error.message = Some("URL protocol is not allowed for security reasons".into());
            return Err(error);
        }
    }

    // Check URL length
    if url.len() > 2048 {
        let mut error = ValidationError::new("url_too_long");
        error.message = Some("URL cannot exceed 2048 characters".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for phone numbers
pub fn validate_phone_number(phone: &str) -> Result<(), ValidationError> {
    static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
    });

    if !PHONE_REGEX.is_match(phone) {
        let mut error = ValidationError::new("invalid_phone");
        error.message = Some("Invalid phone number format. Use international format with optional + prefix".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for role names in IAM system
pub fn validate_role_name(role: &str) -> Result<(), ValidationError> {
    if role.trim().is_empty() {
        let mut error = ValidationError::new("empty_role");
        error.message = Some("Role name cannot be empty".into());
        return Err(error);
    }

    if role.len() > 50 {
        let mut error = ValidationError::new("role_too_long");
        error.message = Some("Role name cannot exceed 50 characters".into());
        return Err(error);
    }

    // Role names should be alphanumeric with underscores and dashes
    static ROLE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()
    });

    if !ROLE_REGEX.is_match(role) {
        let mut error = ValidationError::new("invalid_role_format");
        error.message = Some("Role name can only contain letters, numbers, underscores, and dashes".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for permission strings (resource:action format)
pub fn validate_permission_string(permission: &str) -> Result<(), ValidationError> {
    if permission.trim().is_empty() {
        let mut error = ValidationError::new("empty_permission");
        error.message = Some("Permission string cannot be empty".into());
        return Err(error);
    }

    if permission.len() > 100 {
        let mut error = ValidationError::new("permission_too_long");
        error.message = Some("Permission string cannot exceed 100 characters".into());
        return Err(error);
    }

    // Check for resource:action format or wildcards
    static PERMISSION_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9_*-]+:[a-zA-Z0-9_*-]+$").unwrap()
    });

    if !PERMISSION_REGEX.is_match(permission) {
        let mut error = ValidationError::new("invalid_permission_format");
        error.message = Some("Permission must be in format 'resource:action' (e.g., 'users:read', 'api:*')".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for JSON content with depth and size limits
pub fn validate_json_content(json_str: &str) -> Result<(), ValidationError> {
    // Check JSON size
    if json_str.len() > 1024 * 1024 { // 1MB limit
        let mut error = ValidationError::new("json_too_large");
        error.message = Some("JSON content exceeds maximum size of 1MB".into());
        return Err(error);
    }

    // Try to parse JSON to ensure it's valid
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(_) => {},
        Err(_) => {
            let mut error = ValidationError::new("invalid_json");
            error.message = Some("Invalid JSON format".into());
            return Err(error);
        }
    }

    // Check for suspicious content in JSON
    let suspicious_patterns = [
        "javascript:", "eval(", "function(", "<script", "document.", "window.", 
        "alert(", "confirm(", "prompt(", "setTimeout", "setInterval"
    ];

    let lower_json = json_str.to_lowercase();
    for pattern in &suspicious_patterns {
        if lower_json.contains(pattern) {
            let mut error = ValidationError::new("suspicious_json");
            error.message = Some("JSON contains potentially harmful content".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Custom validator for file names
pub fn validate_file_name(filename: &str) -> Result<(), ValidationError> {
    if filename.trim().is_empty() {
        let mut error = ValidationError::new("empty_filename");
        error.message = Some("Filename cannot be empty".into());
        return Err(error);
    }

    if filename.len() > 255 {
        let mut error = ValidationError::new("filename_too_long");
        error.message = Some("Filename cannot exceed 255 characters".into());
        return Err(error);
    }

    // Check for safe filename characters
    static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9\-_.() ]+\.[a-zA-Z0-9]+$").unwrap()
    });

    if !FILENAME_REGEX.is_match(filename) {
        let mut error = ValidationError::new("invalid_filename");
        error.message = Some("Filename contains invalid characters or missing extension".into());
        return Err(error);
    }

    // Check for dangerous extensions
    let dangerous_extensions = [
        ".exe", ".bat", ".cmd", ".com", ".pif", ".scr", ".vbs", ".js", 
        ".jar", ".php", ".asp", ".jsp", ".sh", ".ps1", ".dll", ".msi"
    ];

    let lower_filename = filename.to_lowercase();
    for ext in &dangerous_extensions {
        if lower_filename.ends_with(ext) {
            let mut error = ValidationError::new("dangerous_file_type");
            error.message = Some("File type is not allowed for security reasons".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Custom validator for API keys
pub fn validate_api_key_format(api_key: &str) -> Result<(), ValidationError> {
    if api_key.len() < 32 {
        let mut error = ValidationError::new("api_key_too_short");
        error.message = Some("API key must be at least 32 characters long".into());
        return Err(error);
    }

    if api_key.len() > 128 {
        let mut error = ValidationError::new("api_key_too_long");
        error.message = Some("API key cannot exceed 128 characters".into());
        return Err(error);
    }

    // API keys should be alphanumeric
    static API_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9]+$").unwrap()
    });

    if !API_KEY_REGEX.is_match(api_key) {
        let mut error = ValidationError::new("invalid_api_key_format");
        error.message = Some("API key can only contain letters and numbers".into());
        return Err(error);
    }

    Ok(())
}

/// Custom validator for decimal amounts (payments, quotas, etc.)
pub fn validate_positive_decimal(value: &str) -> Result<(), ValidationError> {
    match value.parse::<f64>() {
        Ok(num) => {
            if num <= 0.0 {
                let mut error = ValidationError::new("non_positive_amount");
                error.message = Some("Amount must be positive".into());
                return Err(error);
            }
            if num > 999999999.99 {
                let mut error = ValidationError::new("amount_too_large");
                error.message = Some("Amount cannot exceed 999,999,999.99".into());
                return Err(error);
            }
        },
        Err(_) => {
            let mut error = ValidationError::new("invalid_decimal");
            error.message = Some("Invalid decimal number format".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Custom validator for currency codes
pub fn validate_currency_code(currency: &str) -> Result<(), ValidationError> {
    if currency.len() != 3 {
        let mut error = ValidationError::new("invalid_currency_length");
        error.message = Some("Currency code must be exactly 3 characters".into());
        return Err(error);
    }

    // Check if it's all uppercase letters
    if !currency.chars().all(|c| c.is_ascii_uppercase()) {
        let mut error = ValidationError::new("invalid_currency_format");
        error.message = Some("Currency code must be 3 uppercase letters".into());
        return Err(error);
    }

    // Common currency codes validation
    let valid_currencies = [
        "USD", "EUR", "GBP", "JPY", "AUD", "CAD", "CHF", "CNY", "SEK", "NZD",
        "MXN", "SGD", "HKD", "NOK", "TRY", "RUB", "INR", "BRL", "ZAR", "THB"
    ];

    if !valid_currencies.contains(&currency) {
        let mut error = ValidationError::new("unsupported_currency");
        error.message = Some("Currency code is not supported".into());
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_strong_password() {
        // Valid password
        assert!(validate_strong_password("MyPassword123!").is_ok());
        
        // Too short
        assert!(validate_strong_password("Pass1!").is_err());
        
        // No uppercase
        assert!(validate_strong_password("mypassword123!").is_err());
        
        // No special character
        assert!(validate_strong_password("MyPassword123").is_err());
        
        // Weak pattern
        assert!(validate_strong_password("Password123!").is_err());
    }

    #[test]
    fn test_validate_secure_email() {
        // Valid email
        assert!(validate_secure_email("user@example.com").is_ok());
        
        // Invalid format
        assert!(validate_secure_email("invalid-email").is_err());
        
        // Suspicious content
        assert!(validate_secure_email("user+script@example.com").is_err());
        
        // Too long
        let long_email = format!("{}@example.com", "a".repeat(250));
        assert!(validate_secure_email(&long_email).is_err());
    }

    #[test]
    fn test_validate_display_name() {
        // Valid name
        assert!(validate_display_name("John Doe").is_ok());
        
        // Empty name
        assert!(validate_display_name("").is_err());
        assert!(validate_display_name("   ").is_err());
        
        // Too long
        let long_name = "a".repeat(101);
        assert!(validate_display_name(&long_name).is_err());
        
        // Invalid characters
        assert!(validate_display_name("John<script>alert('xss')</script>").is_err());
    }

    #[test]
    fn test_validate_permission_string() {
        // Valid permissions
        assert!(validate_permission_string("users:read").is_ok());
        assert!(validate_permission_string("api:*").is_ok());
        assert!(validate_permission_string("admin:write").is_ok());
        
        // Invalid format
        assert!(validate_permission_string("invalidpermission").is_err());
        assert!(validate_permission_string("users:").is_err());
        assert!(validate_permission_string(":read").is_err());
        
        // Empty
        assert!(validate_permission_string("").is_err());
    }

    #[test]
    fn test_validate_currency_code() {
        // Valid currencies
        assert!(validate_currency_code("USD").is_ok());
        assert!(validate_currency_code("EUR").is_ok());
        
        // Invalid length
        assert!(validate_currency_code("US").is_err());
        assert!(validate_currency_code("USDX").is_err());
        
        // Invalid case
        assert!(validate_currency_code("usd").is_err());
        
        // Unsupported currency
        assert!(validate_currency_code("XXX").is_err());
    }

    #[test]
    fn test_validate_positive_decimal() {
        // Valid amounts
        assert!(validate_positive_decimal("123.45").is_ok());
        assert!(validate_positive_decimal("1").is_ok());
        
        // Invalid amounts
        assert!(validate_positive_decimal("-123.45").is_err());
        assert!(validate_positive_decimal("0").is_err());
        assert!(validate_positive_decimal("not_a_number").is_err());
        assert!(validate_positive_decimal("9999999999.99").is_err());
    }
}