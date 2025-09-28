use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};

use super::PaymentError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    /// Payment has been created but not yet initiated
    Created,
    /// Awaiting payment from user (crypto address assigned)
    AwaitingPayment,
    /// Transaction submitted by user, awaiting blockchain verification
    PendingVerification,
    /// Blockchain verification in progress
    Verifying,
    /// Blockchain verification failed
    VerificationFailed,
    /// Payment has been received and confirmed on blockchain
    Confirmed,
    /// Payment is being processed
    Processing,
    /// Payment completed successfully
    Completed,
    /// Payment failed
    Failed,
    /// Payment was cancelled before completion
    Cancelled,
    /// Refund is being processed
    Refunding,
    /// Payment has been refunded
    Refunded,
}

impl std::str::FromStr for PaymentStatus {
    type Err = PaymentError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(PaymentStatus::Created),
            "awaiting_payment" | "awaiting" => Ok(PaymentStatus::AwaitingPayment),
            "pending_verification" | "pending" => Ok(PaymentStatus::PendingVerification),
            "verifying" => Ok(PaymentStatus::Verifying),
            "verification_failed" => Ok(PaymentStatus::VerificationFailed),
            "confirmed" => Ok(PaymentStatus::Confirmed),
            "processing" => Ok(PaymentStatus::Processing),
            "completed" => Ok(PaymentStatus::Completed),
            "failed" => Ok(PaymentStatus::Failed),
            "cancelled" | "canceled" => Ok(PaymentStatus::Cancelled),
            "refunding" => Ok(PaymentStatus::Refunding),
            "refunded" => Ok(PaymentStatus::Refunded),
            _ => Err(PaymentError::InvalidStatusTransition {
                from: PaymentStatus::Created,
                to: PaymentStatus::Created,
                reason: format!("Unknown payment status: {}", s),
            }),
        }
    }
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Created => "created",
            PaymentStatus::AwaitingPayment => "awaiting_payment",
            PaymentStatus::PendingVerification => "pending_verification",
            PaymentStatus::Verifying => "verifying",
            PaymentStatus::VerificationFailed => "verification_failed",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Processing => "processing",
            PaymentStatus::Completed => "completed",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Refunding => "refunding",
            PaymentStatus::Refunded => "refunded",
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Completed | PaymentStatus::Failed | 
            PaymentStatus::Cancelled | PaymentStatus::Refunded |
            PaymentStatus::VerificationFailed
        )
    }
}

impl PartialEq<PaymentStatus> for &PaymentStatus {
    fn eq(&self, other: &PaymentStatus) -> bool {
        **self == *other
    }
}

impl Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}