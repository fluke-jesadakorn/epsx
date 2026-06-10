use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

#[async_trait]
pub trait IdentityProviderPort: Send + Sync {
    /// Generate an access token for the identity provider
    async fn get_access_token(&self) -> Result<String, anyhow::Error>;

    /// Set custom claims for a user (e.g. assigning roles)
    async fn set_custom_claims(
        &self,
        user_id: &str,
        claims: &HashMap<String, Value>,
    ) -> Result<(), anyhow::Error>;

    /// Get current custom claims for a user
    async fn get_user_claims(
        &self,
        user_id: &str,
    ) -> Result<HashMap<String, Value>, anyhow::Error>;
}
