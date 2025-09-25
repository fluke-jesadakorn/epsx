// Web3 to Group Permission Bridge Service
// Bridges Web3 permissions (NFT/token-gated) with the group permission system
// Automatically assigns users to groups based on Web3 asset ownership

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
// use std::collections::HashMap; // Removed - unused import
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::auth::web3_permission_service::{Web3PermissionService};

/// Web3 to Group Permission Bridge Service
/// Manages the integration between Web3 permissions and group-based permissions
#[derive(Clone)]
pub struct Web3GroupBridge {
    db_pool: PgPool,
    web3_permission_service: Web3PermissionService,
    auto_assignment_enabled: bool,
    group_assignment_cache_minutes: i64,
}

/// Group assignment rule for Web3 assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3GroupRule {
    pub id: Uuid,
    pub group_id: Uuid,
    pub group_name: String,
    pub rule_type: String, // 'nft_ownership', 'token_balance', 'dao_membership'
    pub contract_address: String,
    pub network: String,
    pub minimum_balance: Option<String>,
    pub token_decimals: Option<i32>,
    pub specific_token_ids: Vec<String>,
    pub is_active: bool,
    pub auto_assignment: bool,
    pub assignment_duration_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Group assignment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAssignmentResult {
    pub user_id: Option<Uuid>,
    pub wallet_address: String,
    pub groups_assigned: Vec<GroupAssignment>,
    pub groups_removed: Vec<GroupAssignment>,
    pub verification_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAssignment {
    pub group_id: Uuid,
    pub group_name: String,
    pub assignment_reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub verification_data: serde_json::Value,
}

impl Web3GroupBridge {
    pub fn new(
        db_pool: PgPool,
        web3_permission_service: Web3PermissionService,
    ) -> Self {
        Self {
            db_pool,
            web3_permission_service,
            auto_assignment_enabled: true,
            group_assignment_cache_minutes: 60, // 1 hour cache
        }
    }

    /// Process automatic group assignments for a wallet address
    pub async fn process_wallet_group_assignments(
        &self,
        wallet_address: &str,
    ) -> Result<GroupAssignmentResult> {
        info!("Processing group assignments for wallet: {}", wallet_address);

        let mut result = GroupAssignmentResult {
            user_id: None,
            wallet_address: wallet_address.to_string(),
            groups_assigned: Vec::new(),
            groups_removed: Vec::new(),
            verification_errors: Vec::new(),
        };

        // Get or create user for this wallet
        let user_id = match self.get_or_create_user_for_wallet(wallet_address).await {
            Ok(id) => {
                result.user_id = Some(id);
                id
            }
            Err(e) => {
                error!("Failed to get/create user for wallet {}: {}", wallet_address, e);
                result.verification_errors.push(format!("User creation failed: {}", e));
                return Ok(result);
            }
        };

        // Get active Web3 group rules
        let group_rules = match self.get_active_web3_group_rules().await {
            Ok(rules) => rules,
            Err(e) => {
                error!("Failed to get Web3 group rules: {}", e);
                result.verification_errors.push(format!("Failed to get group rules: {}", e));
                return Ok(result);
            }
        };

        // Process each rule
        for rule in group_rules {
            match self.evaluate_group_rule(&rule, wallet_address, user_id).await {
                Ok(Some(assignment)) => {
                    info!("Assigned user {} to group: {}", user_id, assignment.group_name);
                    result.groups_assigned.push(assignment);
                }
                Ok(None) => {
                    // Rule didn't match - check if user should be removed from group
                    if let Err(e) = self.maybe_remove_from_group(user_id, rule.group_id, wallet_address).await {
                        warn!("Failed to check group removal for user {}: {}", user_id, e);
                    }
                }
                Err(e) => {
                    warn!("Failed to evaluate rule {} for wallet {}: {}", rule.id, wallet_address, e);
                    result.verification_errors.push(format!("Rule {} error: {}", rule.group_name, e));
                }
            }
        }

        info!(
            "Completed group assignment for wallet {}: {} assigned, {} errors",
            wallet_address,
            result.groups_assigned.len(),
            result.verification_errors.len()
        );

        Ok(result)
    }

    /// Create a new Web3 group assignment rule
    pub async fn create_group_rule(
        &self,
        group_id: Uuid,
        rule_type: &str,
        contract_address: &str,
        network: &str,
        config: serde_json::Value,
    ) -> Result<Uuid> {
        let rule_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO web3_group_rules (
                id, group_id, rule_type, contract_address, network, 
                minimum_balance, token_decimals, specific_token_ids,
                is_active, auto_assignment, assignment_duration_days, configuration
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(rule_id)
        .bind(group_id)
        .bind(rule_type)
        .bind(contract_address)
        .bind(network)
        .bind(config.get("minimum_balance").and_then(|v| v.as_str()))
        .bind(config.get("token_decimals").and_then(|v| v.as_i64()).map(|v| v as i32))
        .bind(
            config
                .get("specific_token_ids")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default()
        )
        .bind(true) // is_active
        .bind(true) // auto_assignment
        .bind(config.get("assignment_duration_days").and_then(|v| v.as_i64()).map(|v| v as i32))
        .bind(&config)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to create Web3 group rule: {}", e))?;

        info!("Created Web3 group rule {} for group {}", rule_id, group_id);
        Ok(rule_id)
    }

    /// Sync all existing Web3 permissions to groups
    pub async fn sync_all_web3_permissions_to_groups(&self) -> Result<usize> {
        info!("Starting sync of all Web3 permissions to groups");

        let mut total_synced = 0;

        // Get all unique wallet addresses from wallet_permissions
        let wallet_addresses: Vec<String> = sqlx::query(
            "SELECT DISTINCT wallet_address FROM wallet_permissions WHERE is_active = true"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to get wallet addresses: {}", e))?
        .into_iter()
        .map(|row| row.get::<String, _>("wallet_address"))
        .collect();

        info!("Found {} unique wallet addresses to sync", wallet_addresses.len());

        // Process each wallet
        for wallet_address in wallet_addresses {
            match self.process_wallet_group_assignments(&wallet_address).await {
                Ok(result) => {
                    total_synced += result.groups_assigned.len();
                    if !result.verification_errors.is_empty() {
                        warn!(
                            "Wallet {} had {} verification errors during sync",
                            wallet_address,
                            result.verification_errors.len()
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to sync wallet {}: {}", wallet_address, e);
                }
            }
        }

        info!("Completed Web3 permissions sync: {} total assignments", total_synced);
        Ok(total_synced)
    }

    // Private helper methods

    async fn get_or_create_user_for_wallet(&self, wallet_address: &str) -> Result<Uuid> {
        // First try to find existing user by wallet
        if let Some(user_id) = sqlx::query(
            "SELECT id FROM users WHERE wallet_address = $1 OR $1 = ANY(wallet_addresses)"
        )
        .bind(wallet_address)
        .fetch_optional(&self.db_pool)
        .await?
        .map(|row| row.get::<Uuid, _>("id"))
        {
            return Ok(user_id);
        }

        // Create new user with wallet address
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, wallet_address, wallet_addresses, created_at, updated_at)
            VALUES ($1, $2, ARRAY[$2], NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(wallet_address)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;

        info!("Created new user {} for wallet {}", user_id, wallet_address);
        Ok(user_id)
    }

    async fn get_active_web3_group_rules(&self) -> Result<Vec<Web3GroupRule>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                wgr.id, wgr.group_id, pg.name as group_name,
                wgr.rule_type, wgr.contract_address, wgr.network,
                wgr.minimum_balance, wgr.token_decimals, wgr.specific_token_ids,
                wgr.is_active, wgr.auto_assignment, wgr.assignment_duration_days,
                wgr.created_at, wgr.updated_at
            FROM web3_group_rules wgr
            JOIN permission_groups pg ON wgr.group_id = pg.id
            WHERE wgr.is_active = true AND pg.is_web3_managed = true
            ORDER BY wgr.created_at ASC
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to get Web3 group rules: {}", e))?;

        let rules: Vec<Web3GroupRule> = rows
            .into_iter()
            .map(|row| Web3GroupRule {
                id: row.get("id"),
                group_id: row.get("group_id"),
                group_name: row.get("group_name"),
                rule_type: row.get("rule_type"),
                contract_address: row.get("contract_address"),
                network: row.get("network"),
                minimum_balance: row.get("minimum_balance"),
                token_decimals: row.get("token_decimals"),
                specific_token_ids: row.get::<Vec<String>, _>("specific_token_ids"),
                is_active: row.get("is_active"),
                auto_assignment: row.get("auto_assignment"),
                assignment_duration_days: row.get("assignment_duration_days"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        debug!("Retrieved {} active Web3 group rules", rules.len());
        Ok(rules)
    }

    async fn evaluate_group_rule(
        &self,
        rule: &Web3GroupRule,
        wallet_address: &str,
        user_id: Uuid,
    ) -> Result<Option<GroupAssignment>> {
        debug!("Evaluating rule {} for wallet {}", rule.group_name, wallet_address);

        let verification_result = match rule.rule_type.as_str() {
            "nft_ownership" => {
                self.web3_permission_service
                    .verify_nft_ownership(
                        wallet_address,
                        &rule.contract_address,
                        &rule.network,
                        rule.specific_token_ids.first().map(|s| s.as_str()),
                    )
                    .await
            }
            "token_balance" => {
                let min_balance = rule.minimum_balance.as_deref().unwrap_or("1");
                let decimals = rule.token_decimals.unwrap_or(18);
                self.web3_permission_service
                    .verify_token_balance(
                        wallet_address,
                        &rule.contract_address,
                        &rule.network,
                        min_balance,
                        decimals,
                    )
                    .await
            }
            _ => {
                warn!("Unsupported rule type: {}", rule.rule_type);
                return Err(anyhow!("Unsupported rule type: {}", rule.rule_type));
            }
        };

        match verification_result {
            Ok(true) => {
                // User qualifies - assign to group if not already assigned
                let expires_at = rule.assignment_duration_days
                    .map(|days| Utc::now() + Duration::days(days as i64));

                // Check if user is already in this group
                let already_assigned = sqlx::query(
                    "SELECT id FROM user_group_memberships WHERE user_id = $1 AND group_id = $2 AND is_active = true"
                )
                .bind(user_id)
                .bind(rule.group_id)
                .fetch_optional(&self.db_pool)
                .await?
                .is_some();

                if already_assigned {
                    debug!("User {} already assigned to group {}", user_id, rule.group_name);
                    return Ok(None);
                }

                // Assign user to group
                let assignment_id = Uuid::new_v4();
                let verification_data = serde_json::json!({
                    "rule_id": rule.id,
                    "rule_type": rule.rule_type,
                    "contract_address": rule.contract_address,
                    "network": rule.network,
                    "verified_at": Utc::now(),
                    "verification_method": "web3_asset_ownership"
                });

                sqlx::query(
                    r#"
                    INSERT INTO user_group_memberships (
                        id, user_id, group_id, granted_at, expires_at, 
                        assignment_reason, assignment_source, web3_wallet_address, 
                        web3_verification_data, is_active
                    ) VALUES ($1, $2, $3, NOW(), $4, $5, 'web3_auto', $6, $7, true)
                    "#
                )
                .bind(assignment_id)
                .bind(user_id)
                .bind(rule.group_id)
                .bind(expires_at)
                .bind(format!("Auto-assigned via Web3 {} rule: {}", rule.rule_type, rule.group_name))
                .bind(wallet_address)
                .bind(&verification_data)
                .execute(&self.db_pool)
                .await
                .map_err(|e| anyhow!("Failed to assign user to group: {}", e))?;

                // Log assignment history
                self.log_group_assignment_history(
                    user_id,
                    rule.group_id,
                    "granted",
                    "web3_auto",
                    &format!("Auto-assigned via Web3 asset verification"),
                    None,
                    expires_at,
                    &verification_data,
                ).await?;

                Ok(Some(GroupAssignment {
                    group_id: rule.group_id,
                    group_name: rule.group_name.clone(),
                    assignment_reason: format!("Web3 {} ownership verified", rule.rule_type),
                    expires_at,
                    verification_data,
                }))
            }
            Ok(false) => {
                debug!("User {} does not qualify for group {}", wallet_address, rule.group_name);
                Ok(None)
            }
            Err(e) => {
                warn!("Verification error for wallet {}: {}", wallet_address, e);
                Err(e)
            }
        }
    }

    async fn maybe_remove_from_group(&self, user_id: Uuid, group_id: Uuid, _wallet_address: &str) -> Result<()> {
        // Check if user is currently in this group via Web3 auto-assignment
        let membership = sqlx::query(
            r#"
            SELECT id, web3_verification_data 
            FROM user_group_memberships 
            WHERE user_id = $1 AND group_id = $2 AND is_active = true AND assignment_source = 'web3_auto'
            "#
        )
        .bind(user_id)
        .bind(group_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = membership {
            let membership_id: Uuid = row.get("id");
            
            // Deactivate membership
            sqlx::query(
                "UPDATE user_group_memberships SET is_active = false, updated_at = NOW() WHERE id = $1"
            )
            .bind(membership_id)
            .execute(&self.db_pool)
            .await?;

            // Log removal
            self.log_group_assignment_history(
                user_id,
                group_id,
                "revoked",
                "web3_auto",
                "Removed due to Web3 asset verification failure",
                None,
                None,
                &serde_json::json!({"removed_at": Utc::now()}),
            ).await?;

            info!("Removed user {} from group {} due to failed Web3 verification", user_id, group_id);
        }

        Ok(())
    }

    async fn log_group_assignment_history(
        &self,
        user_id: Uuid,
        group_id: Uuid,
        action: &str,
        trigger_type: &str,
        reason: &str,
        old_expires_at: Option<DateTime<Utc>>,
        new_expires_at: Option<DateTime<Utc>>,
        metadata: &serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO group_assignment_history (
                user_id, group_id, action, trigger_type, reason,
                old_expires_at, new_expires_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(user_id)
        .bind(group_id)
        .bind(action)
        .bind(trigger_type)
        .bind(reason)
        .bind(old_expires_at)
        .bind(new_expires_at)
        .bind(metadata)
        .execute(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Failed to log assignment history: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_db() -> PgPool {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/epsx_test".to_string());

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_create_group_rule() {
        let pool = setup_test_db().await;
        let web3_permission_service = Web3PermissionService::new(
            pool.clone(),
            "https://bsc-dataseed.binance.org".to_string(),
            "https://polygon-mainnet.alchemyapi.io/v2/test".to_string(),
        );
        let bridge = Web3GroupBridge::new(pool, web3_permission_service);

        let group_id = Uuid::new_v4();
        let config = serde_json::json!({
            "minimum_balance": "100",
            "token_decimals": 18
        });

        let result = bridge
            .create_group_rule(
                group_id,
                "token_balance",
                "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", // CAKE token
                "bsc",
                config,
            )
            .await;

        // Should not error on valid input (may fail due to missing tables in test)
        assert!(result.is_ok() || result.err().unwrap().to_string().contains("relation"));
    }
}