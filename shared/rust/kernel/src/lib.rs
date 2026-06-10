use alloy_primitives::{Address, U256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, KernelError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainId(pub u64);

impl ChainId {
    pub const BSC: Self = Self(56);
    pub const BSC_TESTNET: Self = Self(97);
    pub const ETH: Self = Self(1);
    pub const ARBITRUM: Self = Self(42161);
    pub const BASE: Self = Self(8453);
    pub const POLYGON: Self = Self(137);

    pub fn name(&self) -> &str {
        match self.0 {
            56 => "BSC",
            97 => "BSC Testnet",
            1 => "Ethereum",
            42161 => "Arbitrum",
            8453 => "Base",
            137 => "Polygon",
            _ => "Unknown",
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self.0 {
            56 => "https://bsc-dataseed1.binance.org",
            97 => "https://data-seed-prebsc-1-s1.binance.org:8545",
            1 => "https://eth.llamarpc.com",
            42161 => "https://arb1.arbitrum.io/rpc",
            8453 => "https://mainnet.base.org",
            137 => "https://polygon-rpc.com",
            _ => "",
        }
    }

    pub fn is_testnet(&self) -> bool {
        matches!(self.0, 97)
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ChainId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<ChainId> for u64 {
    fn from(c: ChainId) -> Self {
        c.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AddressStr(pub String);

impl AddressStr {
    pub fn new(addr: &str) -> Result<Self> {
        let addr = addr.to_lowercase();
        if !addr.starts_with("0x") || addr.len() != 42 {
            return Err(KernelError::InvalidAddress(addr));
        }
        Ok(Self(addr))
    }

    pub fn as_alloy(&self) -> Address {
        self.0.parse().unwrap_or_default()
    }
}

impl fmt::Display for AddressStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TxHash(pub String);

impl TxHash {
    pub fn new(hash: &str) -> Result<Self> {
        if !hash.starts_with("0x") || hash.len() != 66 {
            return Err(KernelError::InvalidAddress(hash.to_string()));
        }
        Ok(Self(hash.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockNumber(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    pub amount: String,
    pub decimals: u8,
    pub symbol: String,
}

impl Money {
    pub fn new(amount: &str, decimals: u8, symbol: &str) -> Self {
        Self {
            amount: amount.to_string(),
            decimals,
            symbol: symbol.to_string(),
        }
    }

    pub fn parse_amount(&self) -> Result<U256> {
        U256::from_str_radix(&self.amount, 10)
            .map_err(|_| KernelError::InvalidAmount(self.amount.clone()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Token {
    USDT,
    USDC,
    BNB,
    ETH,
}

impl Token {
    pub fn symbol(&self) -> &str {
        match self {
            Self::USDT => "USDT",
            Self::USDC => "USDC",
            Self::BNB => "BNB",
            Self::ETH => "ETH",
        }
    }

    pub fn decimals(&self) -> u8 {
        match self {
            Self::USDT | Self::USDC => 18,
            Self::BNB | Self::ETH => 18,
        }
    }

    pub fn address(&self, chain_id: ChainId) -> Option<AddressStr> {
        match (self, chain_id.0) {
            (Self::USDT, 56) => AddressStr::new("0x55d398326f99059fF775485246999027B3197955").ok(),
            (Self::USDC, 56) => AddressStr::new("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d").ok(),
            (Self::USDT, 97) => AddressStr::new("0x7ef863a75a2d596b4d54d4d04a0e1d2b2b3a3d3d").ok(),
            (Self::USDC, 97) => AddressStr::new("0x64544969ed70487f6cef3fe96a9bbe2037f41ac0").ok(),
            _ => None,
        }
    }

    pub fn for_chain(chain_id: u64) -> Vec<Token> {
        match chain_id {
            56 | 97 => vec![Token::BNB, Token::USDT, Token::USDC],
            _ => vec![],
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentStatus {
    Created,
    Pending,
    Confirmed,
    Failed,
    Cancelled,
    Expired,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Pending => write!(f, "pending"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscrowStatus {
    Funded,
    Released,
    Disputed,
    Resolved,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Paused,
    Cancelled,
    Expired,
    PastDue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Editor,
    ContentManager,
    Merchant,
    User,
}

impl UserRole {
    pub fn can_edit_content(&self) -> bool {
        matches!(self, Self::Admin | Self::Editor | Self::ContentManager)
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_manage_payments(&self) -> bool {
        matches!(self, Self::Admin | Self::Merchant)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self { page: 1, page_size: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_name() {
        assert_eq!(ChainId(56).name(), "BSC");
        assert_eq!(ChainId(97).name(), "BSC Testnet");
        assert_eq!(ChainId(1).name(), "Ethereum");
    }

    #[test]
    fn test_token_address() {
        let usdt = Token::USDT.address(ChainId(56)).unwrap();
        assert_eq!(usdt.0.to_lowercase(), "0x55d398326f99059ff775485246999027b3197955");
    }

    #[test]
    fn test_token_for_chain() {
        assert_eq!(Token::for_chain(56), vec![Token::BNB, Token::USDT, Token::USDC]);
        assert_eq!(Token::for_chain(1), Vec::<Token>::new());
    }

    #[test]
    fn test_user_role_permissions() {
        assert!(UserRole::Admin.can_admin());
        assert!(UserRole::Editor.can_edit_content());
        assert!(UserRole::Merchant.can_manage_payments());
        assert!(!UserRole::User.can_admin());
    }

    #[test]
    fn test_money_new() {
        let m = Money::new("1000000", 18, "USDC");
        assert_eq!(m.amount, "1000000");
        assert_eq!(m.symbol, "USDC");
    }

    #[test]
    fn test_address_str_validation() {
        assert!(AddressStr::new("0x55d398326f99059fF775485246999027B3197955").is_ok());
        assert!(AddressStr::new("not-an-address").is_err());
    }

    #[test]
    fn test_payment_status() {
        assert_eq!(PaymentStatus::Pending.to_string(), "pending");
        assert_eq!(PaymentStatus::Confirmed.to_string(), "confirmed");
    }

    #[test]
    fn test_escrow_status() {
        assert_eq!(EscrowStatus::Released, EscrowStatus::Released);
        assert_ne!(EscrowStatus::Funded, EscrowStatus::Released);
    }
}
