// Infrastructure layer auth implementations

use crate::dom::entities::Session;
use crate::dom::values::{SessId};
use std::collections::HashMap;
use tokio::sync::RwLock;

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

