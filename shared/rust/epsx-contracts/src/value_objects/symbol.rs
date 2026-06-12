// kernel extraction wave9 — moved from apps/backend/src/domain/shared_kernel/value_objects/symbol.rs
// Import-path adjustments:
//   * `crate::core::errors::{AppError, AppResult}` → `crate::errors::{AppError, AppResult}`
//   * `crate::domain::shared_kernel::ValueObject` → `crate::ValueObject`
//     (re-exported at the crate root, mirroring the old shared_kernel re-export).
use serde::{ Deserialize, Serialize };
use crate::errors::{AppError, AppResult};
use crate::ValueObject;

/// Stock symbol value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
  value: String,
}

impl Symbol {
  pub fn new(value: impl Into<String>) -> AppResult<Self> {
    let value = value.into().trim().to_uppercase();

    if value.is_empty() {
      return Err(
        AppError::validation_error("Symbol cannot be empty".to_string())
      );
    }

    if value.len() > 10 {
      return Err(
        AppError::validation_error("Symbol cannot be longer than 10 characters".to_string())
      );
    }

    // Basic validation for stock symbol format
    if
      !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
      return Err(
        AppError::validation_error("Symbol contains invalid characters".to_string())
      );
    }

    Ok(Self { value })
  }

  pub fn as_str(&self) -> &str {
    &self.value
  }
}

impl ValueObject for Symbol {
  type Error = AppError;

  fn validate(&self) -> Result<(), Self::Error> {
    if self.value.is_empty() {
      return Err(
        AppError::validation_error("Symbol cannot be empty".to_string())
      );
    }
    Ok(())
  }
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.value)
  }
}

impl From<Symbol> for String {
  fn from(symbol: Symbol) -> Self {
    symbol.value
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_symbol() {
    let symbol = Symbol::new("AAPL").unwrap();
    assert_eq!(symbol.as_str(), "AAPL");
  }

  #[test]
  fn test_symbol_case_normalization() {
    let symbol = Symbol::new("aapl").unwrap();
    assert_eq!(symbol.as_str(), "AAPL");
  }

  #[test]
  fn test_invalid_empty_symbol() {
    assert!(Symbol::new("").is_err());
  }

  #[test]
  fn test_invalid_long_symbol() {
    assert!(Symbol::new("VERYLONGSYMBOL").is_err());
  }
}
