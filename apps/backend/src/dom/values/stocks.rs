// Stock-related value objects

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Market {
    NASDAQ,
    NYSE,
    AMEX,
    LSE,    // London Stock Exchange
    TSE,    // Tokyo Stock Exchange
    HKE,    // Hong Kong Exchange
    SSE,    // Shanghai Stock Exchange
    SZSE,   // Shenzhen Stock Exchange
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sector {
    Technology,
    Healthcare,
    Financial,
    Energy,
    Consumer,
    Industrial,
    Materials,
    Utilities,
    RealEstate,
    Telecommunications,
}

// Symbol implementation
impl Symbol {
    pub fn new(symbol: &str) -> Result<Self, SymbolError> {
        let cleaned = symbol.trim().to_uppercase();
        
        if cleaned.is_empty() {
            return Err(SymbolError::Empty);
        }
        
        if cleaned.len() > 10 {
            return Err(SymbolError::TooLong);
        }
        
        if !cleaned.chars().all(|c| c.is_ascii_alphanumeric() || c == '.') {
            return Err(SymbolError::InvalidCharacters);
        }
        
        Ok(Self { value: cleaned })
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn is_valid(&self) -> bool {
        !self.value.is_empty() && 
        self.value.len() <= 10 &&
        self.value.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::str::FromStr for Symbol {
    type Err = SymbolError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// Market implementation
impl Market {
    pub fn name(&self) -> &'static str {
        match self {
            Market::NASDAQ => "NASDAQ",
            Market::NYSE => "New York Stock Exchange",
            Market::AMEX => "American Stock Exchange",
            Market::LSE => "London Stock Exchange",
            Market::TSE => "Tokyo Stock Exchange",
            Market::HKE => "Hong Kong Exchange",
            Market::SSE => "Shanghai Stock Exchange",
            Market::SZSE => "Shenzhen Stock Exchange",
        }
    }
    
    pub fn short_name(&self) -> &'static str {
        match self {
            Market::NASDAQ => "NASDAQ",
            Market::NYSE => "NYSE",
            Market::AMEX => "AMEX",
            Market::LSE => "LSE",
            Market::TSE => "TSE",
            Market::HKE => "HKE",
            Market::SSE => "SSE",
            Market::SZSE => "SZSE",
        }
    }
    
    pub fn timezone(&self) -> &'static str {
        match self {
            Market::NASDAQ | Market::NYSE | Market::AMEX => "America/New_York",
            Market::LSE => "Europe/London",
            Market::TSE => "Asia/Tokyo",
            Market::HKE => "Asia/Hong_Kong",
            Market::SSE | Market::SZSE => "Asia/Shanghai",
        }
    }
    
    pub fn currency(&self) -> &'static str {
        match self {
            Market::NASDAQ | Market::NYSE | Market::AMEX => "USD",
            Market::LSE => "GBP",
            Market::TSE => "JPY",
            Market::HKE => "HKD",
            Market::SSE | Market::SZSE => "CNY",
        }
    }
    
    pub fn trading_hours(&self) -> (u8, u8, u8, u8) {
        // Returns (open_hour, open_minute, close_hour, close_minute) in local time
        match self {
            Market::NASDAQ | Market::NYSE | Market::AMEX => (9, 30, 16, 0),
            Market::LSE => (8, 0, 16, 30),
            Market::TSE => (9, 0, 15, 0),
            Market::HKE => (9, 30, 16, 0),
            Market::SSE | Market::SZSE => (9, 30, 15, 0),
        }
    }
}

impl std::fmt::Display for Market {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl std::str::FromStr for Market {
    type Err = MarketError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NASDAQ" => Ok(Market::NASDAQ),
            "NYSE" => Ok(Market::NYSE),
            "AMEX" => Ok(Market::AMEX),
            "LSE" => Ok(Market::LSE),
            "TSE" => Ok(Market::TSE),
            "HKE" => Ok(Market::HKE),
            "SSE" => Ok(Market::SSE),
            "SZSE" => Ok(Market::SZSE),
            _ => Err(MarketError::UnknownMarket(s.to_string())),
        }
    }
}

