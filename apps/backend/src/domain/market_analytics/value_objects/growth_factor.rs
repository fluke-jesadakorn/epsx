use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Growth Factor Value Object
/// Represents growth percentage (e.g., quarter-over-quarter EPS growth)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrowthFactor {
    percentage: f64,
}

impl GrowthFactor {
    /// Create new growth factor with validation
    pub fn new(percentage: f64) -> Result<Self, String> {
        if percentage.is_nan() || percentage.is_infinite() {
            return Err("Growth factor cannot be NaN or infinite".to_string());
        }
        
        // Allow reasonable range for growth (-1000% to +10000%)
        if !(-1000.0..=10000.0).contains(&percentage) {
            return Err("Growth factor is outside reasonable range (-1000% to +10000%)".to_string());
        }
        
        Ok(Self { percentage })
    }
    
    /// Get the raw percentage value
    pub fn percentage(&self) -> f64 {
        self.percentage
    }
    
    /// Get the value (alias for percentage)
    pub fn value(&self) -> f64 {
        self.percentage
    }
    
    /// Get as decimal (e.g., 50% -> 0.5)
    pub fn as_decimal(&self) -> f64 {
        self.percentage / 100.0
    }
    
    /// Check if growth is positive
    pub fn is_positive_growth(&self) -> bool {
        self.percentage > 0.0
    }
    
    /// Check if growth is negative (decline)
    pub fn is_decline(&self) -> bool {
        self.percentage < 0.0
    }
    
    /// Check if growth is flat (near zero)
    pub fn is_flat(&self) -> bool {
        self.percentage.abs() < 0.1 // Within 0.1%
    }
    
    /// Classify growth rate
    pub fn classify(&self) -> GrowthClassification {
        match self.percentage {
            p if p > 100.0 => GrowthClassification::Explosive,
            p if p > 50.0 => GrowthClassification::Strong,
            p if p > 15.0 => GrowthClassification::Moderate,
            p if p > 0.1 => GrowthClassification::Weak,
            p if p > -0.1 => GrowthClassification::Flat,
            p if p > -15.0 => GrowthClassification::SlightDecline,
            p if p > -50.0 => GrowthClassification::Decline,
            _ => GrowthClassification::Collapse,
        }
    }
    
    /// Get investment attractiveness score (0-100)
    pub fn investment_score(&self) -> u8 {
        match self.classify() {
            GrowthClassification::Explosive => 100,
            GrowthClassification::Strong => 85,
            GrowthClassification::Moderate => 70,
            GrowthClassification::Weak => 55,
            GrowthClassification::Flat => 40,
            GrowthClassification::SlightDecline => 25,
            GrowthClassification::Decline => 10,
            GrowthClassification::Collapse => 0,
        }
    }
    
    /// Compare with another growth factor to determine relative performance
    pub fn compare_to(&self, other: &GrowthFactor) -> GrowthComparison {
        let diff = self.percentage - other.percentage;
        
        match diff {
            d if d > 10.0 => GrowthComparison::SignificantlyBetter,
            d if d > 2.0 => GrowthComparison::Better,
            d if d > -2.0 => GrowthComparison::Similar,
            d if d > -10.0 => GrowthComparison::Worse,
            _ => GrowthComparison::SignificantlyWorse,
        }
    }
    
    /// Format for display
    pub fn format_display(&self) -> String {
        if self.percentage >= 0.0 {
            format!("+{:.1}%", self.percentage)
        } else {
            format!("{:.1}%", self.percentage)
        }
    }
    
    /// Create from basis points (1 basis point = 0.01%)
    pub fn from_basis_points(bp: i32) -> Result<Self, String> {
        let percentage = bp as f64 / 100.0;
        Self::new(percentage)
    }
    
    /// Convert to basis points
    pub fn to_basis_points(&self) -> i32 {
        (self.percentage * 100.0).round() as i32
    }
}

/// Growth Classification based on percentage ranges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthClassification {
    Explosive,      // >100%
    Strong,         // 50-100%
    Moderate,       // 15-50%
    Weak,          // 0-15%
    Flat,          // -0.1% to +0.1%
    SlightDecline, // -15% to -0.1%
    Decline,       // -50% to -15%
    Collapse,      // <-50%
}

