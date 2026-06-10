// Core types used across the application

use serde::{Deserialize, Serialize};
use std::fmt;

/// Application-wide error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub message: String,
    pub error_type: String,
    pub context: Option<String>,
}

impl AppError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: "GeneralError".to_string(),
            context: None,
        }
    }
    
    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }
    
    pub fn unauthorized(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: "Unauthorized".to_string(),
            context: None,
        }
    }
    
    pub fn forbidden(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: "Forbidden".to_string(),
            context: None,
        }
    }
    
    pub fn bad_request(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: "BadRequest".to_string(),
            context: None,
        }
    }
    
    pub fn internal_error(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error_type: "InternalServerError".to_string(),
            context: None,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for AppError {}

/// Email type with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, &'static str> {
        if email.contains('@') && email.len() > 3 {
            Ok(Self(email))
        } else {
            Err("Invalid email format")
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}