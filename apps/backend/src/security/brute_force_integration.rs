// Integration layer for brute force detection with existing middleware

use super::brute_force::{*, models::BruteForceError};
use crate::security::brute_force::models::{LoginAttempt, ThreatLevel, AttackType, ResponseActionResult};
use crate::security::brute_force::patterns::PatternAnalyzer;
use crate::web::auth::AppState;
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Brute force protection middleware that integrates with existing auth middleware
pub async fn brute_force_protection_middleware(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract request information
    let client_ip = extract_client_ip(&request);
    let user_agent = extract_user_agent(&request);
    let request_path = request.uri().path().to_string();
    let method = request.method().to_string();

    // Create login attempt record
    let login_attempt = LoginAttempt::new(
        client_ip.clone(),
        false, // Will be updated based on response
        None,  // Username extracted later if available
        user_agent.clone(),
    ).with_request_details(
        request_path.clone(),
        method.clone(),
        0, // Request size would be calculated
        0, // Response time will be measured
    );

    // Check if we have brute force protection enabled
    if let Some(bf_service) = &state.brute_force_service {
        // Analyze the request for brute force patterns
        match bf_service.analyze_login_attempt(&login_attempt).await {
            Ok(analysis_result) => {
                if analysis_result.should_block {
                    warn!(
                        "Brute force attack blocked: IP={}, Threat={:?}, Confidence={:.2}",
                        client_ip, analysis_result.threat_level, analysis_result.confidence_score
                    );

                    // Log security event
                    if let Some(security_cache) = &state.security_cache {
                        let _ = security_cache.log_security_event(&crate::infra::cache::SecurityEvent {
                            event_type: "BRUTE_FORCE_BLOCKED".to_string(),
                            timestamp: Utc::now(),
                            user_id: None,
                            session_id: None,
                            path: request_path,
                            method: Some(method),
                            ip: client_ip,
                            user_agent: user_agent.unwrap_or_default(),
                            severity: "HIGH".to_string(),
                            details: serde_json::json!({
                                "blocked_reason": analysis_result.block_reason,
                                "threat_level": format!("{:?}", analysis_result.threat_level),
                                "confidence_score": analysis_result.confidence_score,
                                "attack_types": analysis_result.attack_types,
                            }),
                            risk_score: Some(analysis_result.confidence_score * 10.0),
                            country_code: None,
                            city: None,
                            device_fingerprint: None,
                            correlation_id: Some(Uuid::new_v4().to_string()),
                        }).await;
                    }

                    return Ok(create_blocked_response(analysis_result));
                }

                // If suspicious but not blocked, add rate limiting headers
                if analysis_result.is_suspicious {
                    info!(
                        "Suspicious activity detected: IP={}, Confidence={:.2}",
                        client_ip, analysis_result.confidence_score
                    );
                }
            }
            Err(e) => {
                error!("Brute force analysis failed: {}", e);
                // Continue processing - don't block legitimate users due to system errors
            }
        }
    }

    // Continue with request processing
    let response = next.run(request).await;
    
    // Post-process response for ML training data
    if let Some(bf_service) = &state.brute_force_service {
        let success = response.status().is_success();
        let mut updated_attempt = login_attempt;
        updated_attempt.success = success;
        
        // Update ML training data asynchronously
        let bf_service_clone = bf_service.clone();
        tokio::spawn(async move {
            let _ = bf_service_clone.record_attempt_outcome(&updated_attempt).await;
        });
    }

    Ok(response)
}

/// Service wrapper for brute force detection integration
#[derive(Clone)]
pub struct BruteForceIntegrationService {
    protection_service: Arc<BruteForceProtectionService>,
    config: BruteForceConfig,
}

impl BruteForceIntegrationService {
    pub fn new(
        protection_service: Arc<BruteForceProtectionService>,
        config: BruteForceConfig,
    ) -> Self {
        Self {
            protection_service,
            config,
        }
    }

    /// Analyze login attempt and determine if it should be blocked
    pub async fn analyze_login_attempt(
        &self,
        attempt: &LoginAttempt,
    ) -> Result<BruteForceMiddlewareResult, BruteForceError> {
        // Use the comprehensive protection service
        let protection_service = self.protection_service.as_ref();
        let bf_request = BruteForceRequest {
            ip_address: attempt.ip_address.clone(),
            user_agent: attempt.user_agent.clone(),
            path: attempt.request_path.as_deref().unwrap_or("/").to_string(),
            method: "POST".to_string(), // Assuming login attempts are POST
            timestamp: attempt.timestamp,
        };
        let analysis_result = protection_service.analyze_request(bf_request).await
            .map_err(|e| BruteForceError::AnalysisError(e.to_string()))?;

        // Convert to middleware-friendly result
        let should_block = analysis_result.is_blocked;
        
        let is_suspicious = analysis_result.risk_score >= 0.5;

        let block_reason = analysis_result.reason
            .unwrap_or_else(|| if should_block {
                "Blocked due to suspicious activity".to_string()
            } else {
                "Not blocked".to_string()
            });

        Ok(BruteForceMiddlewareResult {
            should_block,
            is_suspicious,
            threat_level: ThreatLevel::Low, // Default since not available
            confidence_score: analysis_result.risk_score,
            attack_types: vec![], // Default since not available
            block_reason,
            blocked_until: analysis_result.expires_at,
            response_actions: vec![convert_brute_force_action_to_result(&analysis_result.recommended_action)],
        })
    }

