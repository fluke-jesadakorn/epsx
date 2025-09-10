use std::sync::Arc;

use crate::{
    infrastructure::{
        adapters::repositories::diesel::{
            pool::DbPool,
            marketing_repository::MarketingRepository,
        },
        models::marketing::{
            PricingPlan, PlanWithPromotions, 
        },
    },
};

#[derive(Debug, Clone)]
pub struct PlanFilters {
    pub plan_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_highlighted: Option<bool>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

pub struct PlanService {
    repository: MarketingRepository,
}

impl PlanService {
    pub fn new(db_pool: Arc<DbPool>, _cache: Arc<dyn crate::infrastructure::cache::Cache>) -> Self {
        Self {
            repository: MarketingRepository::new(db_pool),
        }
    }

    pub async fn list_plans(&self, filters: Option<PlanFilters>) -> Result<Vec<PlanWithPromotions>, Box<dyn std::error::Error + Send + Sync>> {
        let plans = if let Some(filters) = filters {
            if let Some(plan_type) = filters.plan_type {
                self.repository.get_plans_by_type(&plan_type).await?
            } else {
                self.repository.get_all_plans().await?
            }
        } else {
            self.repository.get_all_plans().await?
        };

        let plans_with_promotions = self.repository.convert_to_plan_with_promotions(plans);
        Ok(plans_with_promotions)
    }

    pub async fn get_plans_by_type(&self, plan_type: &str) -> Result<Vec<PlanWithPromotions>, Box<dyn std::error::Error + Send + Sync>> {
        let filters = PlanFilters {
            plan_type: Some(plan_type.to_string()),
            is_active: Some(true),
            is_highlighted: None,
            min_price: None,
            max_price: None,
        };
        self.list_plans(Some(filters)).await
    }

    pub async fn get_plan(&self, plan_id: i32) -> Result<Option<PlanWithPromotions>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(plan) = self.repository.get_plan_by_id(plan_id).await? {
            let plans_with_promotions = self.repository.convert_to_plan_with_promotions(vec![plan]);
            Ok(plans_with_promotions.into_iter().next())
        } else {
            Ok(None)
        }
    }

    pub async fn calculate_effective_price(&self, plan_id: i32, _user_id: Option<i32>) -> Result<rust_decimal::Decimal, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(plan) = self.repository.get_plan_by_id(plan_id).await? {
            // Convert BigDecimal to Decimal for the return type
            let effective_price = plan.current_price.to_string().parse::<rust_decimal::Decimal>()?;
            Ok(effective_price)
        } else {
            Err("Plan not found".into())
        }
    }
}