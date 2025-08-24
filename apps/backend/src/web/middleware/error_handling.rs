// Enhanced error handling middleware with comprehensive logging and context

use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

use crate::core::errors::{AppError, ErrorKind, ErrorLogger, ErrorSanitizer, ErrorContextBuilder};

/// Error response format for API clients
#[derive(serde::Serialize)]
pub struct ErrorResponseFormat {
    pub error: String,
    pub message: String,
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Enhanced error handling middleware that provides:
/// - Structured error logging with correlation IDs  
/// - Proper HTTP status code mapping
/// - Error sanitization for user-facing responses
/// - Request context preservation
/// - Performance metrics for error scenarios
pub async fn error_handling_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    let start_time = Instant::now();
    let correlation_id = Uuid::new_v4().to_string();
    let request_path = req.uri().path().to_string();
    let request_method = req.method().to_string();
    
    // Add correlation ID to request headers for downstream use
    req.headers_mut().insert(
        "x-correlation-id",
        correlation_id.parse().unwrap()
    );
    
    // Run the request
    let response = next.run(req).await;
    let duration = start_time.elapsed();
    
    // Check if response indicates an error
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        // Log error details with context
        tracing::error!(
            correlation_id = %correlation_id,
            status_code = %status.as_u16(),
            request_method = %request_method,
            request_path = %request_path,
            duration_ms = duration.as_millis(),
            "HTTP error response"
        );
        
        // Track error metrics
        tracing::info!(
            target: "error_metrics",
            correlation_id = %correlation_id,
            error_type = "http_error",
            status_code = %status.as_u16(),
            endpoint = %request_path,
            duration_ms = duration.as_millis(),
            "Error metrics"
        );
    }
    
    Ok(response)
}

