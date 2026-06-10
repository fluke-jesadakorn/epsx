// Wallet Management Aggregates
// Aggregates are consistency boundaries that encapsulate business rules and behavior

pub mod wallet_user; // NEW - Web3 wallet-based user aggregate  


// Web3 wallet user aggregate
pub use wallet_user::{WalletUser, WalletMetadata};