use std::collections::HashMap;
use std::time::Duration;
use anyhow::{Result, Context};
use ethers::providers::{Provider, Http, Middleware};
use ethers::types::{Address, U256};
use crate::core::errors::AppError;

#[derive(Clone)]
pub struct BlockchainValidationClient {
    providers: HashMap<u64, Provider<Http>>,
    timeout_ms: u64,
}

impl BlockchainValidationClient {
    pub fn new() -> Self {
        let mut providers = HashMap::new();

        // BSC Mainnet
        let bsc_mainnet_url = std::env::var("BSC_MAINNET_RPC_URL")
            .unwrap_or_else(|_| "https://bsc-dataseed.binance.org".to_string());

        // BSC Testnet
        let bsc_testnet_url = std::env::var("BSC_TESTNET_RPC_URL")
            .unwrap_or_else(|_| "https://data-seed-prebsc-1-s1.binance.org:8545".to_string());

        // Ethereum Mainnet
        let eth_mainnet_url = std::env::var("ETH_MAINNET_RPC_URL")
            .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());

        // Initialize providers (will be connected on first use)
        providers.insert(56, Provider::<Http>::try_from(bsc_mainnet_url).unwrap());
        providers.insert(97, Provider::<Http>::try_from(bsc_testnet_url).unwrap());
        providers.insert(1, Provider::<Http>::try_from(eth_mainnet_url).unwrap());

