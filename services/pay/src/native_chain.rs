//! RPC is evidence, never a signing authority. Finalization uses canonical, confirmed receipts.
use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol,
    sol_types::{SolCall, SolValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, str::FromStr, time::Duration};
sol! {
    function deposit(bytes32 salt,address payee,address token,uint256 amount);
    function release(bytes32 id);
    function refund(bytes32 id);
    function dispute(bytes32 id);
    function resolve(bytes32 id,bool toPayee);
    function arbiter() view returns(address);
    function treasury() view returns(address);
    function FEE_BPS() view returns(uint256);
    function decimals() view returns(uint256);
    function VERSION() view returns(uint256);
    function supportedTokens(address token) view returns(bool);
    function setPaused(bool paused_);
    function paused() view returns(bool);
    function approve(address spender,uint256 amount) returns(bool);
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub decimals: u8,
}
#[derive(Clone)]
pub struct Chain {
    pub chain_id: u64,
    pub contract: Address,
    pub admin: Address,
    pub treasury: Address,
    pub tokens: BTreeMap<String, Token>,
    pub confirmations: u64,
    pub deployment_block: u64,
    pub scan_blocks: u64,
    rpc_url: String,
    archive_rpc_url: Option<String>,
    client: reqwest::Client,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// Inclusive log range compatible with the configured free RPC tier.
pub const LOG_SCAN_BLOCKS: u64 = 10;
pub fn default_scan_blocks() -> u64 {
    LOG_SCAN_BLOCKS
}
pub fn validate_scan_blocks(blocks: u64) -> Result<(), Error> {
    if !(1..=50).contains(&blocks) {
        return Err("scan block limit must be between 1 and 50".into());
    }
    Ok(())
}
pub fn address(value: &str) -> Result<Address, Error> {
    Ok(Address::from_str(value)?)
}
pub fn amount(value: &str) -> Result<U256, Error> {
    if value.is_empty()
        || value.starts_with('0')
        || value.len() > 78
        || !value.bytes().all(|c| c.is_ascii_digit())
    {
        return Err("amount must be a positive canonical integer in token base units".into());
    }
    Ok(U256::from_str_radix(value, 10)?)
}
pub fn fee(amount: U256) -> U256 {
    let d = U256::from(10000);
    let n = U256::from(30);
    (amount / d) * n + ((amount % d) * n) / d
}
pub fn number(value: &Value) -> Result<u64, Error> {
    Ok(u64::from_str_radix(
        value
            .as_str()
            .ok_or("missing RPC number")?
            .strip_prefix("0x")
            .ok_or("invalid RPC number")?,
        16,
    )?)
}
pub fn bytes(value: &Value) -> Result<Vec<u8>, Error> {
    Ok(hex::decode(
        value
            .as_str()
            .ok_or("missing RPC bytes")?
            .strip_prefix("0x")
            .ok_or("invalid RPC bytes")?,
    )?)
}
impl Chain {
    pub fn from_env() -> Result<Option<Self>, Error> {
        let Ok(contract) = std::env::var("PAY_ESCROW_V1_CONTRACT") else {
            return Ok(None);
        };
        let chain_id = std::env::var("CHAIN_ID")?.parse()?;
        if ![56, 97, 31337].contains(&chain_id) {
            return Err("only BSC mainnet, testnet and local Anvil are supported".into());
        }
        let contract = address(&contract)?;
        let admin = address(&std::env::var("PAY_ESCROW_ADMIN")?)?;
        let treasury = std::env::var("PAY_ESCROW_TREASURY")
            .ok()
            .map(|value| address(&value))
            .transpose()?
            .unwrap_or(admin);
        if contract.is_zero() || admin.is_zero() || treasury.is_zero() {
            return Err("zero contract or admin address".into());
        }
        let tokens: BTreeMap<String, Token> =
            serde_json::from_str(&std::env::var("PAY_ESCROW_TOKENS")?)?;
        if tokens.is_empty() {
            return Err("token allowlist required".into());
        }
        for (symbol, token) in &tokens {
            let addr = address(&token.address)?;
            if !["BNB", "USDT", "USDC"].contains(&symbol.as_str())
                || token.decimals > 36
                || (symbol == "BNB") != addr.is_zero()
                || symbol == "BNB" && token.decimals != 18
            {
                return Err("invalid token allowlist".into());
            }
        }
        let rpc_url = std::env::var("PAY_ESCROW_RPC_URL")?;
        let url = reqwest::Url::parse(&rpc_url)?;
        let local = url.host_str().is_some_and(|h| {
            h.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
        });
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.scheme() != "https" && !(url.scheme() == "http" && local)
        {
            return Err("RPC must use HTTPS or loopback HTTP".into());
        }
        let minimum = match chain_id {
            56 => 15,
            97 => 3,
            _ => 1,
        };
        let confirmations = std::env::var("PAY_ESCROW_CONFIRMATIONS")
            .ok()
            .map(|s| s.parse())
            .transpose()?
            .unwrap_or(minimum);
        if confirmations < minimum {
            return Err("confirmations below chain policy".into());
        }
        let archive_rpc_url = std::env::var("PAY_ESCROW_ARCHIVE_RPC_URL")
            .ok()
            .filter(|value| !value.is_empty());
        if let Some(url) = &archive_rpc_url {
            crate::rpc_transport::validate_endpoint(url)?;
        }
        let scan_blocks = std::env::var("PAY_ESCROW_SCAN_BLOCKS")
            .ok()
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or(LOG_SCAN_BLOCKS);
        validate_scan_blocks(scan_blocks)?;
        Ok(Some(Self {
            chain_id,
            contract,
            admin,
            treasury,
            tokens,
            confirmations,
            deployment_block: std::env::var("PAY_ESCROW_DEPLOYMENT_BLOCK")?.parse()?,
            rpc_url,
            archive_rpc_url,
            scan_blocks,
            client: reqwest::Client::builder()
                .user_agent("EPSX-Pay/1.0")
                .timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        }))
    }
    pub fn id(&self, payer: Address, salt: B256) -> B256 {
        keccak256((U256::from(self.chain_id), self.contract, payer, salt).abi_encode())
    }
    pub async fn rpc(&self, method: &str, params: Value) -> Result<Value, Error> {
        crate::rpc_transport::read(
            &self.client,
            &self.rpc_url,
            self.archive_rpc_url.as_deref(),
            self.chain_id,
            method,
            params,
        )
        .await
    }
    pub async fn deposits_paused(&self) -> Result<bool, Error> {
        Ok(bool::abi_decode(
            &self.call(pausedCall {}.abi_encode()).await?,
        )?)
    }
    pub async fn validate(&self) -> Result<(), Error> {
        if number(&self.rpc("eth_chainId", json!([])).await?)? != self.chain_id {
            return Err("RPC chain mismatch".into());
        }
        if bytes(
            &self
                .rpc("eth_getCode", json!([self.contract, "latest"]))
                .await?,
        )?
        .is_empty()
        {
            return Err("escrow contract missing".into());
        }
        let version = self.call(VERSIONCall {}.abi_encode()).await?;
        if U256::abi_decode(&version)? != U256::from(1) {
            return Err("escrow version mismatch".into());
        }
        if Address::abi_decode(&self.call(arbiterCall {}.abi_encode()).await?)? != self.admin {
            return Err("escrow admin mismatch".into());
        }
        if Address::abi_decode(&self.call(treasuryCall {}.abi_encode()).await?)? != self.treasury {
            return Err("escrow treasury mismatch".into());
        }
        if U256::abi_decode(&self.call(FEE_BPSCall {}.abi_encode()).await?)? != U256::from(30) {
            return Err("escrow fee mismatch".into());
        }
        for token in self.tokens.values() {
            let token_address = address(&token.address)?;
            if !token_address.is_zero() {
                let result = self.rpc("eth_call",json!([{"to":token_address,"data":format!("0x{}",hex::encode(decimalsCall {}.abi_encode()))},"latest"])).await?;
                if U256::abi_decode(&bytes(&result)?)? != U256::from(token.decimals) {
                    return Err("token decimals do not match the allowlist".into());
                }
            }
            if !bool::abi_decode(
                &self
                    .call(
                        supportedTokensCall {
                            token: address(&token.address)?,
                        }
                        .abi_encode(),
                    )
                    .await?,
            )? {
                return Err("token not allowed by contract".into());
            }
        }
        Ok(())
    }
    pub async fn call(&self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        bytes(
            &self
                .rpc(
                    "eth_call",
                    json!([{"to":self.contract,"data":format!("0x{}",hex::encode(data))},"latest"]),
                )
                .await?,
        )
    }
    pub async fn block_hash(&self, height: u64) -> Result<String, Error> {
        self.rpc(
            "eth_getBlockByNumber",
            json!([format!("0x{height:x}"), false]),
        )
        .await?
        .get("hash")
        .and_then(Value::as_str)
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "block unavailable".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_validation_and_fee_boundaries() {
        for value in ["0", "01", "-1", "0x10", "1.5", ""] {
            assert!(amount(value).is_err())
        }
        assert_eq!(fee(amount("10000").unwrap()), U256::from(30));
        assert_eq!(
            fee(U256::MAX),
            U256::MAX / U256::from(10000) * U256::from(30)
                + (U256::MAX % U256::from(10000) * U256::from(30)) / U256::from(10000)
        );
    }
}
