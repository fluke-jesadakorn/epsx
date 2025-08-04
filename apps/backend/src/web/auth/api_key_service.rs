use std::sync::Arc;
use thiserror::Error;
use crate::config::Config;
use rand::{Rng, thread_rng};

#[derive(Error, Debug)]
pub enum ApiKeyError {
    #[error("Invalid API key")]
    InvalidKey,
    #[error("API key generation failed: {0}")]
    GenerationFailed(String),
    #[error("API key validation failed: {0}")]
    ValidationFailed(String),
}

pub struct ApiKeyService {
    _config: Arc<Config>,
}

impl ApiKeyService {
    pub fn new(config: Arc<Config>) -> Self {
        Self { _config: config }
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<bool, ApiKeyError> {
        // Simple validation - in production this would check against database
        if api_key.is_empty() || api_key.len() < 32 {
            return Err(ApiKeyError::InvalidKey);
        }

        // Check if it's a valid format (alphanumeric)
        if !api_key.chars().all(|c| c.is_alphanumeric()) {
            return Err(ApiKeyError::InvalidKey);
        }

        // For testing purposes, reject obviously invalid keys
        if api_key == "invalid_api_key" {
            return Err(ApiKeyError::InvalidKey);
        }

        Ok(true)
    }

    pub async fn generate_api_key(&self) -> Result<String, ApiKeyError> {
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = thread_rng();
        
        let key: String = (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();
        
        Ok(key)
    }

    pub async fn revoke_api_key(&self, api_key: &str) -> Result<(), ApiKeyError> {
        // In production, this would mark the key as revoked in database
        tracing::info!("API key revoked: {}", &api_key[..8]);
        Ok(())
    }
}