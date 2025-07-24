// Unit tests for EPSX backend components
use epsx::dom::entities::{User, RoleTemplate, Permission};
use epsx::dom::values::{Email, UserId, Role, SubTier};
use epsx::dom::entities::iam::{PackageTier};
use epsx::dom::entities::template::TemplateCategory;

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
mod role_template_tests {
    use super::*;

    #[test]
    fn should_create_role_template() {
        let creator_id = UserId::new("admin123".to_string());
        let template = RoleTemplate::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        assert_eq!(template.name(), "Basic User");
        assert_eq!(template.target_tier(), &PackageTier::Bronze);
        assert_eq!(template.category(), &TemplateCategory::User);
        assert!(template.is_active());
    }

    #[test]
    fn should_add_permissions_to_template() {
        let creator_id = UserId::new("admin123".to_string());
        let mut template = RoleTemplate::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        let permission = Permission::new("read".to_string(), "posts".to_string());
        template.add_permission(permission);
        
        assert_eq!(template.default_permissions().len(), 1);
        assert_eq!(template.default_permissions()[0].action(), "read");
        assert_eq!(template.default_permissions()[0].resource(), "posts");
    }

    #[test]
    fn should_update_template_metadata() {
        let creator_id = UserId::new("admin123".to_string());
        let mut template = RoleTemplate::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        let metadata = epsx::dom::entities::template::TemplateMetadata {
            prerequisites: vec!["email_verified".to_string()],
            warnings: vec!["Limited access".to_string()],
            use_cases: vec!["New user registration".to_string()],
            max_assignments: Some(1000),
            requires_approval: false,
            auto_expire_days: Some(365),
            custom_fields: std::collections::HashMap::new(),
        };
        
        template.update_metadata(metadata);
        
        assert_eq!(template.metadata().prerequisites.len(), 1);
        assert_eq!(template.metadata().max_assignments, Some(1000));
        assert_eq!(template.metadata().auto_expire_days, Some(365));
    }

    #[test]
    fn should_deactivate_template() {
        let creator_id = UserId::new("admin123".to_string());
        let mut template = RoleTemplate::new(
            "Basic User".to_string(),
            "Standard user permissions".to_string(),
            PackageTier::Bronze,
            TemplateCategory::User,
            creator_id,
        );
        
        assert!(template.is_active());
        template.set_active(false);
        assert!(!template.is_active());
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