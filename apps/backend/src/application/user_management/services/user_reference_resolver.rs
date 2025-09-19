use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::domain::shared_kernel::value_objects::UserId;
use crate::domain::shared_kernel::DomainError;
use crate::domain::user_management::{
    User, UserRepositoryPort, Email, FirebaseUid
};

/// Service for resolving user references in multiple formats
/// Supports UUID, email, and Firebase UID lookups with fallback logic
pub struct UserReferenceResolver {
    user_repository: Arc<dyn UserRepositoryPort>,
}

impl UserReferenceResolver {
    pub fn new(user_repository: Arc<dyn UserRepositoryPort>) -> Self {
        Self { user_repository }
    }

    /// Resolve a user reference string to a User object
    /// Tries multiple lookup methods in order: UUID -> Email -> Firebase UID
    pub async fn resolve_user(&self, reference: &str) -> Result<Option<User>, DomainError> {
        debug!("Attempting to resolve user reference: {}", reference);

        // 1. Try UUID lookup first (most common for internal APIs)
        if let Ok(user_id) = UserId::from_string(reference.to_string()) {
            debug!("Attempting UUID lookup for: {}", reference);
            match self.user_repository.find_by_id(&user_id).await {
                Ok(Some(user)) => {
                    debug!("Found user by UUID: {}", reference);
                    return Ok(Some(user));
                }
                Ok(None) => {
                    debug!("No user found by UUID: {}", reference);
                }
                Err(e) => {
                    warn!("UUID lookup failed for {}: {:?}", reference, e);
                }
            }
        } else {
            debug!("Reference is not a valid UUID: {}", reference);
        }

        // 2. Try email lookup
        if let Ok(email) = Email::new(reference.to_string()) {
            debug!("Attempting email lookup for: {}", reference);
            match self.user_repository.find_by_email(&email).await {
                Ok(Some(user)) => {
                    debug!("Found user by email: {}", reference);
                    return Ok(Some(user));
                }
                Ok(None) => {
                    debug!("No user found by email: {}", reference);
                }
                Err(e) => {
                    warn!("Email lookup failed for {}: {:?}", reference, e);
                }
            }
        } else {
            debug!("Reference is not a valid email: {}", reference);
        }

        // 3. Try Firebase UID lookup
        if let Ok(firebase_uid) = FirebaseUid::new(reference.to_string()) {
            debug!("Attempting Firebase UID lookup for: {}", reference);
            match self.user_repository.find_by_firebase_uid(&firebase_uid).await {
                Ok(Some(user)) => {
                    debug!("Found user by Firebase UID: {}", reference);
                    return Ok(Some(user));
                }
                Ok(None) => {
                    debug!("No user found by Firebase UID: {}", reference);
                }
                Err(e) => {
                    warn!("Firebase UID lookup failed for {}: {:?}", reference, e);
                }
            }
        } else {
            debug!("Reference is not a valid Firebase UID: {}", reference);
        }

        debug!("User not found with any lookup method: {}", reference);
        Ok(None)
    }

    /// Resolve a user reference and return an error if not found
    pub async fn resolve_user_required(&self, reference: &str) -> Result<User, DomainError> {
        match self.resolve_user(reference).await? {
            Some(user) => Ok(user),
            None => Err(DomainError::entity_not_found(
                "User",
                format!("User not found with reference: {}", reference)
            ))
        }
    }

    /// Batch resolve multiple user references
    pub async fn resolve_users(&self, references: &[String]) -> Result<Vec<(String, Option<User>)>, DomainError> {
        let mut results = Vec::new();
        
        for reference in references {
            let user = self.resolve_user(reference).await?;
            results.push((reference.clone(), user));
        }
        
        Ok(results)
    }

    /// Get the best identifier for a user (prefers UUID, falls back to email)
    pub fn get_user_identifier(&self, user: &User) -> String {
        user.id().to_string()
    }

    /// Validate if a reference string could be a valid user identifier
    pub fn is_valid_user_reference(&self, reference: &str) -> bool {
        // Check if it could be any of the valid formats
        UserId::from_string(reference.to_string()).is_ok() ||
        Email::new(reference.to_string()).is_ok() ||
        FirebaseUid::new(reference.to_string()).is_ok()
    }
}

