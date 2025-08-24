// Webhook Authentication
// HMAC, Basic Auth, Bearer Token authentication for webhooks

use std::collections::HashMap;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use reqwest::RequestBuilder;

use super::models::*;

type HmacSha256 = Hmac<Sha256>;

/// Webhook authenticator for adding authentication to requests
pub struct WebhookAuthenticator;

impl WebhookAuthenticator {
    pub fn new() -> Self {
        Self
    }

    /// Add authentication to webhook request
    pub async fn add_authentication(
        &self,
        request: RequestBuilder,
        webhook: &WebhookEndpointConfig,
        payload: &serde_json::Value,
    ) -> WebhookResult<RequestBuilder> {
        let auth_type = WebhookAuthType::from(webhook.auth_type.clone());
        
        match auth_type {
            WebhookAuthType::None => Ok(request),
            WebhookAuthType::Basic => self.add_basic_auth(request, webhook).await,
            WebhookAuthType::Bearer => self.add_bearer_auth(request, webhook).await,
            WebhookAuthType::Hmac => self.add_hmac_auth(request, webhook, payload).await,
            WebhookAuthType::Custom => self.add_custom_auth(request, webhook).await,
        }
    }

    /// Add Basic Authentication
    async fn add_basic_auth(
        &self,
        request: RequestBuilder,
        webhook: &WebhookEndpointConfig,
    ) -> WebhookResult<RequestBuilder> {
        if let Ok(auth_config) = serde_json::from_value::<HashMap<String, String>>(webhook.auth_config.clone()) {
            if let (Some(username), Some(password)) = (auth_config.get("username"), auth_config.get("password")) {
                return Ok(request.basic_auth(username, Some(password)));
            }
        }

        Err(WebhookError::InvalidConfiguration {
            message: "Invalid Basic Auth configuration".to_string(),
        })
    }

    /// Add Bearer Token Authentication
    async fn add_bearer_auth(
        &self,
        request: RequestBuilder,
        webhook: &WebhookEndpointConfig,
    ) -> WebhookResult<RequestBuilder> {
        if let Ok(auth_config) = serde_json::from_value::<HashMap<String, String>>(webhook.auth_config.clone()) {
            if let Some(token) = auth_config.get("token") {
                return Ok(request.header("Authorization", format!("Bearer {}", token)));
            }
        }

        Err(WebhookError::InvalidConfiguration {
            message: "Invalid Bearer Auth configuration".to_string(),
        })
    }

    /// Add HMAC Authentication
    async fn add_hmac_auth(
        &self,
        request: RequestBuilder,
        webhook: &WebhookEndpointConfig,
        payload: &serde_json::Value,
    ) -> WebhookResult<RequestBuilder> {
        if let Ok(auth_config) = serde_json::from_value::<HashMap<String, String>>(webhook.auth_config.clone()) {
            if let Some(secret) = auth_config.get("secret") {
                let algorithm = auth_config.get("algorithm").unwrap_or(&"sha256".to_string()).clone();
                let signature = self.generate_hmac_signature(payload, secret, &algorithm)?;
                
                // Add signature headers
                let mut headers = vec![
                    ("X-EPSX-Signature", signature.clone()),
                    ("X-EPSX-Timestamp", Utc::now().timestamp().to_string()),
                ];

                if let Some(header_name) = auth_config.get("signature_header") {
                    headers.push((header_name, signature));
                }

                let mut request_with_headers = request;
                for (header_name, header_value) in headers {
                    request_with_headers = request_with_headers.header(header_name, header_value);
                }

                return Ok(request_with_headers);
            }
        }

        Err(WebhookError::InvalidConfiguration {
            message: "Invalid HMAC Auth configuration".to_string(),
        })
    }

    /// Add Custom Authentication
    async fn add_custom_auth(
        &self,
        request: RequestBuilder,
        webhook: &WebhookEndpointConfig,
    ) -> WebhookResult<RequestBuilder> {
        if let Ok(auth_config) = serde_json::from_value::<HashMap<String, serde_json::Value>>(webhook.auth_config.clone()) {
            if let Some(headers_value) = auth_config.get("headers") {
                if let Ok(headers) = serde_json::from_value::<HashMap<String, String>>(headers_value.clone()) {
                    let mut request_with_headers = request;
                    for (header_name, header_value) in headers {
                        request_with_headers = request_with_headers.header(&header_name, &header_value);
                    }
                    return Ok(request_with_headers);
                }
            }
        }

        Ok(request) // No custom auth configured, return as-is
    }

