use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Market Sector Value Object
/// Represents the industry sector of a stock (e.g., Technology, Healthcare)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketSector {
    sector: String,
    category: SectorCategory,
}

impl MarketSector {
    /// Create new market sector with validation
    pub fn new(sector: String) -> Result<Self, String> {
        let normalized = sector.trim().to_string();
        
        if normalized.is_empty() {
            return Err("Market sector cannot be empty".to_string());
        }
        
        if normalized.len() > 100 {
            return Err("Market sector name cannot exceed 100 characters".to_string());
        }
        
        let category = Self::classify_sector(&normalized);
        
        Ok(Self {
            sector: normalized,
            category,
        })
    }
    
    /// Get the sector name
    pub fn name(&self) -> &str {
        &self.sector
    }
    
    /// Get the sector category
    pub fn category(&self) -> &SectorCategory {
        &self.category
    }
    
    /// Check if this is a technology sector
    pub fn is_technology(&self) -> bool {
        matches!(self.category, SectorCategory::Technology)
    }
    
    /// Check if this is a defensive sector (typically stable during downturns)
    pub fn is_defensive(&self) -> bool {
        matches!(
            self.category,
            SectorCategory::Utilities
                | SectorCategory::ConsumerStaples
                | SectorCategory::Healthcare
                | SectorCategory::RealEstate
        )
    }
    
    /// Check if this is a cyclical sector (sensitive to economic cycles)
    pub fn is_cyclical(&self) -> bool {
        matches!(
            self.category,
            SectorCategory::Materials
                | SectorCategory::Industrials
                | SectorCategory::ConsumerDiscretionary
                | SectorCategory::Energy
        )
    }
    
    /// Get growth potential rating based on historical patterns
    pub fn growth_potential(&self) -> GrowthPotential {
        match self.category {
            SectorCategory::Technology => GrowthPotential::High,
            SectorCategory::Healthcare => GrowthPotential::High,
            SectorCategory::ConsumerDiscretionary => GrowthPotential::Medium,
            SectorCategory::Communication => GrowthPotential::Medium,
            SectorCategory::Industrials => GrowthPotential::Medium,
            SectorCategory::Financials => GrowthPotential::Medium,
            SectorCategory::Materials => GrowthPotential::Low,
            SectorCategory::Energy => GrowthPotential::Low,
            SectorCategory::ConsumerStaples => GrowthPotential::Low,
            SectorCategory::Utilities => GrowthPotential::Low,
            SectorCategory::RealEstate => GrowthPotential::Low,
            SectorCategory::Other => GrowthPotential::Unknown,
        }
    }
    
    /// Get volatility level typically associated with this sector
    pub fn typical_volatility(&self) -> VolatilityLevel {
        match self.category {
            SectorCategory::Technology => VolatilityLevel::High,
            SectorCategory::Energy => VolatilityLevel::High,
            SectorCategory::Materials => VolatilityLevel::High,
            SectorCategory::ConsumerDiscretionary => VolatilityLevel::Medium,
            SectorCategory::Industrials => VolatilityLevel::Medium,
            SectorCategory::Financials => VolatilityLevel::Medium,
            SectorCategory::Communication => VolatilityLevel::Medium,
            SectorCategory::Healthcare => VolatilityLevel::Low,
            SectorCategory::ConsumerStaples => VolatilityLevel::Low,
            SectorCategory::Utilities => VolatilityLevel::Low,
            SectorCategory::RealEstate => VolatilityLevel::Low,
            SectorCategory::Other => VolatilityLevel::Unknown,
        }
    }
    
    /// Classify sector string into standard category
    fn classify_sector(sector: &str) -> SectorCategory {
        let sector_lower = sector.to_lowercase();
        
        // Technology patterns
        if sector_lower.contains("technolog")
            || sector_lower.contains("software")
            || sector_lower.contains("internet")
            || sector_lower.contains("semiconduct")
            || sector_lower.contains("computer")
        {
            return SectorCategory::Technology;
        }
        
        // Healthcare patterns
        if sector_lower.contains("health")
            || sector_lower.contains("medical")
            || sector_lower.contains("pharmaceut")
            || sector_lower.contains("biotechnol")
            || sector_lower.contains("drug")
        {
            return SectorCategory::Healthcare;
        }
        
        // Financial patterns
        if sector_lower.contains("financ")
            || sector_lower.contains("bank")
            || sector_lower.contains("insurance")
            || sector_lower.contains("investment")
        {
            return SectorCategory::Financials;
        }
        
        // Energy patterns
        if sector_lower.contains("energy")
            || sector_lower.contains("oil")
            || sector_lower.contains("gas")
            || sector_lower.contains("petroleum")
        {
            return SectorCategory::Energy;
        }
        
        // Consumer patterns
        if sector_lower.contains("consumer discretionary")
            || sector_lower.contains("retail")
            || sector_lower.contains("automotive")
        {
            return SectorCategory::ConsumerDiscretionary;
        }
        
        if sector_lower.contains("consumer staples")
            || sector_lower.contains("food")
            || sector_lower.contains("beverage")
        {
            return SectorCategory::ConsumerStaples;
        }
        
        // Industrial patterns
        if sector_lower.contains("industrial")
            || sector_lower.contains("manufacturing")
            || sector_lower.contains("aerospace")
            || sector_lower.contains("defense")
        {
            return SectorCategory::Industrials;
        }
        
        // Materials patterns
        if sector_lower.contains("materials")
            || sector_lower.contains("mining")
            || sector_lower.contains("chemical")
            || sector_lower.contains("metal")
        {
            return SectorCategory::Materials;
        }
        
        // Utilities patterns
        if sector_lower.contains("utilit")
            || sector_lower.contains("electric")
            || sector_lower.contains("water")
            || sector_lower.contains("power")
        {
            return SectorCategory::Utilities;
        }
        
        // Real Estate patterns
        if sector_lower.contains("real estate") || sector_lower.contains("reit") {
            return SectorCategory::RealEstate;
        }
        
        // Communication patterns
        if sector_lower.contains("communication")
            || sector_lower.contains("telecom")
            || sector_lower.contains("media")
        {
            return SectorCategory::Communication;
        }
        
        SectorCategory::Other
    }
}

