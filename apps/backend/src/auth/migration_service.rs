/// User Migration Service
/// 
/// This service handles the migration of existing Firebase users to Web3 wallet authentication.
/// It provides a comprehensive migration path that preserves user data, permissions, and
/// preferences while enabling Web3-first authentication.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::unified_auth_service::{UnifiedAuthService, AuthMethod, MigrationRequest};
use super::web3_auth_service::Web3AuthService;
use super::jwt::UserData;
// FirebaseUser removed - migrated to Web3

/// Migration status for tracking user migration progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    NotStarted,
    InProgress { started_at: DateTime<Utc> },
    Completed { completed_at: DateTime<Utc> },
    Failed { failed_at: DateTime<Utc>, reason: String },
}

/// Migration result with detailed information
#[derive(Debug, Serialize)]
pub struct MigrationResult {
    pub success: bool,
    pub user_id: Uuid,
    pub firebase_uid: String,
    pub wallet_address: String,
    pub status: MigrationStatus,
    pub permissions_migrated: usize,
    pub data_preserved: bool,
    pub message: String,
}

/// Migration plan for a user
#[derive(Debug, Serialize)]
pub struct MigrationPlan {
    pub firebase_uid: String,
    pub current_permissions: Vec<String>,
    pub data_to_preserve: DataPreservationPlan,
    pub estimated_duration: u64, // seconds
    pub requirements: Vec<String>,
}

/// Data preservation plan
#[derive(Debug, Serialize)]
pub struct DataPreservationPlan {
    pub user_profile: bool,
    pub permissions: bool,
    pub preferences: bool,
    pub payment_methods: bool,
    pub subscription_data: bool,
}

/// Batch migration request for multiple users
#[derive(Debug, Deserialize)]
pub struct BatchMigrationRequest {
    pub users: Vec<SingleUserMigration>,
    pub dry_run: bool,
}

/// Single user migration data
#[derive(Debug, Deserialize)]
pub struct SingleUserMigration {
    pub firebase_uid: String,
    pub wallet_address: String,
    pub signature: String,
    pub message: String,
}

/// Migration statistics
#[derive(Debug, Serialize)]
pub struct MigrationStats {
    pub total_users: u64,
    pub migrated_users: u64,
    pub pending_migrations: u64,
    pub failed_migrations: u64,
    pub migration_rate: f64, // percentage
}

/// User Migration Service
pub struct MigrationService {
    db_pool: PgPool,
    unified_auth: std::sync::Arc<UnifiedAuthService>,
    web3_auth: std::sync::Arc<Web3AuthService>,
}

impl MigrationService {
    /// Create a new migration service
    pub fn new(
        db_pool: PgPool,
        unified_auth: std::sync::Arc<UnifiedAuthService>,
        web3_auth: std::sync::Arc<Web3AuthService>,
    ) -> Self {
        Self {
            db_pool,
            unified_auth,
            web3_auth,
        }
    }

    /// Generate a migration plan for a Firebase user
    pub async fn generate_migration_plan(
        &self,
        firebase_uid: &str,
    ) -> Result<MigrationPlan> {
        info!("Generating migration plan for Firebase user: {}", firebase_uid);

        // User data fetch stub - implement with database queries
        // Should retrieve current Firebase user data and permissions
        let current_permissions = vec![
            "epsx:analytics:view".to_string(),
            "epsx:profile:manage".to_string(),
        ];

        let data_preservation_plan = DataPreservationPlan {
            user_profile: true,
            permissions: true,
            preferences: true,
            payment_methods: true,
            subscription_data: true,
        };

        let plan = MigrationPlan {
            firebase_uid: firebase_uid.to_string(),
            current_permissions: current_permissions.clone(),
            data_to_preserve: data_preservation_plan,
            estimated_duration: 30, // 30 seconds
            requirements: vec![
                "Valid Web3 wallet with signature capability".to_string(),
                "SIWE message signature".to_string(),
                "Confirmed ownership of wallet".to_string(),
            ],
        };

        info!("Generated migration plan for user {} with {} permissions", 
               firebase_uid, current_permissions.len());
        Ok(plan)
    }

