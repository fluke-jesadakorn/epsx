use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use crate::infrastructure::{
    adapters::repositories::diesel::{pool::DbPool, schema::pricing_plans},
    models::marketing::{PricingPlan, PlanWithPromotions},
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = pricing_plans)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PricingPlanEntity {
    pub id: i32,
    pub name: String,
    pub plan_type: String,
    pub base_price: bigdecimal::BigDecimal,
    pub current_price: bigdecimal::BigDecimal,
    pub currency: String,
    pub features: serde_json::Value,
    pub affiliate_commission_rate: Option<bigdecimal::BigDecimal>,
    pub display_order: Option<i32>,
    pub is_active: Option<bool>,
    pub is_highlighted: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<PricingPlanEntity> for PricingPlan {
    fn from(entity: PricingPlanEntity) -> Self {
        PricingPlan {
            id: entity.id,
            name: entity.name,
            plan_type: entity.plan_type,
            base_price: entity.base_price,
            current_price: entity.current_price,
            currency: entity.currency,
            features: entity.features,
            affiliate_commission_rate: entity.affiliate_commission_rate,
            display_order: entity.display_order,
            is_active: entity.is_active.unwrap_or(false),
            is_highlighted: entity.is_highlighted.unwrap_or(false),
            created_at: entity.created_at.unwrap_or_else(chrono::Utc::now),
            updated_at: entity.updated_at,
        }
    }
}

pub struct MarketingRepository {
    db_pool: Arc<DbPool>,
}

impl MarketingRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    pub async fn get_all_plans(&self) -> Result<Vec<PricingPlan>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get().await?;
        
        let plans_entities = pricing_plans::table
            .filter(pricing_plans::is_active.eq(Some(true)))
            .order_by((pricing_plans::display_order.asc().nulls_last(), pricing_plans::id.asc()))
            .select(PricingPlanEntity::as_select())
            .load(&mut conn)
            .await?;

        let plans: Vec<PricingPlan> = plans_entities.into_iter().map(|entity| entity.into()).collect();
        Ok(plans)
    }

    pub async fn get_plans_by_type(&self, plan_type: &str) -> Result<Vec<PricingPlan>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get().await?;
        
        let plans_entities = pricing_plans::table
            .filter(pricing_plans::is_active.eq(Some(true)))
            .filter(pricing_plans::plan_type.eq(plan_type))
            .order_by((pricing_plans::display_order.asc().nulls_last(), pricing_plans::id.asc()))
            .select(PricingPlanEntity::as_select())
            .load(&mut conn)
            .await?;

        let plans: Vec<PricingPlan> = plans_entities.into_iter().map(|entity| entity.into()).collect();
        Ok(plans)
    }

    pub async fn get_plan_by_id(&self, plan_id: i32) -> Result<Option<PricingPlan>, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.db_pool.get().await?;
        
        let plan_entity = pricing_plans::table
            .filter(pricing_plans::id.eq(plan_id))
            .filter(pricing_plans::is_active.eq(Some(true)))
            .select(PricingPlanEntity::as_select())
            .first(&mut conn)
            .await
            .optional()?;

        Ok(plan_entity.map(|entity| entity.into()))
    }

    pub fn convert_to_plan_with_promotions(&self, plans: Vec<PricingPlan>) -> Vec<PlanWithPromotions> {
        plans.into_iter().map(|plan| {
            let has_discount = plan.current_price < plan.base_price;
            let mut active_promotions = Vec::new();
            let mut promotional_badges = Vec::new();
            let mut campaign_summary = None;

            // Check if plan has a discount
            if has_discount {
                active_promotions.push("Special Offer".to_string());
                let savings = &plan.base_price - &plan.current_price;
                campaign_summary = Some(format!("Save {} {}", savings, plan.currency));
            }

            // Check if plan is highlighted (popular)
            if plan.is_highlighted {
                promotional_badges.push("POPULAR".to_string());
                promotional_badges.push("MOST POPULAR".to_string());
            }

            PlanWithPromotions {
                effective_price: plan.current_price.clone(),
                plan,
                active_promotions,
                promotional_badges,
                campaign_summary,
            }
        }).collect()
    }
}