/// Convert AppError to proper HTTP response with sanitization
pub fn app_error_to_response(error: AppError, _request_id: Option<String>) -> Response {
    let status_code = StatusCode::from_u16(error.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    // Log the full error with all context
    ErrorLogger::log_error(&error);
    
    // Create sanitized error for user response
    let sanitized_error = ErrorSanitizer::sanitize_for_user(&error);
    
    let error_response = ErrorResponseFormat {
        error: format!("{}", sanitized_error.kind),
        message: sanitized_error.message.clone(),
        correlation_id: sanitized_error.correlation_id.clone(),
        timestamp: sanitized_error.timestamp,
        details: create_error_details(&sanitized_error),
    };
    
    // Add correlation ID to response headers
    let mut response = (status_code, Json(error_response)).into_response();
    if let Ok(correlation_header) = error.correlation_id.parse() {
        response.headers_mut().insert("x-correlation-id", correlation_header);
    }
    
    response
}

/// Create error details for response (only safe, non-sensitive information)
fn create_error_details(error: &AppError) -> Option<serde_json::Value> {
    match error.kind {
        ErrorKind::ValidationError => {
            Some(json!({
                "type": "validation_error",
                "help": "Please check your input and try again"
            }))
        },
        ErrorKind::AuthenticationError => {
            Some(json!({
                "type": "authentication_error", 
                "help": "Please verify your credentials and try again"
            }))
        },
        ErrorKind::AuthorizationError => {
            Some(json!({
                "type": "authorization_error",
                "help": "You don't have permission to access this resource"
            }))
        },
        ErrorKind::AggregateNotFound => {
            Some(json!({
                "type": "not_found_error",
                "help": "The requested resource was not found"
            }))
        },
        ErrorKind::RateLimitExceeded => {
            Some(json!({
                "type": "rate_limit_error",
                "help": "Too many requests. Please wait before trying again"
            }))
        },
        ErrorKind::ServiceUnavailable => {
            Some(json!({
                "type": "service_unavailable",
                "help": "Service is temporarily unavailable. Please try again later"
            }))
        },
        _ => {
            Some(json!({
                "type": "internal_error",
                "help": "An unexpected error occurred. Please contact support if the problem persists"
            }))
        }
    }
}

/// Error context extraction from request headers
pub fn extract_error_context_from_request(headers: &HeaderMap, operation: &str, service: &str) -> crate::core::errors::ErrorContext {
    let mut builder = ErrorContextBuilder::new(operation, service);
    
    // Extract correlation ID if present
    if let Some(correlation_id) = headers.get("x-correlation-id") {
        if let Ok(correlation_str) = correlation_id.to_str() {
            builder = builder.request_id(correlation_str);
        }
    }
    
    // Extract user ID from auth headers if present
    if let Some(user_header) = headers.get("x-user-id") {
        if let Ok(user_id) = user_header.to_str() {
            builder = builder.user_id(user_id);
        }
    }
    
    // Add request metadata
    if let Some(user_agent) = headers.get("user-agent") {
        if let Ok(agent_str) = user_agent.to_str() {
            builder = builder.metadata("user_agent", agent_str);
        }
    }
    
    if let Some(client_ip) = headers.get("x-forwarded-for") {
        if let Ok(ip_str) = client_ip.to_str() {
            builder = builder.metadata("client_ip", ip_str.split(',').next().unwrap_or(ip_str));
        }
    }
    
    builder.build()
}

/// Error recovery middleware that attempts to recover from certain types of errors
pub async fn error_recovery_middleware(
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    let response = next.run(req).await;
    
    // For now, just pass through - could implement retry logic for transient failures
    // TODO: Implement actual recovery strategies based on error type
    Ok(response)
}

/// Circuit breaker for error handling to prevent cascading failures
pub struct ErrorCircuitBreaker {
    failure_threshold: u32,
    failure_count: std::sync::atomic::AtomicU32,
    last_failure_time: std::sync::atomic::AtomicU64,
    recovery_timeout: std::time::Duration,
}

impl ErrorCircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: std::time::Duration) -> Self {
        Self {
            failure_threshold,
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure_time: std::sync::atomic::AtomicU64::new(0),
            recovery_timeout,
        }
    }
    
    pub fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::Release);
    }
    
    pub fn record_failure(&self) -> bool {
        let count = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.last_failure_time.store(now, std::sync::atomic::Ordering::Release);
        
        count + 1 >= self.failure_threshold
    }
    
    pub fn is_open(&self) -> bool {
        let count = self.failure_count.load(std::sync::atomic::Ordering::Acquire);
        if count < self.failure_threshold {
            return false;
        }
        
        let last_failure = self.last_failure_time.load(std::sync::atomic::Ordering::Acquire);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now - last_failure < self.recovery_timeout.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::{AppError, ErrorKind};
    
    #[test]
    fn test_error_response_format() {
        let error = AppError::new(ErrorKind::ValidationError, "Test validation error");
        let response = app_error_to_response(error, None);
        
        // Response should have proper status code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // Should have correlation ID header
        assert!(response.headers().contains_key("x-correlation-id"));
    }
    
    #[test]
    fn test_error_sanitization() {
        let error = AppError::new(
            ErrorKind::DatabaseError, 
            "Connection failed: password=secret123 in connection string"
        );
        
        let sanitized = ErrorSanitizer::sanitize_for_user(&error);
        
        // Should not contain sensitive information
        assert!(!sanitized.message.contains("secret123"));
        assert!(sanitized.message.contains("[REDACTED]"));
        
        // Should not have stack trace
        assert!(sanitized.stack_trace.is_none());
        
        // Should not have internal metadata
        assert!(sanitized.context.metadata.is_empty());
    }
    
    #[test]
    fn test_circuit_breaker() {
        let breaker = ErrorCircuitBreaker::new(3, std::time::Duration::from_secs(60));
        
        // Initially closed
        assert!(!breaker.is_open());
        
        // Record failures
        assert!(!breaker.record_failure()); // 1st failure
        assert!(!breaker.record_failure()); // 2nd failure  
        assert!(breaker.record_failure());  // 3rd failure - should open circuit
        
        // Circuit should be open
        assert!(breaker.is_open());
        
        // Record success should reset
        breaker.record_success();
        assert!(!breaker.is_open());
    }
}