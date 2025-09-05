use chrono::Utc;
use uuid::Uuid;
use std::str::FromStr;
use std::collections::HashSet;

use crate::domain::shared_kernel::{DomainResult, AggregateRoot};
use crate::domain::user_management::{
    User, UserId, Email, FirebaseUid, Permission
};
use crate::infra::db::diesel::models::{DieselUser, NewDieselUser, UpdateDieselUser};

/// Maps between domain User aggregate and database models
pub struct UserMapper;

impl UserMapper {
    /// Convert database model to domain aggregate
    pub fn to_domain(diesel_user: DieselUser, permissions: Vec<String>) -> DomainResult<User> {
        let user_id = UserId::from_string(&diesel_user.id.to_string())?;
        let email = Email::new(&diesel_user.email)?;
        let firebase_uid = FirebaseUid::new(&diesel_user.firebase_uid)?;
        
        // Convert string permissions to Permission value objects
        let mut domain_permissions = HashSet::new();
        for perm_str in permissions {
            let permission = Permission::new(&perm_str)?;
            domain_permissions.insert(permission);
        }
        
        // Create user from existing data
        let user = User::load(
            user_id,
            firebase_uid,
            email,
            diesel_user.is_active.unwrap_or(true),
            diesel_user.email_verified.unwrap_or(false),
            domain_permissions,
            diesel_user.created_at,
            diesel_user.updated_at,
            None, // last_login_at - not stored in this table
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
        
        Ok(NewDieselUser {
            id: uuid,
            firebase_uid: user.firebase_uid().to_string(),
            email: user.email().to_string(),
            display_name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            avatar_url: None,
            email_verified: Some(user.is_email_verified()),
            is_active: Some(user.is_active()),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
            primary_platform_id: None,
        })
    }
    
    /// Convert domain aggregate to update database model
    pub fn to_update_diesel(user: &User) -> UpdateDieselUser {
        UpdateDieselUser {
            display_name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            avatar_url: None,
            email_verified: Some(user.is_email_verified()),
            is_active: Some(user.is_active()),
            last_login_at: None, // Would need to be set separately
            updated_at: Utc::now(),
        }
    }
    
    /// Extract permission strings from domain aggregate
    pub fn extract_permissions(user: &User) -> Vec<String> {
        user.permissions().iter()
            .map(|p| p.to_string())
            .collect()
    }
}