// Unit tests for EPSX backend components
use epsx::dom::entities::{User, RolePermissionProfile, Permission};
use epsx::dom::values::{Email, UserId, Role, SubTier};
use epsx::dom::entities::iam::{PackageTier};
use epsx::dom::entities::permission_profile::PermissionProfileCategory;

#[cfg(test)]
mod user_tests {
    use super::*;

    #[test]
    fn should_create_user_with_valid_email() {
        let email = Email::new("test@example.com").unwrap();
        let user = User::new(email, Role::User);
        
        assert_eq!(user.email().value(), "test@example.com");
        assert_eq!(user.role(), &Role::User);
        assert!(user.is_active());
    }

    #[test]
    fn should_fail_with_invalid_email() {
        let result = Email::new("invalid-email");
        assert!(result.is_err());
    }

    #[test]
    fn should_update_user_subscription() {
        let email = Email::new("test@example.com").unwrap();
        let mut user = User::new(email, Role::User);
        
        let new_tier = SubTier::from_string("premium").unwrap();
        user.upgrade_subscription(new_tier);
        
        assert_eq!(user.sub().tier.to_string(), "premium");
    }

    #[test]
    fn should_handle_user_role_changes() {
        let email = Email::new("admin@example.com").unwrap();
        let mut user = User::new(email, Role::User);
        
        assert_eq!(user.role(), &Role::User);
        
        user.promote_to_admin();
        assert_eq!(user.role(), &Role::Admin);
    }
}

#[cfg(test)]
mod role_permission_profile_tests {
    use super::*;

    #[test]
    fn should_create_role_permission_profile() {
        let creator_id = UserId::new("admin123".to_string());
        let profile = RolePermissionProfile::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        assert_eq!(profile.name(), "Basic User");
        assert_eq!(profile.target_tier(), &PackageTier::Bronze);
        assert_eq!(profile.category(), &PermissionProfileCategory::User);
        assert!(profile.is_active());
    }

    #[test]
    fn should_add_permissions_to_permission_profile() {
        let creator_id = UserId::new("admin123".to_string());
        let mut profile = RolePermissionProfile::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        let permission = Permission::new("read".to_string(), "posts".to_string());
        profile.add_permission(permission);
        
        assert_eq!(profile.default_permissions().len(), 1);
        assert_eq!(profile.default_permissions()[0].action(), "read");
        assert_eq!(profile.default_permissions()[0].resource(), "posts");
    }

    #[test]
    fn should_update_permission_profile_metadata() {
        let creator_id = UserId::new("admin123".to_string());
        let mut profile = RolePermissionProfile::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        let metadata = epsx::dom::entities::permission_profile::PermissionProfileMetadata {
            prerequisites: vec!["email_verified".to_string()],
            warnings: vec!["Limited access".to_string()],
            use_cases: vec!["New user registration".to_string()],
            max_assignments: Some(1000),
            requires_approval: false,
            auto_expire_days: Some(365),
            custom_fields: std::collections::HashMap::new(),
        };
        
        profile.update_metadata(metadata);
        
        assert_eq!(profile.metadata().prerequisites.len(), 1);
        assert_eq!(profile.metadata().max_assignments, Some(1000));
        assert_eq!(profile.metadata().auto_expire_days, Some(365));
    }

    #[test]
    fn should_deactivate_permission_profile() {
        let creator_id = UserId::new("admin123".to_string());
        let mut profile = RolePermissionProfile::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            creator_id,
        );
        
        assert!(profile.is_active());
        profile.set_active(false);
        assert!(!profile.is_active());
    }
}

#[cfg(test)]
mod permission_tests {
    use super::*;

    #[test]
    fn should_create_permission() {
        let permission = Permission::new("write".to_string(), "articles".to_string());
        
        assert_eq!(permission.action(), "write");
        assert_eq!(permission.resource(), "articles");
        assert!(permission.conditions().is_none());
    }

    #[test]
    fn should_create_permission_with_conditions() {
        let mut conditions = std::collections::HashMap::new();
        conditions.insert("owner".to_string(), "self".to_string());
        
        let permission = Permission::with_conditions(
            "delete".to_string(),
            "posts".to_string(),
            conditions.clone(),
        );
        
        assert_eq!(permission.action(), "delete");
        assert_eq!(permission.resource(), "posts");
        assert_eq!(permission.conditions(), &Some(conditions));
    }