        Self {
            providers,
            timeout_ms: 30000, // 30 seconds default
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Get provider for a specific chain
    fn get_provider(&self, chain_id: u64) -> Result<&Provider<Http>> {
        self.providers.get(&chain_id)
            .ok_or_else(|| AppError::blockchain_rpc_error(format!("Unsupported chain ID: {}", chain_id)).into())
    }

    /// Validate if a wallet owns an NFT from a specific contract
    pub async fn validate_nft_ownership(
        &self,
        chain_id: u64,
        wallet_address: &str,
        contract_address: &str,
        token_ids: &[u64],
    ) -> Result<NftValidationResult> {
        let provider = self.get_provider(chain_id)?;

        let wallet_addr = wallet_address.parse::<Address>()
            .context("Invalid wallet address format")?;
        let contract_addr = contract_address.parse::<Address>()
            .context("Invalid contract address format")?;

        let mut owned_tokens = Vec::new();
        let mut balance = U256::zero();

        for &token_id in token_ids {
            let timeout_future = tokio::time::timeout(
                Duration::from_millis(self.timeout_ms),
                self.check_nft_ownership(provider.clone(), contract_addr, wallet_addr, token_id)
            ).await;

            match timeout_future {
                Ok(Ok(owns)) => {
                    if owns {
                        owned_tokens.push(token_id);
                    }
                },
                Ok(Err(e)) => {
                    tracing::warn!("Failed to check NFT ownership for token {}: {:?}", token_id, e);
                },
                Err(_) => {
                    tracing::warn!("Timeout checking NFT ownership for token {}", token_id);
                }
            }
        }

        // Get total balance
        let balance_future = tokio::time::timeout(
            Duration::from_millis(self.timeout_ms),
            self.check_nft_balance(provider.clone(), contract_addr, wallet_addr)
        ).await;

        if let Ok(Ok(bal)) = balance_future {
            balance = bal;
        }

        Ok(NftValidationResult {
            is_valid: !owned_tokens.is_empty(),
            owned_tokens,
            balance,
            contract_address: contract_address.to_string(),
            chain_id,
        })
    }

    /// Validate if a wallet has sufficient token balance
    pub async fn validate_token_balance(
        &self,
        chain_id: u64,
        wallet_address: &str,
        contract_address: &str,
        min_balance: &str,
    ) -> Result<TokenValidationResult> {
        let provider = self.get_provider(chain_id)?;

        let wallet_addr = wallet_address.parse::<Address>()
            .context("Invalid wallet address format")?;
        let contract_addr = contract_address.parse::<Address>()
            .context("Invalid contract address format")?;

        let min_balance_wei = parse_token_amount(min_balance, 18)?; // Default to 18 decimals

        let timeout_future = tokio::time::timeout(
            Duration::from_millis(self.timeout_ms),
            self.check_token_balance(provider.clone(), contract_addr, wallet_addr)
        ).await;

        let balance = match timeout_future {
            Ok(Ok(balance)) => balance,
            Ok(Err(e)) => {
                tracing::warn!("Failed to get token balance: {:?}", e);
                return Err(AppError::blockchain_rpc_error(format!("Failed to get token balance: {}", e)).into());
            },
            Err(_) => {
                return Err(AppError::blockchain_rpc_error("Token balance check timed out".to_string()).into());
            }
        };

        Ok(TokenValidationResult {
            is_valid: balance >= min_balance_wei,
            balance,
            min_balance: min_balance_wei,
            contract_address: contract_address.to_string(),
            chain_id,
        })
    }

    /// Validate DAO governance voting power
    pub async fn validate_dao_voting_power(
        &self,
        chain_id: u64,
        wallet_address: &str,
        dao_contract: &str,
        min_voting_power: &str,
    ) -> Result<DaoValidationResult> {
        let provider = self.get_provider(chain_id)?;

        let wallet_addr = wallet_address.parse::<Address>()
            .context("Invalid wallet address format")?;
        let contract_addr = dao_contract.parse::<Address>()
            .context("Invalid DAO contract address format")?;

        let min_power = parse_token_amount(min_voting_power, 18)?;
        let voting_power = self.try_get_voting_power(contract_addr, wallet_addr, provider).await?;

        Ok(DaoValidationResult {
            is_valid: voting_power >= min_power,
            voting_power,
            min_voting_power: min_power,
            dao_contract: dao_contract.to_string(),
            chain_id,
        })
    }

    async fn check_nft_ownership(
        &self,
        provider: Provider<Http>,
        contract_addr: Address,
        wallet_addr: Address,
        token_id: u64,
    ) -> Result<bool> {
        // For now, return a placeholder to avoid complex contract interaction
        // In production, this would use proper ethers contract calls
        tracing::info!("Checking NFT ownership for wallet {} token {} on contract {}", wallet_addr, token_id, contract_addr);
        Ok(false) // Conservative default
    }

    async fn check_nft_balance(
        &self,
        provider: Provider<Http>,
        contract_addr: Address,
        wallet_addr: Address,
    ) -> Result<U256> {
        // For now, return a placeholder
        tracing::info!("Checking NFT balance for wallet {} on contract {}", wallet_addr, contract_addr);
        Ok(U256::zero())
    }

    async fn check_token_balance(
        &self,
        provider: Provider<Http>,
        contract_addr: Address,
        wallet_addr: Address,
    ) -> Result<U256> {
        // For now, return a placeholder
        tracing::info!("Checking token balance for wallet {} on contract {}", wallet_addr, contract_addr);
        Ok(U256::zero())
    }

    async fn try_get_voting_power(
        &self,
        contract_addr: Address,
        wallet_addr: Address,
        provider: &Provider<Http>,
    ) -> Result<U256> {
        // For now, return a placeholder
        tracing::info!("Checking voting power for wallet {} on contract {}", wallet_addr, contract_addr);
        Ok(U256::zero())
    }
}

// Helper function to parse token amounts
fn parse_token_amount(amount: &str, decimals: u8) -> Result<U256> {
    if amount == "0" || amount.is_empty() {
        return Ok(U256::zero());
    }

    // Try to parse as decimal number first
    if let Ok(decimal_amount) = rust_decimal::Decimal::from_str_radix(amount, 10) {
        let decimals_multiplier = rust_decimal::Decimal::new(10_i64.pow(decimals as u32), 0);
        let wei_amount = (decimal_amount * decimals_multiplier)
            .to_string()
            .split('.')
            .next()
            .unwrap_or("0")
            .parse::<u128>()
            .unwrap_or(0);
        return Ok(U256::from(wei_amount));
    }

    // Fallback: parse as raw integer
    amount.parse::<u128>()
        .map(U256::from)
        .map_err(|_| AppError::blockchain_rpc_error(format!("Invalid token amount: {}", amount)).into())
}


#[derive(Debug, Clone)]
pub struct NftValidationResult {
    pub is_valid: bool,
    pub owned_tokens: Vec<u64>,
    pub balance: U256,
    pub contract_address: String,
    pub chain_id: u64,
}

#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    pub balance: U256,
    pub min_balance: U256,
    pub contract_address: String,
    pub chain_id: u64,
}

#[derive(Debug, Clone)]
pub struct DaoValidationResult {
    pub is_valid: bool,
    pub voting_power: U256,
    pub min_voting_power: U256,
    pub dao_contract: String,
    pub chain_id: u64,
}