//! Shared value objects used across multiple bounded contexts.
//!
//! Source: apps/backend/src/domain/shared_kernel/value_objects/

pub mod user_id;
pub mod session_id;
pub mod email;
pub mod chain_id;
pub mod address_str;
pub mod money;
pub mod token;
pub mod payment_id;
pub mod payment_status;
pub mod currency;
pub mod network;
pub mod symbol;
pub mod market;
pub mod stock_symbol;
pub mod notification_id;
pub mod connection_id;

pub use user_id::UserId;
pub use session_id::{SessionId, SessId};
pub use email::Email;
pub use chain_id::ChainId;
pub use address_str::AddressStr;
pub use money::Money;
pub use token::Token;
pub use payment_id::PaymentId;
pub use payment_status::PaymentStatus;
pub use currency::Currency;
pub use network::Network;
pub use symbol::Symbol;
pub use market::Market;
pub use stock_symbol::StockSymbol;
pub use notification_id::NotificationId;
pub use connection_id::ConnectionId;
