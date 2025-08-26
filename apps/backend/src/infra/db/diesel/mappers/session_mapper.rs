use uuid::Uuid;
// use chrono::Utc;

use crate::dom::entities::Session;
use crate::dom::values::{SessId, UserId};
use crate::infra::db::diesel::models::{DieselSession, NewDieselSession, UpdateDieselSession};
use crate::app::ports::repositories::RepoError;

impl TryFrom<DieselSession> for Session {
    type Error = RepoError;

    fn try_from(diesel_session: DieselSession) -> Result<Self, Self::Error> {
        let session_id = SessId::from_str(&diesel_session.id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid SessId: {}", e)))?;
        
        let user_id = UserId::from_str(&diesel_session.user_id.to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid UserId: {}", e)))?;
        
        Ok(Session::reconstruct(
            session_id,
            user_id,
            diesel_session.access_token,
            diesel_session.session_token,
            diesel_session.expires_at,
            diesel_session.created_at,
            diesel_session.is_active,
            diesel_session.ip_address.map(|ip| ip.to_string()),
        ))
    }
}

impl TryFrom<&Session> for NewDieselSession {
    type Error = RepoError;

    fn try_from(session: &Session) -> Result<Self, Self::Error> {
        let session_uuid = Uuid::parse_str(&session.id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid session UUID: {}", e)))?;
        
        let user_uuid = Uuid::parse_str(&session.user_id().to_string())
            .map_err(|e| RepoError::InvalidData(format!("Invalid user UUID: {}", e)))?;
        
        let ip_address = session.ip_address().as_ref()
            .map(|ip| ip.parse())
            .transpose()
            .map_err(|e| RepoError::InvalidData(format!("Invalid IP address: {}", e)))?;
        
        Ok(NewDieselSession {
            id: session_uuid,
            user_id: user_uuid,
            access_token: session.access_token().to_string(),
            expires_at: session.expires_at(),
            provider: Some(session.provider().to_string()),
            session_token: session.session_token().map(|s| s.to_string()),
            user_agent: session.user_agent().map(|s| s.to_string()),
            ip_address,
            is_active: session.is_active(),
            created_at: session.created_at(),
        })
    }
}

impl From<&Session> for UpdateDieselSession {
    fn from(session: &Session) -> Self {
        UpdateDieselSession {
            access_token: Some(session.access_token().to_string()),
            expires_at: Some(session.expires_at()),
            session_token: session.session_token().map(|s| s.to_string()),
            is_active: Some(session.is_active()),
        }
    }
}