pub mod payment;
pub mod payment_status;
pub mod payment_metadata;
pub mod payment_details;
pub mod payment_context;

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