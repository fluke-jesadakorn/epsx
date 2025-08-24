// User domain entity with minimal naming conventions

use chrono::{ DateTime, Utc };
use serde::{ Serialize, Deserialize };

use crate::dom::values::{ UserId, Email, Subscription };
use crate::dom::events::UserPermissionChangedEvent;

/// User role enum for role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
  User,
  Moderator,
  Admin,
}

impl ToString for UserRole {
  fn to_string(&self) -> String {
    match self {
      UserRole::User => "user".to_string(),
      UserRole::Moderator => "moderator".to_string(),
      UserRole::Admin => "admin".to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  id: UserId,
  firebase_uid: String,
  email: Email,
  admin_modules: Vec<String>,
  package_tier: String,
  subscription: Subscription,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  deleted_at: Option<DateTime<Utc>>,
}

impl User {
  pub fn new(firebase_uid: String, email: Email, package_tier: String) -> Self {
    let id = UserId::generate();
    let now = Utc::now();

    Self {
      id: id.clone(),
      firebase_uid,
      email,
      admin_modules: Vec::new(),
      package_tier,
      subscription: Subscription::free(),
      created_at: now,
      updated_at: now,
      deleted_at: None,
    }
  }

  /// Create user from existing database data (basic version)
  pub fn from_existing(
    id: UserId,
    firebase_uid: String,
    email: Email,
    package_tier: crate::dom::entities::iam::PackageTier
  ) -> Self {
    let now = chrono::Utc::now();
    Self {
      id,
      firebase_uid,
      email,
      admin_modules: Vec::new(),
      package_tier: format!("{:?}", package_tier),
      subscription: Subscription::free(),
      created_at: now,
      updated_at: now,
      deleted_at: None,
    }
  }

  /// Create user from complete existing database data
  pub fn from_existing_complete(
    id: UserId,
    firebase_uid: String,
    email: Email,
    admin_modules: Vec<String>,
    package_tier: String,
    subscription: Subscription,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
      admin_modules,
      package_tier,
      subscription,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  pub fn reconstruct(
    id: UserId,
    firebase_uid: String,
    email: Email,
    admin_modules: Vec<String>,
    package_tier: String,
    subscription: Subscription,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
      admin_modules,
      package_tier,
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
  pub fn admin_modules(&self) -> &Vec<String> {
    &self.admin_modules
  }
  pub fn package_tier(&self) -> &str {
    &self.package_tier
  }
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

  /// Get user role based on admin modules and package tier
  pub fn role(&self) -> UserRole {
    if !self.admin_modules.is_empty() {
      if self.admin_modules.contains(&"system_admin".to_string()) {
        UserRole::Admin
      } else {
        UserRole::Moderator
      }
    } else {
      UserRole::User
    }
  }

  /// Check if user is active (not deleted)
  pub fn is_active(&self) -> bool {
    self.deleted_at.is_none()
  }

  // Business methods
  pub fn assign_admin_module(&mut self, module: String) {
    if !self.admin_modules.contains(&module) {
      self.admin_modules.push(module);
      self.updated_at = Utc::now();
    }
  }

  pub fn remove_admin_module(&mut self, module: &str) {
    self.admin_modules.retain(|m| m != module);
    self.updated_at = Utc::now();
  }

  pub fn update_package_tier(&mut self, new_tier: String) {
    self.package_tier = new_tier;
    self.updated_at = Utc::now();
  }

  pub fn upgrade_package_tier(
    &mut self,
    new_tier: crate::dom::entities::iam::PackageTier,
    new_admin_modules: Option<Vec<String>>
  ) -> Result<UserPermissionChangedEvent, DomainError> {
    // Validate tier upgrade logic
    use crate::dom::entities::iam::PackageTier;
    let _current_tier = self.package_tier
      .parse::<PackageTier>()
      .unwrap_or(PackageTier::Free);

    // Allow same tier or upgrade (no strict downgrade prevention for now)
    // In a more sophisticated system, you'd implement Ord for PackageTier

    let old_tier = self.package_tier.clone();
    let old_modules = self.admin_modules.clone();

    self.package_tier = new_tier.to_string();

    let (modules_added, modules_removed) = if
      let Some(modules) = new_admin_modules
    {
      let added = modules
        .iter()
        .filter(|m| !old_modules.contains(m))
        .cloned()
        .collect();
      let removed = old_modules
        .iter()
        .filter(|m| !modules.contains(m))
        .cloned()
        .collect();
      self.admin_modules = modules;
      (added, removed)
    } else {
      (Vec::new(), Vec::new())
    };

    self.updated_at = Utc::now();

    Ok(
      UserPermissionChangedEvent::new(
        self.id.clone(),
        modules_added,
        modules_removed,
        old_tier,
        new_tier.to_string()
      )
    )
  }

  pub fn has_admin_module(&self, module: &str) -> bool {
    self.admin_modules.contains(&module.to_string())
  }

  pub fn is_admin(&self) -> bool {
    !self.admin_modules.is_empty()
  }

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

  /// Get user permissions based on package tier and admin modules
  pub fn permissions(&self) -> Vec<String> {
    let mut perms = Vec::new();

    // Add base permissions based on package tier
    match self.package_tier.as_str() {
      "BRONZE" => perms.push("basic_access".to_string()),
      "SILVER" => {
        perms.push("basic_access".to_string());
        perms.push("premium_features".to_string());
      }
      "GOLD" => {
        perms.push("basic_access".to_string());
        perms.push("premium_features".to_string());
        perms.push("advanced_features".to_string());
      }
      _ => perms.push("basic_access".to_string()),
    }

    // Add admin permissions based on admin modules
    for module in &self.admin_modules {
      perms.push(format!("admin_{}", module));
    }

    perms
  }

  pub fn has_package_tier_or_higher(&self, required_tier: &str) -> bool {
    let tier_hierarchy = [
      ("FREE", 1),
      ("BRONZE", 2),
      ("SILVER", 3),
      ("GOLD", 4),
      ("PLATINUM", 5),
      ("ENTERPRISE", 6),
    ]
      .iter()
      .cloned()
      .collect::<std::collections::HashMap<_, _>>();

    let user_level = tier_hierarchy
      .get(self.package_tier.as_str())
      .unwrap_or(&0);
    let required_level = tier_hierarchy.get(required_tier).unwrap_or(&1);

    user_level >= required_level
  }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
  #[error("Invalid email: {0}")] InvalidEmail(String),

  #[error("Permission denied: {0}")] PermissionDenied(String),

  #[error("Invalid package tier: {0}")] InvalidPackageTier(String),

  #[error("Invalid admin module: {0}")] InvalidAdminModule(String),

  #[error("Invalid tier downgrade: {0}")] InvalidTierDowngrade(String),
}
