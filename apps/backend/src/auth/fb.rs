use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Fb {
    id: String,
    key: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub uid: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

impl Fb {
    pub fn new() -> Result<Self, String> {
        let id = env::var("FB_PROJECT_ID").map_err(|_| "FB_PROJECT_ID missing")?;
        let key = env::var("FB_PRIVATE_KEY").map_err(|_| "FB_PRIVATE_KEY missing")?;
        let email = env::var("FB_CLIENT_EMAIL").map_err(|_| "FB_CLIENT_EMAIL missing")?;
        
        Ok(Self { id, key, email })
    }
    
    pub async fn verify(&self, token: &str) -> Result<Claims, String> {
        // TODO: Real Firebase verification
        if token.is_empty() {
            return Err("Empty token".to_string());
        }
        
        // Mock for now
        Ok(Claims {
            uid: "test_uid".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["user".to_string()],
        })
    }
}