impl GrowthClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            GrowthClassification::Explosive => "explosive",
            GrowthClassification::Strong => "strong",
            GrowthClassification::Moderate => "moderate",
            GrowthClassification::Weak => "weak",
            GrowthClassification::Flat => "flat",
            GrowthClassification::SlightDecline => "slight_decline",
            GrowthClassification::Decline => "decline",
            GrowthClassification::Collapse => "collapse",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            GrowthClassification::Explosive => "Explosive Growth (>100%)",
            GrowthClassification::Strong => "Strong Growth (50-100%)",
            GrowthClassification::Moderate => "Moderate Growth (15-50%)",
            GrowthClassification::Weak => "Weak Growth (0-15%)",
            GrowthClassification::Flat => "Flat (±0.1%)",
            GrowthClassification::SlightDecline => "Slight Decline (-0.1% to -15%)",
            GrowthClassification::Decline => "Decline (-15% to -50%)",
            GrowthClassification::Collapse => "Collapse (<-50%)",
        }
    }
}

/// Growth Comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthComparison {
    SignificantlyBetter,
    Better,
    Similar,
    Worse,
    SignificantlyWorse,
}

impl Display for GrowthFactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.percentage)
    }
}

impl TryFrom<f64> for GrowthFactor {
    type Error = String;
    
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        GrowthFactor::new(value)
    }
}

// Allow comparison operations
impl PartialOrd for GrowthFactor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.percentage.partial_cmp(&other.percentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_growth_factors() {
        let values = [0.0, 15.5, -25.0, 150.0, -75.5];
        
        for value in &values {
            let growth = GrowthFactor::new(*value);
            assert!(growth.is_ok(), "Failed for growth factor: {}", value);
        }
    }
    
    #[test]
    fn test_invalid_growth_factors() {
        let invalid_values = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 20000.0, -2000.0];
        
        for value in &invalid_values {
            let growth = GrowthFactor::new(*value);
            assert!(growth.is_err(), "Should have failed for growth factor: {}", value);
        }
    }
    
    #[test]
    fn test_growth_classification() {
        assert_eq!(GrowthFactor::new(150.0).unwrap().classify(), GrowthClassification::Explosive);
        assert_eq!(GrowthFactor::new(75.0).unwrap().classify(), GrowthClassification::Strong);
        assert_eq!(GrowthFactor::new(25.0).unwrap().classify(), GrowthClassification::Moderate);
        assert_eq!(GrowthFactor::new(5.0).unwrap().classify(), GrowthClassification::Weak);
        assert_eq!(GrowthFactor::new(0.05).unwrap().classify(), GrowthClassification::Flat);
        assert_eq!(GrowthFactor::new(-5.0).unwrap().classify(), GrowthClassification::SlightDecline);
        assert_eq!(GrowthFactor::new(-25.0).unwrap().classify(), GrowthClassification::Decline);
        assert_eq!(GrowthFactor::new(-75.0).unwrap().classify(), GrowthClassification::Collapse);
    }
    
    #[test]
    fn test_growth_types() {
        let positive = GrowthFactor::new(15.0).unwrap();
        assert!(positive.is_positive_growth());
        assert!(!positive.is_decline());
        assert!(!positive.is_flat());
        
        let negative = GrowthFactor::new(-10.0).unwrap();
        assert!(!negative.is_positive_growth());
        assert!(negative.is_decline());
        assert!(!negative.is_flat());
        
        let flat = GrowthFactor::new(0.05).unwrap();
        assert!(!flat.is_positive_growth());
        assert!(!flat.is_decline());
        assert!(flat.is_flat());
    }
    
    #[test]
    fn test_investment_score() {
        assert_eq!(GrowthFactor::new(150.0).unwrap().investment_score(), 100);
        assert_eq!(GrowthFactor::new(75.0).unwrap().investment_score(), 85);
        assert_eq!(GrowthFactor::new(-75.0).unwrap().investment_score(), 0);
    }
    
    #[test]
    fn test_growth_comparison() {
        let growth1 = GrowthFactor::new(25.0).unwrap();
        let growth2 = GrowthFactor::new(15.0).unwrap();
        let growth3 = GrowthFactor::new(24.0).unwrap();
        
        assert_eq!(growth1.compare_to(&growth2), GrowthComparison::Better);
        assert_eq!(growth1.compare_to(&growth3), GrowthComparison::Similar);
    }
    
    #[test]
    fn test_basis_points_conversion() {
        let growth = GrowthFactor::from_basis_points(1500).unwrap(); // 15%
        assert_eq!(growth.percentage(), 15.0);
        assert_eq!(growth.to_basis_points(), 1500);
    }
    
    #[test]
    fn test_formatting() {
        let positive = GrowthFactor::new(15.7).unwrap();
        assert_eq!(positive.format_display(), "+15.7%");
        
        let negative = GrowthFactor::new(-8.3).unwrap();
        assert_eq!(negative.format_display(), "-8.3%");
    }
    
    #[test]
    fn test_decimal_conversion() {
        let growth = GrowthFactor::new(50.0).unwrap();
        assert_eq!(growth.as_decimal(), 0.5);
    }
}