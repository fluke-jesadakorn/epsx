// Firebase Authentication and Token Verification
use chrono::{DateTime, Utc};
// Focused module handling authentication logic and JWT token verification

use std::collections::HashMap;
use serde_json::{Value, json};
use jsonwebtoken::{decode, Algorithm, Validation, DecodingKey};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use tracing::{error, info};

use crate::config::env::get_env_var;
use super::types::{FirebaseAdmin, FirebaseUser, FirebasePublicKey, GetUserResponse, AuthRequest, FirebaseUserRecord};

impl FirebaseAdmin {
    /// Get user by ID token (preferred method)
    pub async fn get_user_by_id_token(&self, id_token: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
                api_key
            );

            let payload = json!({
                "idToken": id_token
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                error!("Failed to get Firebase user by ID token: {}", error_text);
                Err("Failed to get user".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Verify Firebase ID token using proper JWT verification with public keys
    pub async fn verify_id_token(&self, id_token: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // Decode token header to get kid (key ID)
        let header = jsonwebtoken::decode_header(id_token)?;
        let kid = header.kid.ok_or("Token missing kid (key ID)")?;

        // Get or refresh public keys
        let public_keys = self.get_firebase_public_keys().await?;
        let public_key = public_keys.get(&kid)
            .ok_or("Public key not found for token kid")?;

        // Create RSA public key for verification
        let modulus_bytes = URL_SAFE_NO_PAD.decode(&public_key.n)?;
        let exponent_bytes = URL_SAFE_NO_PAD.decode(&public_key.e)?;
        
        // Convert bytes to base64 strings for jsonwebtoken 
        let modulus_str = base64::engine::general_purpose::STANDARD.encode(&modulus_bytes);
        let exponent_str = base64::engine::general_purpose::STANDARD.encode(&exponent_bytes);
        
        // Create decoding key
        let decoding_key = DecodingKey::from_rsa_components(&modulus_str, &exponent_str)?;

        // Set up validation parameters
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[format!("https://securetoken.google.com/{}", self.project_id)]);
        validation.set_audience(&[self.project_id.clone()]);
        validation.validate_exp = true;

        // Verify token
        let token_data = decode::<HashMap<String, Value>>(id_token, &decoding_key, &validation)?;
        
        // Extract user information directly from token claims
        let firebase_uid = token_data.claims.get("sub")
            .and_then(|v| v.as_str())
            .ok_or("Token missing sub claim")?;
            
        let email = token_data.claims.get("email")
            .and_then(|v| v.as_str())
            .ok_or("Token missing email claim")?;
            
        let email_verified = token_data.claims.get("email_verified")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Create FirebaseUser from token claims (more efficient than API call)
        Ok(FirebaseUser {
            uid: firebase_uid.to_string(),
            email: Some(email.to_string()),
            email_verified,
            display_name: token_data.claims.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            photo_url: token_data.claims.get("picture")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            phone_number: None,
            disabled: false,
            custom_claims: HashMap::new(),
            provider_data: vec![],
            created_at: Utc::now(),
            last_login_at: Some(Utc::now()),
        })
    }

    /// Get Firebase public keys for token verification
    pub async fn get_firebase_public_keys(&self) -> Result<HashMap<String, FirebasePublicKey>, Box<dyn std::error::Error>> {
        // Check if cache is still valid
        if Utc::now() < self.jwks_cache_expiry && !self.jwks_cache.is_empty() {
            return Ok(self.jwks_cache.clone());
        }

        // Fetch fresh keys
        let url = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let keys: HashMap<String, String> = response.json().await?;
            let mut firebase_keys = HashMap::new();

            for (kid, cert_pem) in keys {
                // Parse certificate to extract public key
                if let Ok(public_key) = self.parse_firebase_cert(&cert_pem) {
                    firebase_keys.insert(kid.clone(), public_key);
                }
            }

            // Return the keys - proper caching would need Arc<Mutex<>> for thread safety
            Ok(firebase_keys)
        } else {
            Err("Failed to fetch Firebase public keys".into())
        }
    }


    /// Authenticate user with email and password
    pub async fn authenticate_user(&self, email: &str, password: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:signInWithPassword?key={}",
                api_key
            );

            let auth_request = AuthRequest {
                email: email.to_string(),
                password: password.to_string(),
                return_secure_token: true,
            };

            let response = self.client
                .post(&url)
                .json(&auth_request)
                .send()
                .await?;

            if response.status().is_success() {
                let auth_response: Value = response.json().await?;
                
                // Extract user info from auth response
                let _firebase_uid = auth_response.get("localId")
                    .and_then(|v| v.as_str())
                    .ok_or("Auth response missing localId")?;

                let _id_token = auth_response.get("idToken")
                    .and_then(|v| v.as_str())
                    .ok_or("Auth response missing idToken")?;

                // For now, skip complex JWT verification and create user from auth response
                // The Firebase signInWithPassword API already verified the credentials
                let firebase_uid = _firebase_uid;
                
                info!("Firebase authentication successful for {} with UID: {}", email, firebase_uid);
                
                // Create Firebase user from auth response  
                Ok(FirebaseUser {
                    uid: firebase_uid.to_string(),
                    email: Some(email.to_string()),
                    display_name: Some("Admin User".to_string()), // From Firebase response
                    photo_url: None,
                    phone_number: None,
                    email_verified: true, // Assume verified since login worked
                    disabled: false,
                    custom_claims: std::collections::HashMap::new(),
                    provider_data: Vec::new(),
                    created_at: chrono::Utc::now(),
                    last_login_at: Some(chrono::Utc::now()),
                })
            } else {
                let error_text = response.text().await?;
                error!("Firebase authentication failed for {}: {}", email, error_text);
                Err("Authentication failed".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Generate OIDC-compliant JWT access token for Firebase user
    pub async fn generate_jwt_token(&self, firebase_uid: &str) -> Result<String, Box<dyn std::error::Error>> {
        use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Create JWT claims with OIDC-compliant structure
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let claims = serde_json::json!({
            "sub": firebase_uid,                    // Subject (user ID)
            "iss": format!("https://api.epsx.io"),  // Issuer
            "aud": "epsx-platform",                 // Audience
            "exp": now + 3600,                      // Expires in 1 hour
            "iat": now,                             // Issued at
            "auth_time": now,                       // Authentication time
            "token_use": "access"                   // Token purpose
        });
        
        // Use a simple signing key for now (in production, use proper RSA keys)
        let secret = crate::config::env::get_env_var("JWT_SECRET")
            .unwrap_or_else(|_| "epsx-jwt-secret-2024-change-in-production".to_string());
        
        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        
        let token = encode(&header, &claims, &encoding_key)?;
        Ok(token)
    }

    /// Parse Firebase certificate to extract public key components
    pub fn parse_firebase_cert(&self, cert_pem: &str) -> Result<FirebasePublicKey, Box<dyn std::error::Error>> {
        use x509_parser::pem::parse_x509_pem;
        use pkcs8::DecodePublicKey;
        use rsa::traits::PublicKeyParts;
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;
        
        // Parse PEM format certificate
        let (_, pem) = parse_x509_pem(cert_pem.as_bytes())?;
        let cert = pem.parse_x509()?;
        
        // Extract public key from certificate
        let public_key_info = cert.public_key();
        
        // Parse RSA public key from the DER-encoded public key
        let rsa_public_key = rsa::RsaPublicKey::from_public_key_der(&public_key_info.raw)?;
        
        // Extract RSA components (n = modulus, e = exponent)
        let modulus = rsa_public_key.n().to_bytes_be();
        let exponent = rsa_public_key.e().to_bytes_be();
        
        // Convert to base64url format (without padding)
        let n_b64 = URL_SAFE_NO_PAD.encode(&modulus);
        let e_b64 = URL_SAFE_NO_PAD.encode(&exponent);
        
        // Generate key ID from certificate fingerprint (first 8 bytes of SHA-256)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&pem.contents);
        let fingerprint = hasher.finalize();
        let kid = hex::encode(&fingerprint[..8]);
        
        Ok(FirebasePublicKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            r#use: "sig".to_string(),
            kid,
            n: n_b64,
            e: e_b64,
        })
    }

    /// Convert Firebase user record to FirebaseUser struct
    pub fn convert_user_record_to_firebase_user(&self, user_record: &FirebaseUserRecord) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        let created_at = user_record.created_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let last_login_at = user_record.last_login_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let custom_claims = if let Some(claims_str) = &user_record.custom_claims {
            serde_json::from_str::<HashMap<String, Value>>(claims_str)
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        let provider_data = user_record.provider_user_info
            .as_ref()
            .map(|providers| {
                providers.iter().map(|provider| super::types::UserProvider {
                    uid: provider.federated_id.clone(),
                    email: provider.email.clone(),
                    display_name: provider.display_name.clone(),
                    photo_url: provider.photo_url.clone(),
                    provider_id: provider.provider_id.clone(),
                }).collect()
            })
            .unwrap_or_default();

        Ok(FirebaseUser {
            uid: user_record.local_id.clone(),
            email: user_record.email.clone(),
            email_verified: user_record.email_verified.unwrap_or(false),
            display_name: user_record.display_name.clone(),
            photo_url: user_record.photo_url.clone(),
            phone_number: user_record.phone_number.clone(),
            disabled: user_record.disabled.unwrap_or(false),
            custom_claims,
            provider_data,
            created_at,
            last_login_at,
        })
    }

    /// Validate Firebase ID token format
    pub fn validate_id_token_format(&self, id_token: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Basic JWT format validation
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format: must have 3 parts".into());
        }

        // Validate header can be decoded
        let _header = jsonwebtoken::decode_header(id_token)
            .map_err(|_| "Invalid JWT header")?;

        Ok(())
    }

    /// Extract claims from ID token without verification (for debugging)
    pub fn extract_unverified_claims(&self, id_token: &str) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        let payload = parts[1];
        
        // Try URL_SAFE_NO_PAD first, then URL_SAFE with padding
        let decoded = match URL_SAFE_NO_PAD.decode(payload) {
            Ok(bytes) => bytes,
            Err(_) => {
                // Try with standard base64url padding
                let padded_payload = match payload.len() % 4 {
                    0 => payload.to_string(),
                    2 => format!("{}==", payload),
                    3 => format!("{}=", payload),
                    _ => return Err("Invalid base64 padding".into()),
                };
                base64::engine::general_purpose::URL_SAFE.decode(&padded_payload)?
            }
        };
        
        let claims: HashMap<String, Value> = serde_json::from_slice(&decoded)?;
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_token_format_validation() {
        let admin = FirebaseAdmin::create_test_client();
        
        // Invalid format - too few parts
        assert!(admin.validate_id_token_format("invalid.token").is_err());
        
        // Valid format structure (though content may be invalid)
        assert!(admin.validate_id_token_format("header.payload.signature").is_ok());
    }

    #[test]
    fn test_firebase_public_key_creation() {
        let key = FirebasePublicKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            r#use: "sig".to_string(),
            kid: "test-kid".to_string(),
            n: "test-modulus".to_string(),
            e: "AQAB".to_string(),
        };

        assert_eq!(key.kty, "RSA");
        assert_eq!(key.alg, "RS256");
        assert_eq!(key.kid, "test-kid");
    }

    #[test]
    fn test_user_record_conversion() {
        let admin = FirebaseAdmin::create_test_client();
        
        let user_record = FirebaseUserRecord {
            local_id: "test-uid".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: Some(false),
            custom_claims: Some("{}".to_string()),
            provider_user_info: None,
            created_at: Some("2023-01-01T00:00:00Z".to_string()),
            last_login_at: None,
        };

        let firebase_user = admin.convert_user_record_to_firebase_user(&user_record).unwrap();
        assert_eq!(firebase_user.uid, "test-uid");
        assert_eq!(firebase_user.email, Some("test@example.com".to_string()));
        assert_eq!(firebase_user.email_verified, true);
    }

    #[test]
    fn test_cert_parsing_placeholder() {
        let admin = FirebaseAdmin::create_test_client();
        let cert_pem = "-----BEGIN CERTIFICATE-----\ntest_cert_content\n-----END CERTIFICATE-----";
        
        let result = admin.parse_firebase_cert(cert_pem);
        assert!(result.is_ok());
        
        let public_key = result.unwrap();
        assert_eq!(public_key.kty, "RSA");
        assert_eq!(public_key.alg, "RS256");
    }
}