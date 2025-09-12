use std::sync::Arc;

use crate::{
    infrastructure::{
        adapters::repositories::diesel::pool::DbPool,
        models::marketing::DiscountValidation,
    },
};

pub struct PromotionService {
    db_pool: Arc<DbPool>,
}

impl PromotionService {
    pub fn new(db_pool: Arc<DbPool>, _cache: Arc<dyn crate::infrastructure::cache::Cache>) -> Self {
        Self {
            db_pool,
        }
    }

    pub async fn validate_discount_code(&self, _code: &str, _plan_id: i32, _user_id: Option<i32>) -> Result<DiscountValidation, Box<dyn std::error::Error + Send + Sync>> {
        // For now, return mock validation
        Ok(DiscountValidation {
            is_valid: true,
            discount_type: "percentage".to_string(),
            discount_value: "10.00".parse().unwrap(),
            max_discount_amount: Some("5.00".parse().unwrap()),
            min_purchase_amount: None,
            usage_limit: Some(100),
            current_usage: 5,
            is_expired: false,
            error_message: None,
        })
    }
}