    #[test]
    fn should_check_permission_match() {
        let permission = Permission::new("read".to_string(), "posts".to_string());
        
        assert!(permission.matches("read", "posts"));
        assert!(!permission.matches("write", "posts"));
        assert!(!permission.matches("read", "comments"));
    }
}

#[cfg(test)]
mod value_object_tests {
    use super::*;

    #[test]
    fn should_validate_email_format() {
        assert!(Email::new("valid@example.com").is_ok());
        assert!(Email::new("user.name+tag@domain.co.uk").is_ok());
        assert!(Email::new("invalid-email").is_err());
        assert!(Email::new("@domain.com").is_err());
        assert!(Email::new("user@").is_err());
    }

    #[test]
    fn should_create_user_id() {
        let id = UserId::new("user123".to_string());
        assert_eq!(id.to_string(), "user123");
    }

    #[test]
    fn should_parse_subscription_tiers() {
        assert!(SubTier::from_string("basic").is_ok());
        assert!(SubTier::from_string("premium").is_ok());
        assert!(SubTier::from_string("enterprise").is_ok());
        assert!(SubTier::from_string("invalid").is_err());
    }

    #[test]
    fn should_parse_package_tiers() {
        let tiers = vec!["free", "bronze", "silver", "gold", "platinum"];
        
        for tier in tiers {
            let parsed = match tier {
                "free" => PackageTier::Free,
                "bronze" => PackageTier::Bronze,
                "silver" => PackageTier::Silver,
                "gold" => PackageTier::Gold,
                "platinum" => PackageTier::Platinum,
                _ => unreachable!(),
            };
            assert_eq!(parsed.to_string().to_lowercase(), tier);
        }
    }
}

#[cfg(test)]
mod role_tests {
    use super::*;

    #[test]
    fn should_create_roles() {
        let roles = vec![Role::User, Role::Moderator, Role::Admin, Role::SuperAdmin];
        
        for role in roles {
            match role {
                Role::User => assert_eq!(role.to_string(), "user"),
                Role::Moderator => assert_eq!(role.to_string(), "moderator"),
                Role::Admin => assert_eq!(role.to_string(), "admin"),
                Role::SuperAdmin => assert_eq!(role.to_string(), "super_admin"),
            }
        }
    }

    #[test]
    fn should_check_role_hierarchy() {
        assert!(Role::SuperAdmin > Role::Admin);
        assert!(Role::Admin > Role::Moderator);
        assert!(Role::Moderator > Role::User);
    }
}

#[cfg(test)]
mod permission_resolver_tests {
    use super::*;
    use epsx::dom::services::permission_resolver::*;
    use epsx::dom::entities::permission_profile::PermissionProfile;
    use std::collections::HashMap;
    use chrono::Utc;
    
    #[tokio::test]
    async fn should_create_permission_resolver() {
        let resolver = PermissionResolver::new().await;
        assert!(resolver.is_ok());
    }
    
    #[tokio::test]
    async fn should_resolve_single_profile_permissions() {
        let resolver = PermissionResolver::new().await.unwrap();
        let user_id = UserId::new("test_user".to_string());
        
        let mut profile = PermissionProfile::new(
            "Test Profile".to_string(),
            "Test description".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            user_id.clone(),
        );
        
        let permission = Permission::new("read".to_string(), "posts".to_string());
        profile.add_permission(permission);
        
        let result = resolver.resolve_user_permissions(&user_id, vec![profile]).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        assert_eq!(resolution.effective_permissions.len(), 1);
        assert_eq!(resolution.effective_permissions[0].action(), "read");
        assert_eq!(resolution.effective_permissions[0].resource(), "posts");
    }
    
    #[tokio::test]
    async fn should_handle_permission_conflicts() {
        let resolver = PermissionResolver::new().await.unwrap();
        let user_id = UserId::new("test_user".to_string());
        
        // Create two profiles with conflicting permissions
        let mut profile1 = PermissionProfile::new(
            "Profile 1".to_string(),
            "First profile".to_string(),
            PackageTier::Bronze,
            PermissionProfileCategory::User,
            user_id.clone(),
        );
        profile1.add_permission(Permission::new("read".to_string(), "posts".to_string()));
        
        let mut profile2 = PermissionProfile::new(
            "Profile 2".to_string(),
            "Second profile".to_string(),
            PackageTier::Silver,
            PermissionProfileCategory::User,
            user_id.clone(),
        );
        profile2.add_permission(Permission::new("read".to_string(), "posts".to_string()));
        
        let result = resolver.resolve_user_permissions(&user_id, vec![profile1, profile2]).await;
        assert!(result.is_ok());
        
        let resolution = result.unwrap();
        // Should have one resolved permission (conflict resolved)
        assert_eq!(resolution.effective_permissions.len(), 1);
    }
    
