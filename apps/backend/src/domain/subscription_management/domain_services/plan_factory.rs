use crate::domain::subscription_management::aggregates::{Plan, CreatePlanParams};
use crate::domain::subscription_management::value_objects::{
    plan_id::PlanId,
    price::Price,
    billing_cycle::BillingCycle,
    plan_features::PlanFeatures,
};
// Use GroupId if wrapper or just String. Assuming String needs conversion or mock.
// Since Plan Aggregate expects GroupId, we need to import it.
// Checking aggregates/plan.rs: use crate::domain::permission_management::GroupId;

pub struct PlanFactory;

impl PlanFactory {
    pub fn create_plan(
        name: String,
        description: Option<String>,
        _permission_group: String,
        permissions: Vec<String>,
        price_amount: rust_decimal::Decimal,
        currency: String,
        interval: String,
    ) -> Result<Plan, String> {
        // Value Object creation
        let price = Price::new(price_amount, currency).map_err(|e| e.to_string())?;
        
        let billing_cycle = match interval.to_lowercase().as_str() {
            "month" | "monthly" => BillingCycle::Monthly,
            "year" | "yearly" => BillingCycle::Yearly,
            _ => return Err(format!("Invalid billing interval: {}", interval)),
        };

        // Plan aggregation creation
        // Temporarily generate a new GroupId. In real app, might need to lookup or create group by name.
        let group_id = crate::domain::permission_management::GroupId::from(uuid::Uuid::new_v4());
        
        let params = CreatePlanParams {
            name,
            description: description.unwrap_or_default(),
            group_id, 
            permissions,
            price,
            billing_cycle,
            features: PlanFeatures::default(), // Default for now
            target_audience: "general".to_string(), // Default
            is_active: Some(true),
            is_promoted: Some(false),
            display_order: Some(0),
            metadata: None,
        };

        Plan::create(params).map_err(|e| e.to_string())
    }


    /// Helper to derive a permission group name if one isn't provided
    /// This replaces `derive_group_from_permissions` from the handler layer
    pub fn derive_group_from_permissions(permissions: &[String]) -> String {
        // Simplified Logic: Hash of sorted permissions or a default
        // In a real system, this might check a registry of known groups
        if permissions.is_empty() {
            return "free_tier".to_string();
        }
        
        // Mock implementation to match original intent
        format!("custom_group_{}", permissions.len())
    }
}
