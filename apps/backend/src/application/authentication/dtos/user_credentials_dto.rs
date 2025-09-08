use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentialsDto {
    pub email: String,
    pub firebase_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCredentialsDto {
    pub token: String,
    pub validate_permissions: bool,
}