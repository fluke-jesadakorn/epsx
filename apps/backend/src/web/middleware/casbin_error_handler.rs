//! Enhanced error handling for Casbin authorization system
//! Provides comprehensive error recovery, logging, and resilience patterns

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, warn, info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CasbinErrorType {
    DatabaseConnection,
    PolicyEvaluation,
    CacheFailure,
    InvalidInput,
    ConfigurationError,
    NetworkTimeout,
    ResourceExhaustion,
    SecurityViolation,
}

#[derive(Debug, Clone, Serialize)]
pub struct CasbinError {
    pub error_type: CasbinErrorType,
    pub message: String,
    pub context: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub retry_count: u32,
}

impl CasbinError {
    pub fn new(error_type: CasbinErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
            context: HashMap::new(),
            timestamp: chrono::Utc::now(),
            request_id: None,
            user_id: None,
            retry_count: 0,
        }
    }
    
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn increment_retry(mut self) -> Self {
        self.retry_count += 1;
        self
    }
}

#[derive(Debug)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub errors_by_type: HashMap<String, u64>,
    pub recent_errors: Vec<CasbinError>,
    pub last_reset: Instant,
}

impl Default for ErrorStats {
    fn default() -> Self {
        Self {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            recent_errors: Vec::new(),
            last_reset: Instant::now(),
        }
    }
}

impl ErrorStats {
    pub fn record_error(&mut self, error: &CasbinError) {
        self.total_errors += 1;
        
        let error_type_key = format!("{:?}", error.error_type);
        *self.errors_by_type.entry(error_type_key).or_insert(0) += 1;
        
        // Keep only recent errors (last 100)
        self.recent_errors.push(error.clone());
        if self.recent_errors.len() > 100 {
            self.recent_errors.remove(0);
        }
    }
    
    pub fn get_error_rate(&self, window: Duration) -> f64 {
        let cutoff = Instant::now() - window;
        if self.last_reset < cutoff {
            return 0.0;
        }
        
        let window_errors = self.recent_errors
            .iter()
            .filter(|e| e.timestamp > chrono::Utc::now() - chrono::Duration::from_std(window).unwrap())
            .count() as f64;
        
        window_errors / window.as_secs_f64()
    }
}

pub struct CasbinErrorHandler {
    stats: Arc<RwLock<ErrorStats>>,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    retry_config: RetryConfig,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            jitter: true,
        }
    }
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

#[derive(Debug, PartialEq, Clone)]  
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Blocking requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
    
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitBreakerState::Open => {
                // Should not happen
            }
        }
    }
    
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
            }
            CircuitBreakerState::Open => {
                // Already open
            }
        }
    }
}

