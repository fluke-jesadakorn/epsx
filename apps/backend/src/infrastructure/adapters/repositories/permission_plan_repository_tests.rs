use super::*;
use crate::domain::permission_management::aggregates::plan::{CreatePlanParams, UpdatePlanParams};
use crate::domain::permission_management::{PlanCategory, PlanGroup};

#[tokio::test]
#[ignore = "requires migrated local EPSX_PLAN_REPOSITORY_TEST_URL ending in _shadow"]
async fn native_plan_catalog_round_trip_and_atomic_permissions() {
    let url = std::env::var("EPSX_PLAN_REPOSITORY_TEST_URL").unwrap();
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(parsed.host_str(), Some("127.0.0.1" | "localhost")));
    let admin = PgPool::connect(&url).await.unwrap();
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&admin)
        .await
        .unwrap();
    assert!(database.starts_with("epsx_") && database.ends_with("_shadow"));
    let schema = format!("plan_repository_test_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    for table in ["plans", "permissions", "plan_permissions"] {
        sqlx::query(&format!(
            "CREATE TABLE {schema}.{table} (LIKE public.{table} INCLUDING ALL)"
        ))
        .execute(&admin)
        .await
        .unwrap();
    }
    sqlx::query(&format!("ALTER TABLE {schema}.plan_permissions ADD FOREIGN KEY(plan_id) REFERENCES {schema}.plans(id), ADD FOREIGN KEY(permission_id) REFERENCES {schema}.permissions(id)"))
        .execute(&admin).await.unwrap();
    let search_path = format!("SET search_path TO {schema}, public");
    let pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .after_connect(move |connection, _| {
                let statement = search_path.clone();
                Box::pin(async move {
                    sqlx::query(&statement)
                        .execute(connection)
                        .await
                        .map(|_| ())
                })
            })
            .connect(&url)
            .await
            .unwrap(),
    );
    let repo = PlanRepositoryAdapter::new(pool.clone());
    let permission = PermissionString::new("epsx:rehearsal:read").unwrap();
    let mut plan = Plan::create(CreatePlanParams {
        name: "Production-shaped plan".into(),
        slug: PlanSlug::new("production-shaped-plan").unwrap(),
        description: "Preserve pricing and access".into(),
        plan_type: "subscription".into(),
        plan_category: Some(PlanCategory::parse("exclusive").unwrap()),
        plan_group: Some(PlanGroup::parse("enterprise").unwrap()),
        permissions: vec![permission.clone()],
        price: Some(123.45),
        currency: Some("USD".into()),
        billing_cycle: Some("one_time".into()),
        is_active: Some(true),
        is_promoted: Some(true),
        tier_level: Some(7),
        max_members: Some(25),
        auto_assign_enabled: Some(true),
        metadata: Some(serde_json::json!({"duration_days":30,"promo_marker":"keep"})),
        is_public: Some(false),
        grace_period_hours: Some(48),
    })
    .unwrap();
    repo.save(&plan).await.unwrap();
    let loaded = repo.find_by_id(plan.id()).await.unwrap().unwrap();
    assert_eq!(loaded.price(), 123.45);
    assert_eq!(loaded.billing_cycle(), "one_time");
    assert_eq!(loaded.plan_category().to_string(), "exclusive");
    assert_eq!(loaded.plan_group().to_string(), "enterprise");
    assert_eq!(loaded.max_members(), Some(25));
    assert!(loaded.auto_assign_enabled() && loaded.is_promoted());
    assert!(!loaded.is_public());
    assert_eq!(loaded.grace_period_hours(), 48);
    assert_eq!(loaded.metadata(), plan.metadata());
    assert!(loaded.has_permission(&permission));
    assert_eq!(
        repo.find_by_slug(plan.slug()).await.unwrap().unwrap().id(),
        plan.id()
    );
    let criteria = PlanSearchCriteria {
        plan_group: Some("enterprise".into()),
        ..Default::default()
    };
    assert_eq!(repo.find_all(criteria.clone()).await.unwrap().len(), 1);
    assert_eq!(repo.count(criteria).await.unwrap(), 1);
    let other = PlanSearchCriteria {
        plan_group: Some("personal".into()),
        ..Default::default()
    };
    assert!(repo.find_all(other.clone()).await.unwrap().is_empty());
    assert_eq!(repo.count(other).await.unwrap(), 0);

    // Updating a loaded catalog row must retain unmodeled legacy metadata and
    // the IDs/timestamps/reasons of existing permission links.
    sqlx::query("UPDATE plans SET assignment_rules='{\"preserve\":true}',last_modified_by='legacy-operator' WHERE id=$1")
        .bind(plan.id().value()).execute(pool.as_ref()).await.unwrap();
    sqlx::query("UPDATE plan_permissions SET grant_reason='preserve-grant' WHERE plan_id=$1")
        .bind(plan.id().value())
        .execute(pool.as_ref())
        .await
        .unwrap();
    let before: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(p)-'updated_at' FROM plans p WHERE id=$1")
            .bind(plan.id().value())
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
    let grant: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM plan_permissions p WHERE plan_id=$1")
            .bind(plan.id().value())
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
    repo.save(&loaded).await.unwrap();
    let after: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(p)-'updated_at' FROM plans p WHERE id=$1")
            .bind(plan.id().value())
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
    assert_eq!(before, after);
    let retained: serde_json::Value =
        sqlx::query_scalar("SELECT to_jsonb(p) FROM plan_permissions p WHERE plan_id=$1")
            .bind(plan.id().value())
            .fetch_one(pool.as_ref())
            .await
            .unwrap();
    assert_eq!(grant, retained);

    // Fail after the catalog update to prove the whole save rolls back.
    sqlx::query("ALTER TABLE permissions ADD CONSTRAINT reject_test_permission CHECK (permission_string <> 'epsx:rehearsal:reject')")
        .execute(pool.as_ref()).await.unwrap();
    plan.update(UpdatePlanParams {
        price: Some(456.78),
        permissions: Some(vec![PermissionString::new("epsx:rehearsal:reject").unwrap()]),
        ..Default::default()
    })
    .unwrap();
    assert!(repo.save(&plan).await.is_err());
    let unchanged = repo.find_by_id(plan.id()).await.unwrap().unwrap();
    assert_eq!(unchanged.price(), 123.45);
    assert!(unchanged.has_permission(&permission));
    let replacement = PermissionString::new("epsx:rehearsal:write").unwrap();
    plan.update(UpdatePlanParams {
        permissions: Some(vec![replacement.clone()]),
        ..Default::default()
    })
    .unwrap();
    repo.save(&plan).await.unwrap();
    let updated = repo.find_by_id(plan.id()).await.unwrap().unwrap();
    assert_eq!(updated.price(), 456.78);
    assert!(!updated.has_permission(&permission));
    assert!(updated.has_permission(&replacement));

    // An unreadable legacy grant must fail closed instead of disappearing on save.
    sqlx::query("INSERT INTO permissions(permission_string,platform,resource,action) VALUES('invalid-legacy-grant','epsx','legacy','read')")
        .execute(pool.as_ref()).await.unwrap();
    sqlx::query("INSERT INTO plan_permissions(plan_id,permission_id) SELECT $1,id FROM permissions WHERE permission_string='invalid-legacy-grant'")
        .bind(plan.id().value()).execute(pool.as_ref()).await.unwrap();
    assert!(repo.find_by_id(plan.id()).await.is_err());
    assert!(repo.find_all(Default::default()).await.is_err());
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}
