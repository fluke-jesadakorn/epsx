// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/market.rs
// Import-path adjustments: see symbol.rs (same pattern).
use serde::{ Deserialize, Serialize };
use crate::errors::{AppError, AppResult};
use crate::ValueObject;

/// Market/Exchange enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Market {
  NYSE,
  NASDAQ,
  AMEX,
  LSE, // London Stock Exchange
  TSE, // Tokyo Stock Exchange
  HKSE, // Hong Kong Stock Exchange
  SSE, // Shanghai Stock Exchange
  SZSE, // Shenzhen Stock Exchange
  BSE, // Bombay Stock Exchange
  NSE, // National Stock Exchange of India
  TSX, // Toronto Stock Exchange
  ASX, // Australian Securities Exchange
  Other(String),
}

impl Market {
  pub fn new(market: impl Into<String>) -> AppResult<Self> {
    let market_str = market.into().trim().to_uppercase();

    match market_str.as_str() {
      "NYSE" | "NEW_YORK_STOCK_EXCHANGE" => Ok(Market::NYSE),
      "NASDAQ" | "NASDAQ_COMPOSITE" => Ok(Market::NASDAQ),
      "AMEX" | "AMERICAN_STOCK_EXCHANGE" => Ok(Market::AMEX),
      "LSE" | "LONDON_STOCK_EXCHANGE" => Ok(Market::LSE),
      "TSE" | "TOKYO_STOCK_EXCHANGE" => Ok(Market::TSE),
      "HKSE" | "HONG_KONG_STOCK_EXCHANGE" => Ok(Market::HKSE),
      "SSE" | "SHANGHAI_STOCK_EXCHANGE" => Ok(Market::SSE),
      "SZSE" | "SHENZHEN_STOCK_EXCHANGE" => Ok(Market::SZSE),
      "BSE" | "BOMBAY_STOCK_EXCHANGE" => Ok(Market::BSE),
      "NSE" | "NATIONAL_STOCK_EXCHANGE" => Ok(Market::NSE),
      "TSX" | "TORONTO_STOCK_EXCHANGE" => Ok(Market::TSX),
      "ASX" | "AUSTRALIAN_SECURITIES_EXCHANGE" => Ok(Market::ASX),
      _ => {
        if market_str.is_empty() {
          Err(
            AppError::validation_error("Market cannot be empty".to_string())
          )
        } else {
          Ok(Market::Other(market_str))
        }
      }
    }
  }

  pub fn code(&self) -> &str {
    match self {
      Market::NYSE => "NYSE",
      Market::NASDAQ => "NASDAQ",
      Market::AMEX => "AMEX",
      Market::LSE => "LSE",
      Market::TSE => "TSE",
      Market::HKSE => "HKSE",
      Market::SSE => "SSE",
      Market::SZSE => "SZSE",
      Market::BSE => "BSE",
      Market::NSE => "NSE",
      Market::TSX => "TSX",
      Market::ASX => "ASX",
      Market::Other(code) => code,
    }
  }

  pub fn name(&self) -> &str {
    match self {
      Market::NYSE => "New York Stock Exchange",
      Market::NASDAQ => "NASDAQ",
      Market::AMEX => "American Stock Exchange",
      Market::LSE => "London Stock Exchange",
      Market::TSE => "Tokyo Stock Exchange",
      Market::HKSE => "Hong Kong Stock Exchange",
      Market::SSE => "Shanghai Stock Exchange",
      Market::SZSE => "Shenzhen Stock Exchange",
      Market::BSE => "Bombay Stock Exchange",
      Market::NSE => "National Stock Exchange of India",
      Market::TSX => "Toronto Stock Exchange",
      Market::ASX => "Australian Securities Exchange",
      Market::Other(name) => name,
    }
  }
}

impl ValueObject for Market {
  type Error = AppError;

  fn validate(&self) -> Result<(), Self::Error> {
    match self {
      Market::Other(name) if name.is_empty() => {
        Err(
          AppError::validation_error("Market name cannot be empty".to_string())
        )
      }
      _ => Ok(()),
    }
  }
}

impl std::fmt::Display for Market {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.code())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_known_markets() {
    let nasdaq = Market::new("NASDAQ").unwrap();
    assert_eq!(nasdaq, Market::NASDAQ);
    assert_eq!(nasdaq.code(), "NASDAQ");
  }

  #[test]
  fn test_case_insensitive() {
    let nyse = Market::new("nyse").unwrap();
    assert_eq!(nyse, Market::NYSE);
  }

  #[test]
  fn test_other_market() {
    let custom = Market::new("CUSTOM_EXCHANGE").unwrap();
    assert_eq!(custom, Market::Other("CUSTOM_EXCHANGE".to_string()));
  }

  #[test]
  fn test_empty_market() {
    assert!(Market::new("").is_err());
  }
}
