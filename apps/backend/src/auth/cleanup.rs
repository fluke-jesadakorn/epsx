/*!
 * Token Cleanup Service
 * 
 * Automated background service for cleaning up expired tokens, revoked tokens,
 * and maintaining optimal performance of the authentication system.
 */

use chrono::Utc;
use std::time::Duration;
use tokio::time::interval;
use serde::{Serialize, Deserialize};

/// Token cleanup service configuration
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub refresh_token_cleanup_interval: Duration,
    pub revoked_token_cleanup_interval: Duration,
    pub session_cleanup_interval: Duration,
    pub metrics_collection_interval: Duration,
    pub enable_cleanup: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            refresh_token_cleanup_interval: Duration::from_secs(3600), // 1 hour
            revoked_token_cleanup_interval: Duration::from_secs(1800), // 30 minutes  
            session_cleanup_interval: Duration::from_secs(7200), // 2 hours
            metrics_collection_interval: Duration::from_secs(600), // 10 minutes
            enable_cleanup: true,
        }
    }
}

/// Token cleanup service
pub struct TokenCleanupService {
    config: CleanupConfig,
    stats: CleanupStats,
}

impl TokenCleanupService {
    /// Create a new cleanup service
    pub fn new(config: CleanupConfig) -> Self {
        Self {
            config,
            stats: CleanupStats::default(),
        }
    }

    /// Start the cleanup service with background tasks
    pub async fn start(&mut self) -> Result<(), CleanupError> {
        if !self.config.enable_cleanup {
            tracing::info!("Token cleanup service disabled by configuration");
            return Ok(());
        }

        tracing::info!(
            refresh_interval = ?self.config.refresh_token_cleanup_interval,
            revoked_interval = ?self.config.revoked_token_cleanup_interval,
            session_interval = ?self.config.session_cleanup_interval,
            "Starting token cleanup service"
        );

        // Start background cleanup tasks
        self.start_refresh_token_cleanup().await;
        self.start_revoked_token_cleanup().await;
        self.start_metrics_collection().await;

        tracing::info!("Token cleanup service started successfully");
        Ok(())
    }

    /// Start refresh token cleanup task
    async fn start_refresh_token_cleanup(&mut self) {
        let mut interval = interval(self.config.refresh_token_cleanup_interval);
        
        tokio::spawn(async move {
            tracing::debug!("Refresh token cleanup task started");
            
            loop {
                interval.tick().await;
                
                match cleanup_expired_refresh_tokens().await {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            tracing::info!(
                                cleaned_count = %cleaned_count,
                                "Refresh token cleanup completed"
                            );
                        } else {
                            tracing::debug!("Refresh token cleanup: no expired tokens found");
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Refresh token cleanup failed"
                        );
                    }
                }
            }
        });
    }

    /// Start revoked token cleanup task
    async fn start_revoked_token_cleanup(&mut self) {
        let mut interval = interval(self.config.revoked_token_cleanup_interval);
        
        tokio::spawn(async move {
            tracing::debug!("Revoked token cleanup task started");
            
            loop {
                interval.tick().await;
                
                match cleanup_expired_revoked_tokens().await {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            tracing::info!(
                                cleaned_count = %cleaned_count,
                                "Revoked token cleanup completed"
                            );
                        } else {
                            tracing::debug!("Revoked token cleanup: no expired tokens found");
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Revoked token cleanup failed"
                        );
                    }
                }
            }
        });
    }

    /// Start metrics collection task
    async fn start_metrics_collection(&mut self) {
        let mut interval = interval(self.config.metrics_collection_interval);
        
        tokio::spawn(async move {
            tracing::debug!("Cleanup metrics collection task started");
            
            loop {
                interval.tick().await;
                
                match collect_cleanup_metrics().await {
                    Ok(metrics) => {
                        tracing::debug!(
                            total_refresh_tokens = %metrics.refresh_token_count,
                            total_revoked_tokens = %metrics.revoked_token_count,
                            cleanup_runs = %metrics.total_cleanup_runs,
                            "Cleanup metrics collected"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed to collect cleanup metrics"
                        );
                    }
                }
            }
        });
    }

    /// Get cleanup statistics
    pub async fn get_stats(&self) -> CleanupStats {
        self.stats.clone()
    }

    /// Perform manual cleanup of all token types
    pub async fn manual_cleanup(&mut self) -> Result<CleanupResult, CleanupError> {
        tracing::info!("Starting manual token cleanup");
        
        let start_time = Utc::now();
        
        // Clean up refresh tokens
        let refresh_cleaned = cleanup_expired_refresh_tokens().await
            .map_err(|e| CleanupError::RefreshTokenCleanupFailed { 
                message: e.to_string() 
            })?;

        // Clean up revoked tokens  
        let revoked_cleaned = cleanup_expired_revoked_tokens().await
            .map_err(|e| CleanupError::RevokedTokenCleanupFailed { 
                message: e.to_string() 
            })?;

        let end_time = Utc::now();
        let duration = end_time - start_time;

        let result = CleanupResult {
            refresh_tokens_cleaned: refresh_cleaned,
            revoked_tokens_cleaned: revoked_cleaned,
            session_tokens_cleaned: 0, // TODO: Add session cleanup
            duration_ms: duration.num_milliseconds() as u64,
            completed_at: end_time,
        };

        tracing::info!(
            refresh_cleaned = %refresh_cleaned,
            revoked_cleaned = %revoked_cleaned,
            duration_ms = %result.duration_ms,
            "Manual token cleanup completed"
        );

        Ok(result)
    }
}

