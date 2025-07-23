// Simplified Firestore Session Repository Implementation

use async_trait::async_trait;
use crate::app::ports::repositories::{SessRepo, RepoError};
use crate::dom::entities::Session;
use crate::dom::values::{SessId, UserId};

pub struct FsSessRepo {
    // TODO: Implement actual Firestore connection
    _phantom: std::marker::PhantomData<()>,
}

impl FsSessRepo {
    pub fn new(_db: firestore::FirestoreDb) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl SessRepo for FsSessRepo {
    async fn get(&self, _id: &SessId) -> Result<Option<Session>, RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
    
    async fn save(&self, _session: &Session) -> Result<(), RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
    
    async fn delete(&self, _id: &SessId) -> Result<(), RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
    
    async fn find_by_user(&self, _uid: &UserId) -> Result<Vec<Session>, RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
    
    async fn cleanup_expired(&self) -> Result<u64, RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
    
    async fn deactivate_user_sessions(&self, _uid: &UserId) -> Result<(), RepoError> {
        Err(RepoError::Internal("Not implemented".to_string()))
    }
}