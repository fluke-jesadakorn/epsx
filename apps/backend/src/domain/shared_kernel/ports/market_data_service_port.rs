use async_trait::async_trait;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use crate::core::errors::AppError;

/// Port for external market data services
/// This abstraction allows the domain to access market data without depending on specific implementations
#[async_trait]
pub trait MarketDataServicePort: Send + Sync {
    /// Fetch EPS data for a specific symbol
    async fn fetch_eps_data(&self, symbol: &str) -> Result<Option<EPSGrowthData>, AppError>;
    
    /// Fetch EPS data for multiple symbols
    async fn fetch_batch_eps_data(&self, symbols: &[String]) -> Result<Vec<EPSGrowthData>, AppError>;
    
    /// Check if the service is available/healthy
    async fn health_check(&self) -> Result<(), AppError>;
    
    /// Get available countries from the data provider
    async fn get_countries(&self) -> Result<Vec<String>, AppError>;
    
    /// Get available sectors for a specific country
    async fn get_sectors_by_country(&self, country: Option<&str>) -> Result<Vec<String>, AppError>;
    
    /// Search for symbols by name or ticker
    async fn search_symbols(&self, query: &str, limit: Option<usize>) -> Result<Vec<String>, AppError>;
}

/// Configuration for market data service
#[derive(Debug, Clone)]
pub struct MarketDataConfig {
    pub timeout_seconds: u64,
    pub max_concurrent_requests: usize,
    pub rate_limit_per_minute: u32,
    pub cache_ttl_seconds: u64,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_concurrent_requests: 10,
            rate_limit_per_minute: 100,
            cache_ttl_seconds: 300, // 5 minutes
        }
    }
}