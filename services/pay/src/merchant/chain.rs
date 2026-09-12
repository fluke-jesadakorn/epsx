use super::{api::Payment, bad, Result};
use crate::native_chain::{self as rpc, Error};
use alloy::{
    primitives::{keccak256, Address, B256, U256},
    sol,
    sol_types::{SolCall, SolValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, str::FromStr, time::Duration};
sol! {
    function pay(address payee,address token,uint256 amount,uint256 deadline,bytes32 salt);
    function deposit(address payee,address token,uint256 amount,uint256 deadline,bytes32 salt);
    function release(bytes32 id);
    function refund(bytes32 id);
    function dispute(bytes32 id);
    function resolve(bytes32 id,bool toPayee);
    function approve(address spender,uint256 amount);
    function transfer(address to,uint256 amount);
    function balanceOf(address owner) view returns(uint256);
    function allowance(address owner,address spender) view returns(uint256);
    function decimals() view returns(uint256);
    function VERSION() view returns(uint256);
    function FEE_BPS() view returns(uint256);
    function IS_ESCROW() view returns(bool);
    function arbiter() view returns(address);
    function treasury() view returns(address);
    function supportedTokens(address token) view returns(bool);
    function paused() view returns(bool);
    function receiver(address merchant,address token,uint256 amount,bytes32 salt) view returns(address);
    function refundTransfer(address token,uint256 amount,bytes32 salt,address payer);
    function collect(address merchant,address token,uint256 amount,bytes32 salt);
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Contract {
    pub address: Address,
    pub deployment_block: u64,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Network {
    pub environment: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub admin: Address,
    pub treasury: Address,
    pub direct: Contract,
    pub escrow: Contract,
    #[serde(default)]
    pub qr: Option<Contract>,
    pub tokens: BTreeMap<String, rpc::Token>,
    pub confirmations: u64,
}
impl Network {
    pub fn from_env() -> std::result::Result<Vec<Self>, Error> {
        let Ok(raw) = std::env::var("PAY_MERCHANT_NETWORKS") else {
            return Ok(vec![]);
        };
        let networks: Vec<Self> = serde_json::from_str(&raw)?;
        let mut environments = std::collections::HashSet::new();
        for n in &networks {
            let min = match n.chain_id {
                56 => 15,
                97 => 3,
                31337 => 1,
                _ => return Err("unsupported merchant chain".into()),
            };
            if n.environment != if n.chain_id == 56 { "live" } else { "test" }
                || !environments.insert(n.environment.clone())
                || n.confirmations < min
            {
                return Err("invalid merchant environment/confirmation policy".into());
            }
            let url = reqwest::Url::parse(&n.rpc_url)?;
            let local = url.host_str().is_some_and(|h| {
                h.trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|i| i.is_loopback())
            });
            if url.username() != ""
                || url.password().is_some()
                || url.fragment().is_some()
                || !(url.scheme() == "https" || url.scheme() == "http" && local)
            {
                return Err("RPC requires HTTPS or numeric loopback".into());
            }
            if n.direct.address.is_zero()
                || n.escrow.address.is_zero()
                || n.direct.address == n.escrow.address
                || n.admin.is_zero()
                || n.treasury.is_zero()
                || n.tokens.is_empty()
            {
                return Err("invalid contract configuration".into());
            }
            let mut addresses = std::collections::HashSet::new();
            for (symbol, t) in &n.tokens {
                let a = Address::from_str(&t.address)?;
                if !["BNB", "USDT", "USDC"].contains(&symbol.as_str())
                    || (symbol == "BNB") != a.is_zero()
                    || symbol == "BNB" && t.decimals != 18
                    || !addresses.insert(a)
                    || t.decimals > 36
                {
                    return Err("invalid token allowlist".into());
                }
            }
        }
        Ok(networks)
    }
    pub fn contract(&self, mode: &str) -> &Contract {
        if mode == "qr" {
            self.qr.as_ref().expect("QR network must be configured")
        } else if mode == "escrow" {
            &self.escrow
        } else {
            &self.direct
        }
    }
    pub async fn rpc(&self, method: &str, params: Value) -> std::result::Result<Value, Error> {
        let v: Value = reqwest::Client::builder()
            .user_agent("EPSX-Pay/1.0")
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .post(&self.rpc_url)
            .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .send()
            .await
            .map_err(reqwest::Error::without_url)?
            .error_for_status()
            .map_err(reqwest::Error::without_url)?
            .json()
            .await?;
        if v.get("error").is_some() {
            return Err("merchant RPC error".into());
        }
        v.get("result")
            .cloned()
            .ok_or_else(|| "missing RPC result".into())
    }
    pub async fn block(&self, height: u64) -> std::result::Result<Value, Error> {
        self.rpc(
            "eth_getBlockByNumber",
            json!([format!("0x{height:x}"), false]),
        )
        .await
    }
    pub async fn call(&self, c: Address, data: Vec<u8>) -> std::result::Result<Vec<u8>, Error> {
        rpc::bytes(
            &self
                .rpc(
                    "eth_call",
                    json!([{"to":c,"data":format!("0x{}",hex::encode(data))},"latest"]),
                )
                .await?,
        )
    }
    pub async fn validate(&self, mode: &str) -> std::result::Result<(), Error> {
        if rpc::number(&self.rpc("eth_chainId", json!([])).await?)? != self.chain_id {
            return Err("RPC chain mismatch".into());
        }
        let c = self.contract(mode).address;
        if self
            .rpc("eth_getCode", json!([c, "latest"]))
            .await?
            .as_str()
            == Some("0x")
        {
            return Err("contract missing".into());
        }
        for (data, expected) in [
            (
                VERSIONCall {}.abi_encode(),
                U256::from(if mode == "escrow" { 2 } else { 1 }),
            ),
            (
                FEE_BPSCall {}.abi_encode(),
                U256::from(if mode == "escrow" { 100 } else { 50 }),
            ),
        ] {
            if U256::abi_decode(&self.call(c, data).await?)? != expected {
                return Err("contract version/fee mismatch".into());
            }
        }
        for t in self.tokens.values() {
            let token = Address::from_str(&t.address)?;
            if !token.is_zero()
                && U256::abi_decode(&self.call(token, decimalsCall {}.abi_encode()).await?)?
                    != U256::from(t.decimals)
            {
                return Err("token decimals do not match the allowlist".into());
            }
        }
        if mode == "qr" {
            if Address::abi_decode(&self.call(c, treasuryCall {}.abi_encode()).await?)?
                != self.treasury
            {
                return Err("QR treasury mismatch".into());
            }
            return Ok(());
        }
        if bool::abi_decode(&self.call(c, IS_ESCROWCall {}.abi_encode()).await?)?
            != (mode == "escrow")
        {
            return Err("contract mode mismatch".into());
        }
        for (data, expected) in [
            (arbiterCall {}.abi_encode(), self.admin),
            (treasuryCall {}.abi_encode(), self.treasury),
        ] {
            if Address::abi_decode(&self.call(c, data).await?)? != expected {
                return Err("contract authority mismatch".into());
            }
        }
        for t in self.tokens.values() {
            if !bool::abi_decode(
                &self
                    .call(
                        c,
                        supportedTokensCall {
                            token: Address::from_str(&t.address)?,
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
}
pub fn fee(amount: U256, bps: u16) -> U256 {
    amount / U256::from(10000) * U256::from(bps)
        + (amount % U256::from(10000)) * U256::from(bps) / U256::from(10000)
}
pub fn payment_id(d: &Payment, payer: Address) -> Result<B256> {
    Ok(keccak256(
        (
            U256::from(d.chain_id),
            address(&d.contract_address)?,
            payer,
            address(&d.payee)?,
            address(&d.token_address)?,
            amount(&d.amount)?,
            U256::from(d.expires_at.timestamp()),
            B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
        )
            .abi_encode(),
    ))
}
pub fn address(s: &str) -> Result<Address> {
    Address::from_str(s).map_err(|_| bad("invalid_address"))
}
pub fn amount(s: &str) -> Result<U256> {
    rpc::amount(s).map_err(|_| bad("invalid_amount"))
}
pub fn input(d: &Payment, kind: &str) -> Result<Vec<u8>> {
    if d.deposit_address.is_some() {
        if kind == "collect" {
            return Ok(collectCall {
                merchant: address(&d.payee)?,
                token: address(&d.token_address)?,
                amount: amount(&d.amount)?,
                salt: B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
            }
            .abi_encode());
        }
        if kind != "refund" {
            return Err(bad("use_token_transfer"));
        }
        return Ok(refundTransferCall {
            token: address(&d.token_address)?,
            amount: amount(&d.amount)?,
            salt: B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
            payer: address(d.payer.as_deref().ok_or_else(|| bad("payer_required"))?)?,
        }
        .abi_encode());
    }
    let id = B256::from_str(
        d.on_chain_id
            .as_deref()
            .ok_or_else(|| bad("payer_required"))?,
    )
    .map_err(|_| bad("invalid_id"))?;
    Ok(match kind {
        "pay" => payCall {
            payee: address(&d.payee)?,
            token: address(&d.token_address)?,
            amount: amount(&d.amount)?,
            deadline: U256::from(d.expires_at.timestamp()),
            salt: B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
        }
        .abi_encode(),
        "deposit" => depositCall {
            payee: address(&d.payee)?,
            token: address(&d.token_address)?,
            amount: amount(&d.amount)?,
            deadline: U256::from(d.expires_at.timestamp()),
            salt: B256::from_str(&d.salt).map_err(|_| bad("invalid_salt"))?,
        }
        .abi_encode(),
        "release" => releaseCall { id }.abi_encode(),
        "refund" => refundCall { id }.abi_encode(),
        "dispute" => disputeCall { id }.abi_encode(),
        "resolve-release" => resolveCall { id, toPayee: true }.abi_encode(),
        "resolve-refund" => resolveCall { id, toPayee: false }.abi_encode(),
        _ => return Err(bad("invalid_operation")),
    })
}
pub fn parameters(d: &Payment, actor: &str, kind: &str) -> Result<(Value, Value)> {
    let funding = matches!(kind, "pay" | "deposit") || kind == "refund" && d.mode == "direct";
    let raw = amount(&d.amount)?;
    let token = address(&d.token_address)?;
    let contract = address(&d.contract_address)?;
    let params = json!({"chainId":format!("0x{:x}",d.chain_id),"from":actor,"to":contract,"data":format!("0x{}",hex::encode(input(d,kind)?)),"value":format!("{:#x}",if funding&&token.is_zero(){raw}else{U256::ZERO})});
    let approval = if funding && !token.is_zero() {
        json!({"chainId":format!("0x{:x}",d.chain_id),"from":actor,"to":token,"data":format!("0x{}",hex::encode(approveCall{spender:contract,amount:raw}.abi_encode())),"value":"0x0"})
    } else {
        Value::Null
    };
    Ok((params, approval))
}
