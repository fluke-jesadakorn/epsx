use async_trait::async_trait;
use std::collections::HashMap;

use crate::app::ports::repositories::UsageRepository;
use crate::dom::entities::module::ModuleUsageLog;
use crate::dom::values::UserId;
use crate::dom::error::DomainError;

pub struct DieselUsageRepository {
    // TODO: Add database pool when implementing actual usage tracking
}

impl DieselUsageRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UsageRepository for DieselUsageRepository {
    async fn log_usage(&self, _usage_log: ModuleUsageLog) -> Result<(), DomainError> {
        // TODO: Implement actual usage logging
        Ok(())
    }

    async fn get_usage_stats(&self, _user_id: &UserId, _module_name: &str) -> Result<HashMap<String, i32>, DomainError> {
        // TODO: Implement actual usage stats retrieval
        Ok(HashMap::new())
    }

    async fn get_current_usage(&self, _user_id: &UserId, _module_name: &str, _quota_type: &str) -> Result<i32, DomainError> {
        // TODO: Implement actual current usage retrieval
        Ok(0)
    }
}

/// Stub implementation for backward compatibility
pub struct StubUsageRepository {}

impl StubUsageRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UsageRepository for StubUsageRepository {
    async fn log_usage(&self, _usage_log: ModuleUsageLog) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_usage_stats(&self, _user_id: &UserId, _module_name: &str) -> Result<HashMap<String, i32>, DomainError> {
        Ok(HashMap::new())
    }

    async fn get_current_usage(&self, _user_id: &UserId, _module_name: &str, _quota_type: &str) -> Result<i32, DomainError> {
        Ok(0)
    }
}


/// Stub implementation for StockRepository
pub struct StubStockRepository {}

impl StubStockRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl crate::app::ports::repositories::StockRepository for StubStockRepository {
    async fn get(&self, _symbol: &crate::dom::values::Symbol) -> Result<Option<crate::dom::entities::Stock>, crate::app::ports::repositories::RepoError> {
        Ok(None)
    }
    
    async fn save(&self, _stock: &crate::dom::entities::Stock) -> Result<(), crate::app::ports::repositories::RepoError> {
        Ok(())
    }
    
    async fn list_by_market(&self, _market: &crate::dom::values::Market) -> Result<Vec<crate::dom::entities::Stock>, crate::app::ports::repositories::RepoError> {
        Ok(vec![])
    }
    
    async fn find_top_movers(&self, _limit: u32) -> Result<Vec<crate::dom::entities::Stock>, crate::app::ports::repositories::RepoError> {
        Ok(vec![])
    }
    
    async fn find_by_symbols(&self, _symbols: &[crate::dom::values::Symbol]) -> Result<Vec<crate::dom::entities::Stock>, crate::app::ports::repositories::RepoError> {
        Ok(vec![])
    }
    
    async fn save_price_history(&self, _symbol: &crate::dom::values::Symbol, _prices: &[crate::app::ports::repositories::PricePoint]) -> Result<(), crate::app::ports::repositories::RepoError> {
        Ok(())
    }
    
    async fn get_price_history(&self, _symbol: &crate::dom::values::Symbol, _duration: chrono::Duration) -> Result<Vec<crate::app::ports::repositories::PricePoint>, crate::app::ports::repositories::RepoError> {
        Ok(vec![])
    }
    
    async fn save_batch(&self, _stocks: &[crate::dom::entities::Stock]) -> Result<(), crate::app::ports::repositories::RepoError> {
        Ok(())
    }
}