// Web3 DAO Governance Service
// Handles DAO proposals, voting, and governance-based permission management

use anyhow::Result;
use chrono::{DateTime, Utc};
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::web3_shared_types::{
    DAOProposal, Web3PermissionError, Web3PermissionResult
};

/// DAO governance service for Web3 permissions
#[derive(Clone)]
pub struct Web3DAOService {
    db_pool: PgPool,
}

impl Web3DAOService {
    /// Create new DAO service
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Create DAO permission proposal
    pub async fn create_dao_proposal(&self, proposal: DAOProposal) -> Web3PermissionResult<Uuid> {
        info!("🏛️ Creating DAO proposal: {}", proposal.title);
        
        let dao_address = Address::from_str(&proposal.dao_contract_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid DAO contract address format".to_string()))?;
        let dao_contract = dao_address.to_string().to_lowercase();

        let target_address = Address::from_str(&proposal.target_wallet_address)
            .map_err(|_| Web3PermissionError::Configuration("Invalid target wallet address format".to_string()))?;
        let target_wallet = target_address.to_string().to_lowercase();

        let proposal_uuid = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO dao_permission_proposals
            (id, dao_contract_address, network, proposal_id, title, description,
             proposer_address, target_wallet_address, permission, proposal_status, voting_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active', $10)
            "#,
            proposal_uuid,
            &dao_contract,
            proposal.network,
            &proposal.proposal_id,
            proposal.title,
            proposal.description,
            &dao_contract, // Assuming proposer is the DAO contract for now
            target_wallet,
            &proposal.permission,
            proposal.voting_end
        )
        .execute(&self.db_pool)
        .await?;

        info!("✅ Created DAO proposal {} for permission {}", proposal.proposal_id, proposal.permission);
        Ok(proposal_uuid)
    }

    /// Verify DAO-based permission
    pub async fn verify_dao_permission(
        &self,
        dao_contract: &str,
        proposal_id: &str,
        network: &str,
        wallet_address: &str,
    ) -> Web3PermissionResult<bool> {
        debug!("🏛️ Verifying DAO permission for wallet: {}", wallet_address);
        
        // Check if DAO proposal passed and grants permission to this wallet
        let proposal = sqlx::query!(
            r#"
            SELECT proposal_status, target_wallet_address, executed_at
            FROM dao_permission_proposals
            WHERE dao_contract_address = $1 AND proposal_id = $2 AND network = $3
            "#,
            dao_contract,
            proposal_id,
            network
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(proposal) = proposal {
            let is_valid = proposal.proposal_status == "passed" &&
                proposal.target_wallet_address.to_lowercase() == wallet_address.to_lowercase() &&
                proposal.executed_at.is_some();
                
            if is_valid {
                debug!("✅ Valid DAO permission found for wallet: {}", wallet_address);
            }
            
            return Ok(is_valid);
        }

        debug!("❌ No valid DAO permission found for wallet: {}", wallet_address);
        Ok(false)
    }

    /// Update proposal status (e.g., after voting concludes)
    pub async fn update_proposal_status(
        &self,
        proposal_id: &str,
        dao_contract: &str,
        network: &str,
        new_status: &str,
        updated_by: Option<Uuid>,
    ) -> Web3PermissionResult<()> {
        info!("📊 Updating DAO proposal {} status to: {}", proposal_id, new_status);

        let rows_affected = sqlx::query!(
            r#"
            UPDATE dao_permission_proposals 
            SET proposal_status = $1, updated_at = NOW(), updated_by = $5
            WHERE proposal_id = $2 AND dao_contract_address = $3 AND network = $4
            "#,
            new_status,
            proposal_id,
            dao_contract,
            network,
            updated_by
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(Web3PermissionError::PermissionNotFound(format!("DAO proposal {}", proposal_id)));
        }

        info!("✅ DAO proposal {} status updated to: {}", proposal_id, new_status);
        Ok(())
    }

    /// Execute passed proposal (grant the permission)
    pub async fn execute_proposal(
        &self,
        proposal_id: &str,
        dao_contract: &str,
        network: &str,
        executor: Uuid,
    ) -> Web3PermissionResult<()> {
        info!("⚡ Executing DAO proposal: {}", proposal_id);

        // First verify the proposal is in passed status
        let proposal = sqlx::query!(
            r#"
            SELECT id, target_wallet_address, permission, proposal_status
            FROM dao_permission_proposals
            WHERE proposal_id = $1 AND dao_contract_address = $2 AND network = $3
            "#,
            proposal_id,
            dao_contract,
            network
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let proposal = proposal.ok_or_else(|| {
            Web3PermissionError::PermissionNotFound(format!("DAO proposal {}", proposal_id))
        })?;

        if proposal.proposal_status != "passed" {
            return Err(Web3PermissionError::Configuration(
                format!("Proposal {} is not in passed status", proposal_id)
            ));
        }

        // Mark as executed
        sqlx::query!(
            r#"
            UPDATE dao_permission_proposals 
            SET executed_at = NOW(), executed_by = $4
            WHERE proposal_id = $1 AND dao_contract_address = $2 AND network = $3
            "#,
            proposal_id,
            dao_contract,
            network,
            executor
        )
        .execute(&self.db_pool)
        .await?;

        // In production, this would grant the actual permission to the target wallet
        // This would integrate with the core permission service

        info!("✅ DAO proposal {} executed successfully", proposal_id);
        Ok(())
    }

    /// Get all proposals for a DAO
    pub async fn get_dao_proposals(
        &self,
        dao_contract: &str,
        network: &str,
        status_filter: Option<&str>,
    ) -> Web3PermissionResult<Vec<DAOProposalInfo>> {
        debug!("📋 Fetching DAO proposals for contract: {}", dao_contract);

        let proposals = if let Some(status) = status_filter {
            sqlx::query!(
                r#"
                SELECT proposal_id, title, description, target_wallet_address, permission, 
                       proposal_status, voting_end, created_at, executed_at
                FROM dao_permission_proposals
                WHERE dao_contract_address = $1 AND network = $2 AND proposal_status = $3
                ORDER BY created_at DESC
                "#,
                dao_contract,
                network,
                status
            )
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT proposal_id, title, description, target_wallet_address, permission, 
                       proposal_status, voting_end, created_at, executed_at
                FROM dao_permission_proposals
                WHERE dao_contract_address = $1 AND network = $2
                ORDER BY created_at DESC
                "#,
                dao_contract,
                network
            )
            .fetch_all(&self.db_pool)
            .await?
        };

        let proposal_infos = proposals
            .into_iter()
            .map(|row| DAOProposalInfo {
                proposal_id: row.proposal_id,
                title: row.title,
                description: row.description,
                target_wallet_address: row.target_wallet_address,
                permission: row.permission,
                proposal_status: row.proposal_status,
                voting_end: row.voting_end,
                created_at: row.created_at,
                executed_at: row.executed_at,
            })
            .collect();

        debug!("Found {} DAO proposals for contract: {}", proposal_infos.len(), dao_contract);
        Ok(proposal_infos)
    }

    /// Get proposals affecting a specific wallet
    pub async fn get_wallet_proposals(
        &self,
        wallet_address: &str,
        status_filter: Option<&str>,
    ) -> Web3PermissionResult<Vec<DAOProposalInfo>> {
        debug!("📋 Fetching DAO proposals for wallet: {}", wallet_address);

        let proposals = if let Some(status) = status_filter {
            sqlx::query!(
                r#"
                SELECT proposal_id, title, description, target_wallet_address, permission, 
                       proposal_status, voting_end, created_at, executed_at
                FROM dao_permission_proposals
                WHERE LOWER(target_wallet_address) = LOWER($1) AND proposal_status = $2
                ORDER BY created_at DESC
                "#,
                wallet_address,
                status
            )
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT proposal_id, title, description, target_wallet_address, permission, 
                       proposal_status, voting_end, created_at, executed_at
                FROM dao_permission_proposals
                WHERE LOWER(target_wallet_address) = LOWER($1)
                ORDER BY created_at DESC
                "#,
                wallet_address
            )
            .fetch_all(&self.db_pool)
            .await?
        };

        let proposal_infos = proposals
            .into_iter()
            .map(|row| DAOProposalInfo {
                proposal_id: row.proposal_id,
                title: row.title,
                description: row.description,
                target_wallet_address: row.target_wallet_address,
                permission: row.permission,
                proposal_status: row.proposal_status,
                voting_end: row.voting_end,
                created_at: row.created_at,
                executed_at: row.executed_at,
            })
            .collect();

        debug!("Found {} DAO proposals for wallet: {}", proposal_infos.len(), wallet_address);
        Ok(proposal_infos)
    }

    /// Cancel a proposal (only if not yet passed)
    pub async fn cancel_proposal(
        &self,
        proposal_id: &str,
        dao_contract: &str,
        network: &str,
        cancelled_by: Uuid,
    ) -> Web3PermissionResult<()> {
        info!("❌ Cancelling DAO proposal: {}", proposal_id);

        let rows_affected = sqlx::query!(
            r#"
            UPDATE dao_permission_proposals 
            SET proposal_status = 'cancelled', updated_at = NOW(), updated_by = $4
            WHERE proposal_id = $1 AND dao_contract_address = $2 AND network = $3 
              AND proposal_status NOT IN ('passed', 'executed', 'cancelled')
            "#,
            proposal_id,
            dao_contract,
            network,
            cancelled_by
        )
        .execute(&self.db_pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(Web3PermissionError::Configuration(
                "Proposal cannot be cancelled (not found or already finalized)".to_string()
            ));
        }

        info!("✅ DAO proposal {} cancelled", proposal_id);
        Ok(())
    }

    /// Get DAO statistics
    pub async fn get_dao_stats(&self) -> Web3PermissionResult<DAOStats> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_proposals,
                COUNT(*) FILTER (WHERE proposal_status = 'active') as active_proposals,
                COUNT(*) FILTER (WHERE proposal_status = 'passed') as passed_proposals,
                COUNT(*) FILTER (WHERE proposal_status = 'executed') as executed_proposals,
                COUNT(*) FILTER (WHERE proposal_status = 'cancelled') as cancelled_proposals,
                COUNT(DISTINCT dao_contract_address) as unique_daos
            FROM dao_permission_proposals
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(DAOStats {
            total_proposals: stats.total_proposals.unwrap_or(0) as u32,
            active_proposals: stats.active_proposals.unwrap_or(0) as u32,
            passed_proposals: stats.passed_proposals.unwrap_or(0) as u32,
            executed_proposals: stats.executed_proposals.unwrap_or(0) as u32,
            cancelled_proposals: stats.cancelled_proposals.unwrap_or(0) as u32,
            unique_daos: stats.unique_daos.unwrap_or(0) as u32,
        })
    }

    /// Check if a DAO contract is registered
    pub async fn is_dao_registered(&self, dao_contract: &str, network: &str) -> Web3PermissionResult<bool> {
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM dao_permission_proposals WHERE dao_contract_address = $1 AND network = $2)",
            dao_contract,
            network
        )
        .fetch_one(&self.db_pool)
        .await?
        .exists
        .unwrap_or(false);

        Ok(exists)
    }
}

/// DAO proposal information
#[derive(Debug, Serialize, Deserialize)]
pub struct DAOProposalInfo {
    pub proposal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_wallet_address: String,
    pub permission: String,
    pub proposal_status: String,
    pub voting_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}

/// DAO statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct DAOStats {
    pub total_proposals: u32,
    pub active_proposals: u32,
    pub passed_proposals: u32,
    pub executed_proposals: u32,
    pub cancelled_proposals: u32,
    pub unique_daos: u32,
}