// Feature Flag System for Gradual Migration
// Safe rollout of stateless authentication with monitoring and rollback

use std::env;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    // Core stateless auth features
    pub stateless_auth_enabled: bool,
    pub stateless_auth_percentage: u8,        // Percentage of users to use stateless auth
    pub enhanced_security_enabled: bool,
    pub threat_detection_enabled: bool,
    pub auto_refresh_enabled: bool,
    
    // Security features
    pub permission_integrity_check: bool,
    pub device_binding_enabled: bool,
    pub temporal_permissions_enabled: bool,
    pub rs256_signature_enabled: bool,
    
    // Monitoring and observability
    pub detailed_logging_enabled: bool,
    pub performance_monitoring_enabled: bool,
    pub security_metrics_enabled: bool,
    
    // Migration controls
    pub legacy_fallback_enabled: bool,
    pub migration_phase: MigrationPhase,
    pub emergency_rollback: bool,
    
    // User-specific overrides
    pub force_stateless_users: Vec<String>,   // Force specific users to use stateless
    pub force_legacy_users: Vec<String>,      // Force specific users to use legacy
    pub admin_users_stateless: bool,          // All admin users use stateless
    pub beta_users_stateless: bool,           // Beta users get stateless first
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationPhase {
    Planning,           // Not started
    Preparation,        // Infrastructure ready, not enabled
    Limited(u8),        // Limited rollout (percentage)
    Expanded(u8),       // Expanded rollout (percentage)  
    GeneralAvailability, // Full rollout
    Complete,           // Migration complete, legacy removed
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            // Core features - start disabled for safety
            stateless_auth_enabled: env::var("FEATURE_STATELESS_AUTH")
                .unwrap_or("false".to_string()) == "true",
            stateless_auth_percentage: env::var("STATELESS_AUTH_PERCENTAGE")
                .unwrap_or("0".to_string())
                .parse()
                .unwrap_or(0)
                .min(100),
            enhanced_security_enabled: env::var("FEATURE_ENHANCED_SECURITY")
                .unwrap_or("true".to_string()) == "true",
            threat_detection_enabled: env::var("FEATURE_THREAT_DETECTION")
                .unwrap_or("true".to_string()) == "true",
            auto_refresh_enabled: env::var("FEATURE_AUTO_REFRESH")
                .unwrap_or("false".to_string()) == "true",
            
            // Security features - enable by default for security
            permission_integrity_check: env::var("FEATURE_PERMISSION_INTEGRITY")
                .unwrap_or("true".to_string()) == "true",
            device_binding_enabled: env::var("FEATURE_DEVICE_BINDING")
                .unwrap_or("true".to_string()) == "true",
            temporal_permissions_enabled: env::var("FEATURE_TEMPORAL_PERMISSIONS")
                .unwrap_or("true".to_string()) == "true",
            rs256_signature_enabled: env::var("FEATURE_RS256_SIGNATURE")
                .unwrap_or("true".to_string()) == "true",
            
            // Monitoring
            detailed_logging_enabled: env::var("FEATURE_DETAILED_LOGGING")
                .unwrap_or("true".to_string()) == "true",
            performance_monitoring_enabled: env::var("FEATURE_PERFORMANCE_MONITORING")
                .unwrap_or("true".to_string()) == "true",
            security_metrics_enabled: env::var("FEATURE_SECURITY_METRICS")
                .unwrap_or("true".to_string()) == "true",
            
            // Migration controls
            legacy_fallback_enabled: env::var("FEATURE_LEGACY_FALLBACK")
                .unwrap_or("true".to_string()) == "true",
            migration_phase: Self::parse_migration_phase(),
            emergency_rollback: env::var("EMERGENCY_ROLLBACK")
                .unwrap_or("false".to_string()) == "true",
            
            // User overrides
            force_stateless_users: Self::parse_user_list("FORCE_STATELESS_USERS"),
            force_legacy_users: Self::parse_user_list("FORCE_LEGACY_USERS"),
            admin_users_stateless: env::var("ADMIN_USERS_STATELESS")
                .unwrap_or("false".to_string()) == "true",
            beta_users_stateless: env::var("BETA_USERS_STATELESS")
                .unwrap_or("false".to_string()) == "true",
        }
    }
}

impl FeatureFlags {
    /// Create new feature flags with custom overrides
    pub fn new() -> Self {
        let flags = Self::default();
        
        // Log current configuration
        info!(
            stateless_auth_enabled = flags.stateless_auth_enabled,
            stateless_percentage = flags.stateless_auth_percentage,
            migration_phase = ?flags.migration_phase,
            emergency_rollback = flags.emergency_rollback,
            "Feature flags initialized"
        );
        
        // Validate configuration
        flags.validate();
        
        flags
    }
    
