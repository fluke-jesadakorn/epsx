// DAO voting power validation for Web3 permissions

use crate::prelude::*;
use crate::domain::wallet_management::value_objects::{WalletAddress, Permission};
use crate::domain::wallet_management::domain_services::{Web3ValidationResult, Web3ValidationType};
use super::cache::{Web3CacheMgr, DaoResult};
use super::config::BlockchainCfg;
use tracing::{error, info, warn, debug};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, U256, Bytes, TransactionRequest};
use ethers::types::transaction::eip2718::TypedTransaction;
use std::str::FromStr;
use std::time::Duration;
use std::collections::HashMap;
use tokio::time::timeout;

pub struct DaoValidator {
    cache: Web3CacheMgr,
    cfg: BlockchainCfg,
}

impl DaoValidator {
    pub fn new(cache: Web3CacheMgr, cfg: BlockchainCfg) -> Self {
        Self { cache, cfg }
    }

    /// Validate DAO membership on blockchain
    pub async fn validate(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min_power: &str,
        chain: u64,
    ) -> AppResult<DaoResult> {
        debug!("Validating DAO membership for wallet {} on contract {} (min voting power: {}, chain: {})", wallet.as_str(), contract, min_power, chain);

        // Check cache first
        if let Some(cached) = self.cache.get_dao(wallet, contract, min_power, chain).await? {
            return Ok(cached);
        }

        // Get RPC endpoint
        let rpc = self.cfg.rpc_endpoints.get(&chain)
            .ok_or_else(|| AppError::blockchain_rpc_error(
                format!("No RPC endpoint for chain {}", chain)
            ).with_component("dao_validator"))?;

        // Validate on blockchain
        let result = self.check_blockchain(wallet, contract, min_power, rpc).await?;

        // Cache result
        self.cache.set_dao(wallet, contract, min_power, chain, &result).await?;

        Ok(result)
    }

