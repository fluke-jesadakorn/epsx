// Stub Stock Repository - Stock functionality moved to EPS analytics system
use async_trait::async_trait;
use crate::app::ports::repositories::{StockRepository, RepoError, PricePoint};
use crate::dom::entities::stock::Stock;
use crate::dom::values::{Symbol, Market};

pub struct StubStockRepository;

impl StubStockRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StockRepository for StubStockRepository {
    async fn get(&self, _symbol: &Symbol) -> Result<Option<Stock>, RepoError> {
        Ok(None)
    }

    async fn save(&self, _stock: &Stock) -> Result<(), RepoError> {
        Ok(())
    }

    async fn list_by_market(&self, _market: &Market) -> Result<Vec<Stock>, RepoError> {
        Ok(vec![])
    }

    async fn find_top_movers(&self, _limit: u32) -> Result<Vec<Stock>, RepoError> {
        Ok(vec![])
    }

    async fn find_by_symbols(&self, _symbols: &[Symbol]) -> Result<Vec<Stock>, RepoError> {
        Ok(vec![])
    }

    async fn save_price_history(&self, _symbol: &Symbol, _prices: &[PricePoint]) -> Result<(), RepoError> {
        Ok(())
    }

    async fn get_price_history(&self, _symbol: &Symbol, _duration: chrono::Duration) -> Result<Vec<PricePoint>, RepoError> {
        Ok(vec![])
    }

    async fn save_batch(&self, _stocks: &[Stock]) -> Result<(), RepoError> {
        Ok(())
    }
}