    /// Check if user should use stateless authentication
    pub fn should_use_stateless_auth(&self, wallet_address: &str, is_admin: bool, is_beta: bool) -> bool {
        // Emergency rollback - force everyone to legacy
        if self.emergency_rollback {
            warn!(wallet_address = wallet_address, "Emergency rollback active - forcing legacy auth");
            return false;
        }
        
        // Not enabled at all
        if !self.stateless_auth_enabled {
            return false;
        }
        
        // Explicit user overrides
        if self.force_legacy_users.contains(&wallet_address.to_string()) {
            debug!(wallet_address = wallet_address, "User explicitly forced to legacy auth");
            return false;
        }
        
        if self.force_stateless_users.contains(&wallet_address.to_string()) {
            debug!(wallet_address = wallet_address, "User explicitly forced to stateless auth");
            return true;
        }
        
        // Admin users override
        if is_admin && self.admin_users_stateless {
            debug!(wallet_address = wallet_address, "Admin user using stateless auth");
            return true;
        }
        
        // Beta users override
        if is_beta && self.beta_users_stateless {
            debug!(wallet_address = wallet_address, "Beta user using stateless auth");
            return true;
        }
        
        // Percentage-based rollout
        if self.stateless_auth_percentage >= 100 {
            return true;
        }
        
        if self.stateless_auth_percentage == 0 {
            return false;
        }
        
        // Use consistent hashing based on user ID for stable assignment
        let hash = self.hash_wallet_address(wallet_address);
        let user_percentage = hash % 100;
        
        let should_use = user_percentage < self.stateless_auth_percentage as u32;
        
        debug!(
            wallet_address = wallet_address,
            user_percentage = user_percentage,
            threshold = self.stateless_auth_percentage,
            result = should_use,
            "Percentage-based stateless auth decision"
        );
        
        should_use
    }
    
    /// Get feature flag summary for monitoring
    pub fn get_summary(&self) -> FeatureFlagSummary {
        FeatureFlagSummary {
            timestamp: Utc::now(),
            stateless_auth_enabled: self.stateless_auth_enabled,
            rollout_percentage: self.stateless_auth_percentage,
            migration_phase: self.migration_phase.clone(),
            emergency_rollback: self.emergency_rollback,
            security_features_enabled: self.count_security_features(),
            monitoring_enabled: self.performance_monitoring_enabled,
            legacy_fallback_available: self.legacy_fallback_enabled,
        }
    }
    
    /// Update feature flags dynamically (for admin interface)
    pub fn update_flags(&mut self, updates: HashMap<String, String>) -> Result<(), String> {
        for (key, value) in &updates {
            match key.as_str() {
                "stateless_auth_enabled" => {
                    self.stateless_auth_enabled = value.parse().map_err(|_| "Invalid boolean for stateless_auth_enabled")?;
                },
                "stateless_auth_percentage" => {
                    let percentage: u8 = value.parse().map_err(|_| "Invalid percentage")?;
                    if percentage > 100 {
                        return Err("Percentage cannot exceed 100".to_string());
                    }
                    self.stateless_auth_percentage = percentage;
                },
                "emergency_rollback" => {
                    self.emergency_rollback = value.parse().map_err(|_| "Invalid boolean for emergency_rollback")?;
                    if self.emergency_rollback {
                        warn!("Emergency rollback activated via dynamic update");
                    }
                },
                "threat_detection_enabled" => {
                    self.threat_detection_enabled = value.parse().map_err(|_| "Invalid boolean for threat_detection_enabled")?;
                },
                _ => return Err(format!("Unknown feature flag: {}", key)),
            }
        }
        
        info!("Feature flags updated dynamically: {:?}", updates);
        self.validate();
        Ok(())
    }
    
    /// Validate feature flag configuration
    fn validate(&self) {
        if self.stateless_auth_enabled && !self.rs256_signature_enabled {
            warn!("Stateless auth enabled but RS256 signatures disabled - security risk!");
        }
        
        if self.stateless_auth_enabled && !self.permission_integrity_check {
            warn!("Stateless auth enabled but permission integrity checks disabled - security risk!");
        }
        
        if self.stateless_auth_percentage > 0 && !self.legacy_fallback_enabled {
            warn!("Partial rollout without legacy fallback - risky configuration!");
        }
        
        if self.emergency_rollback && self.stateless_auth_enabled {
            warn!("Emergency rollback active but stateless auth still enabled - conflicting settings");
        }
    }
    
    // Private helper methods
    
