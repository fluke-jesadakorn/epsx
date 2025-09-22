use serde::{ Deserialize, Serialize };
use crate::domain::shared_kernel::{ ValueObject, DomainError, DomainResult };

/// Stock symbol value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
  value: String,
}

impl Symbol {
  pub fn new(value: impl Into<String>) -> DomainResult<Self> {
    let value = value.into().trim().to_uppercase();

    if value.is_empty() {
      return Err(
        DomainError::validation_error(
          "Symbol cannot be empty".to_string(),
          "Symbol"
        )
      );
    }

    if value.len() > 10 {
      return Err(
        DomainError::validation_error(
          "Symbol cannot be longer than 10 characters".to_string(),
          "Symbol"
        )
      );
    }

    // Basic validation for stock symbol format
    if
      !value
        .chars()
        .all(|c| (c.is_ascii_alphanumeric() || c == '.' || c == '-'))
    {
      return Err(
        DomainError::validation_error(
          "Symbol contains invalid characters".to_string(),
          "Symbol"
        )
      );
    }

    Ok(Self { value })
  }

  pub fn as_str(&self) -> &str {
    &self.value
  }

  pub fn to_string(&self) -> String {
    self.value.clone()
  }
}

impl ValueObject for Symbol {
  type Error = crate::domain::shared_kernel::DomainError;

  fn validate(&self) -> Result<(), Self::Error> {
    if self.value.is_empty() {
      return Err(
        crate::domain::shared_kernel::DomainError::validation_error(
          "Symbol cannot be empty".to_string(),
          "Symbol"
        )
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
