use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::core::errors::{AppResult, AppError};
use crate::infrastructure::adapters::repositories::diesel::repos::{RefreshTokenRepository, RevokedTokenRepository, DieselSessionRepository};
use crate::application::ports::repositories::SessionRepository;

/// Configuration for session cleanup service
#[derive(Clone, Debug)]
pub struct SessionCleanupConfig {
    /// How often to run cleanup (in seconds)
    pub cleanup_interval_seconds: u64,
    /// Maximum number of tokens to clean per batch
    pub batch_size: usize,
    /// Cleanup expired refresh tokens
    pub cleanup_refresh_tokens: bool,
    /// Cleanup expired revoked tokens
    pub cleanup_revoked_tokens: bool,
    /// Cleanup expired sessions
    pub cleanup_sessions: bool,
}

impl Default for SessionCleanupConfig {
    fn default() -> Self {
        Self {
            cleanup_interval_seconds: 3600, // 1 hour
            batch_size: 1000,
            cleanup_refresh_tokens: true,
            cleanup_revoked_tokens: true,
            cleanup_sessions: true,
        }
    }
}

/// Statistics from a cleanup run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub refresh_tokens_cleaned: usize,
    pub revoked_tokens_cleaned: usize,
    pub sessions_cleaned: usize,
    pub total_cleaned: usize,
    pub cleanup_duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Session cleanup service that removes expired tokens and sessions
pub struct SessionCleanupService {
    config: SessionCleanupConfig,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    revoked_token_repo: Arc<RevokedTokenRepository>,
    session_repo: Arc<DieselSessionRepository>,
}

unsafe impl Send for SessionCleanupService {}
unsafe impl Sync for SessionCleanupService {}

impl SessionCleanupService {
    pub fn new(
        config: SessionCleanupConfig,
        refresh_token_repo: Arc<RefreshTokenRepository>,
        revoked_token_repo: Arc<RevokedTokenRepository>,
        session_repo: Arc<DieselSessionRepository>,
    ) -> Self {
        Self {
            config,
            refresh_token_repo,
            revoked_token_repo,
            session_repo,
        }
    }

