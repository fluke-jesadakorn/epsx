# EPSX Backend - New Services Documentation

This document covers the newly implemented services in the EPSX backend that enhance reliability, security, and error handling capabilities.

---

## 🔄 Error Recovery Service

**Location**: `src/core/error_recovery.rs`

### Overview

The Error Recovery Service provides comprehensive error recovery strategies with retry logic, circuit breakers, and fallback mechanisms. It implements sophisticated recovery patterns to handle transient failures gracefully.

### Key Features

- **Multiple Recovery Strategies**: Retry, circuit breaker, and fallback strategies
- **Exponential Backoff**: Configurable backoff with jitter
- **Circuit Breaker Pattern**: Prevents cascading failures
- **Recovery Orchestration**: Combines multiple strategies
- **Comprehensive Logging**: Detailed recovery attempt tracking

### Architecture

```rust
┌─────────────────────────────────────────────────────────────┐
│                 Recovery Orchestrator                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌────────┐│
│  │   Retry Strategy    │  │ Circuit Breaker     │  │Fallback││
│  │                     │  │ Strategy            │  │Strategy││
│  │ - Exponential       │  │ - Failure tracking  │  │ - Safe ││
│  │   backoff           │  │ - Recovery timeout  │  │   value││
│  │ - Jitter support    │  │ - Status monitoring │  │        ││
│  │ - Max retries       │  │                     │  │        ││
│  └─────────────────────┘  └─────────────────────┘  └────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Configuration

#### `RecoveryConfig`

```rust
pub struct RecoveryConfig {
    pub max_retries: u32,           // Maximum retry attempts (default: 3)
    pub initial_delay: Duration,    // Initial delay (default: 100ms)
    pub max_delay: Duration,        // Maximum delay (default: 30s)
    pub backoff_multiplier: f64,    // Backoff multiplier (default: 2.0)
    pub jitter: bool,               // Enable jitter (default: true)
}
```

**Example Configuration:**
```rust
let config = RecoveryConfig::default()
    .with_max_retries(5)
    .with_initial_delay(Duration::from_millis(200))
    .with_backoff_multiplier(1.5)
    .no_jitter();
```

### Recovery Strategies

#### 1. Retry Recovery Strategy

Implements exponential backoff retry logic for transient failures:

```rust
let retry_strategy = RetryRecoveryStrategy::new(
    || async { database_operation().await },
    "database_query".to_string()
).with_config(config);
```

**Recoverable Error Types:**
- `NetworkError`
- `ServiceUnavailable`
- `TimeoutError`
- `ResourceExhausted`
- `DatabaseError` (transient)

#### 2. Circuit Breaker Recovery

Prevents cascading failures by temporarily stopping requests to failing services:

```rust
let circuit_breaker = CircuitBreakerRecovery::new(
    "external_api".to_string(),
    failure_threshold: 5,
    recovery_timeout: Duration::from_secs(60)
);
```

**Circuit States:**
- **Closed**: Normal operation, requests pass through
- **Open**: Failures exceed threshold, requests fail immediately
- **Half-Open**: Recovery timeout elapsed, testing service availability

#### 3. Fallback Recovery

Provides safe fallback values when operations fail:

```rust
let fallback_strategy = FallbackRecovery::new(
    default_market_data,
    "market_data_fetch".to_string()
);
```

### Usage Examples

#### Basic Retry with Macro

```rust
use crate::with_retry;

let result = with_retry!(
    || async { external_api_call().await },
    "external_api_call"
).await?;
```

#### Advanced Recovery Orchestration

```rust
let orchestrator = RecoveryOrchestrator::new()
    .add_strategy(Box::new(retry_strategy))
    .add_strategy(Box::new(circuit_breaker))
    .add_strategy(Box::new(fallback_strategy));

