use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Earnings Per Share (EPS) Value Object
/// Represents the earnings per share for a company
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EPSValue {
    value: f64,
}

impl EPSValue {
    /// Create new EPS value with validation
    pub fn new(value: f64) -> Result<Self, String> {
        if value.is_nan() || value.is_infinite() {
            return Err("EPS value cannot be NaN or infinite".to_string());
        }
        
        // Allow reasonable range for EPS (can be negative for losses)
        if !(-10000.0..=10000.0).contains(&value) {
            return Err("EPS value is outside reasonable range (-10000 to 10000)".to_string());
        }
        
        Ok(Self { value })
    }
    
    /// Get the raw EPS value
    pub fn value(&self) -> f64 {
        self.value
    }
    
    /// Get the current EPS value (alias for value)
    pub fn current_eps(&self) -> f64 {
        self.value
    }
    
    /// Check if EPS is positive (profitable)
    pub fn is_profitable(&self) -> bool {
        self.value >= 0.01
    }
    
    /// Check if EPS is negative (loss-making)
    pub fn is_loss_making(&self) -> bool {
        self.value <= -0.01
    }
    
    /// Check if EPS is near zero (break-even)
    pub fn is_break_even(&self) -> bool {
        self.value.abs() < 0.01 // Within 1 cent
    }
    
    /// Calculate percentage change to another EPS value
    pub fn percentage_change_to(&self, other: EPSValue) -> Option<f64> {
        if self.value.abs() < 0.0001 {
            // Cannot calculate percentage change from zero
            return None;
        }
        
        Some(((other.value - self.value) / self.value.abs()) * 100.0)
    }
    
    /// Get EPS quality rating based on value and consistency
    pub fn quality_rating(&self) -> EPSQuality {
        match self.value {
            v if v > 10.0 => EPSQuality::Excellent,
            v if v > 2.0 => EPSQuality::Good,
            v if v > 0.5 => EPSQuality::Fair,
            v if v > 0.0 => EPSQuality::Poor,
            _ => EPSQuality::Negative,
        }
    }
    
    /// Format EPS for display with appropriate precision
    pub fn format_currency(&self, currency: Option<&str>) -> String {
        let currency_symbol = currency.unwrap_or("$");
        if self.value.abs() >= 10.0 {
            format!("{}{:.2}", currency_symbol, self.value)
        } else {
            format!("{}{:.3}", currency_symbol, self.value)
        }
    }
}

/// EPS Quality Rating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EPSQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Negative,
}

impl EPSQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            EPSQuality::Excellent => "excellent",
            EPSQuality::Good => "good",
            EPSQuality::Fair => "fair",
            EPSQuality::Poor => "poor",
            EPSQuality::Negative => "negative",
        }
    }
    
    pub fn score(&self) -> u8 {
        match self {
            EPSQuality::Excellent => 5,
            EPSQuality::Good => 4,
            EPSQuality::Fair => 3,
            EPSQuality::Poor => 2,
            EPSQuality::Negative => 1,
        }
    }
}

impl Display for EPSValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}", self.value)
    }
}

impl TryFrom<f64> for EPSValue {
    type Error = String;
    
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        EPSValue::new(value)
    }
}

// Allow comparison operations
impl PartialOrd for EPSValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

// Mathematical operations
impl std::ops::Add for EPSValue {
    type Output = Result<EPSValue, String>;
    
    fn add(self, other: EPSValue) -> Self::Output {
        EPSValue::new(self.value + other.value)
    }
}

impl std::ops::Sub for EPSValue {
    type Output = Result<EPSValue, String>;
    
    fn sub(self, other: EPSValue) -> Self::Output {
        EPSValue::new(self.value - other.value)
    }
}

impl std::ops::Mul<f64> for EPSValue {
    type Output = Result<EPSValue, String>;
    
    fn mul(self, scalar: f64) -> Self::Output {
        EPSValue::new(self.value * scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_eps_values() {
        let values = [0.0, 1.52, -0.75, 25.0, -5.2];
        
        for value in &values {
            let eps = EPSValue::new(*value);
            assert!(eps.is_ok(), "Failed for EPS value: {}", value);
        }
    }
    
    #[test]
    fn test_invalid_eps_values() {
        let invalid_values = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 20000.0, -20000.0];
        
        for value in &invalid_values {
            let eps = EPSValue::new(*value);
            assert!(eps.is_err(), "Should have failed for EPS value: {}", value);
        }
    }
    
    #[test]
    fn test_eps_classification() {
        let profitable = EPSValue::new(1.52).unwrap();
        assert!(profitable.is_profitable());
        assert!(!profitable.is_loss_making());
        assert!(!profitable.is_break_even());
        
        let loss_making = EPSValue::new(-0.75).unwrap();
        assert!(!loss_making.is_profitable());
        assert!(loss_making.is_loss_making());
        assert!(!loss_making.is_break_even());
        
        let break_even = EPSValue::new(0.005).unwrap();
        assert!(!break_even.is_profitable());
        assert!(!break_even.is_loss_making());
        assert!(break_even.is_break_even());
    }
    
    #[test]
    fn test_percentage_change() {
        let eps1 = EPSValue::new(1.0).unwrap();
        let eps2 = EPSValue::new(1.5).unwrap();
        
        let change = eps1.percentage_change_to(eps2).unwrap();
        assert!((change - 50.0).abs() < 0.01); // 50% increase
        
        let eps_zero = EPSValue::new(0.0).unwrap();
        assert!(eps_zero.percentage_change_to(eps2).is_none());
    }
    
    #[test]
    fn test_quality_rating() {
        assert_eq!(EPSValue::new(15.0).unwrap().quality_rating(), EPSQuality::Excellent);
        assert_eq!(EPSValue::new(3.0).unwrap().quality_rating(), EPSQuality::Good);
        assert_eq!(EPSValue::new(1.0).unwrap().quality_rating(), EPSQuality::Fair);
        assert_eq!(EPSValue::new(0.1).unwrap().quality_rating(), EPSQuality::Poor);
        assert_eq!(EPSValue::new(-0.5).unwrap().quality_rating(), EPSQuality::Negative);
    }
    
    #[test]
    fn test_currency_formatting() {
        let eps = EPSValue::new(1.523).unwrap();
        assert_eq!(eps.format_currency(None), "$1.523");
        assert_eq!(eps.format_currency(Some("€")), "€1.523");
        
        let high_eps = EPSValue::new(15.7).unwrap();
        assert_eq!(high_eps.format_currency(None), "$15.70");
    }
    
    #[test]
    fn test_mathematical_operations() {
        let eps1 = EPSValue::new(1.5).unwrap();
        let eps2 = EPSValue::new(0.5).unwrap();
        
        let sum = (eps1 + eps2).unwrap();
        assert_eq!(sum.value(), 2.0);
        
        let diff = (eps1 - eps2).unwrap();
        assert_eq!(diff.value(), 1.0);
        
        let product = (eps1 * 2.0).unwrap();
        assert_eq!(product.value(), 3.0);
    }
}