    fn parse_migration_phase() -> MigrationPhase {
        let phase_str = env::var("MIGRATION_PHASE").unwrap_or("Planning".to_string());
        
        match phase_str.as_str() {
            "Planning" => MigrationPhase::Planning,
            "Preparation" => MigrationPhase::Preparation,
            "Complete" => MigrationPhase::Complete,
            "GeneralAvailability" => MigrationPhase::GeneralAvailability,
            s if s.starts_with("Limited:") => {
                let percentage = s.strip_prefix("Limited:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                MigrationPhase::Limited(percentage.min(100))
            },
            s if s.starts_with("Expanded:") => {
                let percentage = s.strip_prefix("Expanded:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                MigrationPhase::Expanded(percentage.min(100))
            },
            _ => {
                warn!("Unknown migration phase: {}, defaulting to Planning", phase_str);
                MigrationPhase::Planning
            }
        }
    }
    
    fn parse_user_list(env_var: &str) -> Vec<String> {
        env::var(env_var)
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    }
    
    fn hash_wallet_address(&self, wallet_address: &str) -> u32 {
        // Simple hash function for consistent user assignment
        let mut hash: u32 = 0;
        for byte in wallet_address.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
    
    fn count_security_features(&self) -> u8 {
        let mut count = 0;
        if self.permission_integrity_check { count += 1; }
        if self.device_binding_enabled { count += 1; }
        if self.temporal_permissions_enabled { count += 1; }
        if self.rs256_signature_enabled { count += 1; }
        if self.threat_detection_enabled { count += 1; }
        count
    }
}

#[derive(Debug, Serialize)]
pub struct FeatureFlagSummary {
    pub timestamp: DateTime<Utc>,
    pub stateless_auth_enabled: bool,
    pub rollout_percentage: u8,
    pub migration_phase: MigrationPhase,
    pub emergency_rollback: bool,
    pub security_features_enabled: u8,
    pub monitoring_enabled: bool,
    pub legacy_fallback_available: bool,
}

/// Middleware to conditionally use stateless vs legacy auth
pub async fn conditional_auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // use crate::web::middleware::web3_auth_middleware;
    
    let flags = get_feature_flags();
    
    // Emergency rollback - use legacy for everyone
    if flags.emergency_rollback {
        return legacy_auth_middleware(request, next).await;
    }
    
    // Extract user info for decision making
    let user_info = extract_user_info_from_request(&request).await;
    
    let use_stateless = match user_info {
        Some((wallet_address, is_admin, is_beta)) => {
            flags.should_use_stateless_auth(&wallet_address, is_admin, is_beta)
        },
        None => {
            // No user info available - use percentage-based decision
            // For anonymous requests, default to legacy
            false
        }
    };
    
    if use_stateless {
        warn!("Stateless auth routing enabled but actual authentication not implemented - falling back to passthrough");

        let _headers = request.headers().clone();

        // NOTE: This is a feature flag routing layer only. Actual authentication
        // must be implemented elsewhere in the middleware chain.
        // TODO: Implement actual Web3 stateless authentication middleware here
        // when the signature compatibility issues with web3_auth_middleware are resolved.

        // Monitor stateless auth usage
        if flags.performance_monitoring_enabled {
            let start_time = std::time::Instant::now();
            let result = Ok(next.run(request).await);
            let duration = start_time.elapsed();

            // Log performance metrics
            debug!(
                duration_ms = duration.as_millis(),
                auth_type = "stateless_passthrough",
                "Feature flag routing performance"
            );

            result
        } else {
            Ok(next.run(request).await)
        }
    } else {
        debug!("Using legacy authentication middleware");
        
        // Monitor legacy auth usage
        if flags.performance_monitoring_enabled {
            let start_time = std::time::Instant::now();
            let result = legacy_auth_middleware(request, next).await;
            let duration = start_time.elapsed();
            
            // Log performance metrics
            debug!(
                duration_ms = duration.as_millis(),
                auth_type = "legacy",
                "Authentication middleware performance"
            );
            
            result
        } else {
            legacy_auth_middleware(request, next).await
        }
    }
}

/// Legacy authentication middleware (passthrough placeholder)
///
/// NOTE: This is a passthrough implementation. Actual authentication
/// must be handled by other middleware in the chain. This function exists
/// for feature flag routing compatibility.
/// TODO: Integrate with the existing authentication middleware or implement proper auth here
async fn legacy_auth_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    debug!("Legacy auth routing (passthrough) - actual auth should be handled elsewhere");
    Ok(next.run(request).await)
}

/// Extract user information from request for decision making
async fn extract_user_info_from_request<B>(
    request: &axum::extract::Request<B>
) -> Option<(String, bool, bool)> {
    use axum::http::header::AUTHORIZATION;
    
    // Try to get user info from Authorization header
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // Parse token to get user info (simplified)
                if let Some((wallet_address, is_admin, is_beta)) = parse_token_for_user_info(token) {
                    return Some((wallet_address, is_admin, is_beta));
                }
            }
        }
    }
    
    // Try to get user info from session/cookie
    // Implementation would depend on your session management
    None
}

