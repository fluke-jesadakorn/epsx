use std::collections::HashMap;
use std::time::Duration;
use anyhow::{Result, Context};
use ethers::providers::{Provider, Http, Middleware};
use ethers::types::{Address, U256};
use epsx_contracts::errors::AppError;

#[derive(Clone)]
pub struct BlockchainValidationClient {
    providers: HashMap<u64, Provider<Http>>,
    timeout_ms: u64,
}



impl BlockchainValidationClient {
    pub fn new() -> Result<Self> {
        let mut providers = HashMap::new();

        // BSC Mainnet
        let bsc_mainnet_url = std::env::var("BSC_MAINNET_RPC_URL")
            .ok().filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://bsc-dataseed.binance.org".to_string());

        // BSC Testnet
        let bsc_testnet_url = std::env::var("BSC_TESTNET_RPC_URL")
            .ok().filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://data-seed-prebsc-1-s1.binance.org:8545".to_string());

        // Ethereum Mainnet
        let eth_mainnet_url = std::env::var("ETH_MAINNET_RPC_URL")
            .ok().filter(|s| !s.is_empty())
            .unwrap_or_else(|| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());

        // Initialize providers
        providers.insert(56, Provider::<Http>::try_from(bsc_mainnet_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create BSC provider: {}", e)))?);
        providers.insert(97, Provider::<Http>::try_from(bsc_testnet_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create BSC Testnet provider: {}", e)))?);
        providers.insert(1, Provider::<Http>::try_from(eth_mainnet_url)
            .map_err(|e| AppError::blockchain_rpc_error(format!("Failed to create ETH provider: {}", e)))?);

        Ok(Self {
            providers,
            timeout_ms: 30000, // 30 seconds default
        })
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
        // ABI for ownerOf(uint256) -> address
        // 0x6352211e
        use ethers::abi::{Function, Param, ParamType, StateMutability, Token};
        
        // Manual ABI construction to avoid macro complexities/dependencies
        // function ownerOf(uint256 tokenId) external view returns (address owner)
        #[allow(deprecated)]
        let function = Function {
            name: "ownerOf".to_owned(),
            inputs: vec![Param {
                name: "tokenId".to_owned(),
                kind: ParamType::Uint(256),
                internal_type: None,
            }],
            outputs: vec![Param {
                name: "owner".to_owned(),
                kind: ParamType::Address,
                internal_type: None,
            }],

            constant: None,
            state_mutability: StateMutability::View,
        };

        let token_id_u256 = U256::from(token_id);
        
        // Encode calldata
        let data = function.encode_input(&[Token::Uint(token_id_u256)])?;
        
        let tx = ethers::types::TransactionRequest::new()
            .to(contract_addr)
            .data(data);

        let result = provider.call(&tx.into(), None).await?;
        
        let decoded = function.decode_output(&result)?;
        let owner = decoded.first()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from ownerOf"))?
            .clone()
            .into_address()
            .ok_or_else(|| anyhow::anyhow!("Invalid response type from ownerOf"))?;

        Ok(owner == wallet_addr)
    }

    async fn check_nft_balance(
        &self,
        provider: Provider<Http>,
        contract_addr: Address,
        wallet_addr: Address,
    ) -> Result<U256> {
        // balanceOf(address) -> uint256
        let function_selector = ethers::utils::id("balanceOf(address)");


        // simpler: use ethers generic contract/middleware if possible, or build tx manually
        // Let's use simple manual construction for reliability
        
        let mut data = function_selector[0..4].to_vec();
        // Pack address to 32 bytes
        let mut addr_bytes = [0u8; 32];
        addr_bytes[12..32].copy_from_slice(wallet_addr.as_bytes());
        data.extend_from_slice(&addr_bytes);

        let tx = ethers::types::TransactionRequest::new()
            .to(contract_addr)
            .data(data);

        let result = provider.call(&tx.into(), None).await?;
        
        if result.len() < 32 {
            return Ok(U256::zero());
        }
        
        Ok(U256::from_big_endian(&result))
    }

    async fn check_token_balance(
        &self,
        provider: Provider<Http>,
        contract_addr: Address,
        wallet_addr: Address,
    ) -> Result<U256> {
        // ERC20 balanceOf(address)
        self.check_nft_balance(provider, contract_addr, wallet_addr).await
    }

    async fn try_get_voting_power(
        &self,
        contract_addr: Address,
        wallet_addr: Address,
        provider: &Provider<Http>,
    ) -> Result<U256> {
        // Try getting votes(address) for Comp/Gov standard
        // selector for votes(address) is usually 0x9852595c or getVotes(address) 0x9ab24eb0
        
        // Try getVotes(address) (OpenZeppelin Governor) - 0x9ab24eb0
        let selector = ethers::utils::id("getVotes(address)");
        let mut data = selector[0..4].to_vec();
        let mut addr_bytes = [0u8; 32];
        addr_bytes[12..32].copy_from_slice(wallet_addr.as_bytes());
        data.extend_from_slice(&addr_bytes);

        let tx = ethers::types::TransactionRequest::new()
            .to(contract_addr)
            .data(data);

        match provider.call(&tx.into(), None).await {
            Ok(result) if result.len() >= 32 => {
                Ok(U256::from_big_endian(&result))
            }
            _ => {
                // Fallback to balanceOf(address) if not a governance token
                self.check_token_balance(provider.clone(), contract_addr, wallet_addr).await
            }
        }
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