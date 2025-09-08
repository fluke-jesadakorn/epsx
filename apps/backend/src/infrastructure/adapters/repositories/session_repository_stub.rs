// Stub implementation for SessionRepository with legacy method names
// This bridges the gap between the domain interface and actual implementation

use async_trait::async_trait;
use std::fmt;

use crate::domain::user_management::aggregates::session::Session;

#[derive(Debug)]
pub struct SessionRepositoryStubError {
    pub message: String,
}

impl fmt::Display for SessionRepositoryStubError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Session Repository Stub Error: {}", self.message)
    }
}

impl std::error::Error for SessionRepositoryStubError {}

pub struct SessionRepositoryStub;

impl SessionRepositoryStub {
    pub fn new() -> Self {
        Self
    }
    
    // Legacy method names that adapters are trying to call
    pub async fn find_session_by_id(&self, _id: i32) -> Result<Option<Session>, Box<dyn std::error::Error + Send + Sync>> {
        // Return None for now - would integrate with actual session storage
        Ok(None)
    }
    
    pub async fn find_sessions_by_user_id(&self, _user_id: i32) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty vector for now
        Ok(vec![])
    }
    
    pub async fn create_session(&self, _session: Session) -> Result<Session, Box<dyn std::error::Error + Send + Sync>> {
        // Return the session back for now - would save to storage
        Err("Session creation not implemented".into())
    }
    
    pub async fn delete_session(&self, _id: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op for now
        Ok(())
    }
    
    pub async fn cleanup_expired_sessions(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Return 0 cleaned up sessions
        Ok(0)
    }
    
    pub async fn update_last_accessed(&self, _id: i32, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // No-op for now
        Ok(())
    }
    
    pub async fn find_expired_sessions(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>> {
        // Return empty vector for now
        Ok(vec![])
    }
}