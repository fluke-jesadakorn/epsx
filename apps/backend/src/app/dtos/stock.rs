// Stock DTOs

use chrono::{ DateTime, Utc };
use rust_decimal::Decimal;
use serde::{ Serialize, Deserialize };

use crate::dom::entities::Stock;
use crate::dom::values::Symbol;

// Stock DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockDto {
  pub sym: String,
  pub px: Decimal,
  pub vol: u64,
  pub market: String,
  pub ts: DateTime<Utc>,
  pub change: Option<Decimal>,
  pub change_pct: Option<Decimal>,
}

// Get stock data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockReq {
  pub sym: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStockRes {
  pub stock: StockDto,
}

// Get multiple stocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStocksReq {
  pub syms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStocksRes {
  pub stocks: Vec<StockDto>,
}

// Search stocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStocksReq {
  pub query: String,
  pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStocksRes {
  pub results: Vec<StockSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSearchResult {
  pub sym: String,
  pub name: String,
  pub market: String,
  pub sector: Option<String>,
  pub px: Option<Decimal>,
}

// Top movers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopMoversReq {
  pub limit: u32,
  pub mover_type: String, // "gainers", "losers", "volume"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopMoversRes {
  pub stocks: Vec<StockDto>,
}

// Price history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryReq {
  pub sym: String,
  pub period: String, // "1d", "1w", "1m", "3m", "1y"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryRes {
  pub sym: String,
  pub prices: Vec<PricePointDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePointDto {
  pub px: Decimal,
  pub vol: u64,
  pub ts: DateTime<Utc>,
}

// Market status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatusReq {
  pub market: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatusRes {
  pub markets: Vec<MarketStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
  pub market: String,
  pub is_open: bool,
  pub next_open: Option<DateTime<Utc>>,
  pub next_close: Option<DateTime<Utc>>,
  pub timezone: String,
}

// Real-time subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeReq {
  pub syms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRes {
  pub success: bool,
  pub subscribed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeReq {
  pub syms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeRes {
  pub success: bool,
  pub unsubscribed: Vec<String>,
}

// Real-time data message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockUpdate {
  pub sym: String,
  pub px: Decimal,
  pub vol: u64,
  pub ts: DateTime<Utc>,
  pub change: Option<Decimal>,
  pub change_pct: Option<Decimal>,
}

// Utility implementations
impl StockDto {
  pub fn from_entity(stock: &Stock) -> Self {
    Self {
      sym: stock.sym().value().to_string(),
      px: stock.px(),
      vol: stock.vol(),
      market: stock.market().to_string(),
      ts: stock.ts(),
      change: None, // Would be calculated from previous price
      change_pct: None, // Would be calculated from previous price
    }
  }

  pub fn with_change(mut self, prev_price: Option<Decimal>) -> Self {
    if let Some(prev) = prev_price {
      if prev != Decimal::ZERO {
        self.change = Some(self.px - prev);
        self.change_pct = Some(((self.px - prev) / prev) * Decimal::from(100));
      }
    }
    self
  }
}

impl GetStockReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.sym.is_empty() {
      return Err(ValidationError::EmptySymbol);
    }

    Symbol::new(&self.sym).map_err(|_|
      ValidationError::InvalidSymbol(self.sym.clone())
    )?;

    Ok(())
  }
}

impl GetStocksReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.syms.is_empty() {
      return Err(ValidationError::NoSymbols);
    }

    if self.syms.len() > 100 {
      return Err(ValidationError::TooManySymbols(self.syms.len()));
    }

    for sym in &self.syms {
      Symbol::new(sym).map_err(|_|
        ValidationError::InvalidSymbol(sym.clone())
      )?;
    }

    Ok(())
  }
}

impl SearchStocksReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.query.trim().is_empty() {
      return Err(ValidationError::EmptyQuery);
    }

    if self.query.len() > 100 {
      return Err(ValidationError::QueryTooLong);
    }

    if let Some(limit) = self.limit {
      if limit > 1000 {
        return Err(ValidationError::LimitTooLarge(limit));
      }
    }

    Ok(())
  }
}

impl TopMoversReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.limit == 0 {
      return Err(ValidationError::InvalidLimit);
    }

    if self.limit > 1000 {
      return Err(ValidationError::LimitTooLarge(self.limit));
    }

    match self.mover_type.as_str() {
      "gainers" | "losers" | "volume" => Ok(()),
      _ => Err(ValidationError::InvalidMoverType(self.mover_type.clone())),
    }
  }
}

impl PriceHistoryReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    Symbol::new(&self.sym).map_err(|_|
      ValidationError::InvalidSymbol(self.sym.clone())
    )?;

    match self.period.as_str() {
      "1d" | "1w" | "1m" | "3m" | "6m" | "1y" | "2y" | "5y" => Ok(()),
      _ => Err(ValidationError::InvalidPeriod(self.period.clone())),
    }
  }
}

impl SubscribeReq {
  pub fn validate(&self) -> Result<(), ValidationError> {
    if self.syms.is_empty() {
      return Err(ValidationError::NoSymbols);
    }

    if self.syms.len() > 50 {
      return Err(ValidationError::TooManySymbols(self.syms.len()));
    }

    for sym in &self.syms {
      Symbol::new(sym).map_err(|_|
        ValidationError::InvalidSymbol(sym.clone())
      )?;
    }

    Ok(())
  }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
  #[error("Symbol cannot be empty")]
  EmptySymbol,

  #[error("Invalid symbol: {0}")] InvalidSymbol(String),

  #[error("No symbols provided")]
  NoSymbols,

  #[error(
    "Too many symbols: {0} (max 100 for batch, 50 for subscription)"
  )] TooManySymbols(usize),

  #[error("Search query cannot be empty")]
  EmptyQuery,

  #[error("Query too long: max 100 characters")]
  QueryTooLong,

  #[error("Limit too large: {0} (max 1000)")] LimitTooLarge(u32),

  #[error("Invalid limit: must be greater than 0")]
  InvalidLimit,

  #[error(
    "Invalid mover type: {0} (valid: gainers, losers, volume)"
  )] InvalidMoverType(String),

  #[error(
    "Invalid period: {0} (valid: 1d, 1w, 1m, 3m, 6m, 1y, 2y, 5y)"
  )] InvalidPeriod(String),
}
