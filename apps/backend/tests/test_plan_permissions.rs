use epsx::domain::subscription_management::{
    aggregates::{Plan, CreatePlanParams, UpdatePlanParams},
    value_objects::{Price, BillingCycle, PlanFeatures},
    value_objects::quota::Quota,
};
use epsx::domain::permission_management::GroupId;

use rust_decimal::Decimal; // Correct crate import
use uuid::Uuid;
use std::collections::HashMap;

#[test]
fn test_plan_creation_syncs_permissions_from_metadata() {
    // 1. Setup metadata with ranking_offset
    let metadata = serde_json::json!({
        "ranking_offset": 50,
        "rankings_limit": 100
    });

    // 2. Create Plan
    let params = CreatePlanParams {
        name: "Test Plan".to_string(),
        description: "Test Description".to_string(),
        group_id: GroupId::new(),
        permissions: vec!["epsx:base:perm".to_string()], // Base permission
        price: Price::new(Decimal::new(1000, 2), "USD".to_string()).unwrap(),
        billing_cycle: BillingCycle::Monthly,
        features: PlanFeatures::default(),
        target_audience: "all".to_string(),
        is_active: Some(true),
        is_promoted: Some(false),
        display_order: Some(1),
        metadata: Some(metadata),
    };

    let plan = Plan::create(params).expect("Failed to create plan");

    // 3. Verify permissions were synced
    assert!(plan.permissions.contains(&"epsx:rankings:offset:50".to_string()));
    assert!(plan.permissions.contains(&"epsx:rankings:limit:100".to_string()));
    assert!(plan.permissions.contains(&"epsx:analytics:view:100".to_string())); // Legacy support
    assert!(plan.permissions.contains(&"epsx:base:perm".to_string())); // Base preserved
}

#[test]
fn test_plan_update_syncs_permissions_from_metadata() {
    // 1. Validation: Setup initial plan
    let initial_metadata = serde_json::json!({
        "ranking_offset": 50
    });
    
    let params = CreatePlanParams {
        name: "Test Plan".to_string(),
        description: "Test Description".to_string(),
        group_id: GroupId::new(),
        permissions: vec![],
        price: Price::new(Decimal::new(1000, 2), "USD".to_string()).unwrap(),
        billing_cycle: BillingCycle::Monthly,
        features: PlanFeatures::default(),
        target_audience: "all".to_string(),
        is_active: Some(true),
        is_promoted: Some(false),
        display_order: Some(1),
        metadata: Some(initial_metadata),
    };

    let mut plan = Plan::create(params).expect("Failed to create plan");
    assert!(plan.permissions.contains(&"epsx:rankings:offset:50".to_string()));

    // 2. Validation: Update metadata
    let new_metadata = serde_json::json!({
        "ranking_offset": 1, // Upgrade to top tier
        "rankings_limit": 500
    });

    let update_params = UpdatePlanParams {
        metadata: Some(new_metadata),
        ..Default::default()
    };

    plan.update(update_params).expect("Failed to update plan");

    // 3. Verify permissions updated
    assert!(plan.permissions.contains(&"epsx:rankings:offset:1".to_string()));
    assert!(plan.permissions.contains(&"epsx:rankings:limit:500".to_string()));
    
    // 4. Verify old permissions removed
    assert!(!plan.permissions.contains(&"epsx:rankings:offset:50".to_string()));
}

#[test]
fn test_plan_update_features_object_structure() {
    // Test nested "features" structure support
    let metadata = serde_json::json!({
        "features": {
            "ranking_offset": 25,
            "rankings_limit": 50
        }
    });

    let params = CreatePlanParams {
        name: "Feature Object Plan".to_string(),
        description: "Desc".to_string(),
        group_id: GroupId::new(),
        permissions: vec![],
        price: Price::new(Decimal::new(1000, 2), "USD".to_string()).unwrap(),
        billing_cycle: BillingCycle::Monthly,
        features: PlanFeatures::default(),
        target_audience: "all".to_string(),
        is_active: Some(true),
        is_promoted: Some(false),
        display_order: Some(1),
        metadata: Some(metadata),
    };

    let plan = Plan::create(params).expect("Failed to create plan");

    assert!(plan.permissions.contains(&"epsx:rankings:offset:25".to_string()));
    assert!(plan.permissions.contains(&"epsx:rankings:limit:50".to_string()));
}