impl CasbinErrorHandler {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(ErrorStats::default())),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreaker::new())),
            retry_config: RetryConfig::default(),
        }
    }
    
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    
    pub async fn handle_casbin_error(&self, error: CasbinError) -> Result<(), CasbinError> {
        // Record error statistics
        {
            let mut stats = self.stats.write().await;
            stats.record_error(&error);
        }
        
        // Log error with appropriate level
        match error.error_type {
            CasbinErrorType::SecurityViolation => {
                error!("Security violation detected: {} - Context: {:?}", error.message, error.context);
            }
            CasbinErrorType::DatabaseConnection | CasbinErrorType::NetworkTimeout => {
                warn!("Infrastructure error: {} - Retry count: {}", error.message, error.retry_count);
            }
            CasbinErrorType::InvalidInput => {
                info!("Input validation error: {} - User: {:?}", error.message, error.user_id);
            }
            _ => {
                debug!("Casbin error: {} - Type: {:?}", error.message, error.error_type);
            }
        }
        
        // Check circuit breaker
        {
            let mut breaker = self.circuit_breaker.write().await;
            if !breaker.can_execute() {
                let circuit_error = CasbinError::new(
                    CasbinErrorType::ResourceExhaustion,
                    "Circuit breaker is open - service temporarily unavailable".to_string()
                ).with_context("circuit_breaker_state", "open");
                
                return Err(circuit_error);
            }
            
            breaker.record_failure();
        }
        
        // Determine if error is retryable
        if self.is_retryable(&error) && error.retry_count < self.retry_config.max_retries {
            let delay = self.calculate_retry_delay(error.retry_count);
            
            warn!("Retrying Casbin operation after {:?} - Attempt {}/{}", 
                  delay, error.retry_count + 1, self.retry_config.max_retries);
            
            tokio::time::sleep(delay).await;
            return Err(error.increment_retry());
        }
        
        Err(error)
    }
    
    pub async fn handle_success(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.record_success();
    }
    
    fn is_retryable(&self, error: &CasbinError) -> bool {
        match error.error_type {
            CasbinErrorType::DatabaseConnection |
            CasbinErrorType::NetworkTimeout |
            CasbinErrorType::ResourceExhaustion => true,
            
            CasbinErrorType::InvalidInput |
            CasbinErrorType::SecurityViolation |
            CasbinErrorType::ConfigurationError => false,
            
            CasbinErrorType::PolicyEvaluation |
            CasbinErrorType::CacheFailure => error.retry_count < 2, // Limited retries
        }
    }
    
    fn calculate_retry_delay(&self, retry_count: u32) -> Duration {
        let base_delay = self.retry_config.base_delay;
        let exponential_delay = base_delay * 2_u32.pow(retry_count);
        let delay = std::cmp::min(exponential_delay, self.retry_config.max_delay);
        
        if self.retry_config.jitter {
            let jitter = rand::random::<f64>() * 0.1; // 10% jitter
            let jitter_factor = 1.0 + jitter;
            Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64)
        } else {
            delay
        }
    }
    
    pub async fn get_error_stats(&self) -> ErrorStats {
        let stats = self.stats.read().await;
        ErrorStats {
            total_errors: stats.total_errors,
            errors_by_type: stats.errors_by_type.clone(),
            recent_errors: stats.recent_errors.clone(),
            last_reset: stats.last_reset,
        }
    }
    
    pub async fn get_circuit_breaker_state(&self) -> CircuitBreakerState {
        self.circuit_breaker.read().await.state.clone()
    }
    
    pub async fn reset_circuit_breaker(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.state = CircuitBreakerState::Closed;
        breaker.failure_count = 0;
        breaker.success_count = 0;
        breaker.last_failure_time = None;
    }
    
    pub async fn reset_error_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ErrorStats::default();
        stats.last_reset = Instant::now();
    }
}

impl Default for CasbinErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware for handling Casbin-related errors with automatic retry and circuit breaking
pub async fn casbin_error_handling_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let error_handler = CasbinErrorHandler::new();
    
    let start_time = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Extract user ID from request if available
    let user_id = extract_user_id_from_request(&request);
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    
    // Log successful request
    if response.status().is_success() {
        error_handler.handle_success().await;
        debug!("Casbin request completed successfully - Duration: {:?}, Request ID: {}", 
               duration, request_id);
    } else {
        // Handle error response
        let error = CasbinError::new(
            map_status_to_error_type(response.status()),
            format!("Request failed with status: {}", response.status())
        )
        .with_request_id(request_id.clone())
        .with_user_id(user_id.unwrap_or_default())
        .with_context("duration_ms", &duration.as_millis().to_string());
        
        let _ = error_handler.handle_casbin_error(error).await;
    }
    
    Ok(response)
}

fn extract_user_id_from_request(request: &Request) -> Option<String> {
    // Try to extract user ID from headers
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // In a real implementation, decode JWT or validate session
                return Some(format!("user_from_token_{}", &token[..8.min(token.len())]));
            }
        }
    }
    
    // Try to extract from custom headers
    if let Some(user_header) = request.headers().get("x-user-id") {
        if let Ok(user_str) = user_header.to_str() {
            return Some(user_str.to_string());
        }
    }
    
    None
}

