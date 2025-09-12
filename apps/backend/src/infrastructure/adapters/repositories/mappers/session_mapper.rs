use crate::domain::shared_kernel::value_objects::{UserId, SessionId};
use uuid::Uuid;
use std::str::FromStr;
use sha2::{Sha256, Digest};

use crate::domain::shared_kernel::DomainResult;
use crate::domain::user_management::{
    Session
};
use crate::infrastructure::adapters::repositories::diesel::models::{DieselSession, NewDieselSession, UpdateDieselSession};
use crate::infrastructure::adapters::repositories::diesel::types::DieselIpAddr;

/// Maps between domain Session aggregate and database models
pub struct SessionMapper;

impl SessionMapper {
    /// Convert SessionId to UUID for database storage
    /// Handles both UUID-format SessionIds and prefixed SessionIds (like "auth_code:xyz")
    fn session_id_to_uuid(session_id: &str) -> Uuid {
        // First try to parse as regular UUID
        if let Ok(uuid) = Uuid::from_str(session_id) {
            return uuid;
        }
        
        // For prefixed SessionIds, create deterministic UUID using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(session_id.as_bytes());
        let hash = hasher.finalize();
        
        // Use first 16 bytes of hash to create UUID
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash[..16]);
        
        // Set version to 4 (random) and variant bits
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40; // Version 4
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80; // Variant 10
        
        Uuid::from_bytes(uuid_bytes)
    }
    
    /// Convert database model to domain aggregate
    pub fn to_domain(diesel_session: DieselSession) -> DomainResult<Session> {
        let session_id = SessionId::from_string(diesel_session.id.to_string());
        let user_id = UserId::from_string(diesel_session.user_id.to_string())?;
        
        // Create session from existing data using new schema fields
        let session = Session::load(
            session_id,
            user_id,
            diesel_session.access_token,
            diesel_session.session_token, // This can be refresh_token
            diesel_session.created_at,
            diesel_session.created_at, // updated_at (using created_at as proxy)
            diesel_session.expires_at,
            diesel_session.created_at, // last_accessed_at (using created_at as proxy)
            diesel_session.ip_address.map(|ip| ip.0.to_string()),
            diesel_session.user_agent,
            !diesel_session.is_active, // is_revoked is opposite of is_active
            1, // version
        );
        
        Ok(session)
    }
    
    /// Convert domain aggregate to new database model
    pub fn to_new_diesel(session: &Session) -> DomainResult<NewDieselSession> {
        let session_uuid = Self::session_id_to_uuid(&session.id().to_string());
        let user_uuid = Self::session_id_to_uuid(&session.user_id().to_string());
        
        // Convert IP address string to DieselIpAddr
        let ip_address = match session.ip_address() {
            Some(ip_str) if !ip_str.is_empty() => {
                let ip_addr: std::net::IpAddr = ip_str.parse().map_err(|e| crate::domain::shared_kernel::DomainError::validation_error(
                        "ip_address", &format!("Invalid IP address: {}", e)
                    ))?;
                Some(DieselIpAddr(ip_addr))
            }
            _ => None
        };
        
        Ok(NewDieselSession {
            id: session_uuid,
            user_id: user_uuid,
            access_token: session.access_token().to_string(),
            expires_at: session.expires_at(),
            provider: Some("oauth".to_string()),
            session_token: session.refresh_token().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            ip_address,
            is_active: !session.is_revoked(),
        })
    }
    
    /// Convert domain aggregate to update database model
    pub fn to_update_diesel(session: &Session) -> UpdateDieselSession {
        let ip_address = match session.ip_address() {
            Some(ip_str) if !ip_str.is_empty() => {
                let ip_addr: std::net::IpAddr = ip_str.parse().unwrap_or("127.0.0.1".parse().unwrap());
                Some(DieselIpAddr(ip_addr))
            }
            _ => None
        };
        
        UpdateDieselSession {
            access_token: Some(session.access_token().to_string()),
            expires_at: Some(session.expires_at()),
            provider: Some("oauth".to_string()),
            session_token: session.refresh_token().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            ip_address,
            is_active: Some(!session.is_revoked()),
        }
    }
}