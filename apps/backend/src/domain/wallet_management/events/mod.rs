// Wallet Management Domain Events
// Events that are raised when significant things happen in the user management domain

// Web3 wallet-based events
pub mod wallet_user_events;
pub mod web3_permission_events;

// NEW - Web3 wallet user and permission events (primary)
pub use wallet_user_events::{
    WalletPermissionsUpdatedEvent, WalletUserActivatedEvent, WalletUserCreatedEvent,
    WalletUserDeactivatedEvent,
};

pub use web3_permission_events::{
    CrossChainPermissionValidatedEvent, DaoPermissionGrantedEvent, NftPermissionGrantedEvent,
    TokenPermissionGrantedEvent, Web3PermissionValidatedEvent, Web3PermissionValidationFailedEvent,
    Web3PermissionsSyncedEvent,
};

// Session events (unchanged)