/// Parse token to extract user information (simplified)
fn parse_token_for_user_info(_token: &str) -> Option<(String, bool, bool)> {
    // This is a simplified implementation
    // In practice, you might want to decode the JWT payload
    // without validating (since we just need user ID for decision)
    
    let parts: Vec<&str> = _token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    
    // Decode payload (simplified)
    use base64::{Engine as _, engine::general_purpose};
    if let Ok(payload) = general_purpose::STANDARD.decode(parts[1]) {
        if let Ok(claims) = serde_json::from_slice::<serde_json::Value>(&payload) {
            let wallet_address = claims.get("sub")?.as_str()?.to_string();
            let roles = claims.get("roles")?.as_array()?;
            let is_admin = roles.iter().any(|r| r.as_str() == Some("admin"));
            let is_beta = roles.iter().any(|r| r.as_str() == Some("beta"));
            
            return Some((wallet_address, is_admin, is_beta));
        }
    }
    
    None
}

// Singleton pattern for global feature flags
use std::sync::OnceLock;
static FEATURE_FLAGS: OnceLock<FeatureFlags> = OnceLock::new();

/// Get global feature flags instance
pub fn get_feature_flags() -> &'static FeatureFlags {
    FEATURE_FLAGS.get_or_init(FeatureFlags::new)
}

/// Update global feature flags (for admin interface)
pub fn update_global_feature_flags(_updates: HashMap<String, String>) -> Result<(), String> {
    // Note: This is simplified - in production you'd want proper synchronization
    // and possibly external configuration management
    warn!("Dynamic feature flag updates not fully implemented in this version");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_feature_flags() {
        let flags = FeatureFlags::default();
        
        // Security features should be enabled by default
        assert!(flags.permission_integrity_check);
        assert!(flags.threat_detection_enabled);
        assert!(flags.rs256_signature_enabled);
        
        // Auth should be disabled by default for safety
        assert!(!flags.stateless_auth_enabled);
        assert_eq!(flags.stateless_auth_percentage, 0);
    }
    
    #[test]
    fn test_user_assignment_consistency() {
        let flags = FeatureFlags {
            stateless_auth_enabled: true,
            stateless_auth_percentage: 50,
            ..Default::default()
        };
        
        // Same user should get consistent assignment
        let result1 = flags.should_use_stateless_auth("user123", false, false);
        let result2 = flags.should_use_stateless_auth("user123", false, false);
        assert_eq!(result1, result2);
        
        // Different users should get different assignments (statistically)
        let mut stateless_count = 0;
        for i in 0..100 {
            if flags.should_use_stateless_auth(&format!("user{}", i), false, false) {
                stateless_count += 1;
            }
        }
        
        // Should be roughly 50% (allow some variance)
        assert!(stateless_count >= 40 && stateless_count <= 60);
    }
    
    #[test]
    fn test_user_overrides() {
        let flags = FeatureFlags {
            stateless_auth_enabled: true,
            stateless_auth_percentage: 0, // Disabled for everyone by default
            force_stateless_users: vec!["special_user".to_string()],
            force_legacy_users: vec!["legacy_user".to_string()],
            admin_users_stateless: true,
            ..Default::default()
        };
        
        // Force stateless user should use stateless
        assert!(flags.should_use_stateless_auth("special_user", false, false));
        
        // Force legacy user should use legacy
        assert!(!flags.should_use_stateless_auth("legacy_user", false, false));
        
        // Admin should use stateless
        assert!(flags.should_use_stateless_auth("admin_user", true, false));
        
        // Regular user should use legacy (0% rollout)
        assert!(!flags.should_use_stateless_auth("regular_user", false, false));
    }
    
    #[test]
    fn test_emergency_rollback() {
        let flags = FeatureFlags {
            stateless_auth_enabled: true,
            stateless_auth_percentage: 100,
            emergency_rollback: true,
            force_stateless_users: vec!["special_user".to_string()],
            admin_users_stateless: true,
            ..Default::default()
        };
        
        // Emergency rollback should override everything
        assert!(!flags.should_use_stateless_auth("special_user", false, false));
        assert!(!flags.should_use_stateless_auth("admin_user", true, false));
        assert!(!flags.should_use_stateless_auth("regular_user", false, false));
    }
}