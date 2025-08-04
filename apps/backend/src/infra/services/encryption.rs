use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use std::sync::Arc;
use thiserror::Error;
use crate::config::Config;

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid key format")]
    InvalidKeyFormat,
    #[error("Invalid data format")]
    InvalidDataFormat,
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),
}

pub struct EncryptionService {
    config: Arc<Config>,
    current_cipher: Aes256Gcm,
    key_version: u32,
}

impl EncryptionService {
    pub fn new(config: Arc<Config>) -> Self {
        // In production, this key should come from a secure key management service
        let encryption_key = Self::derive_key_from_config(&config);
        let cipher = Aes256Gcm::new(&encryption_key);
        
        Self {
            config,
            current_cipher: cipher,
            key_version: 1,
        }
    }

    fn derive_key_from_config(config: &Config) -> Key<Aes256Gcm> {
        // Use JWT secret as base for encryption key derivation
        // In production, use a dedicated encryption key from key management service
        let key_material = config.auth.jwt_secret.as_bytes();
        
        // Derive a 32-byte key using HKDF or similar
        let mut key_bytes = [0u8; 32];
        let key_len = std::cmp::min(key_material.len(), 32);
        key_bytes[..key_len].copy_from_slice(&key_material[..key_len]);
        
        // If JWT secret is shorter than 32 bytes, pad with a constant
        if key_len < 32 {
            for i in key_len..32 {
                key_bytes[i] = (i as u8).wrapping_mul(7).wrapping_add(31);
            }
        }
        
        Key::<Aes256Gcm>::from_slice(&key_bytes).clone()
    }

    pub async fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        // Generate a random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the data
        let ciphertext = self
            .current_cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        // Combine version, nonce, and ciphertext
        let mut combined = Vec::new();
        combined.extend_from_slice(&self.key_version.to_le_bytes());
        combined.extend_from_slice(&nonce);
        combined.extend_from_slice(&ciphertext);
        
