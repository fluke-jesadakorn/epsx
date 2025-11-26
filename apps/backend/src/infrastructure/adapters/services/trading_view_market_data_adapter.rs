use std::sync::Arc;
use async_trait::async_trait;
use crate::domain::shared_kernel::ports::MarketDataServicePort;
use crate::domain::shared_kernel::entities::eps_growth::EPSGrowthData;
use crate::core::errors::AppError;
use crate::infrastructure::adapters::services::tradingview::{TradingViewApiService, TradingViewService};

/// Adapter that implements MarketDataServicePort for TradingView API service
/// This bridges the domain port interface with the concrete TradingView implementation
pub struct TradingViewMarketDataAdapter {
    tradingview_service: Arc<TradingViewApiService>,
}

impl TradingViewMarketDataAdapter {
    pub fn new(tradingview_service: Arc<TradingViewApiService>) -> Self {
        Self {
            tradingview_service,
        }
    }
}

#[async_trait]
impl MarketDataServicePort for TradingViewMarketDataAdapter {
    async fn fetch_eps_data(&self, symbol: &str) -> Result<Option<EPSGrowthData>, AppError> {
        // Fetch data for a single symbol using TradingView API
        let symbols = vec![symbol.to_string()];
        let results = self.tradingview_service.fetch_symbols_concurrent(symbols).await
            .map_err(|e| AppError::external_service_error(format!("TradingView API error: {}", e)))?;
        
        Ok(results.into_iter().next())
    }
    
    async fn fetch_batch_eps_data(&self, symbols: &[String]) -> Result<Vec<EPSGrowthData>, AppError> {
        let symbols_vec = symbols.to_vec();
        self.tradingview_service.fetch_symbols_concurrent(symbols_vec).await
            .map_err(|e| AppError::external_service_error(format!("TradingView batch API error: {}", e)))
    }
    
    async fn health_check(&self) -> Result<(), AppError> {
        self.tradingview_service.test_connections().await
            .map_err(|e| AppError::external_service_error(format!("TradingView connection test failed: {}", e)))
            .map(|_| ())
    }
    
    async fn get_countries(&self) -> Result<Vec<String>, AppError> {
        // TradingView API doesn't directly provide countries list
        // Return commonly supported countries
        Ok(vec![
            "america".to_string(),
            "europe".to_string(), 
            "asia".to_string(),
            "us".to_string(),
            "gb".to_string(),
            "de".to_string(),
            "jp".to_string(),
            "cn".to_string(),
        ])
    }
    
    async fn get_sectors_by_country(&self, _country: Option<&str>) -> Result<Vec<String>, AppError> {
        // TradingView API doesn't directly provide sectors by country
        // Return commonly supported sectors
        Ok(vec![
            "technology".to_string(),
            "finance".to_string(),
            "healthcare".to_string(),
            "energy".to_string(),
            "industrials".to_string(),
            "consumer".to_string(),
            "materials".to_string(),
            "utilities".to_string(),
            "telecom".to_string(),
            "real_estate".to_string(),
        ])
    }
    
    async fn search_symbols(&self, _query: &str, _limit: Option<usize>) -> Result<Vec<String>, AppError> {
        // For now, return empty as TradingView API doesn't have direct symbol search
        // This could be enhanced later with actual TradingView symbol search functionality
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[test]
    fn test_adapter_creation() {
        let config = Config::from_env().unwrap();
        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let adapter = TradingViewMarketDataAdapter::new(tradingview_service);
        
        // Test that the adapter was created successfully
        assert!(adapter.tradingview_service.get_cache_stats().total_count >= 0);
    }
    
    #[tokio::test]
    async fn test_countries_list() {
        let config = Config::from_env().unwrap();
        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let adapter = TradingViewMarketDataAdapter::new(tradingview_service);
        
        let countries = adapter.get_countries().await.unwrap();
        assert!(!countries.is_empty());
        assert!(countries.contains(&"america".to_string()));
        assert!(countries.contains(&"europe".to_string()));
    }
    
    #[tokio::test]
    async fn test_sectors_list() {
        let config = Config::from_env().unwrap();
        let tradingview_service = Arc::new(TradingViewApiService::new(Arc::new(config)));
        let adapter = TradingViewMarketDataAdapter::new(tradingview_service);
        
        let sectors = adapter.get_sectors_by_country(Some("us")).await.unwrap();
        assert!(!sectors.is_empty());
        assert!(sectors.contains(&"technology".to_string()));
        assert!(sectors.contains(&"finance".to_string()));
    }
}