use std::collections::HashMap;use jsonwebtoken::{EncodingKey, DecodingKey};
use uuid::Uuid;

use rsa::{RsaPrivateKey, RsaPublicKey, pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, DecodePublicKey}, traits::PublicKeyParts};

use serde::{Serialize, Deserialize};


use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use sha2::{Sha256, Digest};

use crate::config::env::get_env_var;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JWK {
    pub kty: String,  // Key Type
    pub use_: String, // Public Key Use
    pub alg: String,  // Algorithm  
    pub kid: String,  // Key ID
    pub n: String,    // Modulus
    pub e: String,    // Exponent
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
            Ok(private_pem) => {
                match get_env_var("RSA_PUBLIC_KEY") {
                    Ok(public_pem) => {
                        match get_env_var("RSA_KEY_ID") {
                            Ok(kid) => {
                                match Self::from_pem(&private_pem, &public_pem, &kid) {
                                    Ok(key_manager) => {
                                        tracing::info!("Loaded RSA keys from environment variables");
                                        return Ok(key_manager);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to load RSA keys from environment: {}", e);
                                    }
                                }
                            }
                            Err(_) => {
                                tracing::debug!("RSA_KEY_ID not found in environment, will generate new keys");
                            }
                        }
                    }
                    Err(_) => {
                        tracing::debug!("RSA_PUBLIC_KEY not found in environment, will generate new keys");
                    }
                }
            }
            Err(_) => {
                tracing::debug!("RSA_PRIVATE_KEY not found in environment, will generate new keys");
            }
        }
        
        // Generate new keys if environment loading failed
        tracing::info!("Generating new RSA key pair for JWT signing");
        let key_manager = Self::new()?;
        
        // Log the keys for environment setup (in development only)
        if cfg!(debug_assertions) {
            let private_pem = key_manager.current_key.private_key
                .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;
            let public_pem = key_manager.current_key.public_key
                .to_public_key_pem(rsa::pkcs8::LineEnding::LF)?;
                
            tracing::info!("Generated RSA Key ID: {}", key_manager.current_key.kid);
            tracing::info!("To persist these keys, set these environment variables:");
            tracing::info!("RSA_KEY_ID={}", key_manager.current_key.kid);
            tracing::info!("RSA_PRIVATE_KEY={}", private_pem.as_str().replace('\n', "\\n"));
            tracing::info!("RSA_PUBLIC_KEY={}", public_pem.replace('\n', "\\n"));
        }
        
        Ok(key_manager)
    }
    
    /// Create KeyManager from PEM strings
    pub fn from_pem(private_pem: &str, public_pem: &str, kid: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse PEM strings
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_pem)?;
        let public_key = RsaPublicKey::from_public_key_pem(public_pem)?;
        
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
    
    /// Rotate keys (move current to backup, generate new current)
    pub fn rotate_keys(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // Move current key to backup
        let old_kid = self.current_key.kid.clone();
        let old_key = std::mem::replace(&mut self.current_key, Self::generate_key_pair()?);
        self.backup_keys.insert(old_kid.clone(), old_key);
        
        // Keep only last 3 backup keys
        if self.backup_keys.len() > 3 {
            let keys_to_remove: Vec<String> = self.backup_keys.keys()
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
        Self::from_env_or_generate()
            .expect("Failed to create default KeyManager")
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
    fn test_key_rotation() {
        let manager = KeyManager::new().unwrap();
        let original_kid = manager.current_key.kid.clone();
        
        let new_kid = manager.rotate_keys().unwrap();
        
        assert_ne!(original_kid, new_kid);
        assert_eq!(manager.current_key.kid, new_kid);
        assert!(manager.backup_keys.contains_key(&original_kid));
    }
}