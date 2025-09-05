/// Payment Value Objects Module
/// 
/// This module contains all value objects for the Payment bounded context.
/// Value objects represent concepts with identity defined by their attributes
/// rather than a unique identifier.

pub mod payment_id;
pub mod payment_amount;
pub mod crypto_address;
pub mod crypto_address_id;
pub mod crypto_network;
pub mod payment_method_id;
pub mod transaction_hash;
pub mod payment_method;

// Public exports from payment_id
pub use payment_id::{PaymentId, PaymentReference};

// Public exports from payment_amount  
pub use payment_amount::{
    PaymentAmount, PaymentAmountError, ExchangeRates,
    Currency, Network
};

// Public exports from crypto_address
pub use crypto_address::{
    CryptoAddress, CryptoAddressError, PaymentAddress, AddressType
};

// Public exports from crypto_address_id
pub use crypto_address_id::{CryptoAddressId, CryptoAddressIdError};

// Public exports from crypto_network
pub use crypto_network::{CryptoNetwork, CryptoNetworkError};

// Public exports from payment_method_id  
pub use payment_method_id::{PaymentMethodId, PaymentMethodIdError};

// Public exports from transaction_hash
pub use transaction_hash::{
    TransactionHash, TransactionHashError, TransactionReceipt, TransactionStatus,
    ConfirmationStatus
};

// Public exports from payment_method
pub use payment_method::{
    PaymentMethod, PaymentMethodError, PaymentMethodType, PaymentMethodConfig,
    PaymentInstructions
};