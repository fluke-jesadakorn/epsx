use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
// use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::app::ports::repositories::{StockRepo, RepoError, PricePoint};
use crate::dom::entities::Stock;
use crate::dom::values::{Symbol, Market};
use crate::infra::db::diesel::{
    DbPool,
    schema::stocks,
    models::{DieselStock, NewDieselStock},
};

pub struct DieselStockRepo {
    pool: Arc<DbPool>,
}

impl DieselStockRepo {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StockRepo for DieselStockRepo {
    async fn get(&self, symbol: &Symbol) -> Result<Option<Stock>, RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let diesel_stock = stocks::table
            .filter(stocks::symbol.eq(symbol.value()))
            .first::<DieselStock>(&mut conn)
            .await
            .optional()
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        match diesel_stock {
            Some(diesel_stock) => {
                let stock = diesel_stock.try_into()
                    .map_err(|e| RepoError::SerializationError(format!("Failed to convert DieselStock: {:?}", e)))?;
                Ok(Some(stock))
            }
            None => Ok(None)
        }
    }
    
    async fn save(&self, stock: &Stock) -> Result<(), RepoError> {
        let mut conn = self.pool.get().await
            .map_err(|e| RepoError::ConnectionError(e.to_string()))?;
        
        let new_stock = NewDieselStock::from(stock);
        
        diesel::insert_into(stocks::table)
            .values(&new_stock)
            .on_conflict(stocks::symbol)
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn list_by_market(&self, _market: &Market) -> Result<Vec<Stock>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn find_top_movers(&self, _limit: u32) -> Result<Vec<Stock>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn find_by_symbols(&self, _symbols: &[Symbol]) -> Result<Vec<Stock>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn save_price_history(&self, _symbol: &Symbol, _prices: &[PricePoint]) -> Result<(), RepoError> {
        // Stub implementation
        Ok(())
    }
    
    async fn get_price_history(&self, _symbol: &Symbol, _duration: chrono::Duration) -> Result<Vec<PricePoint>, RepoError> {
        // Stub implementation
        Ok(vec![])
    }
    
    async fn save_batch(&self, _stocks: &[Stock]) -> Result<(), RepoError> {
        // Stub implementation
        Ok(())
    }
}