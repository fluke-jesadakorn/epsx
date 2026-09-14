use alloy::primitives::{Address, B256, U256};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use super::super::ReceiptOutcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CodecError(String);

impl Display for CodecError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CodecError {}

pub(super) fn encode_b256(value: B256) -> Vec<u8> {
    value.as_slice().to_vec()
}

pub(super) fn encode_address(value: Address) -> Vec<u8> {
    value.as_slice().to_vec()
}

pub(super) fn decode_b256(field: &str, value: Vec<u8>) -> Result<B256, CodecError> {
    if value.len() != 32 {
        return Err(CodecError(format!(
            "{field} must contain exactly 32 bytes, found {}",
            value.len()
        )));
    }
    Ok(B256::from_slice(&value))
}

pub(super) fn decode_address(field: &str, value: Vec<u8>) -> Result<Address, CodecError> {
    if value.len() != 20 {
        return Err(CodecError(format!(
            "{field} must contain exactly 20 bytes, found {}",
            value.len()
        )));
    }
    Ok(Address::from_slice(&value))
}

pub(super) fn decode_nonnegative_i64(field: &str, value: i64) -> Result<u64, CodecError> {
    u64::try_from(value)
        .map_err(|_| CodecError(format!("{field} must be non-negative, found {value}")))
}

pub(super) fn decode_nonnegative_i32(field: &str, value: i32) -> Result<u64, CodecError> {
    u64::try_from(value)
        .map_err(|_| CodecError(format!("{field} must be non-negative, found {value}")))
}

pub(super) fn decode_timestamp_seconds(
    field: &str,
    value: DateTime<Utc>,
) -> Result<u64, CodecError> {
    if value.timestamp_subsec_nanos() != 0 {
        return Err(CodecError(format!(
            "{field} must have whole-second precision"
        )));
    }
    u64::try_from(value.timestamp())
        .map_err(|_| CodecError(format!("{field} is earlier than the Unix epoch")))
}

pub(super) fn encode_u256_decimal(value: U256) -> String {
    value.to_string()
}

pub(super) fn decode_u256_decimal(field: &str, value: &str) -> Result<U256, CodecError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CodecError(format!(
            "{field} must be an unsigned base-10 integer, found {value:?}"
        )));
    }
    U256::from_str_radix(value, 10)
        .map_err(|_| CodecError(format!("{field} exceeds the unsigned 256-bit range")))
}

pub(super) fn encode_receipt_outcome(outcome: &ReceiptOutcome) -> (i16, Option<Vec<u8>>) {
    match outcome {
        ReceiptOutcome::Reverted => (0, None),
        ReceiptOutcome::Succeeded => (1, None),
        ReceiptOutcome::PostStateRoot(root) => (2, Some(encode_b256(*root))),
    }
}

pub(super) fn decode_receipt_outcome(
    outcome: i16,
    post_state_root: Option<Vec<u8>>,
) -> Result<ReceiptOutcome, CodecError> {
    match (outcome, post_state_root) {
        (0, None) => Ok(ReceiptOutcome::Reverted),
        (1, None) => Ok(ReceiptOutcome::Succeeded),
        (2, Some(root)) => Ok(ReceiptOutcome::PostStateRoot(decode_b256(
            "receipt.post_state_root",
            root,
        )?)),
        (0 | 1, Some(_)) => Err(CodecError(
            "receipt outcome without a state root stored one".to_string(),
        )),
        (2, None) => Err(CodecError(
            "post-state receipt is missing its state root".to_string(),
        )),
        (value, _) => Err(CodecError(format!(
            "receipt outcome is outside 0..=2: {value}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u256_decimal_round_trips_storage_boundaries() {
        for value in [U256::ZERO, U256::from(1), U256::MAX] {
            let encoded = encode_u256_decimal(value);
            assert_eq!(decode_u256_decimal("value", &encoded), Ok(value));
        }
    }

    #[test]
    fn u256_decimal_rejects_fractional_signed_and_oversized_values() {
        for value in [
            "1.0",
            "+1",
            "-1",
            "1e2",
            "115792089237316195423570985008687907853269984665640564039457584007913129639936",
        ] {
            assert!(decode_u256_decimal("value", value).is_err(), "{value}");
        }
    }

    #[test]
    fn fixed_width_decoders_reject_malformed_bytea_values() {
        assert!(decode_b256("hash", vec![0; 31]).is_err());
        assert!(decode_b256("hash", vec![0; 33]).is_err());
        assert!(decode_address("address", vec![0; 19]).is_err());
        assert!(decode_address("address", vec![0; 21]).is_err());
    }

    #[test]
    fn integer_decoders_reject_negative_storage_values() {
        assert!(decode_nonnegative_i64("number", -1).is_err());
        assert!(decode_nonnegative_i32("index", -1).is_err());
        assert_eq!(
            decode_nonnegative_i64("number", i64::MAX),
            Ok(i64::MAX as u64)
        );
        assert_eq!(
            decode_nonnegative_i32("index", i32::MAX),
            Ok(i32::MAX as u64)
        );
    }

    #[test]
    fn timestamp_decoder_rejects_fractional_and_pre_epoch_values() {
        let fractional = DateTime::from_timestamp(1, 1).expect("timestamp");
        let pre_epoch = DateTime::from_timestamp(-1, 0).expect("timestamp");
        assert!(decode_timestamp_seconds("block.timestamp", fractional).is_err());
        assert!(decode_timestamp_seconds("block.timestamp", pre_epoch).is_err());
    }

    #[test]
    fn receipt_outcome_requires_the_exact_root_shape() {
        assert_eq!(
            decode_receipt_outcome(0, None),
            Ok(ReceiptOutcome::Reverted)
        );
        assert!(decode_receipt_outcome(2, None).is_err());
        assert!(decode_receipt_outcome(1, Some(vec![0; 32])).is_err());
        assert!(decode_receipt_outcome(2, Some(vec![0; 31])).is_err());
    }
}
