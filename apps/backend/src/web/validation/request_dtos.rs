// Validated request DTOs with comprehensive validation rules
use serde::Deserialize;
use validator::{Validate, ValidationError};

use super::validators::*;

/// Validated login request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedLoginRequest {
    #[validate(custom(function = "validate_secure_email"))]
    pub email: String,
    
    #[validate(custom(function = "validate_strong_password"))]
    pub password: String,
}

/// Validated user registration request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedRegisterRequest {
    #[validate(custom(function = "validate_secure_email"))]
    pub email: String,
    
    #[validate(custom(function = "validate_strong_password"))]
    pub password: String,
    
    #[validate(custom(function = "validate_display_name"))]
    pub display_name: String,
    
    #[validate(custom(function = "validate_phone_number"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    
    #[validate(length(min = 1, max = 50))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
}

/// Validated password reset request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedPasswordResetRequest {
    #[validate(custom(function = "validate_secure_email"))]
    pub email: String,
}

/// Validated password change request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedPasswordChangeRequest {
    #[validate(length(min = 1, max = 128))]
    pub current_password: String,
    
    #[validate(custom(function = "validate_strong_password"))]
    pub new_password: String,
}

/// Validated user profile update request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedUserProfileUpdateRequest {
    #[validate(custom(function = "validate_display_name"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    
    #[validate(custom(function = "validate_phone_number"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    
    #[validate(custom(function = "validate_secure_url"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

// ValidatedRoleCreateRequest removed - using permissions-based system only

/// Custom validator for permissions list
fn validate_permissions_list(permissions: &[String]) -> Result<(), ValidationError> {
    if permissions.is_empty() {
        let mut error = ValidationError::new("empty_permissions");
        error.message = Some("At least one permission is required".into());
        return Err(error);
    }

    if permissions.len() > 100 {
        let mut error = ValidationError::new("too_many_permissions");
        error.message = Some("Cannot assign more than 100 permissions to a group".into());
        return Err(error);
    }

    for permission in permissions {
        validate_permission_string(permission)?;
    }

    Ok(())
}

/// Validated permission assignment request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedPermissionAssignRequest {
    #[validate(length(min = 1))]
    pub wallet_address: String,
    
    #[validate(custom(function = "validate_permissions_list"))]
    pub permissions: Vec<String>,
    
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Validated payment creation request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedPaymentCreateRequest {
    #[validate(custom(function = "validate_positive_decimal"))]
    pub amount: String,
    
    #[validate(custom(function = "validate_currency_code"))]
    pub currency: String,
    
    #[validate(length(min = 1, max = 200))]
    pub description: String,
    
    #[validate(length(max = 100))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_email: Option<String>,
    
    #[validate(custom(function = "validate_json_content"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

/// Validated stock symbol query request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedStockQueryRequest {
    #[validate(length(min = 1, max = 10))]
    #[validate(regex(path = *STOCK_SYMBOL_REGEX))]
    pub symbol: String,
    
    #[validate(range(min = 1, max = 1000))]
    #[serde(default = "default_limit")]
    pub limit: i32,
    
    #[validate(range(min = 0))]
    #[serde(default)]
    pub offset: i32,
    
    #[validate(custom(function = "validate_date_range"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    
    #[validate(custom(function = "validate_date_range"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

fn default_limit() -> i32 {
    100
}

/// Stock symbol regex pattern
use once_cell::sync::Lazy;
use regex::Regex;

static STOCK_SYMBOL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z]{1,5}$").unwrap()
});

/// Custom validator for date range
fn validate_date_range(date: &str) -> Result<(), ValidationError> {
    use chrono::DateTime;
    
    match DateTime::parse_from_rfc3339(date) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Try alternative formats
            match chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                Ok(_) => Ok(()),
                Err(_) => {
                    let mut error = ValidationError::new("invalid_date_format");
                    error.message = Some("Date must be in YYYY-MM-DD or ISO 8601 format".into());
                    Err(error)
                }
            }
        }
    }
}

/// Validated webhook request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedWebhookRequest {
    #[validate(length(min = 1, max = 100))]
    pub event_type: String,
    
    #[validate(custom(function = "validate_json_content"))]
    pub payload: String,
    
    #[validate(length(min = 32, max = 128))]
    pub signature: String,
    
    #[validate(range(min = 1))]
    pub timestamp: i64,
}

/// Validated API key creation request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedApiKeyCreateRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[validate(custom(function = "validate_permissions_list"))]
    pub permissions: Vec<String>,
    
    #[validate(custom(function = "validate_expiry_date"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Custom validator for expiry date
fn validate_expiry_date(date: &str) -> Result<(), ValidationError> {
    use chrono::{DateTime, Utc};
    
    match DateTime::parse_from_rfc3339(date) {
        Ok(parsed_date) => {
            let now = Utc::now();
            if parsed_date.with_timezone(&Utc) <= now {
                let mut error = ValidationError::new("expiry_in_past");
                error.message = Some("Expiry date must be in the future".into());
                return Err(error);
            }
            
            // Check if expiry is too far in the future (max 1 year)
            let max_expiry = now + chrono::Duration::days(365);
            if parsed_date.with_timezone(&Utc) > max_expiry {
                let mut error = ValidationError::new("expiry_too_far");
                error.message = Some("Expiry date cannot be more than 1 year from now".into());
                return Err(error);
            }
            
            Ok(())
        },
        Err(_) => {
            let mut error = ValidationError::new("invalid_expiry_format");
            error.message = Some("Expiry date must be in ISO 8601 format".into());
            Err(error)
        }
    }
}

