use alloy::providers::utils::Eip1559Estimation;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::rpc::types::{Filter, Log};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolCall;
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

sol! {
    interface IERC20 {
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function symbol() external view returns (string);
    }

    interface IERC20Events {
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}

pub fn bsc_provider() -> Result<Box<dyn Provider + Send + Sync>> {
    let url: reqwest::Url = "https://bsc-dataseed1.binance.org".parse().map_err(|e: url::ParseError| Web3Error::Provider(e.to_string()))?;
    Ok(Box::new(ProviderBuilder::new().connect_http(url)))
}

pub fn bsc_testnet_provider() -> Result<Box<dyn Provider + Send + Sync>> {
    let url: reqwest::Url = "https://data-seed-prebsc-1-s1.binance.org:8545".parse().map_err(|e: url::ParseError| Web3Error::Provider(e.to_string()))?;
    Ok(Box::new(ProviderBuilder::new().connect_http(url)))
}

pub fn provider_for_url(url: &str) -> Result<Box<dyn Provider + Send + Sync>> {
    let parsed: reqwest::Url = url.parse().map_err(|e: url::ParseError| Web3Error::Provider(e.to_string()))?;
    Ok(Box::new(ProviderBuilder::new().connect_http(parsed)))
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

pub async fn fetch_block_full<P: Provider + ?Sized>(provider: &P, number: u64) -> Result<Option<alloy_rpc_types::Block>> {
    provider.get_block_by_number(alloy_rpc_types::BlockNumberOrTag::Number(number))
        .full()
        .await
        .map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn fetch_tx_receipt<P: Provider + ?Sized>(provider: &P, hash: B256) -> Result<Option<alloy_rpc_types::TransactionReceipt>> {
    provider.get_transaction_receipt(hash).await.map_err(|e| Web3Error::Provider(e.to_string()))
}

pub async fn send_raw_transaction<P: Provider + ?Sized>(provider: &P, raw: Bytes) -> Result<B256> {
    let pending = provider.send_raw_transaction(&raw).await.map_err(|e| Web3Error::Transaction(e.to_string()))?;
    Ok(*pending.tx_hash())
}

pub async fn fetch_token_balance<P: Provider + ?Sized>(provider: &P, token: Address, holder: Address) -> Result<U256> {
    let call = IERC20::balanceOfCall { account: holder };
    let req = TransactionRequest::default()
        .to(token)
        .input(Bytes::from(call.abi_encode()).into());
    let result = provider.call(req).await
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    let decoded = IERC20::balanceOfCall::abi_decode_returns(&result)
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    Ok(decoded)
}

pub async fn fetch_token_decimals<P: Provider + ?Sized>(provider: &P, token: Address) -> Result<u8> {
    let call = IERC20::decimalsCall {};
    let req = TransactionRequest::default()
        .to(token)
        .input(Bytes::from(call.abi_encode()).into());
    let result = provider.call(req).await
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    let decoded = IERC20::decimalsCall::abi_decode_returns(&result)
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    Ok(decoded)
}

pub async fn fetch_token_symbol<P: Provider + ?Sized>(provider: &P, token: Address) -> Result<String> {
    let call = IERC20::symbolCall {};
    let req = TransactionRequest::default()
        .to(token)
        .input(Bytes::from(call.abi_encode()).into());
    let result = provider.call(req).await
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    let decoded = IERC20::symbolCall::abi_decode_returns(&result)
        .map_err(|e| Web3Error::Contract(e.to_string()))?;
    Ok(decoded)
}

pub async fn fetch_logs<P: Provider + ?Sized>(provider: &P, filter: Filter) -> Result<Vec<Log>> {
    provider.get_logs(&filter).await.map_err(|e| Web3Error::Provider(e.to_string()))
}