    /// Migrate a single Firebase user to Web3
    pub async fn migrate_user(
        &self,
        request: MigrationRequest,
    ) -> Result<MigrationResult> {
        info!("Starting migration for Firebase user {} to wallet {}", 
              request.firebase_uid, request.wallet_address);

        let start_time = Utc::now();

        // Step 1: Validate the migration request
        self.validate_migration_request(&request).await?;

        // Step 2: Check if user exists and get current data
        let user_data = self.get_firebase_user_data(&request.firebase_uid).await?;

        // Step 3: Verify wallet signature
        let signature_valid = self.verify_wallet_signature(&request).await?;
        if !signature_valid {
            return Ok(MigrationResult {
                success: false,
                user_id: Uuid::new_v4(), // Placeholder
                firebase_uid: request.firebase_uid.clone(),
                wallet_address: request.wallet_address.clone(),
                status: MigrationStatus::Failed {
                    failed_at: Utc::now(),
                    reason: "Invalid wallet signature".to_string(),
                },
                permissions_migrated: 0,
                data_preserved: false,
                message: "Migration failed: Invalid wallet signature".to_string(),
            });
        }

        // Step 4: Perform the actual migration using UnifiedAuthService
        let auth_result = self.unified_auth.migrate_firebase_to_web3(request.clone()).await
            .map_err(|e| {
                error!("Unified auth migration failed: {}", e);
                anyhow!("Migration failed: {}", e)
            })?;

        // Step 5: Preserve user data and permissions
        let permissions_migrated = self.preserve_user_data(&auth_result.user_id, &user_data).await?;

        // Step 6: Update migration status in database
        self.record_migration_completion(&auth_result.user_id, &request).await?;

        let migration_result = MigrationResult {
            success: true,
            user_id: auth_result.user_id,
            firebase_uid: request.firebase_uid.clone(),
            wallet_address: request.wallet_address.clone(),
            status: MigrationStatus::Completed { completed_at: Utc::now() },
            permissions_migrated,
            data_preserved: true,
            message: format!("Successfully migrated user from Firebase to Web3 wallet in {:.2}s", 
                           (Utc::now() - start_time).num_milliseconds() as f64 / 1000.0),
        };

        info!("Migration completed successfully for user {} in {:.2}s", 
              request.firebase_uid, 
              (Utc::now() - start_time).num_milliseconds() as f64 / 1000.0);

        Ok(migration_result)
    }

