use alloy::providers::utils::Eip1559Estimation;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy_primitives::{Address, U256};
use epsx_kernel::{ChainId, Token};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Web3Error {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Contract error: {0}")]
    Contract(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("Signer error: {0}")]
    Signer(String),
}

pub type Result<T> = std::result::Result<T, Web3Error>;

pub fn bsc_provider() -> Result<Box<dyn Provider + Send + Sync>> {
    let url: reqwest::Url = "https://bsc-dataseed1.binance.org".parse().map_err(|e: url::ParseError| Web3Error::Provider(e.to_string()))?;
    Ok(Box::new(ProviderBuilder::new().connect_http(url)))
}

pub fn bsc_testnet_provider() -> Result<Box<dyn Provider + Send + Sync>> {
    let url: reqwest::Url = "https://data-seed-prebsc-1-s1.binance.org:8545".parse().map_err(|e: url::ParseError| Web3Error::Provider(e.to_string()))?;
    Ok(Box::new(ProviderBuilder::new().connect_http(url)))
}

pub fn provider_for_chain(chain_id: ChainId) -> Result<Box<dyn Provider + Send + Sync>> {
    match chain_id.0 {
        56 => bsc_provider(),
        97 => bsc_testnet_provider(),
        _ => Err(Web3Error::Provider(format!("Chain {} not supported", chain_id))),
    }
}

pub fn signer_from_private_key(pk: &str) -> Result<PrivateKeySigner> {
    PrivateKeySigner::from_str(pk).map_err(|e| Web3Error::Signer(e.to_string()))
}

pub fn usdt_address(chain_id: ChainId) -> Option<Address> {
    Token::USDT.address(chain_id).and_then(|a| Address::from_str(&a.0).ok())
}

pub fn usdc_address(chain_id: ChainId) -> Option<Address> {
    Token::USDC.address(chain_id).and_then(|a| Address::from_str(&a.0).ok())
}

pub fn parse_ether(value: &str) -> Result<U256> {
    U256::from_str_radix(value, 10).map_err(|e| Web3Error::Contract(e.to_string()))
}

pub fn format_ether(value: U256) -> String {
    value.to_string()
}

pub async fn fetch_eth_balance<P: Provider + ?Sized>(provider: &P, address: Address) -> Result<U256> {
    provider.get_balance(address).await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_chain_id<P: Provider + ?Sized>(provider: &P) -> Result<u64> {
    provider.get_chain_id().await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_gas_price<P: Provider + ?Sized>(provider: &P) -> Result<u128> {
    provider.get_gas_price().await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_nonce<P: Provider + ?Sized>(provider: &P, address: Address) -> Result<u64> {
    provider.get_transaction_count(address).await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_block_number<P: Provider + ?Sized>(provider: &P) -> Result<u64> {
    provider.get_block_number().await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_balance<P: Provider + ?Sized>(provider: &P, address: Address) -> Result<U256> {
    provider.get_balance(address).await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn estimate_eip1559<P: Provider + ?Sized>(provider: &P) -> Result<Eip1559Estimation> {
    provider.estimate_eip1559_fees().await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_block<P: Provider + ?Sized>(provider: &P, number: u64) -> Result<Option<alloy_rpc_types::Block>> {
    provider.get_block_by_number(alloy_rpc_types::BlockNumberOrTag::Number(number)).await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_tx_receipt<P: Provider + ?Sized>(provider: &P, hash: alloy_primitives::B256) -> Result<Option<alloy_rpc_types::TransactionReceipt>> {
    provider.get_transaction_receipt(hash).await.map_err(|e| Web3Error::Provider(e.to_string()))
}
