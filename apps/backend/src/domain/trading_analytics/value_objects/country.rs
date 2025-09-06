use chrono::{DateTime, Utc};
use std::fmt::{self, Display};
use chrono::Timelike;
use serde::{Serialize, Deserialize};

/// Country Value Object
/// Represents the country/market where a stock is traded
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Country {
    name: String,
    code: String,
    region: MarketRegion,
}

impl Country {
    /// Create new country with validation
    pub fn new(name: String) -> Result<Self, String> {
        let normalized_name = name.trim().to_string();
        
        if normalized_name.is_empty() {
            return Err("Country name cannot be empty".to_string());
        }
        
        if normalized_name.len() > 100 {
            return Err("Country name cannot exceed 100 characters".to_string());
        }
        
        let (code, region) = Self::derive_country_info(&normalized_name);
        
        Ok(Self {
            name: normalized_name,
            code,
            region,
        })
    }
    
    /// Create country with explicit code and region
    pub fn with_details(name: String, code: String, region: MarketRegion) -> Result<Self, String> {
        let normalized_name = name.trim().to_string();
        let normalized_code = code.trim().to_uppercase();
        
        if normalized_name.is_empty() {
            return Err("Country name cannot be empty".to_string());
        }
        
        if normalized_code.len() != 2 && normalized_code.len() != 3 {
            return Err("Country code must be 2 or 3 characters".to_string());
        }
        
        Ok(Self {
            name: normalized_name,
            code: normalized_code,
            region,
        })
    }
    
    /// Get country name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get country code (ISO standard when available)
    pub fn code(&self) -> &str {
        &self.code
    }
    
    /// Get market region
    pub fn region(&self) -> &MarketRegion {
        &self.region
    }
    
    /// Check if this is a developed market
    pub fn is_developed_market(&self) -> bool {
        matches!(
            self.region,
            MarketRegion::NorthAmerica | MarketRegion::Europe | MarketRegion::DevelopedAsia
        )
    }
    
    /// Check if this is an emerging market
    pub fn is_emerging_market(&self) -> bool {
        matches!(self.region, MarketRegion::EmergingMarkets)
    }
    
    /// Get market complexity level (affects regulatory requirements)
    pub fn market_complexity(&self) -> MarketComplexity {
        match self.name.as_str() {
            "United States" | "USA" | "America" => MarketComplexity::High,
            "United Kingdom" | "UK" | "Britain" => MarketComplexity::High,
            "Germany" | "Japan" | "Switzerland" => MarketComplexity::High,
            "Canada" | "Australia" | "France" | "Netherlands" => MarketComplexity::Medium,
            "China" | "India" | "Brazil" | "Russia" => MarketComplexity::Medium,
            _ => MarketComplexity::Low,
        }
    }
    
    /// Get typical trading hours offset from UTC
    pub fn typical_utc_offset(&self) -> Option<i8> {
        match self.name.as_str() {
            "United States" | "USA" | "America" => Some(-5), // EST
            "United Kingdom" | "UK" | "Britain" => Some(0),  // GMT
            "Germany" | "France" | "Netherlands" => Some(1), // CET
            "Japan" => Some(9),                               // JST
            "China" => Some(8),                               // CST
            "India" => Some(5),                               // IST (5:30, but simplified)
            "Australia" => Some(10),                          // AEST
            "Canada" => Some(-5),                             // EST (varies by region)
            _ => None,
        }
    }
    
    /// Check if market is currently likely to be open (simplified calculation)
    pub fn is_market_likely_open(&self) -> bool {
        let now = chrono::Utc::now();
        let hour = now.hour() as i8;
        
        if let Some(offset) = self.typical_utc_offset() {
            let local_hour = (hour + offset + 24) % 24;
            // Most markets are open roughly 9 AM to 4 PM local time
            local_hour >= 9 && local_hour <= 16
        } else {
            false // Unknown timezone, assume closed
        }
    }
    
