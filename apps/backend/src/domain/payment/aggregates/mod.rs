pub mod payment;
pub mod payment_context;
pub mod payment_details;
pub mod payment_metadata;
pub mod payment_status;

// wave11(track-b): subscription aggregate + stock-ranking-assignment
// value object moved from the central infrastructure layer and
// from `application::market_analytics` into the payments domain.
// See `docs/wave8-service-boundary/audit-payments.md` §3 row 3
// and row 4.
pub mod stock_ranking_assignment;
pub mod subscription;

#[cfg(test)]
pub mod payment_tests;

// Re-export types from separate modules
pub use payment_details::{BlockchainVerificationStatus, CryptoPaymentDetails, FiatPaymentDetails};
pub use payment_metadata::PaymentMetadata;
pub use payment_status::PaymentStatus;

// Re-export the main aggregate and its types
pub use payment::{
    Payment,
    PaymentAddressAssigned,
    PaymentBlockchainVerified,
    PaymentCancelled,
    PaymentCompleted,
    PaymentConfirmed,
    // Domain Events
    PaymentCreated,
    PaymentError,

    PaymentFailed,
    PaymentRefundCompleted,
    PaymentRefundInitiated,
    PaymentVerificationFailed,
    PaymentVerificationStarted,
};

// Re-export payment context aggregate
pub use payment_context::{
    CreatePaymentContextParams, LoadPaymentContextParams, PaymentContext, PaymentContextError,
    PaymentContextId, PaymentContextType, UpdatePaymentContextParams, DEFAULT_EXPIRATION_HOURS,
};

// wave11(track-b) re-exports: subscription aggregate +
// stock-ranking-assignment value object. Used by the new
// `SubscriptionRepositoryPort` (see
// `repository_ports/subscription_port.rs`).
pub use stock_ranking_assignment::StockRankingAssignment;
pub use subscription::{CreateSubscriptionCommand, Subscription, SubscriptionId};
