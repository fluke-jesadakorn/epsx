// Multi-Chain Permission Service
// Advanced cross-chain permission validation for enterprise Web3 authentication

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use futures::future::join_all;
use ethers::providers::{Provider, Http};

use super::web3_permission_service::{Web3PermissionService, TokenConfig, NFTConfig};
use super::enterprise_permission_engine::{EnterpriseTier, TokenRequirement, NFTRequirement, DAORequirement};

/// Comprehensive multi-chain verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiChainVerificationResult {
    pub wallet_address: String,
    pub total_value_usd: u64,
    pub chain_breakdown: HashMap<String, ChainAssets>,
    pub cross_chain_tier: EnterpriseTier,
    pub aggregated_permissions: Vec<String>,
    pub verification_timestamp: DateTime<Utc>,
    pub data_sources: Vec<String>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainAssets {
    pub network: String,
    pub tokens: Vec<TokenHolding>,
    pub nfts: Vec<NFTHolding>,
    pub defi_positions: Vec<DeFiPosition>,
    pub governance_power: Vec<GovernancePower>,
    pub total_value_usd: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolding {
    pub contract_address: String,
    pub symbol: String,
    pub balance: String,
    pub decimals: u8,
    pub price_usd: f64,
    pub value_usd: u64,
    pub is_staked: bool,
    pub staking_rewards: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTHolding {
    pub contract_address: String,
    pub collection_name: String,
    pub token_ids: Vec<String>,
    pub floor_price_usd: Option<u64>,
    pub estimated_value_usd: Option<u64>,
    pub metadata_attributes: HashMap<String, String>,
    pub enterprise_benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiPosition {
    pub protocol: String,
    pub position_type: String, // "liquidity", "lending", "borrowing", "staking"
    pub pool_address: String,
    pub liquidity_usd: u64,
    pub rewards_earned_usd: u64,
    pub apy: f32,
    pub risk_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernancePower {
    pub dao_name: String,
    pub governance_token: String,
    pub voting_power: String,
    pub proposal_count: u32,
    pub votes_cast: u32,
    pub delegation_received: String,
    pub is_delegate: bool,
}

/// Cross-chain aggregation configuration
#[derive(Debug, Clone)]
pub struct CrossChainConfig {
    pub networks: Vec<String>,
    pub parallel_requests: bool,
    pub cache_duration_minutes: i64,
    pub confidence_threshold: f32,
    pub value_aggregation_method: ValueAggregationMethod,
    pub governance_weight_distribution: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub enum ValueAggregationMethod {
    Sum,           // Simple sum across chains
    WeightedSum,   // Weighted by network importance
    MaxChain,      // Take highest value chain
    Conservative,  // Take minimum to avoid double counting
}

/// Multi-chain permission service with enterprise features
pub struct MultiChainPermissionService {
    web3_service: Web3PermissionService,
    db_pool: PgPool,
    config: CrossChainConfig,
    provider_cache: HashMap<String, Provider<Http>>,
}

impl MultiChainPermissionService {
    /// Create new multi-chain permission service
    pub fn new(
        web3_service: Web3PermissionService,
        db_pool: PgPool,
        config: CrossChainConfig,
    ) -> Self {
        Self {
            web3_service,
            db_pool,
            config,
            provider_cache: HashMap::new(),
        }
    }

    /// Verify enterprise tier across all supported chains
    pub async fn verify_enterprise_tier_multi_chain(
        &self,
        wallet_address: &str,
    ) -> Result<MultiChainVerificationResult> {
        info!("Starting multi-chain enterprise tier verification for wallet: {}", wallet_address);

        // Check cache first
        if let Some(cached_result) = self.get_cached_multi_chain_result(wallet_address).await? {
            debug!("Using cached multi-chain verification result for {}", wallet_address);
            return Ok(cached_result);
        }

        // Verify across all chains in parallel
        let verification_futures = self.config.networks.iter().map(|network| {
            self.verify_chain_assets(wallet_address, network)
        });

        let chain_results = if self.config.parallel_requests {
            join_all(verification_futures).await
        } else {
            // Sequential processing for rate-limited APIs
            let mut results = Vec::new();
            for future in verification_futures {
                results.push(future.await);
            }
            results
        };

        // Aggregate results across chains
        let mut chain_breakdown = HashMap::new();
        let mut total_value_usd = 0;
        let mut data_sources = Vec::new();
        let mut confidence_scores = Vec::new();

        for (network, result) in self.config.networks.iter().zip(chain_results.iter()) {
            match result {
                Ok(assets) => {
                    total_value_usd += assets.total_value_usd;
                    chain_breakdown.insert(network.clone(), assets.clone());
                    data_sources.push(format!("{}_rpc", network));
                    confidence_scores.push(0.9); // High confidence for successful verification
                }
                Err(e) => {
                    warn!("Failed to verify assets on {}: {}", network, e);
                    confidence_scores.push(0.1); // Low confidence for failed verification
                    
                    // Create empty asset record
                    chain_breakdown.insert(network.clone(), ChainAssets {
                        network: network.clone(),
                        tokens: vec![],
                        nfts: vec![],
                        defi_positions: vec![],
                        governance_power: vec![],
                        total_value_usd: 0,
                        last_updated: Utc::now(),
                    });
                }
            }
        }

        // Apply value aggregation method
        total_value_usd = self.apply_value_aggregation(&chain_breakdown);

        // Determine cross-chain enterprise tier
        let cross_chain_tier = self.determine_cross_chain_tier(total_value_usd, &chain_breakdown);

        // Generate aggregated permissions
        let aggregated_permissions = self.generate_aggregated_permissions(&cross_chain_tier, &chain_breakdown);

        // Calculate overall confidence score
        let confidence_score = confidence_scores.iter().sum::<f32>() / confidence_scores.len() as f32;

        let result = MultiChainVerificationResult {
            wallet_address: wallet_address.to_string(),
            total_value_usd,
            chain_breakdown,
            cross_chain_tier,
            aggregated_permissions,
            verification_timestamp: Utc::now(),
            data_sources,
            confidence_score,
        };

        // Cache the result
        self.cache_multi_chain_result(&result).await?;

        info!(
            "Multi-chain verification completed for {}: tier={:?}, total_value=${}, confidence={:.2}",
            wallet_address, result.cross_chain_tier, result.total_value_usd, result.confidence_score
        );

        Ok(result)
    }

    /// Verify cross-chain token requirements
    pub async fn verify_cross_chain_token_requirement(
        &self,
        wallet_address: &str,
        requirements: &[TokenRequirement],
    ) -> Result<bool> {
        debug!("Verifying cross-chain token requirements for {}", wallet_address);

        let multi_chain_result = self.verify_enterprise_tier_multi_chain(wallet_address).await?;

        for requirement in requirements {
            let mut requirement_met = false;

            // Check if requirement can be met on any chain
            for (network, assets) in &multi_chain_result.chain_breakdown {
                if requirement.network == "any" || requirement.network == *network {
                    for token in &assets.tokens {
                        if self.token_meets_requirement(token, requirement) {
                            info!(
                                "Token requirement met: {} on {} with {} {}",
                                requirement.token_symbol, network, token.balance, token.symbol
                            );
                            requirement_met = true;
                            break;
                        }
                    }
                }
                if requirement_met { break; }
            }

            // If USD value requirement, check aggregated value
            if let Some(usd_minimum) = requirement.usd_value_minimum {
                let matching_token_value = self.calculate_cross_chain_token_value(
                    &multi_chain_result.chain_breakdown,
                    &requirement.token_symbol
                );

                if matching_token_value >= usd_minimum {
                    info!(
                        "Cross-chain USD requirement met: ${} >= ${} for {}",
                        matching_token_value, usd_minimum, requirement.token_symbol
                    );
                    requirement_met = true;
                }
            }

            if !requirement_met {
                debug!("Token requirement not met: {:?}", requirement);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify cross-chain NFT requirements
    pub async fn verify_cross_chain_nft_requirement(
        &self,
        wallet_address: &str,
        requirements: &[NFTRequirement],
    ) -> Result<bool> {
        debug!("Verifying cross-chain NFT requirements for {}", wallet_address);

        let multi_chain_result = self.verify_enterprise_tier_multi_chain(wallet_address).await?;

        for requirement in requirements {
            let mut requirement_met = false;

            for (network, assets) in &multi_chain_result.chain_breakdown {
                if requirement.network == "any" || requirement.network == *network {
                    for nft in &assets.nfts {
                        if self.nft_meets_requirement(nft, requirement) {
                            info!(
                                "NFT requirement met: {} on {} with {} tokens",
                                requirement.collection_name, network, nft.token_ids.len()
                            );
                            requirement_met = true;
                            break;
                        }
                    }
                }
                if requirement_met { break; }
            }

            if !requirement_met {
                debug!("NFT requirement not met: {:?}", requirement);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify cross-chain DAO governance requirements
    pub async fn verify_cross_chain_dao_requirement(
        &self,
        wallet_address: &str,
        requirements: &[DAORequirement],
    ) -> Result<bool> {
        debug!("Verifying cross-chain DAO requirements for {}", wallet_address);

        let multi_chain_result = self.verify_enterprise_tier_multi_chain(wallet_address).await?;

        for requirement in requirements {
            let mut requirement_met = false;

            for (network, assets) in &multi_chain_result.chain_breakdown {
                if requirement.network == "any" || requirement.network == *network {
                    for governance in &assets.governance_power {
                        if self.dao_meets_requirement(governance, requirement) {
                            info!(
                                "DAO requirement met: {} on {} with {} voting power",
                                requirement.dao_name, network, governance.voting_power
                            );
                            requirement_met = true;
                            break;
                        }
                    }
                }
                if requirement_met { break; }
            }

            if !requirement_met {
                debug!("DAO requirement not met: {:?}", requirement);
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get comprehensive cross-chain analytics
    pub async fn get_cross_chain_analytics(
        &self,
        wallet_address: &str,
    ) -> Result<CrossChainAnalytics> {
        let multi_chain_result = self.verify_enterprise_tier_multi_chain(wallet_address).await?;

        let analytics = CrossChainAnalytics {
            total_portfolio_value: multi_chain_result.total_value_usd,
            chain_distribution: self.calculate_chain_distribution(&multi_chain_result.chain_breakdown),
            defi_exposure: self.calculate_defi_exposure(&multi_chain_result.chain_breakdown),
            governance_participation: self.calculate_governance_participation(&multi_chain_result.chain_breakdown),
            risk_metrics: self.calculate_risk_metrics(&multi_chain_result.chain_breakdown),
            diversification_score: self.calculate_diversification_score(&multi_chain_result.chain_breakdown),
            enterprise_benefits: self.calculate_enterprise_benefits(&multi_chain_result),
        };

        Ok(analytics)
    }

    // Private helper methods

    async fn verify_chain_assets(&self, wallet_address: &str, network: &str) -> Result<ChainAssets> {
        debug!("Verifying assets for {} on {}", wallet_address, network);

        // Get tokens on this chain
        let tokens = self.get_chain_tokens(wallet_address, network).await?;
        
        // Get NFTs on this chain
        let nfts = self.get_chain_nfts(wallet_address, network).await?;
        
        // Get DeFi positions on this chain
        let defi_positions = self.get_chain_defi_positions(wallet_address, network).await?;
        
        // Get governance power on this chain
        let governance_power = self.get_chain_governance_power(wallet_address, network).await?;

        // Calculate total value for this chain
        let total_value_usd = tokens.iter().map(|t| t.value_usd).sum::<u64>()
            + nfts.iter().map(|n| n.estimated_value_usd.unwrap_or(0)).sum::<u64>()
            + defi_positions.iter().map(|d| d.liquidity_usd + d.rewards_earned_usd).sum::<u64>();

        Ok(ChainAssets {
            network: network.to_string(),
            tokens,
            nfts,
            defi_positions,
            governance_power,
            total_value_usd,
            last_updated: Utc::now(),
        })
    }

    async fn get_chain_tokens(&self, wallet_address: &str, network: &str) -> Result<Vec<TokenHolding>> {
        // Get common tokens for this network
        let token_contracts = self.get_network_token_contracts(network);
        let mut tokens = Vec::new();

        for (symbol, contract_address) in token_contracts {
            match self.web3_service.verify_token_balance_usd(
                wallet_address,
                &contract_address,
                network,
                1, // Check if they have any
                self.get_token_decimals(&symbol)
            ).await {
                Ok(true) => {
                    // They have this token, get the actual balance and value
                    if let Ok(token_info) = self.get_detailed_token_info(
                        wallet_address, 
                        &contract_address, 
                        &symbol, 
                        network
                    ).await {
                        tokens.push(token_info);
                    }
                }
                Ok(false) => {
                    // They don't have this token, skip
                    debug!("Wallet {} has no {} on {}", wallet_address, symbol, network);
                }
                Err(e) => {
                    warn!("Failed to check {} balance for {} on {}: {}", symbol, wallet_address, network, e);
                }
            }
        }

        Ok(tokens)
    }

    async fn get_chain_nfts(&self, wallet_address: &str, network: &str) -> Result<Vec<NFTHolding>> {
        // Get enterprise NFT collections for this network
        let nft_contracts = self.get_network_nft_contracts(network);
        let mut nfts = Vec::new();

        for (collection_name, contract_address) in nft_contracts {
            match self.web3_service.verify_nft_ownership(
                wallet_address,
                &contract_address,
                network,
                None
            ).await {
                Ok(true) => {
                    // They own NFTs from this collection
                    if let Ok(nft_info) = self.get_detailed_nft_info(
                        wallet_address,
                        &contract_address,
                        &collection_name,
                        network
                    ).await {
                        nfts.push(nft_info);
                    }
                }
                Ok(false) => {
                    // They don't own NFTs from this collection
                    debug!("Wallet {} has no {} NFTs on {}", wallet_address, collection_name, network);
                }
                Err(e) => {
                    warn!("Failed to check {} NFTs for {} on {}: {}", collection_name, wallet_address, network, e);
                }
            }
        }

        Ok(nfts)
    }

    async fn get_chain_defi_positions(&self, wallet_address: &str, network: &str) -> Result<Vec<DeFiPosition>> {
        // TODO: Implement DeFi position detection
        // This would integrate with popular DeFi protocols on each chain
        // For now, return empty positions
        Ok(vec![])
    }

    async fn get_chain_governance_power(&self, wallet_address: &str, network: &str) -> Result<Vec<GovernancePower>> {
        // TODO: Implement governance power detection
        // This would check for governance tokens and voting history
        // For now, return empty governance power
        Ok(vec![])
    }

    fn apply_value_aggregation(&self, chain_breakdown: &HashMap<String, ChainAssets>) -> u64 {
        match self.config.value_aggregation_method {
            ValueAggregationMethod::Sum => {
                chain_breakdown.values().map(|assets| assets.total_value_usd).sum()
            }
            ValueAggregationMethod::WeightedSum => {
                let mut total = 0;
                for (network, assets) in chain_breakdown {
                    let weight = self.config.governance_weight_distribution
                        .get(network)
                        .unwrap_or(&1.0);
                    total += (assets.total_value_usd as f32 * weight) as u64;
                }
                total
            }
            ValueAggregationMethod::MaxChain => {
                chain_breakdown.values()
                    .map(|assets| assets.total_value_usd)
                    .max()
                    .unwrap_or(0)
            }
            ValueAggregationMethod::Conservative => {
                // Use 80% of sum to avoid potential double counting
                let sum = chain_breakdown.values().map(|assets| assets.total_value_usd).sum::<u64>();
                (sum as f32 * 0.8) as u64
            }
        }
    }

    fn determine_cross_chain_tier(&self, total_value_usd: u64, chain_breakdown: &HashMap<String, ChainAssets>) -> EnterpriseTier {
        // Base tier determination on total value
        let base_tier = if total_value_usd >= 1_000_000 {
            EnterpriseTier::Whale
        } else if total_value_usd >= 100_000 {
            EnterpriseTier::Enterprise
        } else if total_value_usd >= 10_000 {
            EnterpriseTier::Business
        } else {
            EnterpriseTier::Starter
        };

        // Check for enterprise NFTs or DAO membership upgrades
        let has_enterprise_nfts = chain_breakdown.values()
            .any(|assets| assets.nfts.iter()
                .any(|nft| nft.enterprise_benefits.contains(&"tier_upgrade".to_string())));

        let has_dao_membership = chain_breakdown.values()
            .any(|assets| assets.governance_power.iter()
                .any(|gov| gov.voting_power.parse::<u64>().unwrap_or(0) > 1000));

        // Upgrade tier based on special holdings
        match base_tier {
            EnterpriseTier::Starter if has_enterprise_nfts => EnterpriseTier::Business,
            EnterpriseTier::Business if has_dao_membership => EnterpriseTier::Enterprise,
            _ => base_tier,
        }
    }

    fn generate_aggregated_permissions(&self, tier: &EnterpriseTier, chain_breakdown: &HashMap<String, ChainAssets>) -> Vec<String> {
        let mut permissions = vec![
            "enterprise:analytics:basic".to_string(),
            "enterprise:api:access".to_string(),
        ];

        // Add tier-based permissions
        match tier {
            EnterpriseTier::Business | EnterpriseTier::Enterprise | EnterpriseTier::Whale => {
                permissions.extend(vec![
                    "enterprise:data:real_time".to_string(),
                    "enterprise:analytics:advanced".to_string(),
                    "enterprise:multi_chain:access".to_string(),
                ]);
            }
            _ => {}
        }

        match tier {
            EnterpriseTier::Enterprise | EnterpriseTier::Whale => {
                permissions.extend(vec![
                    "enterprise:integrations:custom".to_string(),
                    "enterprise:webhooks:manage".to_string(),
                    "enterprise:cross_chain:governance".to_string(),
                ]);
            }
            _ => {}
        }

        match tier {
            EnterpriseTier::Whale => {
                permissions.extend(vec![
                    "enterprise:unlimited:access".to_string(),
                    "enterprise:cross_chain:unlimited".to_string(),
                    "enterprise:infrastructure:custom".to_string(),
                ]);
            }
            _ => {}
        }

        // Add chain-specific permissions
        for (network, assets) in chain_breakdown {
            if assets.total_value_usd > 10_000 {
                permissions.push(format!("enterprise:{}:premium", network));
            }
            if !assets.governance_power.is_empty() {
                permissions.push(format!("enterprise:{}:governance", network));
            }
        }

        permissions
    }

    // Additional helper methods for token/NFT/DAO verification
    fn token_meets_requirement(&self, token: &TokenHolding, requirement: &TokenRequirement) -> bool {
        token.symbol.to_uppercase() == requirement.token_symbol.to_uppercase() &&
        token.value_usd >= requirement.usd_value_minimum.unwrap_or(0)
    }

    fn nft_meets_requirement(&self, nft: &NFTHolding, requirement: &NFTRequirement) -> bool {
        nft.collection_name.to_lowercase().contains(&requirement.collection_name.to_lowercase()) &&
        nft.token_ids.len() >= requirement.minimum_tokens as usize
    }

    fn dao_meets_requirement(&self, governance: &GovernancePower, requirement: &DAORequirement) -> bool {
        governance.dao_name.to_lowercase().contains(&requirement.dao_name.to_lowercase()) &&
        governance.voting_power.parse::<u64>().unwrap_or(0) >= 
            requirement.minimum_voting_power.as_ref()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1)
    }

    // Network-specific contract addresses
    fn get_network_token_contracts(&self, network: &str) -> Vec<(String, String)> {
        match network {
            "ethereum" => vec![
                ("USDC".to_string(), "0xa0b86a33e6f2b84af98e67b1f6b40f5af1b1b1b1".to_string()),
                ("USDT".to_string(), "0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()),
                ("WETH".to_string(), "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string()),
            ],
            "polygon" => vec![
                ("USDC".to_string(), "0x2791bca1f2de4661ed88a30c99a7a9449aa84174".to_string()),
                ("USDT".to_string(), "0xc2132d05d31c914a87c6611c10748aeb04b58e8f".to_string()),
                ("WMATIC".to_string(), "0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270".to_string()),
            ],
            _ => vec![],
        }
    }

    fn get_network_nft_contracts(&self, network: &str) -> Vec<(String, String)> {
        match network {
            "ethereum" => vec![
                ("EPSX Enterprise Access".to_string(), "0xb0b86a33e6f2b84af98e67b1f6b40f5af1b1b1b1".to_string()),
                ("EPSX Whale Access".to_string(), "0xd0b86a33e6f2b84af98e67b1f6b40f5af1b1b1b1".to_string()),
            ],
            "polygon" => vec![
                ("EPSX Business Access".to_string(), "0xe0b86a33e6f2b84af98e67b1f6b40f5af1b1b1b1".to_string()),
            ],
            _ => vec![],
        }
    }

    fn get_token_decimals(&self, symbol: &str) -> i32 {
        match symbol.to_uppercase().as_str() {
            "USDC" | "USDT" => 6,
            "WETH" | "DAI" | "WMATIC" => 18,
            _ => 18,
        }
    }

    async fn get_detailed_token_info(&self, wallet_address: &str, contract_address: &str, symbol: &str, network: &str) -> Result<TokenHolding> {
        // TODO: Implement detailed token info retrieval
        // For now, return placeholder data
        Ok(TokenHolding {
            contract_address: contract_address.to_string(),
            symbol: symbol.to_string(),
            balance: "1000000000".to_string(),
            decimals: self.get_token_decimals(symbol) as u8,
            price_usd: 1.0,
            value_usd: 1000,
            is_staked: false,
            staking_rewards: None,
        })
    }

    async fn get_detailed_nft_info(&self, wallet_address: &str, contract_address: &str, collection_name: &str, network: &str) -> Result<NFTHolding> {
        // TODO: Implement detailed NFT info retrieval
        // For now, return placeholder data
        Ok(NFTHolding {
            contract_address: contract_address.to_string(),
            collection_name: collection_name.to_string(),
            token_ids: vec!["1".to_string()],
            floor_price_usd: Some(1000),
            estimated_value_usd: Some(1000),
            metadata_attributes: HashMap::new(),
            enterprise_benefits: vec!["tier_upgrade".to_string()],
        })
    }

    fn calculate_cross_chain_token_value(&self, chain_breakdown: &HashMap<String, ChainAssets>, token_symbol: &str) -> u64 {
        chain_breakdown.values()
            .flat_map(|assets| &assets.tokens)
            .filter(|token| token.symbol.to_uppercase() == token_symbol.to_uppercase())
            .map(|token| token.value_usd)
            .sum()
    }

    // Caching methods
    async fn get_cached_multi_chain_result(&self, wallet_address: &str) -> Result<Option<MultiChainVerificationResult>> {
        let row = sqlx::query!(
            r#"
            SELECT verification_data
            FROM web3_permission_cache
            WHERE wallet_address = $1
              AND permission_type = 'multi_chain_tier'
              AND expires_at > CURRENT_TIMESTAMP
            ORDER BY cached_at DESC
            LIMIT 1
            "#,
            wallet_address.to_lowercase()
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = row {
            if let Some(data) = row.verification_data {
                match serde_json::from_value::<MultiChainVerificationResult>(data) {
                    Ok(result) => return Ok(Some(result)),
                    Err(e) => warn!("Failed to deserialize cached multi-chain verification: {}", e),
                }
            }
        }

        Ok(None)
    }

    async fn cache_multi_chain_result(&self, result: &MultiChainVerificationResult) -> Result<()> {
        let expires_at = Utc::now() + Duration::minutes(self.config.cache_duration_minutes);

        sqlx::query!(
            r#"
            INSERT INTO web3_permission_cache (
                wallet_address, permission_type, contract_address, network,
                verification_result, verification_data, cached_at, expires_at
            )
            VALUES ($1, 'multi_chain_tier', '', 'multi_chain', $2, $3, CURRENT_TIMESTAMP, $4)
            ON CONFLICT (wallet_address, contract_address, network, permission_type)
            DO UPDATE SET
                verification_result = $2,
                verification_data = $3,
                cached_at = CURRENT_TIMESTAMP,
                expires_at = $4
            "#,
            result.wallet_address,
            true,
            serde_json::to_value(result)?,
            expires_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Analytics calculation methods
    fn calculate_chain_distribution(&self, chain_breakdown: &HashMap<String, ChainAssets>) -> HashMap<String, f32> {
        let total_value = chain_breakdown.values()
            .map(|assets| assets.total_value_usd)
            .sum::<u64>() as f32;

        if total_value == 0.0 {
            return HashMap::new();
        }

        chain_breakdown.iter()
            .map(|(network, assets)| {
                (network.clone(), assets.total_value_usd as f32 / total_value)
            })
            .collect()
    }

    fn calculate_defi_exposure(&self, chain_breakdown: &HashMap<String, ChainAssets>) -> f32 {
        let total_value = chain_breakdown.values()
            .map(|assets| assets.total_value_usd)
            .sum::<u64>() as f32;

        let defi_value = chain_breakdown.values()
            .flat_map(|assets| &assets.defi_positions)
            .map(|pos| pos.liquidity_usd + pos.rewards_earned_usd)
            .sum::<u64>() as f32;

        if total_value == 0.0 { 0.0 } else { defi_value / total_value }
    }

    fn calculate_governance_participation(&self, chain_breakdown: &HashMap<String, ChainAssets>) -> u32 {
        chain_breakdown.values()
            .flat_map(|assets| &assets.governance_power)
            .map(|gov| gov.votes_cast)
            .sum()
    }

    fn calculate_risk_metrics(&self, _chain_breakdown: &HashMap<String, ChainAssets>) -> RiskMetrics {
        // TODO: Implement sophisticated risk calculation
        RiskMetrics {
            overall_risk_score: 3.5,
            concentration_risk: 0.2,
            liquidity_risk: 0.1,
            smart_contract_risk: 0.3,
        }
    }

    fn calculate_diversification_score(&self, chain_breakdown: &HashMap<String, ChainAssets>) -> f32 {
        // Simple diversification score based on number of chains and tokens
        let num_chains = chain_breakdown.len() as f32;
        let num_tokens = chain_breakdown.values()
            .map(|assets| assets.tokens.len())
            .sum::<usize>() as f32;

        // Score from 0-1 based on diversification
        (num_chains.sqrt() * num_tokens.sqrt() / 20.0).min(1.0)
    }

    fn calculate_enterprise_benefits(&self, result: &MultiChainVerificationResult) -> Vec<String> {
        let mut benefits = vec![];

        match result.cross_chain_tier {
            EnterpriseTier::Business | EnterpriseTier::Enterprise | EnterpriseTier::Whale => {
                benefits.push("Cross-chain analytics".to_string());
                benefits.push("Multi-network monitoring".to_string());
            }
            _ => {}
        }

        match result.cross_chain_tier {
            EnterpriseTier::Enterprise | EnterpriseTier::Whale => {
                benefits.push("Cross-chain governance".to_string());
                benefits.push("Advanced DeFi insights".to_string());
            }
            _ => {}
        }

        match result.cross_chain_tier {
            EnterpriseTier::Whale => {
                benefits.push("Unlimited cross-chain access".to_string());
                benefits.push("Custom infrastructure".to_string());
            }
            _ => {}
        }

        benefits
    }
}

/// Cross-chain analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainAnalytics {
    pub total_portfolio_value: u64,
    pub chain_distribution: HashMap<String, f32>,
    pub defi_exposure: f32,
    pub governance_participation: u32,
    pub risk_metrics: RiskMetrics,
    pub diversification_score: f32,
    pub enterprise_benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub overall_risk_score: f32,
    pub concentration_risk: f32,
    pub liquidity_risk: f32,
    pub smart_contract_risk: f32,
}