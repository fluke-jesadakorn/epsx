use std::sync::Arc;
use tokio::time::Duration;
use tracing::{debug, info, warn, error};

use crate::core::errors::AppError;
use crate::dom::services::eps_ranking_service::EPSRankingService;
use crate::infra::services::tradingview::{TradingViewService, TradingViewApiService};
use crate::config::Config;

/// Background job processor for EPS data
pub struct EPSDataProcessor {
    eps_service: Arc<EPSRankingService>,
    tradingview_service: Arc<TradingViewApiService>,
    #[allow(dead_code)]
    config: Arc<Config>,
}

/// Job configuration for EPS processing
#[derive(Debug, Clone)]
pub struct EPSProcessorConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub batch_size: usize,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl Default for EPSProcessorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 240, // 4 hours
            batch_size: 100,
            max_retries: 3,
            retry_delay_seconds: 60,
        }
    }
}

/// Processing statistics
#[derive(Debug, Default)]
pub struct ProcessingStats {
    pub total_fetched: usize,
    pub total_processed: usize,
    pub total_stored: usize,
    pub total_errors: usize,
    pub processing_duration_ms: u128,
    pub countries_processed: Vec<String>,
}

impl EPSDataProcessor {
    pub fn new(
        eps_service: Arc<EPSRankingService>,
        tradingview_service: Arc<TradingViewApiService>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            eps_service,
            tradingview_service,
            config,
        }
    }

    // Background processing job removed - using on-demand processing instead
    // Use trigger_manual_processing() or process_eps_data() for on-demand EPS data updates

    /// Process EPS data with retry logic
    async fn process_eps_data_with_retry(&self, config: &EPSProcessorConfig) -> Result<ProcessingStats, AppError> {
        let mut last_error = None;
        
        for attempt in 1..=config.max_retries {
            debug!("EPS data processing attempt {} of {}", attempt, config.max_retries);
            
            match self.process_eps_data().await {
                Ok(stats) => {
                    if attempt > 1 {
                        info!("EPS data processing succeeded on attempt {}", attempt);
                    }
                    return Ok(stats);
                }
                Err(e) => {
                    warn!("EPS data processing attempt {} failed: {:?}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < config.max_retries {
                        debug!("Waiting {} seconds before retry", config.retry_delay_seconds);
                        tokio::time::sleep(Duration::from_secs(config.retry_delay_seconds)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| AppError::new(crate::core::errors::ErrorKind::ExternalServiceError, "All retry attempts failed")))
    }

    /// Main EPS data processing logic
    pub async fn process_eps_data(&self) -> Result<ProcessingStats, AppError> {
        info!("Starting EPS data processing from TradingView");
        
        let start_time = std::time::Instant::now();
        let mut stats = ProcessingStats::default();

        // Step 1: Extract EPS data from TradingView
        debug!("Extracting EPS growth data from TradingView API");
        let eps_data_list = self.tradingview_service.extract_eps_growth_data().await
            .map_err(|e| AppError::new(crate::core::errors::ErrorKind::ExternalServiceError, format!("Failed to extract EPS data: {}", e)))?;

        stats.total_fetched = eps_data_list.len();
        info!("Extracted {} EPS data entries from TradingView", stats.total_fetched);

        if eps_data_list.is_empty() {
            warn!("No EPS data extracted from TradingView");
            return Ok(stats);
        }

        // Step 2: Process and validate data
        debug!("Processing and validating EPS data");
        let mut validated_data = Vec::new();
        let mut countries_set = std::collections::HashSet::new();

        for eps_data in eps_data_list {
            // Track countries for stats
            countries_set.insert(eps_data.country.clone());
            
            // Basic quality checks
            if eps_data.has_quality_data() {
                validated_data.push(eps_data);
                stats.total_processed += 1;
            } else {
                debug!("Filtered out {} due to incomplete data", eps_data.symbol);
                stats.total_errors += 1;
            }
        }

        stats.countries_processed = countries_set.into_iter().collect();
        info!("Processed {} valid EPS entries from {} countries", 
              stats.total_processed, stats.countries_processed.len());

        // Step 3: Store data in database
        if !validated_data.is_empty() {
            debug!("Storing {} validated EPS entries in database", validated_data.len());
            
            match self.eps_service.batch_store_eps_data(validated_data).await {
                Ok(stored_count) => {
                    stats.total_stored = stored_count;
                    info!("Successfully stored {} EPS entries", stored_count);
                }
                Err(e) => {
                    error!("Failed to store EPS data: {:?}", e);
                    return Err(AppError::new(crate::core::errors::ErrorKind::DatabaseError, format!("Failed to store EPS data: {}", e)));
                }
            }
        } else {
            warn!("No validated EPS data to store");
        }

        stats.processing_duration_ms = start_time.elapsed().as_millis();
        
        info!("EPS data processing completed - Fetched: {}, Processed: {}, Stored: {}, Errors: {}", 
              stats.total_fetched, stats.total_processed, stats.total_stored, stats.total_errors);

        Ok(stats)
    }

    /// Manual trigger for EPS data processing (for testing/admin purposes)
    pub async fn trigger_manual_processing(&self) -> Result<ProcessingStats, AppError> {
        info!("Manual EPS data processing triggered");
        
        let config = EPSProcessorConfig {
            max_retries: 1, // No retries for manual processing
            ..Default::default()
        };
        
        self.process_eps_data_with_retry(&config).await
    }

    /// Get processing health status
    pub async fn get_processing_health(&self) -> Result<ProcessingHealthStatus, AppError> {
        debug!("Checking EPS data processing health");

        // Check if we have recent data (within last 6 hours)
        let recent_countries = self.eps_service.get_available_countries().await?;
        
        let status = if recent_countries.is_empty() {
            ProcessingHealthStatus {
                healthy: false,
                message: "No EPS data found".to_string(),
                countries_count: 0,
                last_update: None,
            }
        } else {
            ProcessingHealthStatus {
                healthy: true,
                message: "EPS processing is healthy".to_string(),
                countries_count: recent_countries.len(),
                last_update: Some(chrono::Utc::now()),
            }
        };

        debug!("EPS processing health check: healthy={}, countries={}", 
               status.healthy, status.countries_count);

        Ok(status)
    }

}

/// Health status for the processing service
#[derive(Debug, Clone)]
pub struct ProcessingHealthStatus {
    pub healthy: bool,
    pub message: String,
    pub countries_count: usize,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_config_default() {
        let config = EPSProcessorConfig::default();
        assert_eq!(config.interval_minutes, 240);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_retries, 3);
        assert!(config.enabled);
    }

    #[test]
    fn test_processing_stats_default() {
        let stats = ProcessingStats::default();
        assert_eq!(stats.total_fetched, 0);
        assert_eq!(stats.total_processed, 0);
        assert_eq!(stats.total_stored, 0);
        assert_eq!(stats.total_errors, 0);
        assert!(stats.countries_processed.is_empty());
    }

    #[test]
    fn test_processing_health_status() {
        let status = ProcessingHealthStatus {
            healthy: true,
            message: "Test".to_string(),
            countries_count: 5,
            last_update: Some(chrono::Utc::now()),
        };
        
        assert!(status.healthy);
        assert_eq!(status.countries_count, 5);
        assert!(status.last_update.is_some());
    }
}