match operation().await {
    Ok(result) => Ok(result),
    Err(error) => orchestrator.recover_from_error(error).await
}
```

#### Database Query with Recovery

```rust
async fn get_user_with_recovery(user_id: &str) -> Result<User, AppError> {
    let config = RecoveryConfig::default()
        .with_max_retries(3)
        .with_initial_delay(Duration::from_millis(100));
    
    with_retry!(
        || async { database.get_user(user_id).await },
        "get_user_query",
        config
    ).await
}
```

### Monitoring and Logging

The recovery service provides comprehensive logging:

```rust
tracing::info!(
    operation = %operation_name,
    attempt = attempt + 1,
    max_retries = config.max_retries,
    delay_ms = delay.as_millis(),
    error_kind = %error.kind,
    "Retrying operation after error"
);
```

**Metrics Tracked:**
- Recovery attempt counts
- Success rates after recovery
- Circuit breaker state changes
- Fallback usage frequency
- Average recovery times

---

## 🔐 Encryption Service

**Location**: `src/infra/services/encryption.rs`

### Overview

The Encryption Service provides enterprise-grade data encryption using AES-256-GCM with comprehensive key management, version support, and secure token generation capabilities.

### Key Features

- **AES-256-GCM Encryption**: Industry-standard authenticated encryption
- **Key Versioning**: Support for key rotation without data loss
- **Type-Tagged Encryption**: Data type validation for additional security
- **Secure Token Generation**: Cryptographically secure random tokens
- **Indexable Hashing**: Hash generation for encrypted data indexing

### Architecture

```rust
┌─────────────────────────────────────────────────────────────┐
│                  Encryption Service                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Key Management │  │   Encryption    │  │   Utilities  │ │
│  │                 │  │                 │  │              │ │
│  │ - Key rotation  │  │ - AES-256-GCM   │  │ - Token gen  │ │
│  │ - Version track │  │ - Nonce gen     │  │ - Hashing    │ │
│  │ - Multi-version │  │ - Type tagging  │  │ - Validation │ │
│  │   support       │  │ - Base64 encode │  │              │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Operations

#### 1. Basic Encryption/Decryption

```rust
let service = EncryptionService::new(config);

// Encrypt sensitive data
let encrypted = service.encrypt("sensitive information").await?;

// Decrypt data
let decrypted = service.decrypt(&encrypted).await?;
```

#### 2. Type-Tagged Encryption

Enhanced security with data type validation:

```rust
// Encrypt with type tag
let encrypted_ssn = service
    .encrypt_sensitive_data("123-45-6789", "ssn")
    .await?;

// Decrypt and validate type
let decrypted_ssn = service
    .decrypt_sensitive_data(&encrypted_ssn, "ssn")
    .await?;

// Wrong type will fail
let result = service
    .decrypt_sensitive_data(&encrypted_ssn, "credit_card")
    .await; // Returns error
```

#### 3. Key Rotation

Seamless key rotation without data loss:

```rust
// Encrypt with current key (version 1)
let data_v1 = service.encrypt("data").await?;

// Rotate to new key version
service.rotate_key().await?;

// Encrypt with new key (version 2)
let data_v2 = service.encrypt("data").await?;

// Both versions can still be decrypted
let decrypted_v1 = service.decrypt(&data_v1).await?; // ✓ Works
let decrypted_v2 = service.decrypt(&data_v2).await?; // ✓ Works
```

### Encryption Format

The service uses a structured format for encrypted data:

```
[Key Version: 4 bytes] + [Nonce: 12 bytes] + [Ciphertext: variable] -> Base64
```

**Example:**
- Version 1, random nonce, encrypted data, all base64 encoded
- Format ensures backward compatibility across key rotations

### Security Features

#### 1. Key Derivation

```rust
fn derive_key_from_config(config: &Config) -> Key<Aes256Gcm> {
    // Derives 32-byte AES key from JWT secret
    // Production: Use dedicated key management service
    let key_material = config.auth.jwt_secret.as_bytes();
    // ... secure key derivation logic
}
```

#### 2. Nonce Generation

```rust
// Uses cryptographically secure random nonces
let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
```

#### 3. Type Validation

```rust
// Data format: "type::actual_data"
let tagged_data = format!("{}::{}", data_type, data);
```

### Utility Functions

#### 1. Secure Token Generation

```rust
// Generate cryptographically secure tokens
let api_key = service.generate_secure_token(32);
let session_token = service.generate_secure_token(64);
```

**Features:**
- Alphanumeric character set
- Cryptographically secure randomness
- Configurable length

#### 2. Indexable Hashing

```rust
// Generate consistent hashes for encrypted data indexing
let hash = service.hash_for_indexing("user@example.com");
```

**Use Cases:**
- Database indexing of encrypted fields
- Duplicate detection without decryption
- Search optimization

### Production Deployment

#### Environment Configuration

```env
# Encryption configuration
ENCRYPTION_KEY_SOURCE=key_management_service
KEY_MANAGEMENT_ENDPOINT=https://kms.amazonaws.com
ENCRYPTION_KEY_ID=arn:aws:kms:region:account:key/key-id

# Development/Testing
ENCRYPTION_KEY_SOURCE=config
JWT_SECRET=your-secure-jwt-secret-min-32-chars
```

#### Key Management Best Practices

1. **Production Keys**: Use AWS KMS, Azure Key Vault, or similar
2. **Key Rotation**: Automated monthly rotation
3. **Backup**: Secure key backup and recovery procedures
4. **Access Control**: Strict IAM policies for key access
5. **Audit**: Complete audit trails for key operations

### Usage Examples

#### User Data Encryption

