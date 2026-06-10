// Repository Ports for Wallet Management
// These define the interfaces for data persistence in the Wallet Management bounded context

// Web3 wallet-based repository
pub mod wallet_user_repository_port;


// NEW - Web3 wallet user repository exports (primary)
pub use wallet_user_repository_port::{
    WalletUserRepositoryPort,
    WalletUserSearchPort,
    WalletUserAnalyticsPort,
    WalletUserSearchCriteria,
    WalletUserSearchResult,
    WalletUserStatistics,
    Web3Analytics,
};