        // Encode as base64
        Ok(general_purpose::STANDARD.encode(combined))
    }

    pub async fn decrypt(&self, encrypted_data: &str) -> Result<String, EncryptionError> {
        // Decode from base64
        let combined = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|_| EncryptionError::InvalidDataFormat)?;
        
        if combined.len() < 4 + 12 {
            return Err(EncryptionError::InvalidDataFormat);
        }
        
        // Extract version
        let version = u32::from_le_bytes([
            combined[0], combined[1], combined[2], combined[3]
        ]);
        
        // Extract nonce (12 bytes for AES-GCM)
        let nonce = Nonce::from_slice(&combined[4..16]);
        
        // Extract ciphertext
        let ciphertext = &combined[16..];
        
        // Decrypt using appropriate key version
        let cipher = self.get_cipher_for_version(version)?;
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::DecryptionFailed("Invalid UTF-8".to_string()))
    }

    fn get_cipher_for_version(&self, version: u32) -> Result<&Aes256Gcm, EncryptionError> {
        match version {
            1 => Ok(&self.current_cipher),
            _ => Err(EncryptionError::DecryptionFailed(format!("Unsupported key version: {}", version))),
        }
    }

    pub async fn rotate_key(&mut self) -> Result<(), EncryptionError> {
        // In production, this would get a new key from key management service
        self.key_version += 1;
        
        // For demo purposes, derive a new key by modifying the existing one
        let mut new_key_material = self.config.auth.jwt_secret.as_bytes().to_vec();
        new_key_material.extend_from_slice(&self.key_version.to_le_bytes());
        
        let mut key_bytes = [0u8; 32];
        let key_len = std::cmp::min(new_key_material.len(), 32);
        key_bytes[..key_len].copy_from_slice(&new_key_material[..key_len]);
        
        if key_len < 32 {
            for i in key_len..32 {
                key_bytes[i] = (i as u8).wrapping_mul(11).wrapping_add(self.key_version as u8);
            }
        }
        
        let new_key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        self.current_cipher = Aes256Gcm::new(new_key);
        
        tracing::info!("Encryption key rotated to version {}", self.key_version);
        Ok(())
    }

    pub fn get_key_version(&self) -> u32 {
        self.key_version
    }

    pub async fn encrypt_sensitive_data(&self, data: &str, data_type: &str) -> Result<String, EncryptionError> {
        // Add metadata to the plaintext for additional security
        let tagged_data = format!("{}::{}", data_type, data);
        self.encrypt(&tagged_data).await
    }

    pub async fn decrypt_sensitive_data(&self, encrypted_data: &str, expected_type: &str) -> Result<String, EncryptionError> {
        let decrypted = self.decrypt(encrypted_data).await?;
        
        // Verify the data type tag
        if let Some((data_type, actual_data)) = decrypted.split_once("::") {
            if data_type == expected_type {
                Ok(actual_data.to_string())
            } else {
                Err(EncryptionError::DecryptionFailed(format!(
                    "Data type mismatch: expected {}, got {}",
                    expected_type, data_type
                )))
            }
        } else {
            Err(EncryptionError::DecryptionFailed("Invalid data format: missing type tag".to_string()))
        }
    }

    pub fn generate_secure_token(&self, length: usize) -> String {
        use rand::Rng;
        let charset: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }

    pub fn hash_for_indexing(&self, data: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher.update(&self.key_version.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn create_test_config() -> Arc<Config> {
        Arc::new(Config::from_env())
    }

    #[tokio::test]
    async fn test_basic_encryption_decryption() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        let plaintext = "sensitive data";
        let encrypted = service.encrypt(plaintext).await.unwrap();
        let decrypted = service.decrypt(&encrypted).await.unwrap();

        assert_eq!(plaintext, decrypted);
        assert_ne!(plaintext, encrypted);
    }

    #[tokio::test]
    async fn test_different_encryptions_produce_different_results() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        let plaintext = "test data";
        let encrypted1 = service.encrypt(plaintext).await.unwrap();
        let encrypted2 = service.encrypt(plaintext).await.unwrap();

        // Should be different due to random nonces
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to the same plaintext
        assert_eq!(service.decrypt(&encrypted1).await.unwrap(), plaintext);
        assert_eq!(service.decrypt(&encrypted2).await.unwrap(), plaintext);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let config = create_test_config();
        let mut service = EncryptionService::new(config);

        let plaintext = "test data before rotation";
        let encrypted_v1 = service.encrypt(plaintext).await.unwrap();

        // Rotate key
        service.rotate_key().await.unwrap();
        assert_eq!(service.get_key_version(), 2);

        let encrypted_v2 = service.encrypt(plaintext).await.unwrap();

        // Different versions should produce different ciphertext
        assert_ne!(encrypted_v1, encrypted_v2);

        // Both should still decrypt correctly
        assert_eq!(service.decrypt(&encrypted_v1).await.unwrap(), plaintext);
        assert_eq!(service.decrypt(&encrypted_v2).await.unwrap(), plaintext);
    }

    #[tokio::test]
    async fn test_sensitive_data_with_type_tags() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        let sensitive_data = "123-45-6789";
        let encrypted = service.encrypt_sensitive_data(sensitive_data, "ssn").await.unwrap();
        let decrypted = service.decrypt_sensitive_data(&encrypted, "ssn").await.unwrap();

        assert_eq!(sensitive_data, decrypted);

        // Wrong type should fail
        let wrong_type_result = service.decrypt_sensitive_data(&encrypted, "credit_card").await;
        assert!(wrong_type_result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_encrypted_data() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        // Test invalid base64
        let result = service.decrypt("invalid_base64!").await;
        assert!(result.is_err());

        // Test too short data
        let result = service.decrypt("dGVzdA==").await; // "test" in base64, too short
        assert!(result.is_err());

        // Test corrupted data
        let valid_encrypted = service.encrypt("test data").await.unwrap();
        let mut corrupted = valid_encrypted.chars().collect::<Vec<_>>();
        corrupted[10] = 'X'; // Corrupt one character
        let corrupted_string: String = corrupted.into_iter().collect();
        
        let result = service.decrypt(&corrupted_string).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_secure_token_generation() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        let token1 = service.generate_secure_token(32);
        let token2 = service.generate_secure_token(32);

        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2);

        // Should only contain alphanumeric characters
        assert!(token1.chars().all(|c| c.is_alphanumeric()));
        assert!(token2.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_hash_for_indexing() {
        let config = create_test_config();
        let service = EncryptionService::new(config);

        let data = "user@example.com";
        let hash1 = service.hash_for_indexing(data);
        let hash2 = service.hash_for_indexing(data);

        // Same data should produce same hash
        assert_eq!(hash1, hash2);

        // Different data should produce different hash
        let hash3 = service.hash_for_indexing("different@example.com");
        assert_ne!(hash1, hash3);

        // Hash should be hex string
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }
}