    /// Validate DAO permission
    pub async fn validate_perm(
        &self,
        wallet: &WalletAddress,
        perm: &Permission,
        contract: &str,
        min_power: &str,
        chain: u64,
    ) -> AppResult<Web3ValidationResult> {
        match self.validate(wallet, contract, min_power, chain).await {
            Ok(result) => Ok(Web3ValidationResult {
                permission: perm.clone(),
                is_valid: result.meets_minimum_voting_power,
                validation_type: Web3ValidationType::DaoGovernance,
                blockchain_data: Some(format!(
                    "DAO voting power: {} (required: {})",
                    result.current_voting_power,
                    min_power
                )),
                error_details: result.error_details,
            }),
            Err(e) => {
                error!("DAO validation failed for {}: {}", wallet.as_str(), e);
                Ok(Web3ValidationResult {
                    permission: perm.clone(),
                    is_valid: false,
                    validation_type: Web3ValidationType::DaoGovernance,
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
        min_power: &str,
        rpc: &str,
    ) -> AppResult<DaoResult> {
        info!("🔍 Checking DAO membership for wallet {} on contract {}", wallet.as_str(), contract);

        let timeout_dur = Duration::from_millis(self.cfg.request_timeout_ms);
        let result = timeout(timeout_dur, async {
            self.check_rpc(wallet, contract, min_power, rpc).await
        }).await.map_err(|_| {
            AppError::blockchain_rpc_error("DAO membership check timeout".to_string())
                .with_component("dao_validator")
        })??;

        Ok(result)
    }

    async fn check_rpc(
        &self,
        wallet: &WalletAddress,
        contract: &str,
        min_power: &str,
        rpc: &str,
    ) -> AppResult<DaoResult> {
        debug!("🔗 Making DAO membership RPC call to {}", rpc);

        // Create provider
        let provider = Provider::<Http>::try_from(rpc)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create provider: {}", e)))?;

        // Parse addresses
        let contract_addr = Address::from_str(contract)
            .map_err(|e| AppError::validation_error(format!("Invalid DAO contract address: {}", e)))?;
        let wallet_addr = Address::from_str(wallet.as_str())
            .map_err(|e| AppError::validation_error(format!("Invalid wallet address: {}", e)))?;

        // Parse minimum voting power
        let min_power_u256 = U256::from_dec_str(min_power)
            .map_err(|e| AppError::validation_error(format!("Invalid min_voting_power format: {}", e)))?;

        // Common DAO functions to try (different DAOs use different function names)
        let functions = vec![
            ("getVotes", 0xb9a3c84c), // Common ERC20Votes function
            ("balanceOf", 0x70a08231), // ERC20 balance (governance tokens)
            ("votingPower", 0x0d32dd66), // Custom voting power function
        ];

        let mut current_power = U256::zero();
        let mut found = false;

        for (fn_name, selector) in functions {
            let selector: u32 = selector;
            let call_data = ethers::abi::encode(&[ethers::abi::Token::Address(wallet_addr)]);

            let mut fn_call = selector.to_be_bytes().to_vec();
            fn_call.extend_from_slice(&call_data);

            let req = TransactionRequest::new()
                .to(contract_addr)
                .data(Bytes::from(fn_call));

            let tx = TypedTransaction::Legacy(req);
            match provider.call(&tx, None).await {
                Ok(result) => {
                    if result.len() >= 32 {
                        current_power = U256::from_big_endian(&result);
                        found = true;
                        debug!("✅ Found voting power {} using function {}", current_power, fn_name);
                        break;
                    }
                }
                Err(e) => {
                    debug!("Failed to call {} function: {}", fn_name, e);
                    // Continue trying other functions
                }
            }
        }

        if !found {
            warn!("No supported DAO voting power function found on contract {}", contract);
            // Return zero voting power instead of error for graceful degradation
        }

        let meets_min = current_power >= min_power_u256;
        let is_member = current_power > U256::zero();

        // Check for delegation info - try to find who the wallet has delegated to
        let mut delegation_info = HashMap::new();
        
        // Try common delegation functions
        let delegation_functions = vec![
            ("delegates", 0x587cde1e_u32), // ERC20Votes delegates(address)
            ("getDelegate", 0x9ad54685_u32), // Alternative delegate getter
        ];

        for (fn_name, selector) in delegation_functions {
            let call_data = ethers::abi::encode(&[ethers::abi::Token::Address(wallet_addr)]);
            
            let mut fn_call = selector.to_be_bytes().to_vec();
            fn_call.extend_from_slice(&call_data);

            let req = TransactionRequest::new()
                .to(contract_addr)
                .data(Bytes::from(fn_call));

            let tx = TypedTransaction::Legacy(req);
            if let Ok(result) = provider.call(&tx, None).await {
                if result.len() >= 32 {
                    // Parse address from result (last 20 bytes of 32-byte word)
                    let delegate_bytes = &result[12..32];
                    let delegate_addr = Address::from_slice(delegate_bytes);
                    
                    // Only record if delegate is different from wallet (actually delegated)
                    if delegate_addr != wallet_addr && delegate_addr != Address::zero() {
                        delegation_info.insert(
                            "delegated_to".to_string(),
                            format!("{:?}", delegate_addr)
                        );
                        debug!("✅ Found delegation from {} to {:?} using {}", wallet.as_str(), delegate_addr, fn_name);
                    }
                    break;
                }
            }
        }

        info!("✅ DAO membership check complete: voting power {} >= {} = {}", current_power, min_power_u256, meets_min);

        Ok(DaoResult {
            meets_minimum_voting_power: meets_min,
            current_voting_power: current_power.to_string(),
            min_voting_power_required: min_power.to_string(),
            dao_contract: contract.to_string(),
            chain_id: 1, // Will be detected from RPC
            is_member,
            delegation_info,
            error_details: None,
        })
    }
}