/// Validated search request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedSearchRequest {
    #[validate(length(min = 1, max = 200))]
    pub query: String,
    
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_search_limit")]
    pub limit: i32,
    
    #[validate(range(min = 0))]
    #[serde(default)]
    pub offset: i32,
    
    #[validate(length(max = 50))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    #[validate(custom(function = "validate_sort_field"))]
    #[serde(default = "default_sort")]
    pub sort: String,
    
    #[validate(custom(function = "validate_sort_order"))]
    #[serde(default = "default_sort_order")]
    pub order: String,
}

fn default_search_limit() -> i32 {
    20
}

fn default_sort() -> String {
    "relevance".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

/// Custom validator for sort field
fn validate_sort_field(sort: &str) -> Result<(), ValidationError> {
    let allowed_sorts = ["relevance", "date", "name", "price", "popularity"];
    
    if !allowed_sorts.contains(&sort) {
        let mut error = ValidationError::new("invalid_sort_field");
        error.message = Some(format!("Sort field must be one of: {}", allowed_sorts.join(", ")).into());
        return Err(error);
    }
    
    Ok(())
}

/// Custom validator for sort order
fn validate_sort_order(order: &str) -> Result<(), ValidationError> {
    if !matches!(order, "asc" | "desc") {
        let mut error = ValidationError::new("invalid_sort_order");
        error.message = Some("Sort order must be 'asc' or 'desc'".into());
        return Err(error);
    }
    
    Ok(())
}

/// Validated bulk operation request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedBulkOperationRequest<T: Clone + serde::Serialize> {
    #[validate(length(min = 1, max = 100))]
    pub items: Vec<T>,
    
    #[validate(length(min = 1, max = 50))]
    pub operation: String,
    
    #[serde(default)]
    pub dry_run: bool,
    
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Validated file upload request
#[derive(Debug, Deserialize, Validate)]
pub struct ValidatedFileUploadRequest {
    #[validate(custom(function = "validate_file_name"))]
    pub filename: String,
    
    #[validate(length(min = 1, max = 100))]
    pub content_type: String,
    
    #[validate(range(min = 1, max = 10485760))] // Max 10MB
    pub file_size: usize,
    
    #[validate(length(max = 500))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[serde(default)]
    pub is_public: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_validated_login_request() {
        let valid_request = ValidatedLoginRequest {
            email: "user@example.com".to_string(),
            password: "StrongPass123!".to_string(),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = ValidatedLoginRequest {
            email: "invalid-email".to_string(),
            password: "weak".to_string(),
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_validated_register_request() {
        let valid_request = ValidatedRegisterRequest {
            email: "newuser@example.com".to_string(),
            password: "SecurePass123!".to_string(),
            display_name: "New User".to_string(),
            phone: Some("+1234567890".to_string()),
            company: Some("Example Corp".to_string()),
        };
        assert!(valid_request.validate().is_ok());
    }

    // ValidatedRoleCreateRequest test removed - using permissions-based system

    #[test]
    fn test_validated_payment_create_request() {
        let valid_request = ValidatedPaymentCreateRequest {
            amount: "99.99".to_string(),
            currency: "USD".to_string(),
            description: "Test payment".to_string(),
            customer_email: Some("customer@example.com".to_string()),
            metadata: Some(r#"{"order_id": "12345"}"#.to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = ValidatedPaymentCreateRequest {
            amount: "-50.00".to_string(), // Negative amount should fail
            currency: "INVALID".to_string(), // Invalid currency should fail
            description: "".to_string(), // Empty description should fail
            customer_email: None,
            metadata: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_validated_stock_query_request() {
        let valid_request = ValidatedStockQueryRequest {
            symbol: "AAPL".to_string(),
            limit: 100,
            offset: 0,
            start_date: Some("2023-01-01".to_string()),
            end_date: Some("2023-12-31".to_string()),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_request = ValidatedStockQueryRequest {
            symbol: "INVALID123".to_string(), // Invalid symbol format
            limit: 2000, // Exceeds max limit
            offset: -1, // Negative offset
            start_date: Some("invalid-date".to_string()), // Invalid date format
            end_date: None,
        };
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_validate_permissions_list() {
        // Valid permissions
        let valid_perms = vec!["users:read".to_string(), "api:write".to_string()];
        assert!(validate_permissions_list(&valid_perms).is_ok());

        // Empty permissions
        let empty_perms: Vec<String> = vec![];
        assert!(validate_permissions_list(&empty_perms).is_err());

        // Invalid permission format
        let invalid_perms = vec!["invalid-permission".to_string()];
        assert!(validate_permissions_list(&invalid_perms).is_err());
    }

    #[test]
    fn test_validate_expiry_date() {
        use chrono::{Utc, Duration};

        // Future date should be valid
        let future_date = (Utc::now() + Duration::days(30)).to_rfc3339();
        assert!(validate_expiry_date(&future_date).is_ok());

        // Past date should be invalid
        let past_date = (Utc::now() - Duration::days(30)).to_rfc3339();
        assert!(validate_expiry_date(&past_date).is_err());

        // Invalid format should fail
        assert!(validate_expiry_date("invalid-date").is_err());
    }
}