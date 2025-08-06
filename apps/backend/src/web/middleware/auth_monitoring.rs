// Authorization monitoring and logging middleware

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use crate::web::auth::AppState;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Authorization event types for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEventType {
    AccessGranted,
    AccessDenied,
    AuthenticationFailed,
    PolicyCacheHit,
    PolicyCacheMiss,
    PolicyReload,
    RoleAssigned,
    RoleRemoved,
    PolicyAdded,
    PolicyRemoved,
}

/// Authorization monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    pub event_type: AuthEventType,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub result: Option<bool>,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub additional_data: serde_json::Value,
}

impl AuthEvent {
    pub fn new(event_type: AuthEventType) -> Self {
        Self {
            event_type,
            user_id: None,
            resource: None,
            action: None,
            result: None,
            timestamp: Utc::now(),
            duration_ms: None,
            client_ip: None,
            user_agent: None,
            session_id: None,
            additional_data: serde_json::Value::Null,
        }
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_resource_action(mut self, resource: String, action: String) -> Self {
        self.resource = Some(resource);
        self.action = Some(action);
        self
    }
    
    pub fn with_result(mut self, result: bool) -> Self {
        self.result = Some(result);
        self
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
    
    pub fn with_client_info(mut self, client_ip: Option<String>, user_agent: Option<String>) -> Self {
        self.client_ip = client_ip;
        self.user_agent = user_agent;
        self
    }
    
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    pub fn with_additional_data(mut self, data: serde_json::Value) -> Self {
        self.additional_data = data;
        self
    }
}

/// Authorization monitoring service
#[derive(Debug, Clone)]
pub struct AuthMonitoringService {
    // In production, this could write to metrics systems like Prometheus, OpenTelemetry, etc.
    enabled: bool,
}

impl AuthMonitoringService {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
    
    /// Log authorization event
    pub async fn log_event(&self, event: AuthEvent) {
        if !self.enabled {
            return;
        }
        
        // Log structured event based on type
        match event.event_type {
            AuthEventType::AccessGranted => {
                tracing::info!(
                    user_id = ?event.user_id,
                    resource = ?event.resource,
                    action = ?event.action,
                    duration_ms = ?event.duration_ms,
                    client_ip = ?event.client_ip,
                    "Authorization granted"
                );
            }
            AuthEventType::AccessDenied => {
                tracing::warn!(
                    user_id = ?event.user_id,
                    resource = ?event.resource,
                    action = ?event.action,
                    duration_ms = ?event.duration_ms,
                    client_ip = ?event.client_ip,
                    "Authorization denied"
                );
            }
            AuthEventType::AuthenticationFailed => {
                tracing::warn!(
                    client_ip = ?event.client_ip,
                    user_agent = ?event.user_agent,
                    "Authentication failed"
                );
            }
            AuthEventType::PolicyCacheHit => {
                tracing::debug!(
                    user_id = ?event.user_id,
                    resource = ?event.resource,
                    action = ?event.action,
                    "Policy cache hit"
                );
            }
            AuthEventType::PolicyCacheMiss => {
                tracing::debug!(
                    user_id = ?event.user_id,
                    resource = ?event.resource,
                    action = ?event.action,
                    "Policy cache miss"
                );
            }
            AuthEventType::PolicyReload => {
                tracing::info!(
                    user_id = ?event.user_id,
                    additional_data = ?event.additional_data,
                    "Policies reloaded"
                );
            }
            AuthEventType::RoleAssigned => {
                tracing::info!(
                    user_id = ?event.user_id,
                    additional_data = ?event.additional_data,
                    "Role assigned to user"
                );
            }
            AuthEventType::RoleRemoved => {
                tracing::info!(
                    user_id = ?event.user_id,
                    additional_data = ?event.additional_data,
                    "Role removed from user"
                );
            }
            AuthEventType::PolicyAdded => {
                tracing::info!(
                    resource = ?event.resource,
                    action = ?event.action,
                    additional_data = ?event.additional_data,
                    "Policy added"
                );
            }
            AuthEventType::PolicyRemoved => {
                tracing::info!(
                    resource = ?event.resource,
                    action = ?event.action,
                    additional_data = ?event.additional_data,
                    "Policy removed"
                );
            }
        }
        
        // In production, also send to metrics system
        self.send_to_metrics(&event).await;
    }
    
    /// Send metrics to monitoring system (placeholder)
    async fn send_to_metrics(&self, event: &AuthEvent) {
        // In production, implement integration with:
        // - Prometheus metrics
        // - OpenTelemetry spans
        // - Custom analytics systems
        // - Security information and event management (SIEM) systems
        
        // For now, just log at trace level for debugging
        tracing::trace!(
            event_type = ?event.event_type,
            timestamp = ?event.timestamp,
            duration_ms = ?event.duration_ms,
            "Auth event sent to metrics"
        );
    }
    
    /// Create monitoring event for access control
    pub fn create_access_event(
        user_id: &str,
        resource: &str,
        action: &str,
        granted: bool,
        duration_ms: u64,
        client_ip: Option<String>,
        user_agent: Option<String>,
    ) -> AuthEvent {
        let event_type = if granted {
            AuthEventType::AccessGranted
        } else {
            AuthEventType::AccessDenied
        };
        
        AuthEvent::new(event_type)
            .with_user(user_id.to_string())
            .with_resource_action(resource.to_string(), action.to_string())
            .with_result(granted)
            .with_duration(duration_ms)
            .with_client_info(client_ip, user_agent)
    }
    
    /// Create monitoring event for policy cache operations
    pub fn create_cache_event(
        user_id: &str,
        resource: &str,
        action: &str,
        cache_hit: bool,
    ) -> AuthEvent {
        let event_type = if cache_hit {
            AuthEventType::PolicyCacheHit
        } else {
            AuthEventType::PolicyCacheMiss
        };
        
        AuthEvent::new(event_type)
            .with_user(user_id.to_string())
            .with_resource_action(resource.to_string(), action.to_string())
    }
}

impl Default for AuthMonitoringService {
    fn default() -> Self {
        Self::new(true) // Monitoring enabled by default
    }
}

/// Authorization monitoring middleware
pub async fn auth_monitoring_middleware(
    State(_app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = Instant::now();
    
    // Extract client information
    let client_ip = extract_client_ip(&request);
    let _user_agent = extract_user_agent(&request);
    
    // Store start time in request extensions for duration calculation
    request.extensions_mut().insert(start_time);
    
    // Process request
    let response = next.run(request).await;
    
    // Calculate duration
    let duration = start_time.elapsed();
    let duration_ms = duration.as_millis() as u64;
    
    // Log completion (response status can indicate success/failure)
    let status = response.status();
    tracing::debug!(
        status = status.as_u16(),
        duration_ms = duration_ms,
        client_ip = ?client_ip,
        "Request completed"
    );
    
    Ok(response)
}

/// Extract client IP from request
fn extract_client_ip(request: &Request) -> Option<String> {
    // Check for forwarded headers first (for load balancers/proxies)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return Some(first_ip.trim().to_string());
            }
        }
    }
    
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return Some(ip_str.to_string());
        }
    }
    
    // Extract from connection info if available
    // This would require access to the connection info, which might not be available in all setups
    None
}