/// Trait for services that need user resolution capabilities
#[async_trait]
pub trait UserResolutionCapable {
    fn user_resolver(&self) -> &UserReferenceResolver;
    
    async fn resolve_user(&self, reference: &str) -> Result<Option<User>, DomainError> {
        self.user_resolver().resolve_user(reference).await
    }
    
    async fn resolve_user_required(&self, reference: &str) -> Result<User, DomainError> {
        self.user_resolver().resolve_user_required(reference).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_management::value_objects::*;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock repository for testing
    struct MockUserRepository {
        users: Vec<User>,
    }

    #[async_trait]
    impl UserRepositoryPort for MockUserRepository {
        async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
            Ok(self.users.iter().find(|u| u.id() == id).cloned())
        }

        async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
            Ok(self.users.iter().find(|u| u.email() == email).cloned())
        }

        async fn find_by_firebase_uid(&self, firebase_uid: &FirebaseUid) -> Result<Option<User>, DomainError> {
            Ok(self.users.iter().find(|u| u.firebase_uid() == firebase_uid).cloned())
        }

        // Other methods not needed for tests
        async fn save(&self, _user: &User) -> Result<(), DomainError> { Ok(()) }
        async fn delete(&self, _id: &UserId) -> Result<(), DomainError> { Ok(()) }
        async fn find_by_permission(&self, _permission: &crate::domain::user_management::value_objects::Permission) -> Result<Vec<User>, DomainError> { Ok(vec![]) }
        async fn find_by_criteria(&self, _criteria: &crate::domain::user_management::UserSearchCriteria, _limit: u32, _offset: u32) -> Result<crate::domain::user_management::UserSearchResult, DomainError> { 
            Ok(crate::domain::user_management::UserSearchResult::new(vec![], 0, 0, 10))
        }
        async fn count_by_criteria(&self, _criteria: &crate::domain::user_management::UserSearchCriteria) -> Result<u64, DomainError> { Ok(0) }
        async fn find_eligible_for_auto_assignment(&self) -> Result<Vec<User>, DomainError> { Ok(vec![]) }
        async fn save_batch(&self, _users: &[User]) -> Result<(), DomainError> { Ok(()) }
        async fn next_identity(&self) -> Result<UserId, DomainError> { 
            Ok(UserId::from_uuid(Uuid::new_v4()))
        }
        async fn health_check(&self) -> Result<(), DomainError> { Ok(()) }
        async fn cleanup_expired_permissions(&self) -> Result<u32, DomainError> { Ok(0) }
    }

    #[tokio::test]
    async fn test_resolve_user_by_uuid() {
        let user_id = UserId::from_uuid(Uuid::new_v4());
        let firebase_uid = FirebaseUid::new("test_firebase_uid".to_string()).unwrap();
        let email = Email::new("test@example.com".to_string()).unwrap();
        
        let user = User::create(user_id.clone(), firebase_uid, email).unwrap();
        
        let mock_repo = Arc::new(MockUserRepository {
            users: vec![user.clone()],
        });
        
        let resolver = UserReferenceResolver::new(mock_repo);
        
        let result = resolver.resolve_user(&user_id.to_string()).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id(), &user_id);
    }

    #[tokio::test]
    async fn test_resolve_user_not_found() {
        let mock_repo = Arc::new(MockUserRepository { users: vec![] });
        let resolver = UserReferenceResolver::new(mock_repo);
        
        let result = resolver.resolve_user("nonexistent@example.com").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_resolve_user_required_error() {
        let mock_repo = Arc::new(MockUserRepository { users: vec![] });
        let resolver = UserReferenceResolver::new(mock_repo);
        
        let result = resolver.resolve_user_required("nonexistent@example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_valid_user_reference() {
        let mock_repo = Arc::new(MockUserRepository { users: vec![] });
        let resolver = UserReferenceResolver::new(mock_repo);
        
        assert!(resolver.is_valid_user_reference("550e8400-e29b-41d4-a716-446655440000"));
        assert!(resolver.is_valid_user_reference("test@example.com"));
        assert!(resolver.is_valid_user_reference("firebase_uid_123"));
        assert!(!resolver.is_valid_user_reference("invalid_reference"));
    }
}