    /// Derive country code and region from name
    fn derive_country_info(name: &str) -> (String, MarketRegion) {
        let name_lower = name.to_lowercase();
        
        match name_lower.as_str() {
            "united states" | "usa" | "america" => ("US".to_string(), MarketRegion::NorthAmerica),
            "canada" => ("CA".to_string(), MarketRegion::NorthAmerica),
            "mexico" => ("MX".to_string(), MarketRegion::NorthAmerica),
            
            "united kingdom" | "uk" | "britain" => ("GB".to_string(), MarketRegion::Europe),
            "germany" => ("DE".to_string(), MarketRegion::Europe),
            "france" => ("FR".to_string(), MarketRegion::Europe),
            "netherlands" => ("NL".to_string(), MarketRegion::Europe),
            "switzerland" => ("CH".to_string(), MarketRegion::Europe),
            "italy" => ("IT".to_string(), MarketRegion::Europe),
            "spain" => ("ES".to_string(), MarketRegion::Europe),
            
            "japan" => ("JP".to_string(), MarketRegion::DevelopedAsia),
            "australia" => ("AU".to_string(), MarketRegion::DevelopedAsia),
            "south korea" => ("KR".to_string(), MarketRegion::DevelopedAsia),
            "singapore" => ("SG".to_string(), MarketRegion::DevelopedAsia),
            "hong kong" => ("HK".to_string(), MarketRegion::DevelopedAsia),
            
            "china" => ("CN".to_string(), MarketRegion::EmergingMarkets),
            "india" => ("IN".to_string(), MarketRegion::EmergingMarkets),
            "brazil" => ("BR".to_string(), MarketRegion::EmergingMarkets),
            "russia" => ("RU".to_string(), MarketRegion::EmergingMarkets),
            "south africa" => ("ZA".to_string(), MarketRegion::EmergingMarkets),
            "taiwan" => ("TW".to_string(), MarketRegion::EmergingMarkets),
            
            _ => ("XX".to_string(), MarketRegion::Other),
        }
    }
    
    /// Get popular exchanges in this country
    pub fn major_exchanges(&self) -> Vec<&'static str> {
        match self.code.as_str() {
            "US" => vec!["NYSE", "NASDAQ", "AMEX"],
            "GB" => vec!["LSE"],
            "DE" => vec!["XETRA", "FSE"],
            "JP" => vec!["TSE", "OSE"],
            "CA" => vec!["TSX", "TSXV"],
            "AU" => vec!["ASX"],
            "FR" => vec!["EPA"],
            "NL" => vec!["AMS"],
            "CH" => vec!["SWX"],
            "CN" => vec!["SSE", "SZSE"],
            "HK" => vec!["HKEX"],
            "IN" => vec!["BSE", "NSE"],
            _ => vec![],
        }
    }
}

/// Market regions for geographic classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketRegion {
    NorthAmerica,
    Europe,
    DevelopedAsia,
    EmergingMarkets,
    Other,
}

impl MarketRegion {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketRegion::NorthAmerica => "north_america",
            MarketRegion::Europe => "europe",
            MarketRegion::DevelopedAsia => "developed_asia",
            MarketRegion::EmergingMarkets => "emerging_markets",
            MarketRegion::Other => "other",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            MarketRegion::NorthAmerica => "North America",
            MarketRegion::Europe => "Europe",
            MarketRegion::DevelopedAsia => "Developed Asia",
            MarketRegion::EmergingMarkets => "Emerging Markets",
            MarketRegion::Other => "Other",
        }
    }
    
    /// Get typical market characteristics for this region
    pub fn characteristics(&self) -> MarketCharacteristics {
        match self {
            MarketRegion::NorthAmerica => MarketCharacteristics {
                liquidity: LiquidityLevel::High,
                volatility: VolatilityLevel::Medium,
                regulation_level: RegulationLevel::High,
                transparency: TransparencyLevel::High,
            },
            MarketRegion::Europe => MarketCharacteristics {
                liquidity: LiquidityLevel::High,
                volatility: VolatilityLevel::Medium,
                regulation_level: RegulationLevel::High,
                transparency: TransparencyLevel::High,
            },
            MarketRegion::DevelopedAsia => MarketCharacteristics {
                liquidity: LiquidityLevel::Medium,
                volatility: VolatilityLevel::Medium,
                regulation_level: RegulationLevel::Medium,
                transparency: TransparencyLevel::Medium,
            },
            MarketRegion::EmergingMarkets => MarketCharacteristics {
                liquidity: LiquidityLevel::Low,
                volatility: VolatilityLevel::High,
                regulation_level: RegulationLevel::Medium,
                transparency: TransparencyLevel::Medium,
            },
            MarketRegion::Other => MarketCharacteristics {
                liquidity: LiquidityLevel::Low,
                volatility: VolatilityLevel::High,
                regulation_level: RegulationLevel::Low,
                transparency: TransparencyLevel::Low,
            },
        }
    }
}