/// Extract user agent from request
fn extract_user_agent(request: &Request) -> Option<String> {
    request
        .headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

/// Security monitoring helper functions
pub mod security {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    /// Simple rate limiting for suspicious activity detection
    #[derive(Debug)]
    pub struct SecurityMonitor {
        failed_attempts: Arc<Mutex<HashMap<String, (u32, DateTime<Utc>)>>>,
        max_attempts: u32,
        window_minutes: i64,
    }
    
    impl SecurityMonitor {
        pub fn new(max_attempts: u32, window_minutes: i64) -> Self {
            Self {
                failed_attempts: Arc::new(Mutex::new(HashMap::new())),
                max_attempts,
                window_minutes,
            }
        }
        
        /// Record failed authentication attempt
        pub fn record_failed_attempt(&self, client_ip: &str) -> bool {
            let mut attempts = self.failed_attempts.lock().unwrap();
            let now = Utc::now();
            
            let (count, first_attempt) = attempts
                .entry(client_ip.to_string())
                .or_insert((0, now));
            
            // Reset count if window has passed
            if (now - *first_attempt).num_minutes() > self.window_minutes {
                *count = 1;
                *first_attempt = now;
                return false;
            }
            
            *count += 1;
            
            // Check if threshold exceeded
            if *count >= self.max_attempts {
                tracing::warn!(
                    client_ip = client_ip,
                    attempts = *count,
                    window_minutes = self.window_minutes,
                    "Suspicious activity detected: multiple failed auth attempts"
                );
                return true;
            }
            
            false
        }
        
        /// Clear failed attempts for IP (e.g., after successful auth)
        pub fn clear_failed_attempts(&self, client_ip: &str) {
            let mut attempts = self.failed_attempts.lock().unwrap();
            attempts.remove(client_ip);
        }
        
        /// Check if IP is currently blocked
        pub fn is_blocked(&self, client_ip: &str) -> bool {
            let attempts = self.failed_attempts.lock().unwrap();
            if let Some((count, first_attempt)) = attempts.get(client_ip) {
                let now = Utc::now();
                if (now - *first_attempt).num_minutes() <= self.window_minutes && *count >= self.max_attempts {
                    return true;
                }
            }
            false
        }
    }
    
    impl Default for SecurityMonitor {
        fn default() -> Self {
            Self::new(5, 15) // 5 attempts in 15 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_auth_event_creation() {
        let event = AuthEvent::new(AuthEventType::AccessGranted)
            .with_user("test_user".to_string())
            .with_resource_action("resource1".to_string(), "read".to_string())
            .with_result(true)
            .with_duration(150);
        
        assert_eq!(event.event_type, AuthEventType::AccessGranted);
        assert_eq!(event.user_id, Some("test_user".to_string()));
        assert_eq!(event.resource, Some("resource1".to_string()));
        assert_eq!(event.action, Some("read".to_string()));
        assert_eq!(event.result, Some(true));
        assert_eq!(event.duration_ms, Some(150));
    }
    
    #[tokio::test]
    async fn test_monitoring_service() {
        let monitor = AuthMonitoringService::new(true);
        
        let event = AuthMonitoringService::create_access_event(
            "user1",
            "resource1",
            "read",
            true,
            100,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );
        
        // This should not panic
        monitor.log_event(event).await;
    }
    
    #[test]
    fn test_security_monitor() {
        let monitor = security::SecurityMonitor::new(3, 10);
        
        // Should not be blocked initially
        assert!(!monitor.is_blocked("192.168.1.1"));
        
        // Record failed attempts
        assert!(!monitor.record_failed_attempt("192.168.1.1"));
        assert!(!monitor.record_failed_attempt("192.168.1.1"));
        assert!(monitor.record_failed_attempt("192.168.1.1")); // Should trigger block
        
        // Should be blocked now
        assert!(monitor.is_blocked("192.168.1.1"));
        
        // Clear attempts
        monitor.clear_failed_attempts("192.168.1.1");
        assert!(!monitor.is_blocked("192.168.1.1"));
    }
}