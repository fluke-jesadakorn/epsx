// Firestore User Repository Implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use futures::StreamExt;

use crate::app::ports::repositories::{UserRepo, RepoError};
use crate::dom::entities::User;
use crate::dom::values::{UserId, Email, Role, PermSet, SubTier, Subscription};

#[derive(Debug, Serialize, Deserialize)]
struct UserDoc {
    id: String,
    email: String,
    role: String,
    permissions: Vec<String>,
    subscription_tier: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<&User> for UserDoc {
    fn from(user: &User) -> Self {
        Self {
            id: user.id().to_string(),
            email: user.email().value().to_string(),
            role: user.role().to_string(),
            permissions: user.perms().to_vec(),
            subscription_tier: user.sub().tier.to_string(),
            created_at: user.created_at(),
            updated_at: user.updated_at(),
        }
    }
}

impl TryFrom<UserDoc> for User {
    type Error = RepoError;
    
    fn try_from(doc: UserDoc) -> Result<Self, Self::Error> {
        let user_id = UserId::new(doc.id);
        let email = Email::new(doc.email.clone())
            .map_err(|e| RepoError::SerializationError(format!("Invalid email: {}", e)))?;
        let role = Role::from_string(&doc.role)
            .map_err(|e| RepoError::SerializationError(format!("Invalid role: {}", e)))?;
        let permissions = PermSet::from_vec(doc.permissions);
        let sub_tier = SubTier::from_string(&doc.subscription_tier)
            .map_err(|e| RepoError::SerializationError(format!("Invalid subscription tier: {}", e)))?;
        let subscription = Subscription::paid(sub_tier, doc.created_at + chrono::Duration::days(365));
        
        let user = User::reconstruct(
            user_id,
            email,
            role,
            permissions,
            subscription,
            doc.created_at,
            doc.updated_at,
        );
        Ok(user)
    }
}

pub struct FsUserRepo {
    db: firestore::FirestoreDb,
    collection: String,
}

impl FsUserRepo {
    pub fn new(db: firestore::FirestoreDb) -> Self {
        Self {
            db,
            collection: "users".to_string(),
        }
    }
    
    pub fn with_collection(mut self, collection: String) -> Self {
        self.collection = collection;
        self
    }
}

#[async_trait]
impl UserRepo for FsUserRepo {
    async fn get(&self, id: &UserId) -> Result<Option<User>, RepoError> {
        match self.db
            .fluent()
            .select()
            .by_id_in(self.collection.as_str())
            .obj::<UserDoc>()
            .one(&id.to_string())
            .await
        {
            Ok(Some(doc)) => {
                let user = User::try_from(doc)?;
                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RepoError::QueryError(format!("Failed to get user: {}", e))),
        }
    }
    
    async fn save(&self, user: &User) -> Result<(), RepoError> {
        let doc = UserDoc::from(user);
        
        let _result: () = self.db
            .fluent()
            .insert()
            .into(self.collection.as_str())
            .document_id(&user.id().to_string())
            .object(&doc)
            .execute()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to save user: {}", e)))?;
        
        Ok(())
    }
    
    async fn delete(&self, id: &UserId) -> Result<(), RepoError> {
        self.db
            .fluent()
            .delete()
            .from(self.collection.as_str())
            .document_id(&id.to_string())
            .execute()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to delete user: {}", e)))?;
        
        Ok(())
    }
    
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepoError> {
        let mut docs = self.db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("email").eq(email.value()))
            .obj::<UserDoc>()
            .stream_query()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to query by email: {}", e)))?;
            
