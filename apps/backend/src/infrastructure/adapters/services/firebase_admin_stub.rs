// Firebase Admin Stub - Placeholder for Web3-first migration
// This replaces Firebase Admin SDK functionality during Web3 transition

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Stub implementation of Firebase Admin SDK
/// Used during Web3-first migration to maintain API compatibility
#[derive(Clone, Debug)]
pub struct FirebaseAdminStub {
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub disabled: bool,
    pub email_verified: bool,
    pub custom_claims: Option<HashMap<String, serde_json::Value>>,
}

impl FirebaseAdminStub {
    pub fn new(project_id: String) -> Self {
        tracing::info!("Initializing Firebase Admin Stub for project: {}", project_id);
        Self { project_id }
    }

    // Stub methods that return Web3-compatible responses
    pub async fn verify_id_token(&self, _token: &str) -> Result<serde_json::Value> {
        // Return empty result - Web3 auth handles verification
        Ok(serde_json::json!({
            "error": "Firebase ID token verification is deprecated. Use Web3 wallet authentication."
        }))
    }

    pub async fn get_user(&self, _uid: &str) -> Result<Option<FirebaseUser>> {
        // Return None - users are managed via Web3 wallets
        Ok(None)
    }

    pub async fn set_custom_user_claims(&self, _uid: &str, _claims: HashMap<String, serde_json::Value>) -> Result<()> {
        // No-op - permissions are managed via Web3 blockchain verification
        tracing::warn!("Firebase custom claims are deprecated. Use Web3 permission system.");
        Ok(())
    }

    pub async fn create_user(&self, _email: &str, _password: &str) -> Result<String> {
        // Return error - users should use Web3 wallet registration
        Err(anyhow::anyhow!("Firebase user creation is deprecated. Use Web3 wallet registration."))
    }

    pub async fn delete_user(&self, _uid: &str) -> Result<()> {
        // No-op - Web3 users are managed by wallet ownership
        tracing::warn!("Firebase user deletion is deprecated. Web3 users are managed by wallet ownership.");
        Ok(())
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}