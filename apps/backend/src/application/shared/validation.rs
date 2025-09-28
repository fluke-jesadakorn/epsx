use super::{ApplicationError, ApplicationResult};

/// Validation error for application layer
#[derive(Debug, thiserror::Error)]
#[error("Validation error: {field} - {message}")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validator trait for application-level validation
pub trait Validator<T> {
    /// Validate an object and return validation errors
    fn validate(&self, value: &T) -> Vec<ValidationError>;
    
    /// Validate and return Result
    fn validate_result(&self, value: &T) -> ApplicationResult<()> {
        let errors = self.validate(value);
        if errors.is_empty() {
            Ok(())
        } else {
            // Return the first error (in a real implementation, you might collect all errors)
            let first_error = &errors[0];
            Err(ApplicationError::validation(
                &first_error.field,
                &first_error.message
            ))
        }
    }
}

/// Common validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Validate that a string is not empty
    pub fn required(field: &str, value: &str) -> Option<ValidationError> {
        if value.trim().is_empty() {
            Some(ValidationError::new(field, "Field is required"))
        } else {
            None
        }
    }
    
    /// Validate string length
    pub fn length(field: &str, value: &str, min: usize, max: usize) -> Option<ValidationError> {
        let len = value.len();
        if len < min {
            Some(ValidationError::new(field, format!("Must be at least {} characters", min)))
        } else if len > max {
            Some(ValidationError::new(field, format!("Must be no more than {} characters", max)))
        } else {
            None
        }
    }
    
    /// Validate email format (basic)
    pub fn email_format(field: &str, value: &str) -> Option<ValidationError> {
        if value.contains('@') && value.contains('.') {
            None
        } else {
            Some(ValidationError::new(field, "Invalid email format"))
        }
    }
    
    /// Validate that a collection is not empty
    pub fn not_empty<T>(field: &str, collection: &[T]) -> Option<ValidationError> {
        if collection.is_empty() {
            Some(ValidationError::new(field, "Collection cannot be empty"))
        } else {
            None
        }
    }
    
    /// Validate that a value is within a range
    pub fn range<T: PartialOrd>(field: &str, value: T, min: T, max: T) -> Option<ValidationError> {
        if value < min || value > max {
            Some(ValidationError::new(field, "Value is out of valid range"))
        } else {
            None
        }
    }
}

/// Macro for collecting validation errors
#[macro_export]
macro_rules! validate {
    ($($validation:expr),*) => {
        {
            let mut errors = Vec::new();
            $(
                if let Some(error) = $validation {
                    errors.push(error);
                }
            )*
            errors
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn validation_utils_required() {
        assert!(ValidationUtils::required("name", "").is_some());
        assert!(ValidationUtils::required("name", "   ").is_some());
        assert!(ValidationUtils::required("name", "John").is_none());
    }
    
    #[test]
    fn validation_utils_length() {
        assert!(ValidationUtils::length("name", "Jo", 3, 10).is_some()); // Too short
        assert!(ValidationUtils::length("name", "VeryLongName", 3, 10).is_some()); // Too long
        assert!(ValidationUtils::length("name", "John", 3, 10).is_none()); // Just right
    }
    
    #[test]
    fn validation_utils_email() {
        assert!(ValidationUtils::email_format("email", "invalid").is_some());
        assert!(ValidationUtils::email_format("email", "test@example.com").is_none());
    }
    
    #[test]
    fn validate_macro() {
        let errors = validate![
            ValidationUtils::required("name", ""),
            ValidationUtils::email_format("email", "invalid")
        ];
        
        assert_eq!(errors.len(), 2);
    }
}