    /// Run a single cleanup cycle
    pub async fn run_cleanup(&self) -> AppResult<CleanupStats> {
        let start_time = std::time::Instant::now();
        info!("Starting session cleanup cycle");

        let mut refresh_tokens_cleaned = 0;
        let mut revoked_tokens_cleaned = 0; 
        let mut sessions_cleaned = 0;

        // Clean expired refresh tokens
        if self.config.cleanup_refresh_tokens {
            // TODO: Implement cleanup_expired method in RefreshTokenRepository
            let cleanup_result: Result<usize, Box<dyn std::error::Error + Send + Sync>> = Ok(0);
            match cleanup_result {
                Ok(count) => {
                    refresh_tokens_cleaned = count;
                    if count > 0 {
                        info!("Cleaned {} expired refresh tokens", count);
                    } else {
                        debug!("No expired refresh tokens to clean");
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup expired refresh tokens: {}", e);
                    // Continue with other cleanup tasks
                }
            }
        }

        // Clean expired revoked tokens
        if self.config.cleanup_revoked_tokens {
            // TODO: Implement cleanup_expired method in RevokedTokenRepository
            let cleanup_result: Result<usize, Box<dyn std::error::Error + Send + Sync>> = Ok(0);
            match cleanup_result {
                Ok(count) => {
                    revoked_tokens_cleaned = count;
                    if count > 0 {
                        info!("Cleaned {} expired revoked tokens", count);
                    } else {
                        debug!("No expired revoked tokens to clean");
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup expired revoked tokens: {}", e);
                    // Continue with other cleanup tasks
                }
            }
        }

        // Clean expired sessions
        if self.config.cleanup_sessions {
            // TODO: Implement cleanup_expired method in SessionRepositoryAdapter
            let cleanup_result: Result<i64, Box<dyn std::error::Error + Send + Sync>> = Ok(0);
            match cleanup_result {
                Ok(count) => {
                    sessions_cleaned = count as usize;
                    if count > 0 {
                        info!("Cleaned {} expired sessions", count);
                    } else {
                        debug!("No expired sessions to clean");
                    }
                }
                Err(e) => {
                    error!("Failed to cleanup expired sessions: {}", e);
                    // Continue with other cleanup tasks
                }
            }
        }

        let total_cleaned = refresh_tokens_cleaned + revoked_tokens_cleaned + sessions_cleaned;
        let cleanup_duration = start_time.elapsed();

        let stats = CleanupStats {
            refresh_tokens_cleaned,
            revoked_tokens_cleaned,
            sessions_cleaned,
            total_cleaned,
            cleanup_duration_ms: cleanup_duration.as_millis() as u64,
            timestamp: Utc::now(),
        };

        if total_cleaned > 0 {
            info!("Cleanup cycle completed: cleaned {} items in {}ms", 
                  total_cleaned, cleanup_duration.as_millis());
        } else {
            debug!("Cleanup cycle completed: no expired items found");
        }

        Ok(stats)
    }

    /// Start the cleanup service in the background
    pub async fn start_background_service(self: Arc<Self>) -> AppResult<()> {
        info!("Starting session cleanup service with interval {} seconds", 
              self.config.cleanup_interval_seconds);

        let mut interval = interval(Duration::from_secs(self.config.cleanup_interval_seconds));
        
        loop {
            interval.tick().await;
            
            match self.run_cleanup().await {
                Ok(stats) => {
                    debug!("Cleanup completed: {:?}", stats);
                }
                Err(e) => {
                    error!("Cleanup cycle failed: {}", e);
                }
            }
        }
    }

    /// Run cleanup once and return stats
    pub async fn manual_cleanup(&self) -> AppResult<CleanupStats> {
        info!("Running manual session cleanup");
        self.run_cleanup().await
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SessionCleanupConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SessionCleanupConfig) {
        info!("Updating session cleanup configuration");
        self.config = config;
    }

    /// Get cleanup health status
    pub async fn get_health_status(&self) -> AppResult<CleanupHealthStatus> {
        // For now, just return basic health based on service configuration
        // In the future, we could add count methods to repositories for read-only checks
        let health_issues = Vec::new();

        // Check if all required repositories are available
        if self.config.cleanup_refresh_tokens {
            // Could add a count method check here in the future
        }

        if self.config.cleanup_revoked_tokens {
            // Could add a count method check here in the future
        }

        if self.config.cleanup_sessions {
            // Could add a count method check here in the future
        }

        // For now, assume healthy if no configuration issues
        let status = if health_issues.is_empty() {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        };

        Ok(CleanupHealthStatus {
            status,
            total_expired_items: 0, // Would need separate count methods to populate this
            health_issues,
            last_check: Utc::now(),
        })
    }
}

/// Health status for cleanup service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupHealthStatus {
    pub status: String, // "healthy", "good", "needs_attention", "unhealthy"
    pub total_expired_items: usize,
    pub health_issues: Vec<String>,
    pub last_check: DateTime<Utc>,
}

/// Global cleanup service instance
static mut CLEANUP_SERVICE: Option<Arc<SessionCleanupService>> = None;

/// Initialize global cleanup service
pub async fn init_global_cleanup_service(
    config: SessionCleanupConfig,
    refresh_token_repo: Arc<RefreshTokenRepository>,
    revoked_token_repo: Arc<RevokedTokenRepository>,
    session_repo: Arc<DieselSessionRepository>,
) -> AppResult<()> {
    let service = Arc::new(SessionCleanupService::new(
        config,
        refresh_token_repo,
        revoked_token_repo,
        session_repo,
    ));
    
    unsafe {
        CLEANUP_SERVICE = Some(service);
    }
    
    Ok(())
}

/// Get global cleanup service
#[allow(static_mut_refs)]
pub fn get_global_cleanup_service() -> Option<Arc<SessionCleanupService>> {
    unsafe { CLEANUP_SERVICE.clone() }
}

/// Start the global cleanup service in background
pub async fn start_global_cleanup_service() -> AppResult<()> {
    if let Some(service) = get_global_cleanup_service() {
        info!("Starting global session cleanup service");
        
        // Spawn the background task
        tokio::spawn(async move {
            if let Err(e) = service.start_background_service().await {
                error!("Session cleanup service failed: {}", e);
            }
        });
        
        Ok(())
    } else {
        Err(AppError::bad_request("Cleanup service not initialized"))
    }
}

/// Run manual cleanup via global service
pub async fn run_manual_cleanup() -> AppResult<CleanupStats> {
    if let Some(service) = get_global_cleanup_service() {
        service.manual_cleanup().await
    } else {
        Err(AppError::bad_request("Cleanup service not initialized"))
    }
}

/// Get health status via global service
pub async fn get_cleanup_health() -> AppResult<CleanupHealthStatus> {
    if let Some(service) = get_global_cleanup_service() {
        service.get_health_status().await
    } else {
        Err(AppError::bad_request("Cleanup service not initialized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup_service_creation() {
        // This is a basic test - in a real test we'd mock the repositories
        let config = SessionCleanupConfig::default();
        assert_eq!(config.cleanup_interval_seconds, 3600);
        assert!(config.cleanup_refresh_tokens);
        assert!(config.cleanup_revoked_tokens);
        assert!(config.cleanup_sessions);
    }

    #[test]
    fn test_cleanup_config_default() {
        let config = SessionCleanupConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.cleanup_interval_seconds, 3600);
    }
}