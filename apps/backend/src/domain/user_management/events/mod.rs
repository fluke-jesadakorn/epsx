// User Management Domain Events
// Events that are raised when significant things happen in the user management domain

// Web3 wallet-based events
pub mod wallet_user_events;
pub mod web3_permission_events;
pub mod session_events;

// NEW - Web3 wallet user and permission events (primary)
pub use wallet_user_events::{
    WalletUserCreatedEvent,
    WalletUserActivatedEvent,
    WalletUserDeactivatedEvent,
    WalletPermissionsUpdatedEvent,
};

pub use web3_permission_events::{
    Web3PermissionValidatedEvent,
    NftPermissionGrantedEvent,
    TokenPermissionGrantedEvent,
    DaoPermissionGrantedEvent,
    Web3PermissionValidationFailedEvent,
    Web3PermissionsSyncedEvent,
    CrossChainPermissionValidatedEvent,
};


// Session events (unchanged)
pub use session_events::*;