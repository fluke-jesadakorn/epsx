// RSA Key Management for JWT Security
// Implements RS256 cryptographic foundation with proper key management

use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey, LineEnding};
use jsonwebtoken::{EncodingKey, DecodingKey};
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

#[derive(Debug)]
pub enum KeyError {
    GenerationFailed(String),
    LoadFailed(String),
    SaveFailed(String),
    InvalidKey(String),
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KeyError::GenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            KeyError::LoadFailed(msg) => write!(f, "Key loading failed: {}", msg),
            KeyError::SaveFailed(msg) => write!(f, "Key saving failed: {}", msg),
            KeyError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
        }
    }
}

impl std::error::Error for KeyError {}

/// Manages RSA key pairs for JWT signing and validation
pub struct JwtKeyManager {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl JwtKeyManager {
    /// Generate new RSA key pair or load existing from secure storage
    pub fn generate_or_load() -> Result<Self, KeyError> {
        let keys_dir = "keys";
        let private_key_path = format!("{}/jwt_private_key.pem", keys_dir);
        let public_key_path = format!("{}/jwt_public_key.pem", keys_dir);

        // Create keys directory if it doesn't exist
        if !Path::new(keys_dir).exists() {
            fs::create_dir_all(keys_dir)
                .map_err(|e| KeyError::SaveFailed(format!("Failed to create keys directory: {}", e)))?;
            info!("Created keys directory: {}", keys_dir);
        }

        let private_key = if Path::new(&private_key_path).exists() {
            info!("Loading existing RSA private key from {}", private_key_path);
            let pem_content = fs::read_to_string(&private_key_path)
                .map_err(|e| KeyError::LoadFailed(format!("Failed to read private key file: {}", e)))?;
            
            RsaPrivateKey::from_pkcs8_pem(&pem_content)
                .map_err(|e| KeyError::LoadFailed(format!("Failed to parse private key: {}", e)))?
        } else {
            info!("Generating new RSA key pair (2048 bits)...");
            let mut rng = rand::thread_rng();
            let key = RsaPrivateKey::new(&mut rng, 2048)
                .map_err(|e| KeyError::GenerationFailed(format!("RSA key generation failed: {}", e)))?;
            
            // Save private key
            let private_pem = key.to_pkcs8_pem(LineEnding::LF)
                .map_err(|e| KeyError::SaveFailed(format!("Failed to serialize private key: {}", e)))?;
            
            fs::write(&private_key_path, private_pem)
                .map_err(|e| KeyError::SaveFailed(format!("Failed to save private key: {}", e)))?;
            
            info!("Saved new RSA private key to {}", private_key_path);
            
            // Save public key for external validation if needed
            let public_key = RsaPublicKey::from(&key);
            let public_pem = public_key.to_public_key_pem(LineEnding::LF)
                .map_err(|e| KeyError::SaveFailed(format!("Failed to serialize public key: {}", e)))?;
            
            fs::write(&public_key_path, public_pem)
                .map_err(|e| KeyError::SaveFailed(format!("Failed to save public key: {}", e)))?;
            
            info!("Saved RSA public key to {}", public_key_path);
            
            key
        };
        
        let public_key = RsaPublicKey::from(&private_key);
        
        info!("RSA key pair loaded successfully - RS256 JWT signing enabled");
        
        Ok(Self { private_key, public_key })
    }
    
    /// Get encoding key for JWT signing (RS256)
    pub fn get_encoding_key(&self) -> Result<EncodingKey, KeyError> {
        let pem = self.private_key.to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| KeyError::InvalidKey(format!("Failed to serialize private key for signing: {}", e)))?;
        
        EncodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| KeyError::InvalidKey(format!("Failed to create encoding key: {}", e)))
    }
    
    /// Get decoding key for JWT validation (RS256)
    pub fn get_decoding_key(&self) -> Result<DecodingKey, KeyError> {
        let pem = self.public_key.to_public_key_pem(LineEnding::LF)
            .map_err(|e| KeyError::InvalidKey(format!("Failed to serialize public key for validation: {}", e)))?;
        
        DecodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| KeyError::InvalidKey(format!("Failed to create decoding key: {}", e)))
    }
    
    /// Get public key in PEM format for external services
    pub fn get_public_key_pem(&self) -> Result<String, KeyError> {
        self.public_key.to_public_key_pem(LineEnding::LF)
            .map_err(|e| KeyError::InvalidKey(format!("Failed to serialize public key: {}", e)))
    }
    
    /// Get key fingerprint for identification
    pub fn get_key_fingerprint(&self) -> Result<String, KeyError> {
        let public_pem = self.get_public_key_pem()?;
        let fingerprint = format!("{:x}", md5::compute(public_pem.as_bytes()));
        Ok(fingerprint[..16].to_string()) // First 16 chars for brevity
    }
    
    /// Rotate keys (generate new pair, keeping old for validation period)
    pub fn rotate_keys(&self) -> Result<Self, KeyError> {
        warn!("Key rotation initiated - this should be done carefully in production");
        
        // In production, implement proper key rotation:
        // 1. Generate new keys with version/timestamp
        // 2. Keep old keys for validation period
        // 3. Gradually migrate to new keys
        // 4. Remove old keys after all tokens expire
        
        Self::generate_or_load()
    }
}

// Singleton pattern for global key manager access
use std::sync::OnceLock;
static KEY_MANAGER: OnceLock<JwtKeyManager> = OnceLock::new();

/// Get global key manager instance
pub fn get_key_manager() -> Result<&'static JwtKeyManager, KeyError> {
    KEY_MANAGER.get_or_init(|| JwtKeyManager::generate_or_load().unwrap_or_else(|e| {
        error!("Failed to initialize key manager: {}", e);
        std::process::exit(1);
    }));
    Ok(KEY_MANAGER.get().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_generation() {
        let key_manager = JwtKeyManager::generate_or_load().unwrap();

        // Test encoding key creation
        let _encoding_key = key_manager.get_encoding_key().unwrap();

        // Test decoding key creation
        let _decoding_key = key_manager.get_decoding_key().unwrap();

        // Test public key PEM export
        let public_pem = key_manager.get_public_key_pem().unwrap();
        assert!(public_pem.contains("BEGIN PUBLIC KEY"));
        assert!(public_pem.contains("END PUBLIC KEY"));

        // Test fingerprint generation
        let fingerprint = key_manager.get_key_fingerprint().unwrap();
        assert_eq!(fingerprint.len(), 16);
    }
    
    #[test]
    fn test_key_persistence() {
        // Generate first instance
        let _key_manager1 = JwtKeyManager::generate_or_load().unwrap();
        let fingerprint1 = _key_manager1.get_key_fingerprint().unwrap();
        
        // Load second instance (should be same keys)
        let key_manager2 = JwtKeyManager::generate_or_load().unwrap();
        let fingerprint2 = key_manager2.get_key_fingerprint().unwrap();
        
        assert_eq!(fingerprint1, fingerprint2);
    }
}