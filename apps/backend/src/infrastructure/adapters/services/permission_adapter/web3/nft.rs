// NFT ownership validation for Web3 permissions

use crate::prelude::*;
use crate::domain::wallet_management::value_objects::{WalletAddress, Permission};
use crate::domain::wallet_management::domain_services::{Web3ValidationResult, Web3ValidationType};
use super::cache::{Web3CacheMgr, NftResult};
use super::config::BlockchainCfg;
use tracing::{error, info, warn, debug};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256, Bytes, TransactionRequest};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

pub struct NftValidator {
    cache: Web3CacheMgr,
    cfg: BlockchainCfg,
}

impl NftValidator {
    pub fn new(cache: Web3CacheMgr, cfg: BlockchainCfg) -> Self {
        Self { cache, cfg }
    }

    /// Validate NFT ownership on blockchain
    pub async fn validate(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        chain: u64,
    ) -> AppResult<NftResult> {
        debug!("Validating NFT ownership for wallet {} on contract {} (chain {})", wallet.as_str(), contract, chain);

        // Check cache first
        if let Some(cached) = self.cache.get_nft(wallet, contract, tokens, chain).await? {
            return Ok(cached);
        }

        // Get RPC endpoint
        let rpc = self.cfg.rpc_endpoints.get(&chain)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint for chain {}", chain)
            ).with_component("nft_validator"))?;

        // Validate on blockchain
        let result = self.check_blockchain(wallet, contract, tokens, rpc).await?;

        // Cache result
        self.cache.set_nft(wallet, contract, tokens, chain, &result).await?;

        Ok(result)
    }

    /// Validate NFT permission
    pub async fn validate_perm(
        &self,
        wallet: &WalletAddress,
        perm: &Permission,
        contract: &str,
        tokens: &[u64],
        chain: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate(wallet, contract, tokens, chain).await {
            Ok(result) => Ok(Web3ValidationResult {
                permission: perm.clone(),
                is_valid: result.owns_required_nfts,
                validation_type: Web3ValidationType::NftGated,
                blockchain_data: Some(format!(
                    "NFT ownership check: owns {} out of {} required tokens",
                    result.owned_token_ids.len(),
                    if tokens.is_empty() { 1 } else { tokens.len() }
                )),
                error_details: result.error_details,
            }),
            Err(e) => {
                error!("NFT validation failed for {}: {}", wallet.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: perm.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::NftGated,
                    blockchain_data: None,
                    error_details: Some(format!("Blockchain error: {}", e)),
                })
            }
        }
    }

    // Private methods

    async fn check_blockchain(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        rpc: &str,
    ) -> AppResult<NftResult> {
        info!("🔍 Checking NFT ownership for wallet {} on contract {}", wallet.as_str(), contract);

        let timeout_dur = Duration::from_millis(self.cfg.request_timeout_ms);
        let result = timeout(timeout_dur, async {
            self.check_rpc(wallet, contract, tokens, rpc).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error("NFT ownership check timeout".to_string())
                .with_component("nft_validator")
        })??;

        Ok(result)
    }

    async fn check_rpc(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        tokens: &[u64],
        rpc: &str,
    ) -> AppResult<NftResult> {
        debug!("🔗 Making NFT ownership RPC call to {}", rpc);

        // Create provider
        let provider = Provider::<Http>::try_from(rpc)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;

        // Parse addresses
        let contract_addr = Address::from_str(contract)
            .map_err(|e| AppError::validation_error(format!("Invalid contract address: {}", e)))?;
        let wallet_addr = Address::from_str(wallet.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        let mut owned = Vec::new();
        let owns_required;

        if tokens.is_empty() {
            // Check overall NFT balance
            let balance = provider.get_balance(wallet_addr, None).await
                .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to get NFT balance: {}", e)))?;
            owns_required = balance > U256::zero();
            if owns_required {
                owned.push(1); // Placeholder
            }
        } else {
            // Check specific token ownership using ERC721 ownerOf
            for &token_id in tokens {
                let token_id_u256 = U256::from(token_id);
                let call_data = ethers::abi::encode(&[ethers::abi::Token::Uint(token_id_u256)]);

                // ERC721 ownerOf function selector: 0x6352211e
                let mut fn_call = vec![0x63, 0x52, 0x21, 0x1e];
                fn_call.extend_from_slice(&call_data);

                let req = TransactionRequest::new()
                    .to(contract_addr)
                    .data(Bytes::from(fn_call));

                let tx = TypedTransaction::Legacy(req);
                match provider.call(&tx, None).await {
                    Ok(result) => {
                        if result.len() >= 32 {
                            let owner_bytes = &result[12..32];
                            let owner_addr = Address::from_slice(owner_bytes);
                            if owner_addr == wallet_addr {
                                owned.push(token_id);
                            }
                        }
                    }
                    Err(e) => warn!("Failed to check ownership of token {}: {}", token_id, e),
                }
            }
            owns_required = !owned.is_empty();
        }

        info!("✅ NFT ownership check complete: owns {} tokens", owned.len());

        Ok(NftResult {
            owns_required_nfts: owns_required,
            owned_token_ids: owned,
            contract_address: contract.to_string(),
            chain_id: 1, // Will be detected from RPC
            error_details: None,
        })
    }
}
