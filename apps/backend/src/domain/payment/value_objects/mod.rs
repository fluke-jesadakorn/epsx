pub mod credit;
pub mod crypto_address;
pub mod crypto_address_id;
pub mod crypto_network;
pub mod payment_amount;
/// Payment Value Objects Module
///
/// This module contains all value objects for the Payment bounded context.
/// Value objects represent concepts with identity defined by their attributes
/// rather than a unique identifier.
pub mod payment_id;
pub mod payment_method;
pub mod payment_method_id;
pub mod transaction_hash;

// Public exports from payment_id
pub use payment_id::{PaymentId, PaymentReference};

// Public exports from payment_amount
pub use payment_amount::{Currency, ExchangeRates, Network, PaymentAmount, PaymentAmountError};

// Public exports from crypto_address
pub use crypto_address::{AddressType, CryptoAddress, CryptoAddressError, PaymentAddress};

// Public exports from crypto_address_id
pub use crypto_address_id::{CryptoAddressId, CryptoAddressIdError};

// Public exports from crypto_network
pub use crypto_network::{CryptoNetwork, CryptoNetworkError};

// Public exports from payment_method_id
pub use payment_method_id::{PaymentMethodId, PaymentMethodIdError};

// Public exports from transaction_hash
pub use transaction_hash::{
    ConfirmationStatus, TransactionHash, TransactionHashError, TransactionReceipt,
    TransactionStatus,
};

// Public exports from payment_method
pub use payment_method::{
    PaymentInstructions, PaymentMethod, PaymentMethodConfig, PaymentMethodError, PaymentMethodType,
};

// Public exports from credit
pub use credit::{CreditAmount, CreditError, CreditTransactionType};
