use async_trait::async_trait;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use epsx_contracts::errors::AppError;

/// Port for external market data scanning services
/// This abstraction allows the domain to access market data without depending on specific implementations
#[async_trait]
pub trait MarketDataScannerPort: Send + Sync {
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
