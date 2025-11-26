// Web3 validation result cache manager
// Handles caching of NFT, Token, and DAO validation results

use crate::prelude::*;
use crate::domain::wallet_management::value_objects::WalletAddress;
use crate::infrastructure::cache::Cache;
use tracing::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CACHE_TTL_SECONDS: u64 = 300; // 5 minutes

/// Generic cache manager for Web3 validation results
#[derive(Clone)]
pub struct Web3CacheMgr {
    cache: Option<Arc<dyn Cache>>,
}

impl Web3CacheMgr {
    pub fn new(cache: Option<Arc<dyn Cache>>) -> Self {
        Self { cache }
    }

    // NFT validation cache
    pub async fn get_nft(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        chain: u64,
    ) -> AppResult<Option<NftResult>> {
        self.get_cached("nft", wallet, contract, &format_tokens(tokens), chain).await
    }

    pub async fn set_nft(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        chain: u64,
        result: &NftResult,
    ) -> AppResult<()> {
        self.set_cached("nft", wallet, contract, &format_tokens(tokens), chain, result).await
    }

    // Token balance cache
    pub async fn get_token(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min: &str,
        chain: u64,
    ) -> AppResult<Option<TokenResult>> {
        self.get_cached("token", wallet, contract, min, chain).await
    }

    pub async fn set_token(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min: &str,
        chain: u64,
        result: &TokenResult,
    ) -> AppResult<()> {
        self.set_cached("token", wallet, contract, min, chain, result).await
    }

    // DAO membership cache
    pub async fn get_dao(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min_power: &str,
        chain: u64,
    ) -> AppResult<Option<DaoResult>> {
        self.get_cached("dao", wallet, contract, min_power, chain).await
    }

    pub async fn set_dao(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min_power: &str,
        chain: u64,
        result: &DaoResult,
    ) -> AppResult<()> {
        self.set_cached("dao", wallet, contract, min_power, chain, result).await
    }

    // Generic cache operations
    async fn get_cached<T: for<'de> Deserialize<'de>>(
        &self,
        prefix: &str,
        wallet: &WalletAddress,
        contract: &str,
        param: &str,
        chain: u64,
    ) -> AppResult<Option<T>> {
        if let Some(cache) = &self.cache {
            let key = cache_key(prefix, wallet, contract, param, chain);
            if let Some(data) = cache.get(&key) {
                if let Ok(result) = serde_json::from_str::<T>(&data) {
                    debug!("Cache hit: {}", key);
                    return Ok(Some(result));
                }
            }
        }
        Ok(None)
    }

    async fn set_cached<T: Serialize>(
        &self,
        prefix: &str,
        wallet: &WalletAddress,
        contract: &str,
        param: &str,
        chain: u64,
        result: &T,
    ) -> AppResult<()> {
        if let Some(cache) = &self.cache {
            let key = cache_key(prefix, wallet, contract, param, chain);
            if let Ok(data) = serde_json::to_string(result) {
                cache.set(&key, data, Some(CACHE_TTL_SECONDS));
                debug!("Cached: {}", key);
            }
        }
        Ok(())
    }
}

// Helpers

fn cache_key(prefix: &str, wallet: &WalletAddress, contract: &str, param: &str, chain: u64) -> String {
    format!("{}:{}:{}:{}:{}", prefix, wallet.as_str(), contract, param, chain)
}

fn format_tokens(tokens: &[u64]) -> String {
    tokens.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")
}

// Result types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftResult {
    pub owns_required_nfts: bool,
    pub owned_token_ids: Vec<u64>,
    pub contract_address: String,
    pub chain_id: u64,
    pub error_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResult {
    pub meets_minimum_balance: bool,
    pub current_balance: String,
    pub min_balance_required: String,
    pub contract_address: String,
    pub chain_id: u64,
    pub error_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoResult {
    pub meets_minimum_voting_power: bool,
    pub current_voting_power: String,
    pub min_voting_power_required: String,
    pub dao_contract: String,
    pub chain_id: u64,
    pub is_member: bool,
    pub delegation_info: HashMap<String, String>,
    pub error_details: Option<String>,
}