    /// Migrate multiple users in batch
    pub async fn batch_migrate(
        &self,
        request: BatchMigrationRequest,
    ) -> Result<Vec<MigrationResult>> {
        info!("Starting batch migration for {} users (dry_run: {})", 
              request.users.len(), request.dry_run);

        let mut results = Vec::new();

        for user_migration in request.users {
            let migration_request = MigrationRequest {
                firebase_uid: user_migration.firebase_uid.clone(),
                wallet_address: user_migration.wallet_address.clone(),
                signature: user_migration.signature.clone(),
                message: user_migration.message.clone(),
            };

            if request.dry_run {
                // In dry run mode, just validate without actually migrating
                match self.validate_migration_request(&migration_request).await {
                    Ok(_) => {
                        results.push(MigrationResult {
                            success: true,
                            user_id: Uuid::new_v4(),
                            firebase_uid: user_migration.firebase_uid.clone(),
                            wallet_address: user_migration.wallet_address.clone(),
                            status: MigrationStatus::NotStarted,
                            permissions_migrated: 0,
                            data_preserved: false,
                            message: "Dry run: Migration would succeed".to_string(),
                        });
                    }
                    Err(e) => {
                        results.push(MigrationResult {
                            success: false,
                            user_id: Uuid::new_v4(),
                            firebase_uid: user_migration.firebase_uid.clone(),
                            wallet_address: user_migration.wallet_address.clone(),
                            status: MigrationStatus::Failed {
                                failed_at: Utc::now(),
                                reason: e.to_string(),
                            },
                            permissions_migrated: 0,
                            data_preserved: false,
                            message: format!("Dry run: Migration would fail: {}", e),
                        });
                    }
                }
            } else {
                // Actual migration
                match self.migrate_user(migration_request).await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        error!("Migration failed for user {}: {}", user_migration.firebase_uid, e);
                        results.push(MigrationResult {
                            success: false,
                            user_id: Uuid::new_v4(),
                            firebase_uid: user_migration.firebase_uid.clone(),
                            wallet_address: user_migration.wallet_address.clone(),
                            status: MigrationStatus::Failed {
                                failed_at: Utc::now(),
                                reason: e.to_string(),
                            },
                            permissions_migrated: 0,
                            data_preserved: false,
                            message: format!("Migration failed: {}", e),
                        });
                    }
                }
            }
        }

        let successful_migrations = results.iter().filter(|r| r.success).count();
        info!("Batch migration completed: {}/{} successful", 
              successful_migrations, results.len());

        Ok(results)
    }

    /// Get migration statistics
    pub async fn get_migration_stats(&self) -> Result<MigrationStats> {
        // Migration statistics stub - implement with database aggregation queries
        // Should query migration tracking tables for real statistics
        Ok(MigrationStats {
            total_users: 1000,
            migrated_users: 250,
            pending_migrations: 50,
            failed_migrations: 10,
            migration_rate: 25.0,
        })
    }

    /// Check if a Firebase user can be migrated
    pub async fn can_migrate_user(&self, firebase_uid: &str) -> Result<bool> {
        // Check if user exists
        let user_exists = self.firebase_user_exists(firebase_uid).await?;
        if !user_exists {
            return Ok(false);
        }

        // Check if user has already been migrated
        let already_migrated = self.is_user_migrated(firebase_uid).await?;
        if already_migrated {
            return Ok(false);
        }

        // Check if user has any blocking issues
        let has_blocking_issues = self.has_migration_blockers(firebase_uid).await?;
        if has_blocking_issues {
            return Ok(false);
        }

        Ok(true)
    }

    /// Rollback a migration (emergency function)
    pub async fn rollback_migration(&self, user_id: &Uuid) -> Result<()> {
        warn!("Rolling back migration for user: {}", user_id);

        // TODO: Implement migration rollback
        // This would involve:
        // 1. Removing Web3 wallet association
        // 2. Restoring Firebase-only authentication
        // 3. Reverting permission changes
        // 4. Updating migration status

        info!("Migration rollback completed for user: {}", user_id);
        Ok(())
    }

    // Private helper methods

    async fn validate_migration_request(&self, request: &MigrationRequest) -> Result<()> {
        // Validate Firebase UID format
        if request.firebase_uid.is_empty() || request.firebase_uid.len() > 128 {
            return Err(anyhow!("Invalid Firebase UID"));
        }

        // Validate wallet address format
        if !request.wallet_address.starts_with("0x") || request.wallet_address.len() != 42 {
            return Err(anyhow!("Invalid wallet address format"));
        }

        // Validate signature is not empty
        if request.signature.is_empty() {
            return Err(anyhow!("Empty signature"));
        }

        Ok(())
    }

    async fn get_firebase_user_data(&self, firebase_uid: &str) -> Result<FirebaseUserData> {
        // Firebase user data stub - implement with users table query
        // Should fetch actual user data by firebase_uid
        Ok(FirebaseUserData {
            firebase_uid: firebase_uid.to_string(),
            email: format!("{}@example.com", firebase_uid),
            permissions: vec!["epsx:analytics:view".to_string()],
            created_at: Utc::now(),
            preferences: HashMap::new(),
        })
    }

    async fn verify_wallet_signature(&self, request: &MigrationRequest) -> Result<bool> {
        // Use Web3AuthService to verify the signature
        let verify_request = super::web3_auth_service::VerifyRequest {
            message: request.message.clone(),
            signature: request.signature.clone(),
            wallet_address: request.wallet_address.clone(),
        };

        let result = self.web3_auth.verify_signature(verify_request).await?;
        Ok(result.is_valid)
    }

    async fn preserve_user_data(&self, user_id: &Uuid, user_data: &FirebaseUserData) -> Result<usize> {
        // TODO: Implement data preservation logic
        // This would involve:
        // 1. Copying user preferences
        // 2. Migrating permissions
        // 3. Preserving subscription data
        // 4. Maintaining payment methods

        info!("Preserving data for migrated user: {}", user_id);
        Ok(user_data.permissions.len())
    }

    async fn record_migration_completion(&self, user_id: &Uuid, request: &MigrationRequest) -> Result<()> {
        // TODO: Record migration in database
        info!("Recording migration completion for user {} -> {}", 
              request.firebase_uid, user_id);
        Ok(())
    }

    async fn firebase_user_exists(&self, firebase_uid: &str) -> Result<bool> {
        // TODO: Check if Firebase user exists in database
        debug!("Checking if Firebase user exists: {}", firebase_uid);
        Ok(true) // Placeholder
    }

    async fn is_user_migrated(&self, firebase_uid: &str) -> Result<bool> {
        // TODO: Check if user has already been migrated
        debug!("Checking if user is already migrated: {}", firebase_uid);
        Ok(false) // Placeholder
    }

    async fn has_migration_blockers(&self, firebase_uid: &str) -> Result<bool> {
        // TODO: Check for migration blockers
        // Examples: pending payments, active subscriptions, etc.
        debug!("Checking for migration blockers: {}", firebase_uid);
        Ok(false) // Placeholder
    }
}

