use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("SIWE error: {0}")]
    Siwe(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key error: {0}")]
    Key(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub address: String,
    pub chain_id: String,
    pub roles: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_ttl: i64,
    refresh_ttl: i64,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("access_ttl", &self.access_ttl)
            .field("refresh_ttl", &self.refresh_ttl)
            .finish()
    }
}

impl JwtService {
    pub fn new(secret: &str, access_ttl: i64, refresh_ttl: i64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl,
            refresh_ttl,
        }
    }

    pub fn generate_tokens(
        &self,
        user_id: &str,
        address: &str,
        chain_id: &str,
        roles: Vec<String>,
    ) -> Result<TokenPair> {
        let now = Utc::now();
        let access_exp = (now + Duration::seconds(self.access_ttl)).timestamp() as usize;
        let refresh_exp = (now + Duration::seconds(self.refresh_ttl)).timestamp() as usize;

        let access_claims = Claims {
            sub: user_id.to_string(),
            address: address.to_string(),
            chain_id: chain_id.to_string(),
            roles: roles.clone(),
            exp: access_exp,
            iat: now.timestamp() as usize,
        };

        let refresh_claims = Claims {
            sub: user_id.to_string(),
            address: address.to_string(),
            chain_id: chain_id.to_string(),
            roles,
            exp: refresh_exp,
            iat: now.timestamp() as usize,
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)?;
        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_ttl as u64,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }
}

pub struct SiweVerifier;

impl SiweVerifier {
    pub async fn verify(message: &str, signature: &str, expected_domain: &str) -> Result<String> {
        let parsed = siwe::Message::from_str(message)
            .map_err(|e| CryptoError::Siwe(e.to_string()))?;

        if parsed.domain != expected_domain {
            return Err(CryptoError::Siwe("Domain mismatch".to_string()));
        }

        let sig_bytes = hex::decode(signature.trim_start_matches("0x"))
            .map_err(|e| CryptoError::Siwe(e.to_string()))?;

        let opts = siwe::VerificationOpts::default();
        parsed
            .verify(&sig_bytes, &opts)
            .await
            .map_err(|e| CryptoError::Siwe(e.to_string()))?;

        Ok(format!("{:?}", parsed.address))
    }
}
