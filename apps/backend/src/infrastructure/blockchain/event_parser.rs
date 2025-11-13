use chrono::{DateTime, Utc};
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Parsed PaymentReceived event from smart contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub user_address: String,
    pub plan_id: u32,
    pub token_address: String,
    pub amount: Decimal,
    pub timestamp: DateTime<Utc>,
    pub payment_id: u64,
    pub transaction_hash: String,
    pub block_number: u64,
    pub log_index: u32,
}

impl PaymentEvent {
    /// Create from ethers Log
    pub fn from_log(log: &Log) -> Result<Self, EventParseError> {
        // Validate log has required topics
        if log.topics.len() < 4 {
            return Err(EventParseError::InvalidTopicCount {
                expected: 4,
                actual: log.topics.len(),
            });
        }

        // Topic 0: Event signature (keccak256("PaymentReceived(address,uint256,address,uint256,uint256,uint256)"))
        // Topic 1: user address (indexed)
        // Topic 2: plan_id (indexed)
        // Topic 3: token address (indexed)

        let user_address = format!("0x{}", hex::encode(&log.topics[1][12..]));
        let plan_id_bytes = log.topics[2].as_bytes();
        let plan_id = u32::from_be_bytes([
            plan_id_bytes[28],
            plan_id_bytes[29],
            plan_id_bytes[30],
            plan_id_bytes[31],
        ]);
        let token_address = format!("0x{}", hex::encode(&log.topics[3][12..]));

        // Parse data (non-indexed parameters: amount, timestamp, payment_id)
        if log.data.len() < 96 {
            return Err(EventParseError::InvalidDataLength {
                expected: 96,
                actual: log.data.len(),
            });
        }

        // Amount (first 32 bytes)
        let amount_bytes: [u8; 32] = log.data[0..32].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let amount_u256 = U256::from_big_endian(&amount_bytes);
        let amount = Decimal::from_str_exact(&amount_u256.to_string())
            .map_err(|_| EventParseError::DecimalConversionFailed)?
            / Decimal::from(1_000_000); // Convert from 6 decimals to standard decimal

        // Timestamp (second 32 bytes)
        let timestamp_bytes: [u8; 32] = log.data[32..64].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let timestamp_u256 = U256::from_big_endian(&timestamp_bytes);
        let timestamp_secs = timestamp_u256.as_u64() as i64;
        let timestamp = DateTime::from_timestamp(timestamp_secs, 0)
            .ok_or(EventParseError::InvalidTimestamp)?;

        // Payment ID (third 32 bytes)
        let payment_id_bytes: [u8; 32] = log.data[64..96].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let payment_id_u256 = U256::from_big_endian(&payment_id_bytes);
        let payment_id = payment_id_u256.as_u64();

        // Transaction hash and block number from log metadata
        let transaction_hash = log.transaction_hash
            .ok_or(EventParseError::MissingTransactionHash)?
            .to_string();

        let block_number = log.block_number
            .ok_or(EventParseError::MissingBlockNumber)?
            .as_u64();

        let log_index = log.log_index
            .ok_or(EventParseError::MissingLogIndex)?
            .as_u32();

        Ok(PaymentEvent {
            user_address,
            plan_id,
            token_address,
            amount,
            timestamp,
            payment_id,
            transaction_hash,
            block_number,
            log_index,
        })
    }

    /// Generate unique identifier for this event
    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.transaction_hash, self.log_index)
    }

    /// Check if event is valid and complete
    pub fn is_valid(&self) -> bool {
        !self.user_address.is_empty()
            && self.plan_id > 0
            && !self.token_address.is_empty()
            && self.amount > Decimal::ZERO
            && !self.transaction_hash.is_empty()
    }
}

/// Parse PaymentReceived event from raw log
pub fn parse_payment_event(log: &Log) -> Result<PaymentEvent, EventParseError> {
    PaymentEvent::from_log(log)
}

/// Event parsing errors
#[derive(Debug, thiserror::Error)]
pub enum EventParseError {
    #[error("Invalid topic count: expected {expected}, got {actual}")]
    InvalidTopicCount { expected: usize, actual: usize },

    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },

    #[error("Failed to convert data bytes")]
    DataConversionFailed,

    #[error("Failed to convert to decimal")]
    DecimalConversionFailed,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("Missing transaction hash")]
    MissingTransactionHash,

    #[error("Missing block number")]
    MissingBlockNumber,

    #[error("Missing log index")]
    MissingLogIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_event_unique_id() {
        let event = PaymentEvent {
            user_address: "0x1234".to_string(),
            plan_id: 1,
            token_address: "0x5678".to_string(),
            amount: Decimal::from(29),
            timestamp: Utc::now(),
            payment_id: 1,
            transaction_hash: "0xabc123".to_string(),
            block_number: 12345,
            log_index: 5,
        };

        assert_eq!(event.unique_id(), "0xabc123:5");
    }

    #[test]
    fn test_payment_event_validation() {
        let mut event = PaymentEvent {
            user_address: "0x1234".to_string(),
            plan_id: 1,
            token_address: "0x5678".to_string(),
            amount: Decimal::from(29),
            timestamp: Utc::now(),
            payment_id: 1,
            transaction_hash: "0xabc123".to_string(),
            block_number: 12345,
            log_index: 5,
        };

        assert!(event.is_valid());

        event.amount = Decimal::ZERO;
        assert!(!event.is_valid());
    }
}
