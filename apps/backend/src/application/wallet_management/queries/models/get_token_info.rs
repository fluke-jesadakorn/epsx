// Get Token Info Query Model
use serde::{Deserialize, Serialize};
use crate::application::shared::{Query, ApplicationResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokenInfoQuery {
    pub token: String,
    pub validate_expiry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub wallet_address: String,
    pub session_id: Option<String>,
    pub expires_at: String,
    pub is_valid: bool,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokenInfoResponse {
    pub token_info: TokenInfo,
}

impl Query for GetTokenInfoQuery {
    type Response = GetTokenInfoResponse;

    fn validate(&self) -> ApplicationResult<()> {
        if self.token.is_empty() {
            return Err(crate::application::ApplicationError::validation(
                "token",
                "Token cannot be empty"
            ));
        }
        Ok(())
    }
}