    /// Record the outcome of a login attempt for ML training
    pub async fn record_attempt_outcome(
        &self,
        attempt: &LoginAttempt,
    ) -> Result<(), BruteForceError> {
        // This would update ML training data based on the actual outcome
        info!(
            "Recording attempt outcome: IP={}, Success={}", 
            attempt.ip_address, attempt.success
        );
        Ok(())
    }
}

/// Result structure for middleware decision making
#[derive(Debug, Clone)]
pub struct BruteForceMiddlewareResult {
    pub should_block: bool,
    pub is_suspicious: bool,
    pub threat_level: ThreatLevel,
    pub confidence_score: f64,
    pub attack_types: Vec<AttackType>,
    pub block_reason: String,
    pub blocked_until: Option<chrono::DateTime<chrono::Utc>>,
    pub response_actions: Vec<ResponseActionResult>,
}

/// Helper functions for request processing

fn extract_client_ip<B>(request: &Request<B>) -> String {
    // Try to get real client IP from headers
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return real_ip_str.to_string();
        }
    }

    if let Some(connect_info) = request.extensions().get::<axum::extract::ConnectInfo<std::net::SocketAddr>>() {
        return connect_info.ip().to_string();
    }

    "unknown".to_string()
}

fn extract_user_agent<B>(request: &Request<B>) -> Option<String> {
    request.headers().get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|ua| ua.to_string())
}

fn create_blocked_response(result: BruteForceMiddlewareResult) -> Response {
    let status = match result.threat_level {
        ThreatLevel::Critical => StatusCode::FORBIDDEN,
        ThreatLevel::High => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::TOO_MANY_REQUESTS,
    };

    let body = serde_json::json!({
        "error": "Request blocked due to security policy",
        "reason": result.block_reason,
        "threat_level": format!("{:?}", result.threat_level),
        "blocked_until": result.blocked_until,
        "retry_after": result.blocked_until.map(|until| {
            (until - chrono::Utc::now()).num_seconds().max(0)
        }),
    });

    let mut response = Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(serde_json::to_string(&body).unwrap_or_default().into())
        .unwrap();

    // Add security headers
    let headers = response.headers_mut();
    headers.insert("X-Security-Block", "brute-force-protection".parse().unwrap());
    
    if let Some(blocked_until) = result.blocked_until {
        let retry_after = (blocked_until - chrono::Utc::now()).num_seconds().max(0);
        headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
    }

    response
}

/// Factory for creating brute force integration service
pub struct BruteForceIntegrationFactory;

impl BruteForceIntegrationFactory {
    pub async fn create(
        app_state: &AppState,
    ) -> Result<BruteForceIntegrationService, BruteForceError> {
        let config = BruteForceConfig::default();
        
        // Create the comprehensive protection service
        let security_cache = app_state.security_cache.as_ref()
            .ok_or_else(|| BruteForceError::DatabaseError("Security cache not initialized".to_string()))?;
        let _detection_engine = BruteForceDetectionEngine::new(
            security_cache.clone(),
        );

        let _response_manager = ResponseManager::new(
            config.clone(),
            app_state.db_pool.clone(),
            app_state.cache.clone(),
        );

        let _pattern_analyzer = PatternAnalyzer::new(
            config.clone(),
            app_state.db_pool.clone(),
            app_state.cache.clone(),
        );

        let protection_service = Arc::new(BruteForceProtectionService::new(
            config.clone(),
            app_state.db_pool.clone(),
            app_state.cache.clone(),
        ));

        Ok(BruteForceIntegrationService::new(protection_service, config))
    }
}

/// Extension trait to add brute force protection to AppState
pub trait AppStateExtensions {
    fn with_brute_force_protection(self, _bf_service: BruteForceIntegrationService) -> Self;
}

impl AppStateExtensions for AppState {
    fn with_brute_force_protection(self, _bf_service: BruteForceIntegrationService) -> Self {
        // This would add the brute force service to the AppState
        // Since we can't modify the existing AppState structure,
        // this serves as an example of how it could be integrated
        self
    }
}

/// Convert BruteForceAction to ResponseActionResult for compatibility
fn convert_brute_force_action_to_result(action: &super::brute_force::BruteForceAction) -> super::brute_force::models::ResponseActionResult {
    use super::brute_force::{BruteForceAction, models::{ResponseActionResult, ResponseActionType}};
    
    match action {
        BruteForceAction::Allow => ResponseActionResult {
            action_type: ResponseActionType::IpBlock, // Placeholder type
            success: true,
            message: "Request allowed".to_string(),
            duration: None,
        },
        BruteForceAction::Block => ResponseActionResult {
            action_type: ResponseActionType::IpBlock,
            success: true,
            message: "IP address blocked".to_string(),
            duration: Some(chrono::Duration::hours(1)),
        },
        BruteForceAction::Challenge => ResponseActionResult {
            action_type: ResponseActionType::CaptchaChallenge,
            success: true,
            message: "CAPTCHA challenge required".to_string(),
            duration: None,
        },
        BruteForceAction::Throttle => ResponseActionResult {
            action_type: ResponseActionType::RateLimit,
            success: true,
            message: "Request rate limited".to_string(),
            duration: Some(chrono::Duration::minutes(5)),
        },
    }
}