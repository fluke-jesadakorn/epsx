pub mod aggregates;
pub mod repository_ports;
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

// Public exports from value objects
pub use value_objects::{
    AddressType, ConfirmationStatus, CryptoAddress, CryptoAddressError, Currency, ExchangeRates,
    Network, PaymentAddress, PaymentAmount, PaymentAmountError, PaymentId, PaymentInstructions,
    PaymentMethod, PaymentMethodConfig, PaymentMethodError, PaymentMethodType, PaymentReference,
    TransactionHash, TransactionHashError, TransactionReceipt, TransactionStatus,
};

// Public exports from aggregates
pub use aggregates::{
    CreatePaymentContextParams,
    CryptoPaymentDetails,
    FiatPaymentDetails,
    LoadPaymentContextParams,
    Payment,
    PaymentAddressAssigned,
    PaymentCancelled,
    PaymentCompleted,
    PaymentConfirmed,
    // Payment Context exports
    PaymentContext,
    PaymentContextError,
    PaymentContextId,
    PaymentContextType,
    PaymentCreated,
    PaymentError,
    PaymentFailed,
    PaymentMetadata,
    PaymentRefundCompleted,
    PaymentRefundInitiated,
    PaymentStatus,
    UpdatePaymentContextParams,
    DEFAULT_EXPIRATION_HOURS,
};

// Public exports from repository ports
pub use repository_ports::{
    CryptoAddressRepositoryPort, PaymentContextRepositoryPort, PaymentMethodRepositoryPort,
    PaymentRepositoryPort, PaymentStats, TransactionRecord, TransactionRepositoryPort,
};

// wave11(track-b) re-export: the new
// `SubscriptionRepositoryPort`. See
// `repository_ports/subscription_port.rs` for the full docstring
// and the audit references.
pub use repository_ports::subscription_port::SubscriptionRepositoryPort;
