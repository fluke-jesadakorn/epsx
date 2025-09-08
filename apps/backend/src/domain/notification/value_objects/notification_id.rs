use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use uuid::Uuid;

/// Notification ID Value Object
/// Strong typed identifier for notifications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId {
    id: Uuid,
}

impl NotificationId {
    /// Create a new notification ID
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }

    /// Create from existing UUID
    pub fn from_uuid(id: Uuid) -> Self {
        Self { id }
    }

    /// Get the UUID value
    pub fn value(&self) -> Uuid {
        self.id
    }

    /// Get as string
    pub fn as_str(&self) -> String {
        self.id.to_string()
    }
}

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for NotificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<Uuid> for NotificationId {
    fn from(id: Uuid) -> Self {
        Self::from_uuid(id)
    }
}

impl From<NotificationId> for Uuid {
    fn from(notification_id: NotificationId) -> Self {
        notification_id.id
    }
}

impl TryFrom<String> for NotificationId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(&value)
            .map_err(|e| format!("Invalid notification ID format: {}", e))?;
        Ok(Self::from_uuid(uuid))
    }
}

impl TryFrom<&str> for NotificationId {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let uuid = Uuid::parse_str(value)
            .map_err(|e| format!("Invalid notification ID format: {}", e))?;
        Ok(Self::from_uuid(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_notification_id() {
        let id1 = NotificationId::new();
        let id2 = NotificationId::new();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.value().version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = NotificationId::from_uuid(uuid);
        
        assert_eq!(id.value(), uuid);
    }

    #[test]
    fn test_string_conversion() {
        let id = NotificationId::new();
        let id_str = id.as_str();
        let parsed_id = NotificationId::try_from(id_str).unwrap();
        
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_display() {
        let uuid = Uuid::new_v4();
        let id = NotificationId::from_uuid(uuid);
        
        assert_eq!(format!("{}", id), uuid.to_string());
    }

    #[test]
    fn test_invalid_string_conversion() {
        let result = NotificationId::try_from("invalid-uuid");
        assert!(result.is_err());
    }
}