// User domain entity with minimal naming conventions
// PERMISSION-ONLY SYSTEM - No role field, using structured permissions only

use chrono::{ DateTime, Utc };
use serde::{ Serialize, Deserialize };

use crate::dom::values::{ UserId, Email, Subscription };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  id: UserId,
  firebase_uid: String,
  email: Email,
  // permissions: Vec<String>,  // REMOVED: Now stored in separate user_permissions table
  // package_tier removed - using permissions only
  // role field removed - using permissions only
  subscription: Subscription,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  deleted_at: Option<DateTime<Utc>>,
}

impl User {
  pub fn new(firebase_uid: String, email: Email) -> Self {
    let id = UserId::generate();
    let now = Utc::now();

    // Default permissions are now handled by permission service during user creation
    
    Self {
      id: id.clone(),
      firebase_uid,
      email,
      // permissions field removed - handled by separate table
      subscription: Subscription::free(),
      created_at: now,
      updated_at: now,
      deleted_at: None,
    }
  }

  /// Create user from existing database data (basic version)
  /// Note: Permissions are now handled by separate user_permissions table
  pub fn from_existing(
    id: UserId,
    firebase_uid: String,
    email: Email,
    // permissions parameter removed - handled by separate table
  ) -> Self {
    let now = chrono::Utc::now();
    Self {
      id,
      firebase_uid,
      email,
      // permissions field removed - handled by separate table
      subscription: Subscription::free(),
      created_at: now,
      updated_at: now,
      deleted_at: None,
    }
  }

  /// Create user from complete existing database data
  /// Note: Permissions are now handled by separate user_permissions table
  pub fn from_existing_complete(
    id: UserId,
    firebase_uid: String,
    email: Email,
    // permissions parameter removed - handled by separate table
    subscription: Subscription,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
      // permissions field removed - handled by separate table
      subscription,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  /// Reconstruct user from complete data 
  /// Note: Permissions are now handled by separate user_permissions table
  pub fn reconstruct(
    id: UserId,
    firebase_uid: String,
    email: Email,
    // permissions parameter removed - handled by separate table
    subscription: Subscription,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
      // permissions field removed - handled by separate table
      subscription,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  // Getters
  pub fn id(&self) -> &UserId {
    &self.id
  }
  pub fn firebase_uid(&self) -> &str {
    &self.firebase_uid
  }
  pub fn email(&self) -> &Email {
    &self.email
  }
  // permissions() getter removed - permissions now handled by separate user_permissions table
  // Use PermissionApplicationService to fetch user permissions
  pub fn subscription(&self) -> &Subscription {
    &self.subscription
  }

  // Backward compatibility getters (deprecated)
  #[deprecated(note = "Use subscription() instead")]
  pub fn sub(&self) -> &Subscription {
    &self.subscription
  }
  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }
  pub fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }
  pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
    self.deleted_at
  }
  pub fn is_deleted(&self) -> bool {
    self.deleted_at.is_some()
  }

  /// Check if user is active (not deleted)
  pub fn is_active(&self) -> bool {
    self.deleted_at.is_none()
  }

  // Permission methods moved to PermissionApplicationService
  // Use PermissionApplicationService for all permission operations:
  // - grant_permission_to_user()
  // - revoke_permission_from_user() 
  // - set_user_permissions()
  // - check_user_permission()

  // Package tier system removed - now using permissions only
  
  // derived_tier() method removed - tier derivation now handled by PermissionApplicationService
  // Use PermissionApplicationService.derive_user_tier(firebase_uid) instead
  
  pub fn update_email(&mut self, new_email: Email) {
    self.email = new_email;
    self.updated_at = Utc::now();
  }

  // upgrade_permissions() method removed - permission upgrades now handled by PermissionApplicationService
  // Use PermissionApplicationService.set_user_permissions() instead

  // Permission check methods removed - permission checks now handled by PermissionApplicationService
  // Use PermissionApplicationService.check_user_permission() instead

  pub fn update_subscription(&mut self, subscription: Subscription) {
    self.subscription = subscription;
    self.updated_at = Utc::now();
  }

  // Backward compatibility method (deprecated)
  #[deprecated(note = "Use update_subscription() instead")]
  pub fn update_sub(&mut self, sub: Subscription) {
    self.subscription = sub;
    self.updated_at = Utc::now();
  }

  pub fn soft_delete(&mut self) {
    self.deleted_at = Some(Utc::now());
    self.updated_at = Utc::now();
  }

  pub fn restore(&mut self) {
    self.deleted_at = None;
    self.updated_at = Utc::now();
  }

  // Permission-related methods removed - all permission operations now handled by PermissionApplicationService
  // Use PermissionApplicationService for:
  // - get_user_permissions()
  // - check_user_permission()  
  // - has_admin_permission()
  // - derive_user_role()

  /// Convert legacy role to permissions (for migration)
  pub fn convert_legacy_role_to_permissions(role: &str) -> Vec<String> {
    // Legacy conversion - simplified without external function
    match role.to_lowercase().as_str() {
      "admin" => vec!["admin:*:*".to_string()],
      "user" => vec!["epsx:analytics:view".to_string(), "epsx:profile:manage".to_string()],
      "guest" => vec!["epsx:analytics:view".to_string()],
      _ => vec!["epsx:analytics:view".to_string()],
    }
  }

  // assign_tier_permissions() method removed - tier-based permission assignment now handled by PermissionApplicationService
  // Use PermissionApplicationService.assign_tier_permissions() instead
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
  #[error("Invalid email: {0}")] InvalidEmail(String),

  #[error("Permission denied: {0}")] PermissionDenied(String),

  #[error("Invalid package tier: {0}")] InvalidPackageTier(String),

  #[error("Invalid permission: {0}")] InvalidPermission(String),

  #[error("Invalid tier downgrade: {0}")] InvalidTierDowngrade(String),
}