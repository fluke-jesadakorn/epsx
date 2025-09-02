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
  subscription: Subscription,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
  deleted_at: Option<DateTime<Utc>>,
}

impl User {
  pub fn new(firebase_uid: String, email: Email) -> Self {
    let id = UserId::generate();
    let now = Utc::now();

    
    Self {
      id: id.clone(),
      firebase_uid,
      email,
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
  ) -> Self {
    let now = chrono::Utc::now();
    Self {
      id,
      firebase_uid,
      email,
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
    subscription: Subscription,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    deleted_at: Option<chrono::DateTime<chrono::Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
      subscription,
      created_at,
      updated_at,
      deleted_at,
    }
  }

  /// Reconstruct user from complete data
  pub fn reconstruct(
    id: UserId,
    firebase_uid: String,
    email: Email,
    subscription: Subscription,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>
  ) -> Self {
    Self {
      id,
      firebase_uid,
      email,
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
  pub fn subscription(&self) -> &Subscription {
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

  
  pub fn update_email(&mut self, new_email: Email) {
    self.email = new_email;
    self.updated_at = Utc::now();
  }


  pub fn update_subscription(&mut self, subscription: Subscription) {
    self.subscription = subscription;
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



}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
  #[error("Invalid email: {0}")] InvalidEmail(String),

  #[error("Permission denied: {0}")] PermissionDenied(String),

  #[error("Invalid package tier: {0}")] InvalidPackageTier(String),

  #[error("Invalid permission: {0}")] InvalidPermission(String),

  #[error("Invalid tier downgrade: {0}")] InvalidTierDowngrade(String),
}