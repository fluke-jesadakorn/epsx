use jsonwebtoken::{DecodingKey, EncodingKey};
use std::collections::HashMap;
use uuid::Uuid;

use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey},
    traits::PublicKeyParts,
    RsaPrivateKey, RsaPublicKey,
};

use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

use sha2::{Digest, Sha256};

use crate::config::get_env_var;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWK {
    pub kty: String, // Key Type
    #[serde(rename = "use")]
    pub use_: String, // Public Key Use
    pub alg: String, // Algorithm
    pub kid: String, // Key ID
    pub n: String,   // Modulus
    pub e: String,   // Exponent
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWKS {
    pub keys: Vec<JWK>,
}

pub struct RSAKeyPair {
    pub kid: String,
    pub private_key: RsaPrivateKey,
    pub public_key: RsaPublicKey,
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

pub struct KeyManager {
    current_key: RSAKeyPair,
    backup_keys: HashMap<String, RSAKeyPair>,
}

impl KeyManager {
    fn is_production_environment(value: &str) -> bool {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "prod" | "production"
        )
    }

    fn is_production_runtime() -> bool {
        ["ENV", "APP_ENV", "NODE_ENV", "RUST_ENV"]
            .into_iter()
            .filter_map(|name| std::env::var(name).ok())
            .any(|value| Self::is_production_environment(&value))
    }

    /// Generate a new RSA key pair for JWT signing
    pub fn generate_key_pair() -> Result<RSAKeyPair, Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let bits = 2048;

        // Generate RSA key pair
        let private_key = RsaPrivateKey::new(&mut rng, bits)?;
        let public_key = RsaPublicKey::from(&private_key);

        // Generate unique key ID
        let kid = Uuid::new_v4().to_string();

        // Convert to PEM format for jsonwebtoken
        let private_pem = private_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
        let public_pem = public_key.to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())?;

        Ok(RSAKeyPair {
            kid,
            private_key,
            public_key,
            encoding_key,
            decoding_key,
        })
    }

    /// Create a new KeyManager with a freshly generated key
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let current_key = Self::generate_key_pair()?;

        Ok(Self {
            current_key,
            backup_keys: HashMap::new(),
        })
    }

    /// Load KeyManager from environment or generate new keys
    pub fn from_env_or_generate() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from environment first
        match get_env_var("RSA_PRIVATE_KEY") {
            Ok(private_pem) => match get_env_var("RSA_PUBLIC_KEY") {
                Ok(public_pem) => match get_env_var("RSA_KEY_ID") {
                    Ok(kid) => match Self::from_pem(&private_pem, &public_pem, &kid) {
                        Ok(key_manager) => {
                            tracing::info!("Loaded RSA keys from environment variables");
                            return Ok(key_manager);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load RSA keys from environment: {}", e);
                        }
                    },
                    Err(_) => {
                        tracing::debug!(
                            "RSA_KEY_ID not found in environment, will generate new keys"
                        );
                    }
                },
                Err(_) => {
                    tracing::debug!(
                        "RSA_PUBLIC_KEY not found in environment, will generate new keys"
                    );
                }
            },
            Err(_) => {
                tracing::debug!("RSA_PRIVATE_KEY not found in environment, will generate new keys");
            }
        }

        if Self::is_production_runtime() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "RSA_PRIVATE_KEY, RSA_PUBLIC_KEY, and RSA_KEY_ID must contain one valid persistent signing key pair in production",
            )
            .into());
        }

        // Generate new keys only for non-production development environments.
        tracing::info!("Generating new RSA key pair for JWT signing");
        let key_manager = Self::new()?;

        // Key material must never be written to logs, including development
        // logs. Operators can provision persistent keys through the normal
        // secret-management workflow.
        if cfg!(debug_assertions) {
            tracing::info!("Generated RSA Key ID: {}", key_manager.current_key.kid);
            tracing::warn!(
                "Generated an ephemeral JWT signing key; configure persistent RSA keys through the secret manager when session continuity is required"
            );
        }

        Ok(key_manager)
    }

    /// Create KeyManager from PEM strings
    pub fn from_pem(
        private_pem: &str,
        public_pem: &str,
        kid: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse PEM strings
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_pem)?;
        let public_key = RsaPublicKey::from_public_key_pem(public_pem)?;
        let derived_public_key = RsaPublicKey::from(&private_key);
        if derived_public_key.n() != public_key.n() || derived_public_key.e() != public_key.e() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "RSA private and public keys do not form one signing pair",
            )
            .into());
        }

        // Create encoding/decoding keys
        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())?;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes())?;

        let current_key = RSAKeyPair {
            kid: kid.to_string(),
            private_key,
            public_key,
            encoding_key,
            decoding_key,
        };

        Ok(Self {
            current_key,
            backup_keys: HashMap::new(),
        })
    }

    /// Get the current signing key
    pub fn current_key(&self) -> &RSAKeyPair {
        &self.current_key
    }

    /// Get a key by ID (current or backup)
    pub fn get_key(&self, kid: &str) -> Option<&RSAKeyPair> {
        if self.current_key.kid == kid {
            Some(&self.current_key)
        } else {
            self.backup_keys.get(kid)
        }
    }

    /// List all available key IDs
    pub fn list_key_ids(&self) -> Vec<String> {
        let mut key_ids = vec![self.current_key.kid.clone()];
        key_ids.extend(self.backup_keys.keys().cloned());
        key_ids
    }

    /// Rotate keys (move current to backup, generate new current)
    pub fn rotate_keys(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // Move current key to backup
        let old_kid = self.current_key.kid.clone();
        let old_key = std::mem::replace(&mut self.current_key, Self::generate_key_pair()?);
        self.backup_keys.insert(old_kid.clone(), old_key);

        // Keep only last 3 backup keys
        if self.backup_keys.len() > 3 {
            let keys_to_remove: Vec<String> = self
                .backup_keys
                .keys()
                .take(self.backup_keys.len() - 3)
                .cloned()
                .collect();

            for key_id in keys_to_remove {
                self.backup_keys.remove(&key_id);
            }
        }

        Ok(self.current_key.kid.clone())
    }

    /// Convert RSA public key to JWK format
    pub fn rsa_to_jwk(&self, key_pair: &RSAKeyPair) -> Result<JWK, Box<dyn std::error::Error>> {
        let public_key = &key_pair.public_key;

        // Get modulus (n) and exponent (e) from the public key
        let n = public_key.n().to_bytes_be();
        let e = public_key.e().to_bytes_be();

        // Encode in base64url without padding
        let n_b64 = URL_SAFE_NO_PAD.encode(&n);
        let e_b64 = URL_SAFE_NO_PAD.encode(&e);

        Ok(JWK {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            kid: key_pair.kid.clone(),
            n: n_b64,
            e: e_b64,
        })
    }

    /// Generate JWKS (JSON Web Key Set) for the /.well-known/jwks.json endpoint
    pub fn generate_jwks(&self) -> Result<JWKS, Box<dyn std::error::Error>> {
        let mut keys = vec![];

        // Add current key
        keys.push(self.rsa_to_jwk(&self.current_key)?);

        // Add backup keys
        for key_pair in self.backup_keys.values() {
            keys.push(self.rsa_to_jwk(key_pair)?);
        }

        Ok(JWKS { keys })
    }

    /// Generate key thumbprint for cache-busting
    pub fn generate_key_thumbprint(&self) -> Result<String, Box<dyn std::error::Error>> {
        let jwk = self.rsa_to_jwk(&self.current_key)?;
        let jwk_json = serde_json::to_string(&jwk)?;

        let mut hasher = Sha256::new();
        hasher.update(jwk_json.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash)[..16].to_string()) // First 16 chars
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::from_env_or_generate().expect("Failed to create default KeyManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key_pair = KeyManager::generate_key_pair().unwrap();
        assert!(!key_pair.kid.is_empty());
        assert_eq!(key_pair.private_key.size(), 256); // 2048 bits = 256 bytes
    }

    #[test]
    fn production_environment_values_are_recognized() {
        for value in ["prod", "PROD", "production", "Production"] {
            assert!(KeyManager::is_production_environment(value));
        }

        for value in ["dev", "development", "test", ""] {
            assert!(!KeyManager::is_production_environment(value));
        }
    }

    #[test]
    fn test_jwk_generation() {
        let manager = KeyManager::new().unwrap();
        let jwk = manager.rsa_to_jwk(&manager.current_key).unwrap();

        assert_eq!(jwk.kty, "RSA");
        assert_eq!(jwk.use_, "sig");
        assert_eq!(jwk.alg, "RS256");
        assert!(!jwk.kid.is_empty());
        assert!(!jwk.n.is_empty());
        assert!(!jwk.e.is_empty());
    }

    #[test]
    fn test_jwks_generation() {
        let manager = KeyManager::new().unwrap();
        let jwks = manager.generate_jwks().unwrap();

        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kid, manager.current_key.kid);
    }

    #[test]
    fn test_jwks_serialization_contains_public_material_only() {
        let manager = KeyManager::new().unwrap();
        let serialized = serde_json::to_value(manager.generate_jwks().unwrap()).unwrap();
        let key = &serialized["keys"][0];

        assert_eq!(key["kty"], "RSA");
        assert_eq!(key["use"], "sig");
        assert_eq!(key["alg"], "RS256");
        assert!(key["kid"].as_str().is_some_and(|value| !value.is_empty()));
        assert!(key["n"].as_str().is_some_and(|value| !value.is_empty()));
        assert!(key["e"].as_str().is_some_and(|value| !value.is_empty()));
        assert!(key.get("use_").is_none());

        let serialized = serialized.to_string();
        for forbidden in ["private_key", "encoding_key", "decoding_key", "PRIVATE KEY"] {
            assert!(!serialized.contains(forbidden));
        }
    }

    #[test]
    fn test_jwks_includes_current_and_backup_public_keys() {
        let mut manager = KeyManager::new().unwrap();
        let original_kid = manager.current_key().kid.clone();
        let current_kid = manager.rotate_keys().unwrap();
        let jwks = manager.generate_jwks().unwrap();
        let kids: std::collections::HashSet<_> = jwks.keys.into_iter().map(|key| key.kid).collect();

        assert_eq!(kids.len(), 2);
        assert!(kids.contains(&original_kid));
        assert!(kids.contains(&current_kid));
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = KeyManager::new().unwrap();
        let original_kid = manager.current_key.kid.clone();

        let new_kid = manager.rotate_keys().unwrap();

        assert_ne!(original_kid, new_kid);
        assert_eq!(manager.current_key.kid, new_kid);
        assert!(manager.backup_keys.contains_key(&original_kid));
    }

    #[test]
    fn persistent_key_material_reconstructs_one_identity_across_restarts() {
        let generated = KeyManager::generate_key_pair().unwrap();
        let private_pem = generated
            .private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let public_pem = generated
            .public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let first =
            KeyManager::from_pem(private_pem.as_str(), &public_pem, "persistent-e2e-key").unwrap();
        let restarted =
            KeyManager::from_pem(private_pem.as_str(), &public_pem, "persistent-e2e-key").unwrap();

        assert_eq!(first.current_key().kid, restarted.current_key().kid);
        assert_eq!(
            first.generate_key_thumbprint().unwrap(),
            restarted.generate_key_thumbprint().unwrap()
        );
        assert_eq!(
            serde_json::to_value(first.generate_jwks().unwrap()).unwrap(),
            serde_json::to_value(restarted.generate_jwks().unwrap()).unwrap()
        );
    }

    #[test]
    fn mismatched_persistent_key_pair_is_rejected() {
        let private_pair = KeyManager::generate_key_pair().unwrap();
        let unrelated_public_pair = KeyManager::generate_key_pair().unwrap();
        let private_pem = private_pair
            .private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();
        let unrelated_public_pem = unrelated_public_pair
            .public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();

        assert!(KeyManager::from_pem(
            private_pem.as_str(),
            &unrelated_public_pem,
            "mismatched-e2e-key"
        )
        .is_err());
    }
}
