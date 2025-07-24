// Typed identifiers for domain entities

use serde::{Serialize, Deserialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PayId(pub Uuid);

// Implementations for UserId
impl UserId {
    pub fn new(s: String) -> Self {
        // Try to parse as UUID first, otherwise create from string hash
        if let Ok(uuid) = Uuid::parse_str(&s) {
            Self(uuid)
        } else {
            // For backwards compatibility with string IDs
            Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, s.as_bytes()))
        }
    }
    
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    pub fn from_string(s: String) -> Self {
        Self::new(s)
    }
    
    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for UserId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// Implementations for SessId
impl SessId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    pub fn from_string(s: String) -> Self {
        if let Ok(uuid) = Uuid::parse_str(&s) {
            Self(uuid)
        } else {
            // Fallback to generating a UUID from the string
            Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, s.as_bytes()))
        }
    }
    
    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Display for SessId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

// Implementations for PayId
impl PayId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
    
    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Display for PayId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for PayId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn should_generate_user_id() {
        let id1 = UserId::generate();
        let id2 = UserId::generate();
        
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn should_parse_user_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = UserId::from_str(uuid_str).unwrap();
        
        assert_eq!(id.to_string(), uuid_str);
    }
    
    #[test]
    fn should_convert_from_uuid() {
        let uuid = Uuid::new_v4();
        let user_id = UserId::from(uuid);
        
        assert_eq!(user_id.0, uuid);
    }
}