/// Market complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketComplexity {
    High,
    Medium,
    Low,
}

/// Market characteristics for risk assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketCharacteristics {
    pub liquidity: LiquidityLevel,
    pub volatility: VolatilityLevel,
    pub regulation_level: RegulationLevel,
    pub transparency: TransparencyLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiquidityLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolatilityLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulationLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransparencyLevel {
    High,
    Medium,
    Low,
}

impl Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.code)
    }
}

impl TryFrom<String> for Country {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Country::new(value)
    }
}

impl TryFrom<&str> for Country {
    type Error = String;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Country::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_major_countries() {
        let countries = [
            ("United States", "US", MarketRegion::NorthAmerica),
            ("Germany", "DE", MarketRegion::Europe),
            ("Japan", "JP", MarketRegion::DevelopedAsia),
            ("China", "CN", MarketRegion::EmergingMarkets),
        ];
        
        for (name, expected_code, expected_region) in &countries {
            let country = Country::new(name.to_string()).unwrap();
            assert_eq!(country.code(), *expected_code);
            assert_eq!(country.region(), expected_region);
        }
    }
    
    #[test]
    fn test_developed_vs_emerging() {
        let us = Country::new("United States".to_string()).unwrap();
        assert!(us.is_developed_market());
        assert!(!us.is_emerging_market());
        
        let china = Country::new("China".to_string()).unwrap();
        assert!(!china.is_developed_market());
        assert!(china.is_emerging_market());
    }
    
    #[test]
    fn test_market_complexity() {
        let us = Country::new("United States".to_string()).unwrap();
        assert_eq!(us.market_complexity(), MarketComplexity::High);
        
        let unknown = Country::new("Small Country".to_string()).unwrap();
        assert_eq!(unknown.market_complexity(), MarketComplexity::Low);
    }
    
    #[test]
    fn test_utc_offsets() {
        let us = Country::new("United States".to_string()).unwrap();
        assert_eq!(us.typical_utc_offset(), Some(-5));
        
        let uk = Country::new("United Kingdom".to_string()).unwrap();
        assert_eq!(uk.typical_utc_offset(), Some(0));
        
        let japan = Country::new("Japan".to_string()).unwrap();
        assert_eq!(japan.typical_utc_offset(), Some(9));
    }
    
    #[test]
    fn test_major_exchanges() {
        let us = Country::new("United States".to_string()).unwrap();
        let exchanges = us.major_exchanges();
        assert!(exchanges.contains(&"NYSE"));
        assert!(exchanges.contains(&"NASDAQ"));
        
        let japan = Country::new("Japan".to_string()).unwrap();
        assert!(japan.major_exchanges().contains(&"TSE"));
    }
    
    #[test]
    fn test_custom_country_creation() {
        let custom = Country::with_details(
            "Custom Country".to_string(),
            "CC".to_string(),
            MarketRegion::Other,
        ).unwrap();
        
        assert_eq!(custom.name(), "Custom Country");
        assert_eq!(custom.code(), "CC");
        assert_eq!(custom.region(), &MarketRegion::Other);
    }
    
    #[test]
    fn test_invalid_countries() {
        assert!(Country::new("".to_string()).is_err());
        assert!(Country::new("A".repeat(150)).is_err());
        assert!(Country::with_details("Test".to_string(), "X".to_string(), MarketRegion::Other).is_err());
    }
    
    #[test]
    fn test_market_characteristics() {
        let na_chars = MarketRegion::NorthAmerica.characteristics();
        assert_eq!(na_chars.liquidity, LiquidityLevel::High);
        assert_eq!(na_chars.regulation_level, RegulationLevel::High);
        
        let em_chars = MarketRegion::EmergingMarkets.characteristics();
        assert_eq!(em_chars.volatility, VolatilityLevel::High);
        assert_eq!(em_chars.liquidity, LiquidityLevel::Low);
    }
}