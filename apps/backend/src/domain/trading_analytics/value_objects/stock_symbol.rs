use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Stock Symbol Value Object
/// Represents a unique identifier for a publicly traded stock
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StockSymbol {
    symbol: String,
}

impl StockSymbol {
    /// Create new stock symbol with validation
    pub fn new(symbol: String) -> Result<Self, String> {
        let normalized = symbol.trim().to_uppercase();
        
        // Validate symbol format
        if normalized.is_empty() {
            return Err("Stock symbol cannot be empty".to_string());
        }
        
        if normalized.len() > 20 {
            return Err("Stock symbol cannot exceed 20 characters".to_string());
        }
        
        // Allow alphanumeric characters, dots, and hyphens (common in international markets)
        if !normalized.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err("Stock symbol contains invalid characters".to_string());
        }
        
        Ok(Self { symbol: normalized })
    }
    
    /// Get the symbol as string
    pub fn as_str(&self) -> &str {
        &self.symbol
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        self.symbol.clone()
    }
    
    /// Check if this is a valid US market symbol format
    pub fn is_us_market(&self) -> bool {
        // US symbols are typically 1-5 characters, all letters
        self.symbol.len() <= 5 && self.symbol.chars().all(|c| c.is_ascii_uppercase())
    }
    
    /// Check if this appears to be an international symbol
    pub fn is_international(&self) -> bool {
        // International symbols often contain dots or numbers
        self.symbol.contains('.') || self.symbol.chars().any(|c| c.is_numeric())
    }
}

impl Display for StockSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl TryFrom<String> for StockSymbol {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        StockSymbol::new(value)
    }
}

impl TryFrom<&str> for StockSymbol {
    type Error = String;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        StockSymbol::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_us_symbols() {
        let symbols = ["AAPL", "MSFT", "GOOGL", "TSLA", "BRK.A"];
        
        for symbol in &symbols {
            let stock_symbol = StockSymbol::new(symbol.to_string());
            assert!(stock_symbol.is_ok(), "Failed for symbol: {}", symbol);
        }
    }
    
    #[test]
    fn test_valid_international_symbols() {
        let symbols = ["VOW3.DE", "7203.T", "ASML.AS", "SAP.DE"];
        
        for symbol in &symbols {
            let stock_symbol = StockSymbol::new(symbol.to_string());
            assert!(stock_symbol.is_ok(), "Failed for symbol: {}", symbol);
        }
    }
    
    #[test]
    fn test_invalid_symbols() {
        let invalid_symbols = ["", "A" * 25, "AAPL@", "TEST!", " "];
        
        for symbol in &invalid_symbols {
            let stock_symbol = StockSymbol::new(symbol.to_string());
            assert!(stock_symbol.is_err(), "Should have failed for: {}", symbol);
        }
    }
    
    #[test]
    fn test_symbol_normalization() {
        let symbol = StockSymbol::new("  aapl  ".to_string()).unwrap();
        assert_eq!(symbol.as_str(), "AAPL");
    }
    
    #[test]
    fn test_market_identification() {
        let us_symbol = StockSymbol::new("AAPL".to_string()).unwrap();
        assert!(us_symbol.is_us_market());
        assert!(!us_symbol.is_international());
        
        let intl_symbol = StockSymbol::new("VOW3.DE".to_string()).unwrap();
        assert!(!intl_symbol.is_us_market());
        assert!(intl_symbol.is_international());
    }
}