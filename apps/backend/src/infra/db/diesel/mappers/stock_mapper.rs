use chrono::Utc;

use crate::dom::entities::Stock;
use crate::dom::values::{Symbol, Market};
use crate::infra::db::diesel::models::{DieselStock, NewDieselStock, UpdateDieselStock};
use crate::app::ports::repositories::RepoError;

impl TryFrom<DieselStock> for Stock {
    type Error = RepoError;

    fn try_from(diesel_stock: DieselStock) -> Result<Self, Self::Error> {
        let symbol = Symbol::new(&diesel_stock.symbol)
            .map_err(|e| RepoError::InvalidData(format!("Invalid symbol: {}", e)))?;
        
        let market = diesel_stock.market.parse::<Market>()
            .map_err(|e| RepoError::InvalidData(format!("Invalid market: {}", e)))?;
        
        Ok(Stock::reconstruct(
            symbol,
            diesel_stock.price.into(),
            diesel_stock.volume as u64,
            market,
            diesel_stock.last_updated,
        ))
    }
}

impl From<&Stock> for NewDieselStock {
    fn from(stock: &Stock) -> Self {
        NewDieselStock {
            symbol: stock.sym().value().to_string(),
            name: stock.sym().value().to_string(), // Use symbol as name for minimal entity
            market: stock.market().to_string(),
            price: crate::infra::db::diesel::types::DieselDecimal(stock.px()),
            volume: stock.vol() as i64,
            market_cap: None, // Not available in minimal Stock entity
            sector: None, // Not available in minimal Stock entity
            industry: None, // Not available in minimal Stock entity
            last_updated: stock.ts(),
            created_at: stock.ts(), // Use same timestamp for both
        }
    }
}

impl From<&Stock> for UpdateDieselStock {
    fn from(stock: &Stock) -> Self {
        UpdateDieselStock {
            price: crate::infra::db::diesel::types::DieselDecimal(stock.px()),
            volume: stock.vol() as i64,
            market_cap: None, // Not available in minimal Stock entity
            last_updated: Utc::now(),
        }
    }
}