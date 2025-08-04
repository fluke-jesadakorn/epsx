use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password is too weak: {0}")]
    WeakPassword(String),
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
    #[error("Password verification failed: {0}")]
    VerificationFailed(String),
    #[error("Invalid password hash format")]
    InvalidHashFormat,
}

pub struct PasswordValidator {
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_numbers: bool,
    require_special: bool,
    forbidden_patterns: Vec<String>,
}

impl PasswordValidator {
    pub fn new() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special: true,
            forbidden_patterns: vec![
                "123456".to_string(),
                "password".to_string(),
                "admin".to_string(),
                "user".to_string(),
                "test".to_string(),
                "qwerty".to_string(),
                "abc123".to_string(),
                "password123".to_string(),
            ],
        }
    }

    pub fn validate_strength(&self, password: &str) -> Result<(), PasswordError> {
        // Check minimum length
        if password.len() < self.min_length {
            return Err(PasswordError::WeakPassword(format!(
                "Password must be at least {} characters long",
                self.min_length
            )));
        }

        // Check maximum length to prevent DoS
        if password.len() > 128 {
            return Err(PasswordError::WeakPassword(
                "Password cannot exceed 128 characters".to_string(),
            ));
        }

        // Check character requirements
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if self.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(PasswordError::WeakPassword(
                "Password must contain at least one number".to_string(),
            ));
        }

        if self.require_special && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) {
            return Err(PasswordError::WeakPassword(
                "Password must contain at least one special character".to_string(),
            ));
        }

        // Check for forbidden patterns
        let password_lower = password.to_lowercase();
        for pattern in &self.forbidden_patterns {
            if password_lower.contains(&pattern.to_lowercase()) {
                return Err(PasswordError::WeakPassword(
                    "Password contains common weak patterns".to_string(),
                ));
            }
        }

        // Check for repeating characters
        if self.has_excessive_repeating_chars(password) {
            return Err(PasswordError::WeakPassword(
                "Password contains too many repeating characters".to_string(),
            ));
        }

        // Check for sequential patterns
        if self.has_sequential_patterns(password) {
            return Err(PasswordError::WeakPassword(
                "Password contains sequential patterns".to_string(),
            ));
        }

        Ok(())
    }

    fn has_excessive_repeating_chars(&self, password: &str) -> bool {
        let mut consecutive_count = 1;
        let mut last_char = '\0';

        for ch in password.chars() {
            if ch == last_char {
                consecutive_count += 1;
                if consecutive_count >= 3 {
                    return true;
                }
            } else {
                consecutive_count = 1;
            }
            last_char = ch;
        }

        false
    }

    fn has_sequential_patterns(&self, password: &str) -> bool {
        let password_lower = password.to_lowercase();
        let sequential_patterns = [
            "abcd", "1234", "qwer", "asdf", "zxcv",
            "dcba", "4321", "rewq", "fdsa", "vcxz",
        ];

        for pattern in &sequential_patterns {
            if password_lower.contains(pattern) {
                return true;
            }
        }

        false
    }
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    pub async fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        // Generate a random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password using the trait method
        let password_hash = ArgonPasswordHasher::hash_password(&self.argon2, password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    pub async fn verify_password(&self, password: &str, hash: &str) -> Result<bool, PasswordError> {
        // Parse the stored hash
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| PasswordError::InvalidHashFormat)?;

        // Verify the password
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn estimate_strength(&self, password: &str) -> PasswordStrength {
        let mut score = 0;
        let length = password.len();

        // Length scoring
        if length >= 8 { score += 1; }
        if length >= 12 { score += 1; }
        if length >= 16 { score += 1; }

        // Character diversity scoring
        if password.chars().any(|c| c.is_lowercase()) { score += 1; }
        if password.chars().any(|c| c.is_uppercase()) { score += 1; }
        if password.chars().any(|c| c.is_numeric()) { score += 1; }
        if password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c)) { score += 1; }

        // Entropy estimation
        let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
        if unique_chars >= length / 2 { score += 1; }

        match score {
            0..=3 => PasswordStrength::Weak,
            4..=6 => PasswordStrength::Medium,
            7..=8 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    pub fn as_str(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "weak",
            PasswordStrength::Medium => "medium",
            PasswordStrength::Strong => "strong",
            PasswordStrength::VeryStrong => "very_strong",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            PasswordStrength::Weak => 1,
            PasswordStrength::Medium => 2,
            PasswordStrength::Strong => 3,  
            PasswordStrength::VeryStrong => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation() {
        let validator = PasswordValidator::new();

        // Test weak passwords
        assert!(validator.validate_strength("123456").is_err());
        assert!(validator.validate_strength("password").is_err());
        assert!(validator.validate_strength("abc123").is_err());
        assert!(validator.validate_strength("short").is_err());

        // Test strong password
        assert!(validator.validate_strength("MyStr0ng!P@ssw0rd123").is_ok());
    }

    #[test]
    fn test_repeating_characters() {
        let validator = PasswordValidator::new();
        
        // Should reject passwords with too many repeating characters
        assert!(validator.validate_strength("Passsssword1!").is_err());
        assert!(validator.validate_strength("Pass111word!").is_err());
        
        // Should accept passwords with reasonable repetition
        assert!(validator.validate_strength("Password123!").is_ok());
    }

    #[test]
    fn test_sequential_patterns() {
        let validator = PasswordValidator::new();
        
        // Should reject sequential patterns
        assert!(validator.validate_strength("Password1234!").is_err());
        assert!(validator.validate_strength("Passwordabcd!").is_err());
        
        // Should accept non-sequential passwords
        assert!(validator.validate_strength("MyStr0ng!P@ss").is_ok());
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "TestPassword123!";

        // Test hashing
        let hash1 = hasher.hash_password(password).await.unwrap();
        let hash2 = hasher.hash_password(password).await.unwrap();

        // Hashes should be different (due to different salts)
        assert_ne!(hash1, hash2);

        // Both hashes should verify correctly
        assert!(hasher.verify_password(password, &hash1).await.unwrap());
        assert!(hasher.verify_password(password, &hash2).await.unwrap());

        // Wrong password should not verify
        assert!(!hasher.verify_password("WrongPassword", &hash1).await.unwrap());
    }

    #[test]
    fn test_password_strength_estimation() {
        let hasher = PasswordHasher::new();

        assert_eq!(hasher.estimate_strength("123456"), PasswordStrength::Weak);
        assert_eq!(hasher.estimate_strength("Password123"), PasswordStrength::Medium);
        assert_eq!(hasher.estimate_strength("MyStr0ng!Password"), PasswordStrength::Strong);
        assert_eq!(hasher.estimate_strength("MyV3ry$tr0ng!P@ssw0rd"), PasswordStrength::VeryStrong);
    }
}