/// Standard sector categories (GICS-inspired)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SectorCategory {
    Technology,
    Healthcare,
    Financials,
    ConsumerDiscretionary,
    ConsumerStaples,
    Industrials,
    Materials,
    Energy,
    Utilities,
    RealEstate,
    Communication,
    Other,
}

impl SectorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SectorCategory::Technology => "technology",
            SectorCategory::Healthcare => "healthcare",
            SectorCategory::Financials => "financials",
            SectorCategory::ConsumerDiscretionary => "consumer_discretionary",
            SectorCategory::ConsumerStaples => "consumer_staples",
            SectorCategory::Industrials => "industrials",
            SectorCategory::Materials => "materials",
            SectorCategory::Energy => "energy",
            SectorCategory::Utilities => "utilities",
            SectorCategory::RealEstate => "real_estate",
            SectorCategory::Communication => "communication",
            SectorCategory::Other => "other",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            SectorCategory::Technology => "Technology",
            SectorCategory::Healthcare => "Healthcare",
            SectorCategory::Financials => "Financials",
            SectorCategory::ConsumerDiscretionary => "Consumer Discretionary",
            SectorCategory::ConsumerStaples => "Consumer Staples",
            SectorCategory::Industrials => "Industrials",
            SectorCategory::Materials => "Materials",
            SectorCategory::Energy => "Energy",
            SectorCategory::Utilities => "Utilities",
            SectorCategory::RealEstate => "Real Estate",
            SectorCategory::Communication => "Communication",
            SectorCategory::Other => "Other",
        }
    }
    
    /// Get all standard sector categories
    pub fn all() -> Vec<SectorCategory> {
        vec![
            SectorCategory::Technology,
            SectorCategory::Healthcare,
            SectorCategory::Financials,
            SectorCategory::ConsumerDiscretionary,
            SectorCategory::ConsumerStaples,
            SectorCategory::Industrials,
            SectorCategory::Materials,
            SectorCategory::Energy,
            SectorCategory::Utilities,
            SectorCategory::RealEstate,
            SectorCategory::Communication,
            SectorCategory::Other,
        ]
    }
}

/// Growth potential classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthPotential {
    High,
    Medium,
    Low,
    Unknown,
}

/// Volatility level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolatilityLevel {
    High,
    Medium,
    Low,
    Unknown,
}

impl Display for MarketSector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.sector)
    }
}

impl TryFrom<String> for MarketSector {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        MarketSector::new(value)
    }
}

impl TryFrom<&str> for MarketSector {
    type Error = String;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MarketSector::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sector_classification() {
        let tech_sectors = [
            "Technology",
            "Software",
            "Internet Technology",
            "Semiconductor Equipment",
            "Computer Hardware",
        ];
        
        for sector_name in &tech_sectors {
            let sector = MarketSector::new(sector_name.to_string()).unwrap();
            assert_eq!(sector.category(), &SectorCategory::Technology);
            assert!(sector.is_technology());
        }
    }
    
    #[test]
    fn test_healthcare_classification() {
        let healthcare_sectors = [
            "Healthcare",
            "Medical Devices",
            "Pharmaceuticals",
            "Biotechnology",
            "Drug Manufacturers",
        ];
        
        for sector_name in &healthcare_sectors {
            let sector = MarketSector::new(sector_name.to_string()).unwrap();
            assert_eq!(sector.category(), &SectorCategory::Healthcare);
        }
    }
    
    #[test]
    fn test_defensive_sectors() {
        let defensive = MarketSector::new("Utilities".to_string()).unwrap();
        assert!(defensive.is_defensive());
        assert!(!defensive.is_cyclical());
        
        let staples = MarketSector::new("Consumer Staples".to_string()).unwrap();
        assert!(staples.is_defensive());
    }
    
    #[test]
    fn test_cyclical_sectors() {
        let materials = MarketSector::new("Materials".to_string()).unwrap();
        assert!(materials.is_cyclical());
        assert!(!materials.is_defensive());
        
        let industrials = MarketSector::new("Industrials".to_string()).unwrap();
        assert!(industrials.is_cyclical());
    }
    
    #[test]
    fn test_growth_potential() {
        let tech = MarketSector::new("Technology".to_string()).unwrap();
        assert_eq!(tech.growth_potential(), GrowthPotential::High);
        
        let utilities = MarketSector::new("Utilities".to_string()).unwrap();
        assert_eq!(utilities.growth_potential(), GrowthPotential::Low);
    }
    
    #[test]
    fn test_volatility_levels() {
        let tech = MarketSector::new("Technology".to_string()).unwrap();
        assert_eq!(tech.typical_volatility(), VolatilityLevel::High);
        
        let healthcare = MarketSector::new("Healthcare".to_string()).unwrap();
        assert_eq!(healthcare.typical_volatility(), VolatilityLevel::Low);
    }
    
    #[test]
    fn test_invalid_sectors() {
        assert!(MarketSector::new("".to_string()).is_err());
        assert!(MarketSector::new("A".repeat(150)).is_err());
    }
    
    #[test]
    fn test_all_categories() {
        let all_categories = SectorCategory::all();
        assert_eq!(all_categories.len(), 12);
        assert!(all_categories.contains(&SectorCategory::Technology));
        assert!(all_categories.contains(&SectorCategory::Other));
    }
}