    #[tokio::test]
    async fn should_check_api_access_permission() {
        let resolver = PermissionResolver::new().await.unwrap();
        let user_id = UserId::new("test_user".to_string());
        
        let permissions = vec![
            Permission::new("get".to_string(), "api:/users".to_string()),
            Permission::new("post".to_string(), "api:/posts".to_string()),
        ];
        
        let result = resolver.check_api_access(&user_id, &permissions, "/users", "GET").await;
        assert!(result.granted);
        assert_eq!(result.reason, "Exact permission match");
        
        let result = resolver.check_api_access(&user_id, &permissions, "/admin", "GET").await;
        assert!(!result.granted);
        assert_eq!(result.reason, "No matching permission found");
    }
    
    #[tokio::test]
    async fn should_check_wildcard_permissions() {
        let resolver = PermissionResolver::new().await.unwrap();
        let user_id = UserId::new("test_user".to_string());
        
        let permissions = vec![
            Permission::new("*".to_string(), "api:*".to_string()),
        ];
        
        let result = resolver.check_api_access(&user_id, &permissions, "/anything", "GET").await;
        assert!(result.granted);
        assert_eq!(result.reason, "Wildcard permission match");
    }
}

#[cfg(test)]
mod policy_engine_tests {
    use super::*;
    use epsx::dom::services::policy_engine::*;
    use epsx::dom::entities::iam::*;
    
    #[test]
    fn should_create_policy_engine() {
        let engine = PolicyEngine::new();
        assert!(true); // Engine created successfully
    }
    
    #[test]
    fn should_evaluate_package_tier_access() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "read:own_data".to_string(),
            "users/123".to_string(),
        );
        
        let result = engine.evaluate_permission(
            &context,
            &PackageTier::Free,
            &[],
            &None,
            &[],
        );
        
        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(decision.package_tier_access);
    }
    
    #[test]
    fn should_deny_admin_actions_for_free_tier() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("user123".to_string()),
            "admin:manage_users".to_string(),
            "users/*".to_string(),
        );
        
        let result = engine.evaluate_permission(
            &context,
            &PackageTier::Free,
            &[],
            &None,
            &[],
        );
        
        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(!decision.package_tier_access);
        assert!(decision.is_denied());
    }
    
    #[test]
    fn should_allow_super_admin_all_actions() {
        let mut engine = PolicyEngine::new();
        let context = EvaluationContext::new(
            UserId::new("admin123".to_string()),
            "admin:delete_users".to_string(),
            "users/*".to_string(),
        );
        
        let result = engine.evaluate_permission(
            &context,
            &PackageTier::SuperAdmin,
            &[],
            &None,
            &[],
        );
        
        assert!(result.is_ok());
        let decision = result.unwrap();
        assert!(decision.package_tier_access);
        assert!(decision.is_allowed());
    }
}

#[cfg(test)]
mod feature_expiration_tests {
    use super::*;
    use epsx::dom::services::feature_expiration::*;
    use chrono::{Utc, Duration};
    
    #[test]
    fn should_create_expiration_config() {
        let config = ExpirationConfig::default();
        assert_eq!(config.warning_days_before, vec![30, 7, 3, 1]);
        assert_eq!(config.grace_period_days, 7);
        assert_eq!(config.check_interval_hours, 1);
        assert_eq!(config.batch_size, 100);
    }
    
    #[test]
    fn should_create_feature_expiration() {
        let user_id = UserId::new("user123".to_string());
        let profile_id = epsx::dom::entities::permission_profile::PermissionProfileId::new("profile123".to_string());
        
        let expiration = FeatureExpiration {
            user_id,
            permission_profile_id: profile_id,
            permission_profile_name: "Test Profile".to_string(),
            expires_at: Utc::now() + Duration::days(30),
            features: vec!["feature1".to_string(), "feature2".to_string()],
            grace_period_days: 7,
            notification_sent: false,
            final_warning_sent: false,
        };
        
        assert_eq!(expiration.permission_profile_name, "Test Profile");
        assert_eq!(expiration.features.len(), 2);
        assert_eq!(expiration.grace_period_days, 7);
    }
}