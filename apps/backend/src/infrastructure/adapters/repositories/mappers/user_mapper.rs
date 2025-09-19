use crate::domain::shared_kernel::value_objects::UserId;
use std::collections::HashSet;

use crate::domain::shared_kernel::DomainResult;
use crate::domain::user_management::{
    User, Email, FirebaseUid, Permission
};
use crate::infrastructure::adapters::repositories::database_types::{User as DbUser, NewUser as NewDbUser, UpdateUser as UpdateDbUser};

/// Maps between domain User aggregate and database models
pub struct UserMapper;

impl UserMapper {
    /// Convert database model to domain aggregate
    pub fn to_domain(db_user: DbUser, permissions: Vec<String>) -> DomainResult<User> {
        let user_id = UserId::from_string(db_user.id.to_string())?;
        let email = Email::new(&db_user.email)?;
        let firebase_uid = FirebaseUid::new(&db_user.firebase_uid)?;
        
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
            db_user.is_active, // Use actual is_active field
            db_user.email_verified, // Use actual email_verified field
            domain_permissions,
            db_user.created_at, // Already DateTime<Utc>
            db_user.updated_at, // Already DateTime<Utc>
            db_user.last_login_at, // Already Option<DateTime<Utc>>
            1, // version - would need to be stored in DB for proper event sourcing
        );
        
        Ok(user)
    }
    
    /// Convert domain aggregate to new database model
    pub fn to_new_diesel(user: &User) -> DomainResult<NewDbUser> {
        Ok(NewDbUser {
            email: user.email().to_string(),
            firebase_uid: user.firebase_uid().to_string(),
        })
    }
    
    /// Convert domain aggregate to update database model
    pub fn to_update_diesel(user: &User) -> UpdateDbUser {
        UpdateDbUser {
            email: Some(user.email().to_string()),
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