```rust
impl UserService {
    async fn store_sensitive_user_data(&self, user: &User) -> Result<(), AppError> {
        // Encrypt sensitive fields
        let encrypted_ssn = self.encryption
            .encrypt_sensitive_data(&user.ssn, "ssn")
            .await?;
        
        let encrypted_phone = self.encryption
            .encrypt_sensitive_data(&user.phone, "phone")
            .await?;
        
        // Store encrypted data
        self.repository.store_user_data(UserData {
            id: user.id,
            encrypted_ssn,
            encrypted_phone,
            // ... other fields
        }).await
    }
}
```

#### API Key Management

```rust
impl ApiKeyService {
    async fn generate_api_key(&self, client_info: &ClientInfo) -> Result<String, AppError> {
        // Generate secure API key
        let raw_key = self.encryption.generate_secure_token(40);
        
        // Hash for storage (one-way)
        let key_hash = self.encryption.hash_for_indexing(&raw_key);
        
        // Store hash in database
        self.repository.store_api_key(ApiKeyRecord {
            key_hash,
            client_id: client_info.id,
            created_at: Utc::now(),
        }).await?;
        
        Ok(raw_key)
    }
}
```

#### Payment Data Protection

```rust
impl PaymentService {
    async fn process_payment(&self, payment_data: &PaymentData) -> Result<(), AppError> {
        // Encrypt payment details
        let encrypted_card = self.encryption
            .encrypt_sensitive_data(&payment_data.card_number, "card_number")
            .await?;
        
        // Process payment with encrypted storage
        self.payment_processor.process(EncryptedPaymentData {
            encrypted_card,
            amount: payment_data.amount,
            // ... other non-sensitive fields
        }).await
    }
}
```

### Error Handling

The service provides comprehensive error types:

```rust
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
```

### Testing

Comprehensive test suite covers:

```rust
#[tokio::test]
async fn test_basic_encryption_decryption() { /* ... */ }

#[tokio::test]
async fn test_different_encryptions_produce_different_results() { /* ... */ }

#[tokio::test]
async fn test_key_rotation() { /* ... */ }

#[tokio::test]
async fn test_sensitive_data_with_type_tags() { /* ... */ }

#[tokio::test]
async fn test_invalid_encrypted_data() { /* ... */ }
```

---

## 🔗 Service Integration

### Integration with Core Architecture

Both services integrate seamlessly with the Clean Architecture:

#### Error Recovery in Domain Services

```rust
impl PermissionResolver {
    async fn resolve_permissions(&self, user_id: &UserId) -> Result<Permissions, AppError> {
        with_retry!(
            || async { self.repository.get_user_permissions(user_id).await },
            "get_user_permissions",
            RecoveryConfig::default().with_max_retries(2)
        ).await
    }
}
```

#### Encryption in Infrastructure Layer

```rust
impl UserRepository {
    async fn store_user(&self, user: &User) -> Result<(), AppError> {
        let encrypted_email = self.encryption
            .encrypt_sensitive_data(&user.email, "email")
            .await?;
        
        self.database.execute(
            "INSERT INTO users (id, encrypted_email) VALUES ($1, $2)",
            &[&user.id, &encrypted_email]
        ).await
    }
}
```

### Configuration Integration

Both services respect the centralized configuration system:

```rust
// In main.rs or service initialization
let config = Arc::new(Config::from_env());

let encryption_service = EncryptionService::new(config.clone());
let recovery_config = RecoveryConfig::default()
    .with_max_retries(config.error_recovery.max_retries);
```

### Monitoring Integration

Both services integrate with the telemetry system:

```rust
// Error recovery metrics
tracing::info!(
    service = "error_recovery",
    operation = operation_name,
    attempt = attempt,
    success = true,
    "Operation recovered successfully"
);

// Encryption metrics
tracing::info!(
    service = "encryption",
    operation = "encrypt",
    key_version = key_version,
    data_type = data_type,
    "Data encrypted successfully"
);
```

---

## 🚀 Future Enhancements

### Error Recovery Service

1. **Machine Learning Integration**: Predictive failure detection
2. **Dynamic Configuration**: Runtime adjustment of recovery parameters
3. **Service Mesh Integration**: Istio/Linkerd compatibility
4. **Advanced Circuit Breaker**: Adaptive thresholds based on service health

### Encryption Service

1. **Hardware Security Module (HSM)**: Integration with dedicated hardware
2. **Multi-Key Management**: Support for multiple encryption keys per tenant
3. **Field-Level Encryption**: Automatic encryption based on data annotations
4. **Quantum-Resistant Algorithms**: Future-proofing for post-quantum cryptography

---

*These services provide enterprise-grade reliability and security features essential for production financial data platforms.*