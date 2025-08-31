use uuid::Uuid;
use chrono::Utc;

use crate::dom::entities::User;
use crate::dom::values::{UserId, Email, Subscription};
use crate::infra::db::diesel::models::{DieselUser, NewDieselUser, UpdateDieselUser};
use crate::app::ports::repositories::RepoError;

impl TryFrom<DieselUser> for User {
    type Error = RepoError;

    fn try_from(diesel_user: DieselUser) -> Result<Self, Self::Error> {
        let user_id = UserId::from(diesel_user.id);
        
        let email = Email::new(diesel_user.email)
            .map_err(|e| RepoError::InvalidData(format!("Invalid email: {}", e)))?;
        
        let subscription = Subscription::new(crate::dom::values::SubscriptionTier::Basic);
        
        // Permissions now handled by separate user_permissions table
        
        Ok(User::from_existing_complete(
            user_id,
            diesel_user.firebase_uid,
            email,
            // permissions removed - now handled by separate table
            subscription,
            diesel_user.created_at,
            diesel_user.updated_at,
            None, // deleted_at not in diesel_user
        ))
    }
}

impl From<&User> for NewDieselUser {
    fn from(user: &User) -> Self {
        let uuid = Uuid::parse_str(&user.id().to_string())
            .expect("User ID should be valid UUID");
        
        NewDieselUser {
            id: uuid,
            firebase_uid: user.firebase_uid().to_string(),
            email: user.email().to_string(),
            display_name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            avatar_url: None,
            email_verified: Some(true), // Default to true for simple system
            is_active: Some(user.is_active()),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
            primary_platform_id: None,
        }
    }
}

impl From<&User> for UpdateDieselUser {
    fn from(user: &User) -> Self {
        UpdateDieselUser {
            display_name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()),
            avatar_url: None,
            email_verified: Some(true), // Default to true for simple system
            is_active: Some(user.is_active()),
            last_login_at: None, // This would need to be set separately
            updated_at: Utc::now(),
        }
    }
}