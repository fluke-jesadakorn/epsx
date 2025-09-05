use uuid::Uuid;
use std::str::FromStr;

use crate::domain::shared_kernel::{DomainResult, AggregateRoot};
use crate::domain::user_management::{
    Session, SessionId, UserId
};
use crate::infra::db::diesel::models::{DieselSession, NewDieselSession, UpdateDieselSession};
use crate::infra::db::diesel::types::DieselIpAddr;

/// Maps between domain Session aggregate and database models
pub struct SessionMapper;

impl SessionMapper {
    /// Convert database model to domain aggregate
    pub fn to_domain(diesel_session: DieselSession) -> DomainResult<Session> {
        let session_id = SessionId::from_string(&diesel_session.id.to_string())?;
        let user_id = UserId::from_string(&diesel_session.user_id.to_string())?;
        
        // Create session from existing data
        let session = Session::load(
            session_id,
            user_id,
            diesel_session.access_token,
            None, // refresh_token - not stored in this table
            diesel_session.created_at,
            diesel_session.created_at, // updated_at
            diesel_session.expires_at,
            diesel_session.created_at, // last_accessed_at
            diesel_session.ip_address.map(|ip| ip.to_string()),
            diesel_session.user_agent,
            !diesel_session.is_active, // is_revoked
            1, // version
        );
        
        Ok(session)
    }
    
    /// Convert domain aggregate to new database model
    pub fn to_new_diesel(session: &Session) -> DomainResult<NewDieselSession> {
        let session_uuid = Uuid::from_str(&session.id().to_string())
            .map_err(|e| crate::domain::shared_kernel::DomainError::validation_error(
                "session_id", &format!("Invalid UUID: {}", e)
            ))?;
            
        let user_uuid = Uuid::from_str(&session.user_id().to_string())
            .map_err(|e| crate::domain::shared_kernel::DomainError::validation_error(
                "user_id", &format!("Invalid UUID: {}", e)
            ))?;
        
        // Convert IP address string to DieselIpAddr
        let ip_address = match session.ip_address() {
            Some(ip_str) if !ip_str.is_empty() => {
                Some(DieselIpAddr::from_str(ip_str)
                    .map_err(|e| crate::domain::shared_kernel::DomainError::validation_error(
                        "ip_address", &format!("Invalid IP address: {}", e)
                    ))?)
            }
            _ => None
        };
        
        Ok(NewDieselSession {
            id: session_uuid,
            user_id: user_uuid,
            access_token: session.access_token().to_string(),
            expires_at: session.expires_at(),
            provider: Some("oidc".to_string()),
            session_token: None,
            user_agent: session.user_agent().map(|s| s.to_string()),
            ip_address,
            is_active: session.is_valid(),
            created_at: session.created_at(),
        })
    }
    
    /// Convert domain aggregate to update database model
    pub fn to_update_diesel(session: &Session) -> UpdateDieselSession {
        UpdateDieselSession {
            access_token: Some(session.access_token().to_string()),
            expires_at: Some(session.expires_at()),
            session_token: None,
            is_active: Some(session.is_valid()),
        }
    }
}