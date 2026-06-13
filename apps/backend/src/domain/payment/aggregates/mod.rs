pub mod payment;
pub mod payment_status;
pub mod payment_metadata;
pub mod payment_details;
pub mod payment_context;

// wave11(track-b): subscription aggregate + stock-ranking-assignment
// value object moved from the central infrastructure layer and
// from `application::market_analytics` into the payments domain.
// See `docs/wave8-service-boundary/audit-payments.md` §3 row 3
// and row 4.
pub mod subscription;
pub mod stock_ranking_assignment;

#[cfg(test)]
pub mod payment_tests;

// Re-export types from separate modules
pub use payment_status::PaymentStatus;
pub use payment_metadata::PaymentMetadata;
pub use payment_details::{CryptoPaymentDetails, FiatPaymentDetails, BlockchainVerificationStatus};

// Re-export the main aggregate and its types
pub use payment::{
    Payment, PaymentError,
    
    // Domain Events
    PaymentCreated, PaymentAddressAssigned, PaymentConfirmed,
    PaymentCompleted, PaymentFailed, PaymentCancelled,
    PaymentRefundInitiated, PaymentRefundCompleted,
    PaymentVerificationStarted, PaymentBlockchainVerified, PaymentVerificationFailed
};

// Re-export payment context aggregate
pub use payment_context::{
    PaymentContext, PaymentContextId, PaymentContextType,
    CreatePaymentContextParams, LoadPaymentContextParams, UpdatePaymentContextParams,
    PaymentContextError, DEFAULT_EXPIRATION_HOURS
};

// wave11(track-b) re-exports: subscription aggregate +
// stock-ranking-assignment value object. Used by the new
// `SubscriptionRepositoryPort` (see
// `repository_ports/subscription_port.rs`).
pub use subscription::{
    CreateSubscriptionCommand, Subscription, SubscriptionId,
};
pub use stock_ranking_assignment::StockRankingAssignment;