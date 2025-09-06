use crate::domain::shared_kernel::value_objects::UserId;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::str::FromStr;
use std::collections::HashSet;

use crate::domain::shared_kernel::{DomainResult, AggregateRoot};
use crate::domain::user_management::{
    User, Email, FirebaseUid, Permission
};
use crate::infrastructure::adapters::repositories::diesel::models::{DieselUser, NewDieselUser, UpdateDieselUser};

/// Maps between domain User aggregate and database models
pub struct UserMapper;

impl UserMapper {
    /// Convert database model to domain aggregate
    pub fn to_domain(diesel_user: DieselUser, permissions: Vec<String>) -> DomainResult<User> {
        let user_id = UserId::from_string(diesel_user.id.to_string())?;
        let email = Email::new(&diesel_user.email)?;
        let firebase_uid = FirebaseUid::new(&diesel_user.firebase_uid)?;
        
        // Convert string permissions to Permission value objects
        let mut domain_permissions = HashSet::new();
        for perm_str in permissions {
            let permission = Permission::new(&perm_str)?;
            domain_permissions.insert(permission);
        }
        
        // Create user from existing data using actual database fields
        let user = User::load(
            user_id,
            firebase_uid,
            email,
            diesel_user.is_active.unwrap_or(true), // Use actual is_active field with default
            diesel_user.email_verified.unwrap_or(false), // Use actual email_verified field with default
            domain_permissions,
            diesel_user.created_at, // Already DateTime<Utc>
            diesel_user.updated_at, // Already DateTime<Utc>
            diesel_user.last_login_at, // Already Option<DateTime<Utc>>
            1, // version - would need to be stored in DB for proper event sourcing
        );
        
        Ok(user)
    }
    
    /// Convert domain aggregate to new database model
    pub fn to_new_diesel(user: &User) -> DomainResult<NewDieselUser> {
        let uuid = Uuid::from_str(&user.id().to_string())
            .map_err(|e| crate::domain::shared_kernel::DomainError::validation_error(
                "user_id", &format!("Invalid UUID: {}", e)
            ))?;
        
        // Derive display_name from email
        let display_name = user.email().to_string().split('@').next().unwrap_or("User").to_string();
        
        Ok(NewDieselUser {
            id: uuid,
            firebase_uid: user.firebase_uid().to_string(),
            email: user.email().to_string(),
            display_name: Some(display_name.clone()),
            name: Some(display_name), // Use display_name as name
            avatar_url: None, // No avatar by default
            package_tier: Some(Self::map_subscription_tier(user)), // Map from permissions or default
            email_verified: Some(user.is_email_verified()),
            is_active: Some(user.is_active()),
            primary_platform_id: None, // No primary platform by default
        })
    }
    
    /// Convert domain aggregate to update database model
    pub fn to_update_diesel(user: &User) -> UpdateDieselUser {
        let display_name = user.email().to_string().split('@').next().unwrap_or("User").to_string();
        
        UpdateDieselUser {
            firebase_uid: Some(user.firebase_uid().to_string()),
            email: Some(user.email().to_string()),
            display_name: Some(display_name.clone()),
            name: Some(display_name),
            avatar_url: None, // Keep existing avatar
            package_tier: Some(Self::map_subscription_tier(user)),
            email_verified: Some(user.is_email_verified()),
            is_active: Some(user.is_active()),
            last_login_at: user.last_login_at(),
            updated_at: Some(chrono::Utc::now()),
            primary_platform_id: None, // Keep existing primary platform
        }
    }
    
    /// Extract permission strings from domain aggregate
    pub fn extract_permissions(user: &User) -> Vec<String> {
        user.permissions().iter()
            .map(|p| p.to_string())
            .collect()
    }
    
    /// Map subscription tier from user permissions or domain data
    fn map_subscription_tier(user: &User) -> String {
        // Check if user has admin permissions to determine if they have premium access
        let permissions = user.permissions();
        
        // Look for admin permissions that might indicate premium tier
        for permission in permissions {
            let perm_str = permission.to_string();
            if perm_str.starts_with("admin:") {
                return "premium".to_string();
            }
            if perm_str.contains(":premium:") || perm_str.contains(":pro:") {
                return "premium".to_string();
            }
        }
        
        // Default to free tier
        "free".to_string()
    }
    
    /// Check if user is admin based on permissions (for backwards compatibility)
    pub fn is_admin_from_permissions(user: &User) -> bool {
        let permissions = user.permissions();
        for permission in permissions {
            let perm_str = permission.to_string();
            if perm_str.starts_with("admin:") || perm_str == "admin:*:*" {
                return true;
            }
        }
        false
    }
}