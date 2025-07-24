use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;

use crate::{
    app::ports::repositories::{StockRepo, PricePoint, RepoError},
    dom::{
        entities::Stock,
        values::{Symbol, Market},
    },
};

pub struct PostgresStockRepo {
    pool: PgPool,
}

impl PostgresStockRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row_to_stock(row: &sqlx::postgres::PgRow) -> Result<Stock, RepoError> {
        let symbol: String = row.try_get("symbol").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let market: String = row.try_get("market").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let current_price: Decimal = row.try_get("current_price").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let volume: i64 = row.try_get("volume").map_err(|e| RepoError::QueryError(e.to_string()))?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at").map_err(|e| RepoError::QueryError(e.to_string()))?;

        let sym = Symbol::from_str(&symbol).map_err(|_| RepoError::InvalidData(format!("Invalid symbol: {}", symbol)))?;
        let mkt = Market::from_str(&market).map_err(|_| RepoError::InvalidData(format!("Invalid market: {}", market)))?;

        Ok(Stock::reconstruct(
            sym,
            current_price,
            volume as u64,
            mkt,
            updated_at,
        ))
    }
}

#[async_trait]
impl StockRepo for PostgresStockRepo {
    async fn get(&self, symbol: &Symbol) -> Result<Option<Stock>, RepoError> {
        let query = "SELECT * FROM stocks WHERE symbol = $1";
        
        match sqlx::query(query)
            .bind(symbol.to_string())
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Some(Self::map_row_to_stock(&row)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(RepoError::QueryError(e.to_string())),
        }
    }

    async fn save(&self, stock: &Stock) -> Result<(), RepoError> {
        let query = r#"
            INSERT INTO stocks (symbol, market, current_price, volume, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (symbol) DO UPDATE SET
                market = EXCLUDED.market,
                current_price = EXCLUDED.current_price,
                volume = EXCLUDED.volume,
                updated_at = EXCLUDED.updated_at
        "#;

        sqlx::query(query)
            .bind(stock.sym().to_string())
            .bind(stock.market().to_string())
            .bind(stock.px())
            .bind(stock.vol() as i64)
            .bind(stock.ts())
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn list_by_market(&self, market: &Market) -> Result<Vec<Stock>, RepoError> {
        let query = "SELECT * FROM stocks WHERE market = $1 ORDER BY volume DESC";
        
        let rows = sqlx::query(query)
            .bind(market.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut stocks = Vec::new();
        for row in rows {
            stocks.push(Self::map_row_to_stock(&row)?);
        }

        Ok(stocks)
    }

    async fn find_top_movers(&self, limit: u32) -> Result<Vec<Stock>, RepoError> {
        let query = r#"
            SELECT * FROM stocks 
            ORDER BY volume DESC
            LIMIT $1
        "#;
        
        let rows = sqlx::query(query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut stocks = Vec::new();
        for row in rows {
            stocks.push(Self::map_row_to_stock(&row)?);
        }

        Ok(stocks)
    }

    async fn find_by_symbols(&self, symbols: &[Symbol]) -> Result<Vec<Stock>, RepoError> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }

        let symbol_strings: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        let query = "SELECT * FROM stocks WHERE symbol = ANY($1)";
        
        let rows = sqlx::query(query)
            .bind(&symbol_strings)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut stocks = Vec::new();
        for row in rows {
            stocks.push(Self::map_row_to_stock(&row)?);
        }

        Ok(stocks)
    }

    async fn save_price_history(&self, symbol: &Symbol, prices: &[PricePoint]) -> Result<(), RepoError> {
        if prices.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO stock_price_history (symbol, price, volume, timestamp) "
        );

        query_builder.push_values(prices, |mut b, price| {
            b.push_bind(symbol.to_string())
             .push_bind(price.price)
             .push_bind(price.volume as i64)
             .push_bind(price.timestamp);
        });

        query_builder.push(" ON CONFLICT (symbol, timestamp) DO UPDATE SET price = EXCLUDED.price, volume = EXCLUDED.volume");

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }

    async fn get_price_history(&self, symbol: &Symbol, duration: chrono::Duration) -> Result<Vec<PricePoint>, RepoError> {
        let start_time = Utc::now() - duration;
        let query = r#"
            SELECT price, volume, timestamp 
            FROM stock_price_history 
            WHERE symbol = $1 AND timestamp >= $2
            ORDER BY timestamp ASC
        "#;
        
        let rows = sqlx::query(query)
            .bind(symbol.to_string())
            .bind(start_time)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        let mut prices = Vec::new();
        for row in rows {
            let price: Decimal = row.try_get("price").map_err(|e| RepoError::QueryError(e.to_string()))?;
            let volume: i64 = row.try_get("volume").map_err(|e| RepoError::QueryError(e.to_string()))?;
            let timestamp: DateTime<Utc> = row.try_get("timestamp").map_err(|e| RepoError::QueryError(e.to_string()))?;

            prices.push(PricePoint {
                price,
                volume: volume as u64,
                timestamp,
            });
        }

        Ok(prices)
    }

    async fn save_batch(&self, stocks: &[Stock]) -> Result<(), RepoError> {
        if stocks.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO stocks (symbol, market, current_price, volume, updated_at) "
        );

        query_builder.push_values(stocks, |mut b, stock| {
            b.push_bind(stock.sym().to_string())
             .push_bind(stock.market().to_string())
             .push_bind(stock.px())
             .push_bind(stock.vol() as i64)
             .push_bind(stock.ts());
        });

        query_builder.push(r#" 
            ON CONFLICT (symbol) DO UPDATE SET
                market = EXCLUDED.market,
                current_price = EXCLUDED.current_price,
                volume = EXCLUDED.volume,
                updated_at = EXCLUDED.updated_at
        "#);

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::QueryError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/epsx_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn test_save_and_get_stock() {
        let pool = setup_test_pool().await;
        let repo = PostgresStockRepo::new(pool);

        let symbol = Symbol::from_str("AAPL").unwrap();
        let market = Market::from_str("NASDAQ").unwrap();
        let stock = Stock::new(
            symbol.clone(),
            dec!(150.50),
            1000000,
            market,
        );

        repo.save(&stock).await.unwrap();

        let retrieved = repo.get(&symbol).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_stock = retrieved.unwrap();
        assert_eq!(retrieved_stock.sym(), &symbol);
        assert_eq!(retrieved_stock.px(), dec!(150.50));
    }
}