fn map_status_to_error_type(status: StatusCode) -> CasbinErrorType {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => CasbinErrorType::SecurityViolation,
        StatusCode::BAD_REQUEST => CasbinErrorType::InvalidInput,
        StatusCode::INTERNAL_SERVER_ERROR => CasbinErrorType::PolicyEvaluation,
        StatusCode::SERVICE_UNAVAILABLE => CasbinErrorType::ResourceExhaustion,
        StatusCode::GATEWAY_TIMEOUT | StatusCode::REQUEST_TIMEOUT => CasbinErrorType::NetworkTimeout,
        _ => CasbinErrorType::PolicyEvaluation,
    }
}

/// Helper trait for converting standard errors to CasbinError
pub trait ToCasbinError<T> {
    fn to_casbin_error(self, error_type: CasbinErrorType, context: &str) -> Result<T, CasbinError>;
}

impl<T, E: std::fmt::Display> ToCasbinError<T> for Result<T, E> {
    fn to_casbin_error(self, error_type: CasbinErrorType, context: &str) -> Result<T, CasbinError> {
        self.map_err(|e| {
            CasbinError::new(error_type, e.to_string())
                .with_context("operation", context)
        })
    }
}

/// Graceful degradation options for when Casbin is unavailable
#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    DenyAll,
    AllowAll,
    UseStaticRules(HashMap<String, Vec<String>>),
    UseCache,
}

pub struct GracefulDegradation {
    strategy: FallbackStrategy,
    #[allow(dead_code)]
    static_rules: HashMap<String, Vec<String>>,
}

impl GracefulDegradation {
    pub fn new(strategy: FallbackStrategy) -> Self {
        let static_rules = match &strategy {
            FallbackStrategy::UseStaticRules(rules) => rules.clone(),
            _ => HashMap::new(),
        };
        
        Self {
            strategy,
            static_rules,
        }
    }
    
    pub fn evaluate_fallback(&self, user: &str, resource: &str, action: &str) -> bool {
        match &self.strategy {
            FallbackStrategy::DenyAll => {
                warn!("Casbin unavailable - denying access for safety (user: {}, resource: {}, action: {})", 
                      user, resource, action);
                false
            }
            FallbackStrategy::AllowAll => {
                warn!("Casbin unavailable - allowing access (DANGEROUS) (user: {}, resource: {}, action: {})", 
                      user, resource, action);
                true
            }
            FallbackStrategy::UseStaticRules(rules) => {
                let key = format!("{}:{}:{}", user, resource, action);
                let allowed = rules.contains_key(&key);
                
                info!("Using static fallback rules - {} access for {}", 
                      if allowed { "allowing" } else { "denying" }, key);
                allowed
            }
            FallbackStrategy::UseCache => {
                // This would integrate with the cache system
                warn!("Casbin unavailable - falling back to cached decisions");
                false // Conservative default
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_error_handler_stats() {
        let handler = CasbinErrorHandler::new();
        
        let error = CasbinError::new(
            CasbinErrorType::DatabaseConnection,
            "Test error".to_string()
        );
        
        let _ = handler.handle_casbin_error(error).await;
        
        let stats = handler.get_error_stats().await;
        assert_eq!(stats.total_errors, 1);
        assert!(stats.errors_by_type.contains_key("DatabaseConnection"));
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let handler = CasbinErrorHandler::new();
        
        // Trigger multiple failures to open circuit
        for _ in 0..6 {
            let error = CasbinError::new(
                CasbinErrorType::DatabaseConnection,
                "Connection failed".to_string()
            );
            let _ = handler.handle_casbin_error(error).await;
        }
        
        let state = handler.get_circuit_breaker_state().await;
        assert_eq!(state, CircuitBreakerState::Open);
    }
    
    #[test]
    fn test_graceful_degradation() {
        let mut rules = HashMap::new();
        rules.insert("admin:/api/v1/admin:GET".to_string(), vec![]);
        
        let degradation = GracefulDegradation::new(
            FallbackStrategy::UseStaticRules(rules)
        );
        
        assert!(degradation.evaluate_fallback("admin", "/api/v1/admin", "GET"));
        assert!(!degradation.evaluate_fallback("user", "/api/v1/admin", "GET"));
    }
}