/// Clean up expired refresh tokens
async fn cleanup_expired_refresh_tokens() -> Result<u32, Box<dyn std::error::Error>> {
    // Refresh token service removed for Web3-first authentication
    let cleaned_count = 0;
    Ok(cleaned_count)
}

/// Clean up expired revoked tokens  
async fn cleanup_expired_revoked_tokens() -> Result<u32, Box<dyn std::error::Error>> {
    // Token revocation service removed for Web3-first authentication
    let cleaned_count = 0;
    Ok(cleaned_count)
}

/// Collect metrics about token cleanup
async fn collect_cleanup_metrics() -> Result<CleanupMetrics, Box<dyn std::error::Error>> {
    // Token services removed for Web3-first authentication
    
    let refresh_stats = (0, 0); // (active, expired)
    let revocation_stats = (0, 0); // (revoked, cleaned)
    
    Ok(CleanupMetrics {
        refresh_token_count: refresh_stats.0,
        revoked_token_count: revocation_stats.0,
        active_refresh_tokens: refresh_stats.1,
        expired_refresh_tokens: refresh_stats.0,
        active_revocations: revocation_stats.1,
        expired_revocations: revocation_stats.0,
        total_cleanup_runs: 0, // TODO: Track cleanup run count
        last_cleanup: Utc::now(),
    })
}

// Global cleanup service instance
lazy_static::lazy_static! {
    pub static ref CLEANUP_SERVICE: tokio::sync::Mutex<TokenCleanupService> = 
        tokio::sync::Mutex::new(TokenCleanupService::new(CleanupConfig::default()));
}

/// Cleanup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub total_refresh_tokens_cleaned: u64,
    pub total_revoked_tokens_cleaned: u64,
    pub total_cleanup_runs: u64,
    pub last_cleanup: Option<chrono::DateTime<Utc>>,
}

impl Default for CleanupStats {
    fn default() -> Self {
        Self {
            total_refresh_tokens_cleaned: 0,
            total_revoked_tokens_cleaned: 0,
            total_cleanup_runs: 0,
            last_cleanup: None,
        }
    }
}

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize)]
pub struct CleanupResult {
    pub refresh_tokens_cleaned: u32,
    pub revoked_tokens_cleaned: u32,
    pub session_tokens_cleaned: u32,
    pub duration_ms: u64,
    pub completed_at: chrono::DateTime<Utc>,
}

/// Cleanup metrics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct CleanupMetrics {
    pub refresh_token_count: u32,
    pub revoked_token_count: u32,
    pub active_refresh_tokens: u32,
    pub expired_refresh_tokens: u32,
    pub active_revocations: u32,
    pub expired_revocations: u32,
    pub total_cleanup_runs: u64,
    pub last_cleanup: chrono::DateTime<Utc>,
}

/// Cleanup service errors
#[derive(Debug, thiserror::Error)]
pub enum CleanupError {
    #[error("Refresh token cleanup failed: {message}")]
    RefreshTokenCleanupFailed { message: String },
    
    #[error("Revoked token cleanup failed: {message}")]
    RevokedTokenCleanupFailed { message: String },
    
    #[error("Session cleanup failed: {message}")]
    SessionCleanupFailed { message: String },
    
    #[error("Cleanup service configuration error: {message}")]
    ConfigurationError { message: String },
}

/// Initialize and start the cleanup service
pub async fn start_cleanup_service() -> Result<(), CleanupError> {
    tracing::info!("Initializing token cleanup service");
    
    let mut service = CLEANUP_SERVICE.lock().await;
    service.start().await?;
    
    Ok(())
}

/// Get cleanup service statistics
pub async fn get_cleanup_stats() -> CleanupStats {
    let service = CLEANUP_SERVICE.lock().await;
    service.get_stats().await
}

/// Perform manual cleanup (useful for admin endpoints)
pub async fn manual_cleanup() -> Result<CleanupResult, CleanupError> {
    let mut service = CLEANUP_SERVICE.lock().await;
    service.manual_cleanup().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_config_default() {
        let config = CleanupConfig::default();
        
        assert!(config.enable_cleanup);
        assert_eq!(config.refresh_token_cleanup_interval, Duration::from_secs(3600));
        assert_eq!(config.revoked_token_cleanup_interval, Duration::from_secs(1800));
    }

    #[tokio::test]
    async fn test_cleanup_service_creation() {
        let config = CleanupConfig::default();
        let service = TokenCleanupService::new(config);
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_cleanup_runs, 0);
    }

    #[tokio::test] 
    async fn test_cleanup_service_disabled() {
        let config = CleanupConfig {
            enable_cleanup: false,
            ..CleanupConfig::default()
        };
        
        let service = TokenCleanupService::new(config);
        let result = service.start().await;
        
        assert!(result.is_ok());
    }
}