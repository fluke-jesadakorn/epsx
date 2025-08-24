// Payment-related value objects

use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
  USD,
  USDT,
  USDC,
  ETH,
  BTC,
  BNB,
  TRX,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayStatus {
  Pending,
  Confirmed,
  Completed,
  Failed,
  Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
  Ethereum,
  Binance,
  Tron,
  Arbitrum,
  Polygon,
}

// Currency implementation
impl Currency {
  pub fn symbol(&self) -> &'static str {
    match self {
      Currency::USD => "USD",
      Currency::USDT => "USDT",
      Currency::USDC => "USDC",
      Currency::ETH => "ETH",
      Currency::BTC => "BTC",
      Currency::BNB => "BNB",
      Currency::TRX => "TRX",
    }
  }

  pub fn decimals(&self) -> u8 {
    match self {
      Currency::USD => 2,
      Currency::USDT => 6,
      Currency::USDC => 6,
      Currency::ETH => 18,
      Currency::BTC => 8,
      Currency::BNB => 18,
      Currency::TRX => 6,
    }
  }

  pub fn is_crypto(&self) -> bool {
    !matches!(self, Currency::USD)
  }

  pub fn supported_networks(&self) -> Vec<Network> {
    match self {
      Currency::USD => vec![],
      Currency::USDT =>
        vec![
          Network::Ethereum,
          Network::Binance,
          Network::Tron,
          Network::Arbitrum,
          Network::Polygon
        ],
      Currency::USDC =>
        vec![
          Network::Ethereum,
          Network::Binance,
          Network::Arbitrum,
          Network::Polygon
        ],
      Currency::ETH => vec![Network::Ethereum],
      Currency::BTC => vec![],
      Currency::BNB => vec![Network::Binance],
      Currency::TRX => vec![Network::Tron],
    }
  }
}

impl std::fmt::Display for Currency {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.symbol())
  }
}

impl std::str::FromStr for Currency {
  type Err = CurrencyError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_uppercase().as_str() {
      "USD" => Ok(Currency::USD),
      "USDT" => Ok(Currency::USDT),
      "USDC" => Ok(Currency::USDC),
      "ETH" => Ok(Currency::ETH),
      "BTC" => Ok(Currency::BTC),
      "BNB" => Ok(Currency::BNB),
      "TRX" => Ok(Currency::TRX),
      _ => Err(CurrencyError::UnsupportedCurrency(s.to_string())),
    }
  }
}

// PayStatus implementation
impl PayStatus {
  pub fn is_final(&self) -> bool {
    matches!(
      self,
      PayStatus::Completed | PayStatus::Failed | PayStatus::Cancelled
    )
  }

  pub fn is_successful(&self) -> bool {
    matches!(self, PayStatus::Completed)
  }

  pub fn can_transition_to(&self, target: &PayStatus) -> bool {
    use PayStatus::*;

    match (self, target) {
      (Pending, Confirmed) => true,
      (Pending, Failed) => true,
      (Pending, Cancelled) => true,
      (Confirmed, Completed) => true,
      (Confirmed, Failed) => true,
      _ => false,
    }
  }
}

impl std::fmt::Display for PayStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PayStatus::Pending => write!(f, "pending"),
      PayStatus::Confirmed => write!(f, "confirmed"),
      PayStatus::Completed => write!(f, "completed"),
      PayStatus::Failed => write!(f, "failed"),
      PayStatus::Cancelled => write!(f, "cancelled"),
    }
  }
}

impl std::str::FromStr for PayStatus {
  type Err = PayStatusError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "pending" => Ok(PayStatus::Pending),
      "confirmed" => Ok(PayStatus::Confirmed),
      "completed" => Ok(PayStatus::Completed),
      "failed" => Ok(PayStatus::Failed),
      "cancelled" => Ok(PayStatus::Cancelled),
      _ => Err(PayStatusError::InvalidStatus(s.to_string())),
    }
  }
}

// Network implementation
impl Network {
  pub fn name(&self) -> &'static str {
    match self {
      Network::Ethereum => "Ethereum",
      Network::Binance => "Binance Smart Chain",
      Network::Tron => "Tron",
      Network::Arbitrum => "Arbitrum",
      Network::Polygon => "Polygon",
    }
  }

  pub fn short_name(&self) -> &'static str {
    match self {
      Network::Ethereum => "ETH",
      Network::Binance => "BNB",
      Network::Tron => "TRX",
      Network::Arbitrum => "ARB",
      Network::Polygon => "MATIC",
    }
  }

  pub fn chain_id(&self) -> u64 {
    match self {
      Network::Ethereum => 1,
      Network::Binance => 56,
      Network::Tron => 728126428, // TRON mainnet
      Network::Arbitrum => 42161,
      Network::Polygon => 137,
    }
  }
}

impl std::fmt::Display for Network {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

impl std::str::FromStr for Network {
  type Err = NetworkError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "ethereum" | "eth" => Ok(Network::Ethereum),
      "binance" | "bnb" | "bsc" => Ok(Network::Binance),
      "tron" | "trx" => Ok(Network::Tron),
      "arbitrum" | "arb" => Ok(Network::Arbitrum),
      "polygon" | "matic" => Ok(Network::Polygon),
      _ => Err(NetworkError::UnsupportedNetwork(s.to_string())),
    }
  }
}

// Errors
#[derive(Debug, thiserror::Error)]
pub enum CurrencyError {
  #[error("Unsupported currency: {0}")] UnsupportedCurrency(String),
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
  #[error("Unsupported network: {0}")] UnsupportedNetwork(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PayStatusError {
  #[error("Invalid payment status: {0}")] InvalidStatus(String),
}
