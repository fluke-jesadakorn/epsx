/// Payment Bounded Context
/// 
/// This bounded context handles all aspects of payment processing, transaction management,
/// cryptocurrency payments, and billing for the EPSX platform.
/// 
/// ## Core Concepts
/// 
/// - **Payment**: Main aggregate managing payment lifecycle from creation to completion
/// - **PaymentAmount**: Value object with currency validation and fee calculations  
/// - **CryptoAddress**: Blockchain address validation with network-specific rules
/// - **TransactionHash**: Blockchain transaction tracking with confirmation status
/// - **PaymentMethod**: Payment method configuration with processing rules
/// 
/// ## Supported Payment Methods
/// 
/// - **Cryptocurrency**: Multi-network crypto payments (Ethereum, BSC, TRON, Arbitrum, Polygon)
/// - **Bank Transfer**: Traditional fiat bank transfers with processing delays
/// - **Credit Card**: Instant credit/debit card processing
/// 
/// ## Domain Events
/// 
/// The context publishes events for payment lifecycle, transaction confirmations,
/// refunds, and payment status changes
/// 
/// ## Integration
/// 
/// This bounded context integrates with:
/// - User Management (for user identification and permissions)
/// - Notification (for payment status updates)
/// - External payment processors and blockchain networks

pub mod value_objects;
pub mod aggregates;
pub mod repository_ports;

// Public exports from value objects
pub use value_objects::{
    PaymentId, PaymentReference, PaymentAmount, PaymentAmountError, ExchangeRates,
    CryptoAddress, CryptoAddressError, PaymentAddress, AddressType,
    TransactionHash, TransactionHashError, TransactionReceipt, TransactionStatus,
    ConfirmationStatus, PaymentMethod, PaymentMethodError, PaymentMethodType, 
    PaymentMethodConfig, PaymentInstructions, Currency, Network
};

// Public exports from aggregates
pub use aggregates::{
    Payment, PaymentStatus, PaymentMetadata, PaymentError,
    CryptoPaymentDetails, FiatPaymentDetails,
    PaymentCreated, PaymentAddressAssigned, PaymentConfirmed,
    PaymentCompleted, PaymentFailed, PaymentCancelled,
    PaymentRefundInitiated, PaymentRefundCompleted
};

// Public exports from repository ports
pub use repository_ports::{
    PaymentRepositoryPort, TransactionRepositoryPort, CryptoAddressRepositoryPort,
    PaymentMethodRepositoryPort, PaymentStats, TransactionRecord
};