    /// Generate HMAC signature for payload
    fn generate_hmac_signature(
        &self,
        payload: &serde_json::Value,
        secret: &str,
        algorithm: &str,
    ) -> WebhookResult<String> {
        let payload_string = serde_json::to_string(payload)?;
        
        match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .map_err(|_| WebhookError::InvalidConfiguration {
                        message: "Invalid HMAC secret".to_string(),
                    })?;
                
                mac.update(payload_string.as_bytes());
                let result = mac.finalize();
                let signature = hex::encode(result.into_bytes());
                Ok(format!("sha256={}", signature))
            }
            _ => Err(WebhookError::InvalidConfiguration {
                message: format!("Unsupported HMAC algorithm: {}", algorithm),
            }),
        }
    }

    /// Validate HMAC signature (for incoming webhook validation)
    pub fn validate_hmac_signature(
        &self,
        payload: &str,
        signature: &str,
        secret: &str,
        algorithm: &str,
    ) -> WebhookResult<bool> {
        match algorithm.to_lowercase().as_str() {
            "sha256" => {
                let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                    .map_err(|_| WebhookError::InvalidConfiguration {
                        message: "Invalid HMAC secret".to_string(),
                    })?;
                
                mac.update(payload.as_bytes());
                let expected_signature = hex::encode(mac.finalize().into_bytes());
                let expected_full = format!("sha256={}", expected_signature);
                
                // Remove any "sha256=" prefix from provided signature for comparison
                let provided_signature = signature
                    .strip_prefix("sha256=")
                    .unwrap_or(signature);
                
                Ok(expected_signature == provided_signature || expected_full == signature)
            }
            _ => Err(WebhookError::InvalidConfiguration {
                message: format!("Unsupported HMAC algorithm: {}", algorithm),
            }),
        }
    }

    /// Generate secure random secret for HMAC
    pub fn generate_webhook_secret() -> String {
        use rand::Rng;
        
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const SECRET_LENGTH: usize = 64;
        
        let mut rng = rand::thread_rng();
        (0..SECRET_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Validate webhook configuration
    pub fn validate_auth_config(
        &self,
        auth_type: &WebhookAuthType,
        auth_config: &serde_json::Value,
    ) -> WebhookResult<()> {
        match auth_type {
            WebhookAuthType::None => Ok(()),
            WebhookAuthType::Basic => {
                if let Ok(config) = serde_json::from_value::<HashMap<String, String>>(auth_config.clone()) {
                    if config.contains_key("username") && config.contains_key("password") {
                        Ok(())
                    } else {
                        Err(WebhookError::InvalidConfiguration {
                            message: "Basic Auth requires 'username' and 'password' fields".to_string(),
                        })
                    }
                } else {
                    Err(WebhookError::InvalidConfiguration {
                        message: "Invalid Basic Auth configuration format".to_string(),
                    })
                }
            }
            WebhookAuthType::Bearer => {
                if let Ok(config) = serde_json::from_value::<HashMap<String, String>>(auth_config.clone()) {
                    if config.contains_key("token") {
                        Ok(())
                    } else {
                        Err(WebhookError::InvalidConfiguration {
                            message: "Bearer Auth requires 'token' field".to_string(),
                        })
                    }
                } else {
                    Err(WebhookError::InvalidConfiguration {
                        message: "Invalid Bearer Auth configuration format".to_string(),
                    })
                }
            }
            WebhookAuthType::Hmac => {
                if let Ok(config) = serde_json::from_value::<HashMap<String, String>>(auth_config.clone()) {
                    if let Some(secret) = config.get("secret") {
                        if secret.len() < 32 {
                            return Err(WebhookError::InvalidConfiguration {
                                message: "HMAC secret must be at least 32 characters long".to_string(),
                            });
                        }
                        
                        if let Some(algorithm) = config.get("algorithm") {
                            match algorithm.to_lowercase().as_str() {
                                "sha256" => Ok(()),
                                _ => Err(WebhookError::InvalidConfiguration {
                                    message: format!("Unsupported HMAC algorithm: {}", algorithm),
                                }),
                            }
                        } else {
                            Ok(()) // Default to sha256
                        }
                    } else {
                        Err(WebhookError::InvalidConfiguration {
                            message: "HMAC Auth requires 'secret' field".to_string(),
                        })
                    }
                } else {
                    Err(WebhookError::InvalidConfiguration {
                        message: "Invalid HMAC Auth configuration format".to_string(),
                    })
                }
            }
            WebhookAuthType::Custom => {
                // Custom auth is flexible, just validate it's a valid JSON object
                if auth_config.is_object() {
                    Ok(())
                } else {
                    Err(WebhookError::InvalidConfiguration {
                        message: "Custom Auth configuration must be a JSON object".to_string(),
                    })
                }
            }
        }
    }

    /// Create sample auth configurations for documentation
    pub fn sample_auth_configs() -> HashMap<String, serde_json::Value> {
        let mut samples = HashMap::new();

        samples.insert(
            "basic".to_string(),
            serde_json::json!({
                "username": "webhook_user",
                "password": "secure_password_123"
            }),
        );

        samples.insert(
            "bearer".to_string(),
            serde_json::json!({
                "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
            }),
        );

        samples.insert(
            "hmac".to_string(),
            serde_json::json!({
                "secret": "your_64_character_secret_key_here_must_be_long_enough_for_security",
                "algorithm": "sha256",
                "signature_header": "X-Hub-Signature"
            }),
        );

        samples.insert(
            "custom".to_string(),
            serde_json::json!({
                "headers": {
                    "X-API-Key": "your_api_key_here",
                    "X-Custom-Auth": "custom_auth_value"
                }
            }),
        );

        samples
    }

    /// Extract authentication method from request headers (for incoming webhooks)
    pub fn detect_auth_method(headers: &reqwest::header::HeaderMap) -> Option<WebhookAuthType> {
        if headers.contains_key("Authorization") {
            if let Ok(auth_header) = headers.get("Authorization").unwrap().to_str() {
                if auth_header.starts_with("Bearer ") {
                    return Some(WebhookAuthType::Bearer);
                } else if auth_header.starts_with("Basic ") {
                    return Some(WebhookAuthType::Basic);
                }
            }
        }

        if headers.contains_key("X-EPSX-Signature") || headers.contains_key("X-Hub-Signature") {
            return Some(WebhookAuthType::Hmac);
        }

        // Check for common custom auth headers
        let custom_headers = ["X-API-Key", "X-Auth-Token", "X-Custom-Auth"];
        for header in &custom_headers {
            if headers.contains_key(*header) {
                return Some(WebhookAuthType::Custom);
            }
        }

        Some(WebhookAuthType::None)
    }

    /// Rotate webhook secret (for HMAC)
    pub fn rotate_hmac_secret(current_config: &serde_json::Value) -> WebhookResult<serde_json::Value> {
        if let Ok(mut config) = serde_json::from_value::<HashMap<String, String>>(current_config.clone()) {
            let new_secret = Self::generate_webhook_secret();
            config.insert("secret".to_string(), new_secret);
            config.insert("rotated_at".to_string(), Utc::now().to_rfc3339());
            
            Ok(serde_json::to_value(config)?)
        } else {
            Err(WebhookError::InvalidConfiguration {
                message: "Cannot rotate secret for non-HMAC configuration".to_string(),
            })
        }
    }
}

/// Authentication middleware for verifying incoming webhooks
pub struct WebhookAuthMiddleware {
    authenticator: WebhookAuthenticator,
}

impl WebhookAuthMiddleware {
    pub fn new() -> Self {
        Self {
            authenticator: WebhookAuthenticator::new(),
        }
    }

    /// Verify incoming webhook request
    pub async fn verify_request(
        &self,
        webhook: &WebhookEndpointConfig,
        headers: &reqwest::header::HeaderMap,
        body: &str,
    ) -> WebhookResult<bool> {
        let auth_type = WebhookAuthType::from(webhook.auth_type.clone());
        
        match auth_type {
            WebhookAuthType::None => Ok(true),
            WebhookAuthType::Hmac => self.verify_hmac_request(webhook, headers, body).await,
            WebhookAuthType::Basic | WebhookAuthType::Bearer | WebhookAuthType::Custom => {
                // These are typically for outgoing requests, but we can verify if needed
                Ok(true)
            }
        }
    }

    /// Verify HMAC signature for incoming request
    async fn verify_hmac_request(
        &self,
        webhook: &WebhookEndpointConfig,
        headers: &reqwest::header::HeaderMap,
        body: &str,
    ) -> WebhookResult<bool> {
        if let Ok(auth_config) = serde_json::from_value::<HashMap<String, String>>(webhook.auth_config.clone()) {
            if let Some(secret) = auth_config.get("secret") {
                let default_algorithm = "sha256".to_string();
                let algorithm = auth_config.get("algorithm").unwrap_or(&default_algorithm);
                
                // Look for signature in various possible headers
                let signature_headers = ["X-EPSX-Signature", "X-Hub-Signature", "X-Signature"];
                
                for header_name in &signature_headers {
                    if let Some(signature_header) = headers.get(*header_name) {
                        if let Ok(signature) = signature_header.to_str() {
                            return self.authenticator.validate_hmac_signature(
                                body,
                                signature,
                                secret,
                                algorithm,
                            );
                        }
                    }
                }
                
                return Ok(false); // No signature found
            }
        }

        Err(WebhookError::AuthenticationFailed {
            webhook_id: webhook.id,
        })
    }
}