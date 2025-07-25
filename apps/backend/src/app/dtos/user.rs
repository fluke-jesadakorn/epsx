// User management DTOs

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::dom::entities::User;
use crate::dom::values::{UserId, Role};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    #[serde(rename = "creationTime")]
    pub creation_time: Option<String>,
    #[serde(rename = "lastSignInTime")]
    pub last_sign_in_time: Option<String>,
    #[serde(rename = "lastRefreshTime")]
    pub last_refresh_time: Option<String>,
}

// User DTO for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub uid: String,
    pub email: String,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub disabled: bool,
    pub role: String,
    pub sub_tier: String,
    pub perms: Vec<String>,
    pub created_at: u64,
    pub last_updated: Option<u64>,
    pub metadata: UserMetadata,
}

// Create user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserReq {
    pub email: String,
    pub role: String,
    pub fb_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRes {
    pub usr: UserDto,
}

// Get user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserReq {
    pub usr_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserRes {
    pub usr: UserDto,
}

// Update user role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleReq {
    pub usr_id: UserId,
    pub new_role: String,
    pub admin_id: UserId, // Who is making this change
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRes {
    pub usr: UserDto,
    pub event_id: String,
}

// List users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersReq {
    pub offset: u32,
    pub limit: u32,
    pub role_filter: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersRes {
    pub users: Vec<UserDto>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}

// User profile update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileReq {
    pub usr_id: UserId,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRes {
    pub usr: UserDto,
}

// User subscription update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubReq {
    pub usr_id: UserId,
    pub tier: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubRes {
    pub usr: UserDto,
}

// Bulk user level updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateLevelsReq {
    pub updates: Vec<UserLevelUpdate>,
    pub admin_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLevelUpdate {
    pub usr_id: UserId,
    pub new_role: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateLevelsRes {
    pub updated_users: Vec<UserDto>,
    pub failed_updates: Vec<FailedUpdate>,
    pub total_processed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUpdate {
    pub usr_id: String,
    pub error: String,
}

// Soft delete user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftDeleteUserReq {
    pub usr_id: String,
    pub reason: Option<String>,
}

impl SoftDeleteUserReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.usr_id.is_empty() {
            return Err(ValidationError::EmptyField("usr_id".to_string()));
        }
        
        if let Some(reason) = &self.reason {
            if reason.len() > 500 {
                return Err(ValidationError::ReasonTooLong(reason.len()));
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftDeleteUserRes {
    pub usr: UserDto,
    pub deleted_at: DateTime<Utc>,
}

// User statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatsReq {
    pub include_roles: bool,
    pub include_tiers: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatsRes {
    #[serde(rename = "totalUsers")]
    pub total_users: u64,
    #[serde(rename = "verifiedUsers")]
    pub verified_users: u64,
    #[serde(rename = "disabledUsers")]
    pub disabled_users: u64,
    #[serde(rename = "adminUsers")]
    pub admin_users: u64,
    #[serde(rename = "verificationRate")]
    pub verification_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleCount {
    pub role: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCount {
    pub tier: String,
    pub count: u64,
}

// User level history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLevelHistoryReq {
    pub usr_id: UserId,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLevelHistoryRes {
    pub history: Vec<LevelChangeRecord>,
    pub total_changes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelChangeRecord {
    pub id: String,
    pub usr_id: String,
    pub old_role: String,
    pub new_role: String,
    pub changed_by: String,
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
}

// Enhanced user DTO with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithStatsDto {
    #[serde(flatten)]
    pub user: UserDto,
    pub login_count: u64,
    pub last_login_at: Option<DateTime<Utc>>,
    pub level_changes: u32,
    pub created_days_ago: i64,
}

// Utility implementations
impl UserDto {
    pub fn from_entity(user: &User) -> Self {
        Self {
            uid: user.id().to_string(),
            email: user.email().value().to_string(),
            email_verified: true, // Assume verified by default
            display_name: None, // Not stored in domain entity
            disabled: false, // Not stored in domain entity
            role: user.role().to_string(),
            sub_tier: user.sub().tier().to_string(),
            perms: user.perms().to_vec(),
            created_at: user.created_at().timestamp() as u64,
            last_updated: Some(user.updated_at().timestamp() as u64),
            metadata: UserMetadata {
                creation_time: Some(user.created_at().to_rfc3339()),
                last_sign_in_time: None, // Not available in domain
                last_refresh_time: None, // Not available in domain
            },
        }
    }
}

impl CreateUserReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.email.is_empty() {
            return Err(ValidationError::EmptyField("email".to_string()));
        }
        
        if !self.email.contains('@') {
            return Err(ValidationError::InvalidEmail(self.email.clone()));
        }
        
        // Validate role
        self.role.parse::<Role>()
            .map_err(|_| ValidationError::InvalidRole(self.role.clone()))?;
        
        Ok(())
    }
}

impl UpdateRoleReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.new_role.parse::<Role>()
            .map_err(|_| ValidationError::InvalidRole(self.new_role.clone()))?;
        Ok(())
    }
}

impl ListUsersReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.limit > 1000 {
            return Err(ValidationError::LimitTooLarge(self.limit));
        }
        
        if let Some(role) = &self.role_filter {
            role.parse::<Role>()
                .map_err(|_| ValidationError::InvalidRole(role.clone()))?;
        }
        
        Ok(())
    }
}

