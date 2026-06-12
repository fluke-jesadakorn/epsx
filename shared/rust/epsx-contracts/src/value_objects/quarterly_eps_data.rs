// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/quarterly_eps_data.rs
// Import-path adjustment: `crate::domain::shared_kernel::value_object::*` →
// `crate::value_object::*`.
use serde::{Deserialize, Serialize};
use crate::value_object::{ValueObject, ValueObjectError};

/// Quarterly EPS data value object - Domain representation
/// Represents quarterly earnings data independent of external data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarterlyEPSData {
    pub quarter_number: usize,
    pub period: String, // "2024-Q3", "2025-Q1", etc.
    pub actual_eps: f64,
    pub timestamp: i64, // earnings timestamp (seconds)
    pub estimated_eps: Option<f64>,
    pub is_reported: bool,
    pub beat_estimate: Option<bool>,
    pub quarter_end_date: Option<String>,
    pub estimated_earnings_date: Option<i64>, // earnings announcement timestamp
}

// Manual PartialEq implementation to handle f64 comparison
impl PartialEq for QuarterlyEPSData {
    fn eq(&self, other: &Self) -> bool {
        self.quarter_number == other.quarter_number
            && self.period == other.period
            && (self.actual_eps - other.actual_eps).abs() < f64::EPSILON
            && self.timestamp == other.timestamp
            && self.estimated_eps.zip(other.estimated_eps).map_or(
                self.estimated_eps.is_none() && other.estimated_eps.is_none(),
                |(a, b)| (a - b).abs() < f64::EPSILON
            )
            && self.is_reported == other.is_reported
            && self.beat_estimate == other.beat_estimate
            && self.quarter_end_date == other.quarter_end_date
            && self.estimated_earnings_date == other.estimated_earnings_date
    }
}

// Eq implementation (marker trait)
impl Eq for QuarterlyEPSData {}

impl QuarterlyEPSData {
    pub fn new(
        quarter_number: usize,
        period: String,
        actual_eps: f64,
        timestamp: i64,
    ) -> Result<Self, ValueObjectError> {
        let quarterly_data = Self {
            quarter_number,
            period,
            actual_eps,
            timestamp,
            estimated_eps: None,
            is_reported: false,
            beat_estimate: None,
            quarter_end_date: None,
            estimated_earnings_date: None,
        };
        quarterly_data.validate()?;
        Ok(quarterly_data)
    }

    pub fn with_estimates(mut self, estimated_eps: f64, beat_estimate: bool) -> Self {
        self.estimated_eps = Some(estimated_eps);
        self.beat_estimate = Some(beat_estimate);
        self
    }

    pub fn mark_as_reported(mut self) -> Self {
        self.is_reported = true;
        self
    }

    /// Calculate EPS surprise (difference between actual and estimated)
    pub fn eps_surprise(&self) -> Option<f64> {
        self.estimated_eps.map(|estimated| self.actual_eps - estimated)
    }

    /// Calculate EPS surprise percentage
    pub fn eps_surprise_percentage(&self) -> Option<f64> {
        self.estimated_eps
            .filter(|&estimated| estimated != 0.0)
            .map(|estimated| ((self.actual_eps - estimated) / estimated.abs()) * 100.0)
    }
}

impl ValueObject for QuarterlyEPSData {
    type Error = ValueObjectError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        if self.quarter_number == 0 || self.quarter_number > 4 {
            return Err(ValueObjectError::OutOfRange("Quarter number must be 1-4".to_string()));
        }
        
        if self.period.is_empty() {
            return Err(ValueObjectError::Required("Period cannot be empty".to_string()));
        }
        
        if self.timestamp <= 0 {
            return Err(ValueObjectError::InvalidFormat("Timestamp must be positive".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarterly_eps_data_creation() {
        let eps_data = QuarterlyEPSData::new(
            3,
            "2024-Q3".to_string(),
            2.45,
            1672531200,
        ).unwrap();

        assert_eq!(eps_data.quarter_number, 3);
        assert_eq!(eps_data.actual_eps, 2.45);
        assert!(!eps_data.is_reported);
    }

    #[test]
    fn test_eps_surprise_calculation() {
        let eps_data = QuarterlyEPSData::new(
            2,
            "2024-Q2".to_string(),
            2.50,
            1672531200,
        )
        .unwrap()
        .with_estimates(2.30, true);

        assert!((eps_data.eps_surprise().unwrap() - 0.20).abs() < 1e-10);
        assert!(eps_data.eps_surprise_percentage().unwrap() > 8.0);
    }

    #[test]
    fn test_invalid_quarter_number() {
        let result = QuarterlyEPSData::new(
            5,
            "2024-Q5".to_string(),
            1.0,
            1672531200,
        );

        assert!(result.is_err());
    }
}