        if let Some(doc) = docs.next().await {
            let user = User::try_from(doc)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn find_by_role(&self, role: &Role) -> Result<Vec<User>, RepoError> {
        let docs = self.db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .filter(|q| q.field("role").eq(role.to_string()))
            .obj::<UserDoc>()
            .stream_query()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to query by role: {}", e)))?;
            
        let mut users = Vec::new();
        let docs_vec: Vec<UserDoc> = docs.collect().await;
            
        for doc in docs_vec {
            let user = User::try_from(doc)?;
            users.push(user);
        }
        
        Ok(users)
    }
    
    async fn list(&self, offset: u32, limit: u32) -> Result<Vec<User>, RepoError> {
        let docs = self.db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .obj::<UserDoc>()
            .stream_query()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to list users: {}", e)))?;
            
        let mut users = Vec::new();
        let docs_vec: Vec<UserDoc> = docs
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
            .await;
            
        for doc in docs_vec {
            let user = User::try_from(doc)?;
            users.push(user);
        }
        
        Ok(users)
    }
    
    async fn count(&self) -> Result<u64, RepoError> {
        let docs = self.db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .obj::<UserDoc>()
            .stream_query()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to count users: {}", e)))?;
            
        let count = docs.fold(0, |acc, _| async move { acc + 1 }).await;
        Ok(count)
    }
    
    async fn save_batch(&self, users: &[User]) -> Result<(), RepoError> {
        // For now, use individual inserts as a simple implementation
        // In a production system, you would want to use FirestoreSimpleBatchWriter
        for user in users {
            self.save(user).await?;
        }
        
        Ok(())
    }
    
    async fn find_all(&self) -> Result<Vec<User>, RepoError> {
        let docs = self.db
            .fluent()
            .select()
            .from(self.collection.as_str())
            .obj::<UserDoc>()
            .stream_query()
            .await
            .map_err(|e| RepoError::QueryError(format!("Failed to find all users: {}", e)))?;
            
        let mut users = Vec::new();
        let docs_vec: Vec<UserDoc> = docs.collect().await;
            
        for doc in docs_vec {
            let user = User::try_from(doc)?;
            users.push(user);
        }
        
        Ok(users)
    }
    
    async fn find_by_id(&self, id: &UserId) -> Result<User, RepoError> {
        self.get(id).await?.ok_or(RepoError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use firestore::FirestoreDb;
    use tokio;
    
    async fn setup_test_repo() -> FsUserRepo {
        // Create a test Firestore DB connection
        // Note: This requires FIREBASE_PROJECT_ID environment variable
        let db = FirestoreDb::new("test-project")
            .await
            .expect("Failed to create test DB");
        FsUserRepo::new(db)
    }
    
    fn create_test_user() -> User {
        let email = Email::new("test@example.com").unwrap();
        User::new(email, Role::User)
    }
    
    #[test]
    fn should_convert_user_to_doc() {
        let user = create_test_user();
        let doc = UserDoc::from(&user);
        
        assert_eq!(doc.email, "test@example.com");
        assert_eq!(doc.role, "user");
        assert_eq!(doc.id, user.id().to_string());
    }
    
    #[test] 
    fn should_convert_doc_to_user() {
        let user = create_test_user();
        let doc = UserDoc::from(&user);
        let converted_user = User::try_from(doc).unwrap();
        
        assert_eq!(converted_user.email().value(), user.email().value());
        assert_eq!(converted_user.role(), user.role());
        assert_eq!(converted_user.id().to_string(), user.id().to_string());
    }
    
    #[tokio::test]
    #[ignore] // Integration test - requires Firebase setup
    async fn should_save_and_get_user() {
        let repo = setup_test_repo().await;
        let user = create_test_user();
        let user_id = user.id();
        
        // Save user
        repo.save(&user).await.expect("Should save user");
        
        // Get user
        let retrieved = repo.get(user_id).await.expect("Should get user");
        assert!(retrieved.is_some());
        
        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.email().value(), user.email().value());
        assert_eq!(retrieved_user.role(), user.role());
        
        // Clean up
        repo.delete(user_id).await.expect("Should delete user");
    }
    
    #[tokio::test]
    #[ignore] // Integration test - requires Firebase setup
    async fn should_find_user_by_email() {
        let repo = setup_test_repo().await;
        let user = create_test_user();
        let email = user.email();
        
        // Save user
        repo.save(&user).await.expect("Should save user");
        
        // Find by email
        let found = repo.find_by_email(email).await.expect("Should find user by email");
        assert!(found.is_some());
        
        let found_user = found.unwrap();
        assert_eq!(found_user.id(), user.id());
        
        // Clean up
        repo.delete(user.id()).await.expect("Should delete user");
    }
    
    #[test]
    fn should_implement_user_repo_trait() {
        // Compile-time test to ensure FsUserRepo implements UserRepo
        fn assert_implements_user_repo<T: UserRepo>() {}
        assert_implements_user_repo::<FsUserRepo>();
    }
    
    #[test]
    fn should_be_send_sync() {
        // Test that repository can be used across thread boundaries
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FsUserRepo>();
    }
}