use jsonwebtoken::{ decode, DecodingKey, Validation, Algorithm };
use crate::config::Config;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use tracing::debug;
use anyhow;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    User,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: Option<String>,
    pub roles: Vec<UserRole>,
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    #[serde(rename = "private_key")]
    private_key_pem: String,
    _client_email: String,
    project_id: String,
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    sub: String,
    email: Option<String>,
    claims: Option<Value>,
}

pub struct FirebaseAdmin {
    credentials: Arc<ServiceAccount>,
    validation: Validation,
}

impl FirebaseAdmin {
    pub async fn new(service_account_path: &str) -> anyhow::Result<Self> {
        debug!("Initializing Firebase Admin with service account from: {}", service_account_path);

        let (project_id, _client_email, private_key_pem) =
            Config::load_firebase_config(service_account_path);

        let credentials = ServiceAccount {
            project_id,
            _client_email,
            private_key_pem,
        };

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&credentials.project_id]);
        validation.set_issuer(
            &[&format!("https://securetoken.google.com/{}", credentials.project_id)]
        );
        validation.validate_exp = true;
        validation.validate_nbf = false;

        Ok(Self {
            credentials: Arc::new(credentials),
            validation,
        })
    }

    pub async fn verify_token(&self, token: &str) -> anyhow::Result<FirebaseUser> {
        debug!("Verifying Firebase token");

        let decoding_key = DecodingKey::from_rsa_pem(
            self.credentials.private_key_pem.as_bytes()
        ).map_err(|e| anyhow::anyhow!("Failed to create decoding key: {}", e))?;

        let token_data = decode::<TokenClaims>(token, &decoding_key, &self.validation).map_err(|e|
            anyhow::anyhow!("Failed to verify token: {}", e)
        )?;

        let roles = token_data.claims.claims
            .as_ref()
            .and_then(|claims| claims.get("roles"))
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str().and_then(|s| {
                            match s {
                                "admin" => Some(UserRole::Admin),
                                "user" => Some(UserRole::User),
                                _ => None,
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![UserRole::User]); // Default to User role if none specified

        Ok(FirebaseUser {
            uid: token_data.claims.sub,
            email: token_data.claims.email,
            roles,
            token: token.to_string(),
        })
    }
}