impl BulkUpdateLevelsReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.updates.is_empty() {
            return Err(ValidationError::EmptyField("updates".to_string()));
        }
        
        if self.updates.len() > 100 {
            return Err(ValidationError::BulkUpdateTooLarge(self.updates.len()));
        }
        
        for update in &self.updates {
            update.new_role.parse::<Role>()
                .map_err(|_| ValidationError::InvalidRole(update.new_role.clone()))?;
        }
        
        Ok(())
    }
}

impl UserStatsReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            if start > end {
                return Err(ValidationError::InvalidDateRange);
            }
        }
        Ok(())
    }
}

impl GetLevelHistoryReq {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let Some(limit) = self.limit {
            if limit > 1000 {
                return Err(ValidationError::LimitTooLarge(limit));
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Field cannot be empty: {0}")]
    EmptyField(String),
    
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    #[error("Limit too large: {0} (max 1000)")]
    LimitTooLarge(u32),
    
    #[error("Invalid subscription tier: {0}")]
    InvalidSubTier(String),
    
    #[error("Bulk update too large: {0} updates (max 100)")]
    BulkUpdateTooLarge(usize),
    
    #[error("Invalid date range: start date must be before end date")]
    InvalidDateRange,
    
    #[error("Reason too long: {0} characters (max 500)")]
    ReasonTooLong(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::Email;
    
    #[test]
    fn should_create_user_dto_from_entity() {
        let user = User::new(
            Email::new("test@example.com").unwrap(),
            Role::User
        );
        
        let dto = UserDto::from_entity(&user);
        
        assert_eq!(dto.email, "test@example.com");
        assert_eq!(dto.role, "user");
        assert!(!dto.perms.is_empty());
    }
    
    #[test]
    fn should_validate_create_user_request() {
        let valid_req = CreateUserReq {
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            fb_token: None,
        };
        
        assert!(valid_req.validate().is_ok());
        
        let invalid_req = CreateUserReq {
            email: "invalid-email".to_string(),
            role: "user".to_string(),
            fb_token: None,
        };
        
        assert!(invalid_req.validate().is_err());
    }
    
    #[test]
    fn should_validate_list_users_request() {
        let valid_req = ListUsersReq {
            offset: 0,
            limit: 50,
            role_filter: Some("user".to_string()),
            page_token: None,
        };
        
        assert!(valid_req.validate().is_ok());
        
        let invalid_req = ListUsersReq {
            offset: 0,
            limit: 2000, // Too large
            role_filter: None,
            page_token: None,
        };
        
        assert!(invalid_req.validate().is_err());
    }
}