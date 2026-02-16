// ERC-20 token balance validation for Web3 permissions

use crate::prelude::*;
use crate::domain::wallet_management::value_objects::{WalletAddress, Permission};
use crate::domain::wallet_management::domain_services::{Web3ValidationResult, Web3ValidationType};
use super::cache::{Web3CacheMgr, TokenResult};
use super::config::BlockchainCfg;
use tracing::{error, info, debug};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256, Bytes, TransactionRequest};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

pub struct TokenValidator {
    cache: Web3CacheMgr,
    cfg: BlockchainCfg,
}

impl TokenValidator {
    pub fn new(cache: Web3CacheMgr, cfg: BlockchainCfg) -> Self {
        Self { cache, cfg }
    }

    /// Validate token balance on blockchain
    pub async fn validate(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min: &str,
        chain: u64,
    ) -> AppResult<TokenResult> {
        debug!("Validating token balance for wallet {} on contract {} (min: {}, chain: {})", wallet.as_str(), contract, min, chain);

        // Check cache first
        if let Some(cached) = self.cache.get_token(wallet, contract, min, chain).await? {
            return Ok(cached);
        }

        // Get RPC endpoint
        let rpc = self.cfg.rpc_endpoints.get(&chain)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint for chain {}", chain)
            ).with_component("token_validator"))?;

        // Validate on blockchain
        let result = self.check_blockchain(wallet, contract, min, rpc).await?;

        // Cache result
        self.cache.set_token(wallet, contract, min, chain, &result).await?;

        Ok(result)
    }

    /// Validate token permission
    pub async fn validate_perm(
        &self,
        wallet: &WalletAddress,
        perm: &Permission,
        contract: &str,
        min: &str,
        chain: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate(wallet, contract, min, chain).await {
            Ok(result) => Ok(Web3ValidationResult {
                permission: perm.clone(),
                is_valid: result.meets_minimum_balance,
                validation_type: Web3ValidationType::TokenGated,
                blockchain_data: Some(format!(
                    "Token balance: {} (required: {})",
                    result.current_balance,
                    min
                )),
                error_details: result.error_details,
            }),
            Err(e) => {
                error!("Token validation failed for {}: {}", wallet.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: perm.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::TokenGated,
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
        min: &str,
        rpc: &str,
    ) -> AppResult<TokenResult> {
        info!("Checking token balance for wallet {} on contract {}", wallet.as_str(), contract);

        let timeout_dur = Duration::from_millis(self.cfg.request_timeout_ms);
        let result = timeout(timeout_dur, async {
            self.check_rpc(wallet, contract, min, rpc).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error("Token balance check timeout".to_string())
                .with_component("token_validator")
        })??;

        Ok(result)
    }

    async fn check_rpc(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min: &str,
        rpc: &str,
    ) -> AppResult<TokenResult> {
        debug!("Making token balance RPC call to {}", rpc);

        // Create provider
        let provider = Provider::<Http>::try_from(rpc)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;

        // Parse addresses
        let contract_addr = Address::from_str(contract)
            .map_err(|e| AppError::validation_error(format!("Invalid contract address: {}", e)))?;
        let wallet_addr = Address::from_str(wallet.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        // Parse minimum balance
        let min_u256 = U256::from_dec_str(min)
            .map_err(|e| AppError::validation_error(format!("Invalid min_balance format: {}", e)))?;

        // Call balanceOf(address) on ERC20 contract
        let call_data = ethers::abi::encode(&[ethers::abi::Token::Address(wallet_addr)]);

        // ERC20 balanceOf function selector: 0x70a08231
        let mut fn_call = vec![0x70, 0xa0, 0x82, 0x31];
        fn_call.extend_from_slice(&call_data);

        let req = TransactionRequest::new()
            .to(contract_addr)
            .data(Bytes::from(fn_call));

        let tx = TypedTransaction::Legacy(req);
        let result = provider.call(&tx, None).await
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to get token balance: {}", e)))?;

        let current = if result.len() >= 32 {
            U256::from_big_endian(&result)
        } else {
            U256::zero()
        };

        let meets_min = current >= min_u256;

        info!("Token balance check complete: {} >= {} = {}", current, min_u256, meets_min);

        Ok(TokenResult {
            meets_minimum_balance: meets_min,
            current_balance: current.to_string(),
            min_balance_required: min.to_string(),
            contract_address: contract.to_string(),
            chain_id: 1, // Will be detected from RPC
            error_details: None,
        })
    }
}
