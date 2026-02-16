use chrono::{DateTime, Utc};
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// Re-export from domain for convenience
pub use crate::domain::payment::PaymentContextType;

/// Parsed PaymentWithContext event from smart contract V2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub user_address: String,
    pub context_type: u8,
    pub context_id: u64,
    pub token_address: String,
    pub amount: Decimal,
    pub timestamp: DateTime<Utc>,
    pub payment_id: u64,
    pub link_hash: String,
    pub transaction_hash: String,
    pub block_number: u64,
    pub log_index: u32,
}

impl PaymentEvent {
    /// Create from ethers Log (V2: PaymentWithContext)
    ///
    /// Event signature:
    /// PaymentWithContext(address indexed user, uint8 indexed contextType, uint256 indexed contextId,
    ///                    address token, uint256 amount, uint256 timestamp, uint256 paymentId, bytes32 linkHash)
    ///
    /// Topics: [event_sig, user, contextType, contextId]
    /// Data:   [token(32), amount(32), timestamp(32), paymentId(32), linkHash(32)] = 160 bytes
    pub fn from_log(log: &Log) -> Result<Self, EventParseError> {
        if log.topics.len() < 4 {
            return Err(EventParseError::InvalidTopicCount {
                expected: 4,
                actual: log.topics.len(),
            });
        }

        // Topic 1: user address (indexed)
        let user_address = format!("0x{}", hex::encode(&log.topics[1][12..]));

        // Topic 2: contextType (indexed, uint8 stored as bytes32)
        let context_type = log.topics[2].as_bytes()[31];

        // Topic 3: contextId (indexed, uint256)
        let context_id_bytes = log.topics[3].as_bytes();
        let context_id = U256::from_big_endian(context_id_bytes).as_u64();

        // Parse data: [token(32), amount(32), timestamp(32), paymentId(32), linkHash(32)]
        if log.data.len() < 160 {
            return Err(EventParseError::InvalidDataLength {
                expected: 160,
                actual: log.data.len(),
            });
        }

        // Token address (first 32 bytes, address in last 20 bytes)
        let token_address = format!("0x{}", hex::encode(&log.data[12..32]));

        // Amount (bytes 32..64) - BSC USDT/USDC use 18 decimals
        let amount_bytes: [u8; 32] = log.data[32..64].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let amount_u256 = U256::from_big_endian(&amount_bytes);
        let amount = Decimal::from_str_exact(&amount_u256.to_string())
            .map_err(|_| EventParseError::DecimalConversionFailed)?
            / Decimal::from_str_exact("1000000000000000000")
                .map_err(|_| EventParseError::DecimalConversionFailed)?; // 10^18

        // Timestamp (bytes 64..96)
        let timestamp_bytes: [u8; 32] = log.data[64..96].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let timestamp_u256 = U256::from_big_endian(&timestamp_bytes);
        let timestamp_secs = timestamp_u256.as_u64() as i64;
        let timestamp = DateTime::from_timestamp(timestamp_secs, 0)
            .ok_or(EventParseError::InvalidTimestamp)?;

        // Payment ID (bytes 96..128)
        let payment_id_bytes: [u8; 32] = log.data[96..128].try_into()
            .map_err(|_| EventParseError::DataConversionFailed)?;
        let payment_id = U256::from_big_endian(&payment_id_bytes).as_u64();

        // Link hash (bytes 128..160)
        let link_hash = format!("0x{}", hex::encode(&log.data[128..160]));

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
            context_type,
            context_id,
            token_address,
            amount,
            timestamp,
            payment_id,
            link_hash,
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
            && self.context_type <= 4
            && !self.token_address.is_empty()
            && self.amount > Decimal::ZERO
            && !self.transaction_hash.is_empty()
    }

    /// Get parsed context type
    pub fn parsed_context_type(&self) -> Option<PaymentContextType> {
        PaymentContextType::from_u8(self.context_type)
    }

    /// Check if this is a plan payment
    pub fn is_plan_payment(&self) -> bool {
        self.context_type == PaymentContextType::Plan as u8
    }
}

/// Parse PaymentWithContext event from raw log
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
            context_type: 0,
            context_id: 1,
            token_address: "0x5678".to_string(),
            amount: Decimal::from(29),
            timestamp: Utc::now(),
            payment_id: 1,
            link_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
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
            context_type: 0,
            context_id: 1,
            token_address: "0x5678".to_string(),
            amount: Decimal::from(29),
            timestamp: Utc::now(),
            payment_id: 1,
            link_hash: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transaction_hash: "0xabc123".to_string(),
            block_number: 12345,
            log_index: 5,
        };

        assert!(event.is_valid());
        assert!(event.is_plan_payment());
        assert_eq!(event.parsed_context_type(), Some(PaymentContextType::Plan));

        event.amount = Decimal::ZERO;
        assert!(!event.is_valid());
    }
}
