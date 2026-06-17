use std::fmt;
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

use crate::domain::shared_kernel::{ ValueObject, Identity, new_id };
use epsx_contracts::value_object::ValueObjectError;

/// Session identifier value object
/// Represents a unique identifier for a user session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
  /// Create a new session ID with the given UUID
  pub fn from_uuid(uuid: Uuid) -> Self {
    Self(uuid)
  }

  /// Parse a session ID from a string
  pub fn from_string(s: &str) -> Result<Self, ValueObjectError> {
    let uuid = Uuid::parse_str(s).map_err(|_|
      ValueObjectError::InvalidFormat("Invalid UUID format".to_string())
    )?;
    Ok(Self(uuid))
  }

  /// Get the inner UUID
  pub fn inner(&self) -> Uuid {
    self.0
  }
}

impl ValueObject for SessionId {
  type Error = ValueObjectError;

  fn validate(&self) -> Result<(), Self::Error> {
    // UUID is always valid if it was constructed properly
    Ok(())
  }
}

impl Identity for SessionId {
  fn new() -> Self {
    Self(new_id())
  }

  fn from_uuid(uuid: Uuid) -> Self {
    Self(uuid)
  }

  fn to_uuid(&self) -> Uuid {
    self.0
  }

  fn to_string(&self) -> String {
    self.0.to_string()
  }
}

impl fmt::Display for SessionId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl From<Uuid> for SessionId {
  fn from(uuid: Uuid) -> Self {
    Self(uuid)
  }
}

impl From<SessionId> for Uuid {
  fn from(sid: SessionId) -> Self {
    sid.0
  }
}

impl From<SessionId> for String {
  fn from(sid: SessionId) -> Self {
    sid.0.to_string()
  }
}
