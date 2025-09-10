use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingPlan {
    pub id: i32,
    pub name: String,
    pub plan_type: String,
    pub base_price: bigdecimal::BigDecimal,
    pub current_price: bigdecimal::BigDecimal,
    pub currency: String,
    pub features: serde_json::Value,
    pub affiliate_commission_rate: Option<bigdecimal::BigDecimal>,
    pub display_order: Option<i32>,
    pub is_active: bool,
    pub is_highlighted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affiliate {
    pub id: i32,
    pub user_id: Option<i32>,
    pub affiliate_code: String,
    pub status: String,
    pub commission_rate: Option<bigdecimal::BigDecimal>,
    pub total_referrals: Option<i32>,
    pub total_sales: Option<bigdecimal::BigDecimal>,
    pub total_commissions: Option<bigdecimal::BigDecimal>,
    pub payment_method: Option<String>,
    pub payment_details: Option<serde_json::Value>,
    pub tier_id: Option<i32>,
    pub notes: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// DTO structures for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanWithPromotions {
    pub plan: PricingPlan,
    pub active_promotions: Vec<String>, // Simplified for now
    pub effective_price: bigdecimal::BigDecimal,
    pub promotional_badges: Vec<String>,
    pub campaign_summary: Option<String>, // Simplified for now
}

// Additional DTO types needed by services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountValidation {
    pub is_valid: bool,
    pub discount_type: String,
    pub discount_value: bigdecimal::BigDecimal,
    pub max_discount_amount: Option<bigdecimal::BigDecimal>,
    pub min_purchase_amount: Option<bigdecimal::BigDecimal>,
    pub usage_limit: Option<i32>,
    pub current_usage: i32,
    pub is_expired: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionCalculation {
    pub affiliate_id: i32,
    pub plan_id: i32,
    pub base_amount: rust_decimal::Decimal,
    pub commission_rate: rust_decimal::Decimal,
    pub commission_amount: rust_decimal::Decimal,
    pub commission_type: String,
    pub calculated_at: DateTime<Utc>,
}