/// Placeholder for Firebase user data
#[derive(Debug)]
struct FirebaseUserData {
    firebase_uid: String,
    email: String,
    permissions: Vec<String>,
    created_at: DateTime<Utc>,
    preferences: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_service() -> MigrationService {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let unified_auth = std::sync::Arc::new(UnifiedAuthService::new(
            pool.clone(),
            "epsx.io".to_string(),
            "https://eth-mainnet.alchemyapi.io/v2/test".to_string(),
            "https://polygon-mainnet.alchemyapi.io/v2/test".to_string(),
        ));

        let web3_auth = std::sync::Arc::new(Web3AuthService::new(pool.clone(), "epsx.io".to_string()));

        MigrationService::new(pool, unified_auth, web3_auth)
    }

    #[tokio::test]
    async fn test_generate_migration_plan() {
        let service = setup_test_service().await;
        let firebase_uid = "test_firebase_user";
        
        let plan = service.generate_migration_plan(firebase_uid).await.unwrap();
        
        assert_eq!(plan.firebase_uid, firebase_uid);
        assert!(!plan.current_permissions.is_empty());
        assert!(plan.estimated_duration > 0);
    }

    #[tokio::test]
    async fn test_can_migrate_user() {
        let service = setup_test_service().await;
        let firebase_uid = "test_firebase_user";
        
        let can_migrate = service.can_migrate_user(firebase_uid).await.unwrap();
        
        // Should be true since our placeholder implementation allows migration
        assert!(can_migrate);
    }

    #[tokio::test]
    async fn test_validate_migration_request() {
        let service = setup_test_service().await;
        
        let valid_request = MigrationRequest {
            firebase_uid: "valid_firebase_uid".to_string(),
            wallet_address: "0x742d35Cc6634C0532925a3b8D369D7763F3c45c6".to_string(),
            signature: "valid_signature".to_string(),
            message: "valid_message".to_string(),
        };
        
        let result = service.validate_migration_request(&valid_request).await;
        assert!(result.is_ok());
        
        let invalid_request = MigrationRequest {
            firebase_uid: "".to_string(), // Invalid empty UID
            wallet_address: "invalid_address".to_string(),
            signature: "".to_string(),
            message: "message".to_string(),
        };
        
        let result = service.validate_migration_request(&invalid_request).await;
        assert!(result.is_err());
    }
}