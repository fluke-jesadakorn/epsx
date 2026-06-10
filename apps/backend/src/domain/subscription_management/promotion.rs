use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub promo_type: PromotionType,
    pub value: f64,
    pub price: f64,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromotionType {
    Percentage,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromotionStatus {
    Active,
    Upcoming,
    Expired,
    Disabled,
}

impl Promotion {
    pub fn calculate_effective_price(&self, base_price: f64) -> f64 {
        if !self.enabled || !self.is_active() {
            return base_price;
        }

        // Use custom price if set (> 0), otherwise calculate
        if self.price > 0.0 {
            return self.price;
        }

        match self.promo_type {
            PromotionType::Percentage => {
                let discount_factor = 1.0 - (self.value / 100.0);
                base_price * discount_factor
            }
            PromotionType::Fixed => {
                (base_price - self.value).max(0.0)
            }
        }
    }

    pub fn is_active(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let now = Utc::now();

        // Empty start_date = no start restriction (already started)
        let started = if self.start_date.is_empty() {
            true
        } else {
            match DateTime::parse_from_rfc3339(&self.start_date) {
                Ok(dt) => now >= dt.with_timezone(&Utc),
                Err(_) => true,
            }
        };

        // Empty end_date = no end restriction (never expires)
        let not_ended = if self.end_date.is_empty() {
            true
        } else {
            match DateTime::parse_from_rfc3339(&self.end_date) {
                Ok(dt) => now <= dt.with_timezone(&Utc),
                Err(_) => true,
            }
        };

        started && not_ended
    }

    pub fn get_status(&self) -> PromotionStatus {
        if !self.enabled {
            return PromotionStatus::Disabled;
        }

        let now = Utc::now();

        let start_opt = if self.start_date.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(&self.start_date)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        };

        let end_opt = if self.end_date.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(&self.end_date)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        };

        match (start_opt, end_opt) {
            (Some(start), _) if now < start => PromotionStatus::Upcoming,
            (_, Some(end)) if now > end => PromotionStatus::Expired,
            _ => PromotionStatus::Active,
        }
    }

    pub fn get_discount_percentage(&self, base_price: f64) -> f64 {
        if !self.is_active() || base_price == 0.0 {
            return 0.0;
        }

        let effective = self.calculate_effective_price(base_price);
        let discount = base_price - effective;
        (discount / base_price) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_percentage_discount() {
        let now = Utc::now();
        let promo = Promotion {
            enabled: true,
            promo_type: PromotionType::Percentage,
            value: 20.0,
            price: 0.0,
            start_date: (now - Duration::days(1)).to_rfc3339(),
            end_date: (now + Duration::days(1)).to_rfc3339(),
        };

        assert_eq!(promo.calculate_effective_price(100.0), 80.0);
    }

    #[test]
    fn test_fixed_discount() {
        let now = Utc::now();
        let promo = Promotion {
            enabled: true,
            promo_type: PromotionType::Fixed,
            value: 10.0,
            price: 0.0,
            start_date: (now - Duration::days(1)).to_rfc3339(),
            end_date: (now + Duration::days(1)).to_rfc3339(),
        };

        assert_eq!(promo.calculate_effective_price(50.0), 40.0);
    }

    #[test]
    fn test_custom_price() {
        let now = Utc::now();
        let promo = Promotion {
            enabled: true,
            promo_type: PromotionType::Percentage,
            value: 20.0,
            price: 25.0, // Custom override
            start_date: (now - Duration::days(1)).to_rfc3339(),
            end_date: (now + Duration::days(1)).to_rfc3339(),
        };

        assert_eq!(promo.calculate_effective_price(100.0), 25.0);
    }

    #[test]
    fn test_upcoming_promotion() {
        let now = Utc::now();
        let promo = Promotion {
            enabled: true,
            promo_type: PromotionType::Percentage,
            value: 20.0,
            price: 0.0,
            start_date: (now + Duration::days(1)).to_rfc3339(),
            end_date: (now + Duration::days(7)).to_rfc3339(),
        };

        assert_eq!(promo.get_status(), PromotionStatus::Upcoming);
        assert!(!promo.is_active());
        assert_eq!(promo.calculate_effective_price(100.0), 100.0); // No discount yet
    }

    #[test]
    fn test_expired_promotion() {
        let now = Utc::now();
        let promo = Promotion {
            enabled: true,
            promo_type: PromotionType::Percentage,
            value: 20.0,
            price: 0.0,
            start_date: (now - Duration::days(7)).to_rfc3339(),
            end_date: (now - Duration::days(1)).to_rfc3339(),
        };

        assert_eq!(promo.get_status(), PromotionStatus::Expired);
        assert!(!promo.is_active());
        assert_eq!(promo.calculate_effective_price(100.0), 100.0); // No discount
    }
}
