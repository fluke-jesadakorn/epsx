/*!
 * Token Cleanup Service
 * 
 * Automated background service for cleaning up expired tokens, revoked tokens,
 * and maintaining optimal performance of the authentication system.
 */

use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use crate::infrastructure::adapter_repositories::DbPool;

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

        // Note: For Web3 nonce cleanup, use cleanup_expired_web3_nonces() with a DbPool
        // This manual cleanup focuses on legacy token types

        let end_time = Utc::now();
        let duration = end_time - start_time;

        // Increment the global cleanup counter
        increment_cleanup_runs();
        
        // Update local stats
        self.stats.total_cleanup_runs += 1;
        self.stats.total_refresh_tokens_cleaned += refresh_cleaned as u64;
        self.stats.total_revoked_tokens_cleaned += revoked_cleaned as u64;
        self.stats.last_cleanup = Some(end_time);

        let result = CleanupResult {
            refresh_tokens_cleaned: refresh_cleaned,
            revoked_tokens_cleaned: revoked_cleaned,
            session_tokens_cleaned: 0, // Web3 nonce cleanup tracked separately
            duration_ms: duration.num_milliseconds() as u64,
            completed_at: end_time,
        };

        tracing::info!(
            refresh_cleaned = %refresh_cleaned,
            revoked_cleaned = %revoked_cleaned,
            total_runs = %self.stats.total_cleanup_runs,
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

/// Clean up expired Web3 auth nonces (SIWE challenges)
/// This cleans up nonces that were generated but never used (user abandoned sign-in)
pub async fn cleanup_expired_web3_nonces(pool: &DbPool) -> anyhow::Result<u32> {
    use crate::schemas::primary::web3_auth_nonces;
    
    let now = chrono::Utc::now();
    
    let mut conn = pool.get().await
        .map_err(|e| anyhow::anyhow!("Failed to get connection from pool: {}", e))?;
    
    let deleted_count = diesel::delete(web3_auth_nonces::table)
        .filter(web3_auth_nonces::expires_at.lt(&now))
        .execute(&mut conn)
        .await?;
    
    if deleted_count > 0 {
        tracing::info!(
            deleted_count = %deleted_count,
            "Cleaned up expired Web3 auth nonces"
        );
    }
    
    Ok(deleted_count as u32)
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
        total_cleanup_runs: get_cleanup_run_count(),
        last_cleanup: Utc::now(),
    })
}

// Global cleanup service instance
lazy_static::lazy_static! {
    pub static ref CLEANUP_SERVICE: tokio::sync::Mutex<TokenCleanupService> = 
        tokio::sync::Mutex::new(TokenCleanupService::new(CleanupConfig::default()));
}

/// Global cleanup run counter - atomic for safe concurrent access
static CLEANUP_RUN_COUNT: AtomicU64 = AtomicU64::new(0);

/// Get the total number of cleanup runs
pub fn get_cleanup_run_count() -> u64 {
    AtomicU64::load(&CLEANUP_RUN_COUNT, Ordering::Relaxed)
}

/// Increment the cleanup run counter
fn increment_cleanup_runs() {
    AtomicU64::fetch_add(&CLEANUP_RUN_COUNT, 1, Ordering::Relaxed);
}

/// Cleanup statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanupStats {
    pub total_refresh_tokens_cleaned: u64,
    pub total_revoked_tokens_cleaned: u64,
    pub total_cleanup_runs: u64,
    pub last_cleanup: Option<chrono::DateTime<Utc>>,
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
        
        let mut service = TokenCleanupService::new(config);
        let result = service.start().await;
        
        assert!(result.is_ok());
    }
}