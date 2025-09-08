pub mod payment;

// Re-export the main aggregate and its types
pub use payment::{
    Payment, PaymentStatus, PaymentMetadata, PaymentError,
    CryptoPaymentDetails, FiatPaymentDetails,
    
    // Domain Events
    PaymentCreated, PaymentAddressAssigned, PaymentConfirmed,
    PaymentCompleted, PaymentFailed, PaymentCancelled,
    PaymentRefundInitiated, PaymentRefundCompleted
};