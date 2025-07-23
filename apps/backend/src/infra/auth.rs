// Infrastructure layer auth implementations

use crate::dom::entities::Session;
use crate::dom::values::{SessId};
use crate::app::ports::services::FbAuthSvc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;

pub struct InMemorySessionStore {
    sessions: RwLock<HashMap<SessId, Session>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn store_session(&self, session: Session) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    pub async fn get_session(&self, session_id: &SessId) -> Result<Option<Session>, Box<dyn std::error::Error>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn remove_session(&self, session_id: &SessId) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| !session.is_expired());
        Ok(())
    }
}

/// Firebase Authentication Service Implementation
#[allow(dead_code)]
pub struct FbAuthSvcImpl {
    project_id: String,
    service_account_key: Option<String>,
    api_key: Option<String>,
}

impl FbAuthSvcImpl {
    pub fn new(project_id: String, service_account_key: Option<String>, api_key: Option<String>) -> Self {
        Self {
            project_id,
            service_account_key,
            api_key,
        }
    }

    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let project_id = std::env::var("FIREBASE_PROJECT_ID")
            .map_err(|_| "FIREBASE_PROJECT_ID must be set")?;
        let service_account_key = std::env::var("FIREBASE_SERVICE_ACCOUNT_KEY").ok();
        let api_key = std::env::var("FIREBASE_API_KEY").ok();

        Ok(Self::new(project_id, service_account_key, api_key))
    }
}

#[async_trait]
impl FbAuthSvc for FbAuthSvcImpl {
    async fn verify_token(&self, _token: &str) -> Result<crate::app::ports::services::FbClaims, crate::app::ports::services::AuthServiceError> {
        // TODO: Implement actual Firebase token verification
        Err(crate::app::ports::services::AuthServiceError::InternalError(
            "Token verification not implemented".to_string()
        ))
    }

    async fn create_custom_token(&self, _uid: &str) -> Result<String, crate::app::ports::services::AuthServiceError> {
        // TODO: Implement custom token creation
        Err(crate::app::ports::services::AuthServiceError::InternalError(
            "Custom token creation not implemented".to_string()
        ))
    }

    async fn get_user(&self, _uid: &str) -> Result<crate::app::ports::services::FbUser, crate::app::ports::services::AuthServiceError> {
        // TODO: Implement user retrieval from Firebase
        Err(crate::app::ports::services::AuthServiceError::UserNotFound)
    }

    async fn list_users(&self, _page_token: Option<String>) -> Result<crate::app::ports::services::FbUserList, crate::app::ports::services::AuthServiceError> {
        // TODO: Implement user listing from Firebase
        Ok(crate::app::ports::services::FbUserList {
            users: vec![],
            next_page_token: None,
        })
    }

    async fn delete_user(&self, _uid: &str) -> Result<(), crate::app::ports::services::AuthServiceError> {
        // TODO: Implement user deletion in Firebase
        Err(crate::app::ports::services::AuthServiceError::InternalError(
            "User deletion not implemented".to_string()
        ))
    }
}