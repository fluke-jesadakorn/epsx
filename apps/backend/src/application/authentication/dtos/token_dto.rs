use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDto {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: String,
    pub token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}