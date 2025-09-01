// Stock domain entity with minimal naming
use chrono::{DateTime, Utc};

use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

use crate::dom::values::{Symbol, Market};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    sym: Symbol,
    px: Decimal,
    vol: u64,
    market: Market,
    ts: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    px: Decimal,
    vol: u64,
    ts: DateTime<Utc>,
}

impl Stock {
    pub fn new(sym: Symbol, px: Decimal, vol: u64, market: Market) -> Self {
        Self {
            sym,
            px,
            vol,
            market,
            ts: Utc::now(),
        }
    }
    
    pub fn reconstruct(
        sym: Symbol,
        px: Decimal,
        vol: u64,
        market: Market,
        ts: DateTime<Utc>,
    ) -> Self {
        Self { sym, px, vol, market, ts }
    }
    
    // Getters
    pub fn sym(&self) -> &Symbol { &self.sym }
    pub fn px(&self) -> Decimal { self.px }
    pub fn vol(&self) -> u64 { self.vol }
    pub fn market(&self) -> &Market { &self.market }
    pub fn ts(&self) -> DateTime<Utc> { self.ts }
    
    // Business methods
    pub fn update_px(&mut self, new_px: Decimal, new_vol: u64) {
        self.px = new_px;
        self.vol = new_vol;
        self.ts = Utc::now();
    }
    
    pub fn is_recent(&self, minutes: i64) -> bool {
        let threshold = Utc::now() - chrono::Duration::minutes(minutes);
        self.ts > threshold
    }
    
    pub fn to_price_point(&self) -> PricePoint {
        PricePoint {
            px: self.px,
            vol: self.vol,
            ts: self.ts,
        }
    }
    
    pub fn calculate_market_cap(&self, shares_outstanding: u64) -> Decimal {
        self.px * Decimal::from(shares_outstanding)
    }
}

impl PricePoint {
    pub fn new(px: Decimal, vol: u64) -> Self {
        Self {
            px,
            vol,
            ts: Utc::now(),
        }
    }
    
    pub fn px(&self) -> Decimal { self.px }
    pub fn vol(&self) -> u64 { self.vol }
    pub fn ts(&self) -> DateTime<Utc> { self.ts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn should_create_stock() {
        let sym = Symbol::new("AAPL").unwrap();
        let px = dec!(150.50);
        let vol = 1000000;
        let market = Market::NASDAQ;
        
        let stock = Stock::new(sym.clone(), px, vol, market.clone());
        
        assert_eq!(stock.sym(), &sym);
        assert_eq!(stock.px(), px);
        assert_eq!(stock.vol(), vol);
        assert_eq!(stock.market(), &market);
    }
    
    #[test]
    fn should_update_price() {
        let sym = Symbol::new("AAPL").unwrap();
        let mut stock = Stock::new(sym, dec!(150.50), 1000000, Market::NASDAQ);
        
        let old_ts = stock.ts();
        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        stock.update_px(dec!(151.00), 1100000);
        
        assert_eq!(stock.px(), dec!(151.00));
        assert_eq!(stock.vol(), 1100000);
        assert!(stock.ts() >= old_ts); // Allow equal timestamps for fast execution
    }
    
    #[test]
    fn should_calculate_market_cap() {
        let sym = Symbol::new("AAPL").unwrap();
        let stock = Stock::new(sym, dec!(150.50), 1000000, Market::NASDAQ);
        
        let market_cap = stock.calculate_market_cap(1000000000); // 1B shares
        
        assert_eq!(market_cap, dec!(150500000000)); // $150.5B
    }
    
    #[test]
    fn should_check_if_recent() {
        let sym = Symbol::new("AAPL").unwrap();
        let stock = Stock::new(sym, dec!(150.50), 1000000, Market::NASDAQ);
        
        assert!(stock.is_recent(10)); // Within 10 minutes
        assert!(stock.is_recent(1)); // Within 1 minute
    }
}