// Sector implementation  
impl Sector {
    pub fn name(&self) -> &'static str {
        match self {
            Sector::Technology => "Technology",
            Sector::Healthcare => "Healthcare",
            Sector::Financial => "Financial Services",
            Sector::Energy => "Energy",
            Sector::Consumer => "Consumer Goods",
            Sector::Industrial => "Industrial",
            Sector::Materials => "Materials",
            Sector::Utilities => "Utilities",
            Sector::RealEstate => "Real Estate",
            Sector::Telecommunications => "Telecommunications",
        }
    }
    
    pub fn is_cyclical(&self) -> bool {
        matches!(self, 
            Sector::Technology | 
            Sector::Financial | 
            Sector::Energy | 
            Sector::Consumer | 
            Sector::Industrial |
            Sector::Materials
        )
    }
    
    pub fn is_defensive(&self) -> bool {
        matches!(self, 
            Sector::Healthcare | 
            Sector::Utilities | 
            Sector::Telecommunications
        )
    }
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::str::FromStr for Sector {
    type Err = SectorError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(' ', "_").as_str() {
            "technology" | "tech" => Ok(Sector::Technology),
            "healthcare" | "health" => Ok(Sector::Healthcare),
            "financial" | "financial_services" | "financials" => Ok(Sector::Financial),
            "energy" => Ok(Sector::Energy),
            "consumer" | "consumer_goods" => Ok(Sector::Consumer),
            "industrial" | "industrials" => Ok(Sector::Industrial),
            "materials" => Ok(Sector::Materials),
            "utilities" => Ok(Sector::Utilities),
            "real_estate" | "realestate" => Ok(Sector::RealEstate),
            "telecommunications" | "telecom" => Ok(Sector::Telecommunications),
            _ => Err(SectorError::UnknownSector(s.to_string())),
        }
    }
}

// Errors
#[derive(Debug, thiserror::Error)]
pub enum SymbolError {
    #[error("Symbol cannot be empty")]
    Empty,
    
    #[error("Symbol too long (max 10 characters)")]
    TooLong,
    
    #[error("Symbol contains invalid characters")]
    InvalidCharacters,
}

#[derive(Debug, thiserror::Error)]
pub enum MarketError {
    #[error("Unknown market: {0}")]
    UnknownMarket(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SectorError {
    #[error("Unknown sector: {0}")]
    UnknownSector(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_create_valid_symbol() {
        let symbol = Symbol::new("AAPL").unwrap();
        assert_eq!(symbol, "AAPL");
        assert!(symbol.is_valid());
    }
    
    #[test]
    fn should_reject_invalid_symbols() {
        assert!(Symbol::new("").is_err());
        assert!(Symbol::new("TOOLONGSYMBOL").is_err());
        assert!(Symbol::new("INVALID@").is_err());
    }
    
    #[test]
    fn should_normalize_symbol() {
        let symbol = Symbol::new(" aapl ").unwrap();
        assert_eq!(symbol, "AAPL");
    }
    
    #[test]
    fn should_parse_symbol_from_string() {
        let symbol: Symbol = "MSFT".parse().unwrap();
        assert_eq!(symbol, "MSFT");
    }
    
    #[test]
    fn should_get_market_properties() {
        let market = Market::NASDAQ;
        assert_eq!(market.name(), "NASDAQ");
        assert_eq!(market.short_name(), "NASDAQ");
        assert_eq!(market.currency(), "USD");
        assert_eq!(market.timezone(), "America/New_York");
        
        let (open_h, open_m, close_h, close_m) = market.trading_hours();
        assert_eq!((open_h, open_m, close_h, close_m), (9, 30, 16, 0));
    }
    
    #[test]
    fn should_parse_market_from_string() {
        assert_eq!("NASDAQ".parse::<Market>().unwrap(), Market::NASDAQ);
        assert_eq!("nasdaq".parse::<Market>().unwrap(), Market::NASDAQ);
        assert!("INVALID".parse::<Market>().is_err());
    }
    
    #[test]
    fn should_classify_sectors() {
        assert!(Sector::Technology.is_cyclical());
        assert!(!Sector::Technology.is_defensive());
        
        assert!(Sector::Healthcare.is_defensive());
        assert!(!Sector::Healthcare.is_cyclical());
        
        assert!(!Sector::RealEstate.is_cyclical());
        assert!(!Sector::RealEstate.is_defensive());
    }
    
    #[test]
    fn should_parse_sector_from_string() {
        assert_eq!("technology".parse::<Sector>().unwrap(), Sector::Technology);
        assert_eq!("Financial Services".parse::<Sector>().unwrap(), Sector::Financial);
        assert_eq!("real estate".parse::<Sector>().unwrap(), Sector::RealEstate);
    }
}