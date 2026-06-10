// Web3 Admin Permission Management Handlers - Module Organization
// Leverages existing Web3PermissionService for blockchain-based permission management

// Module declarations
pub mod types;
pub mod permission_handlers;
pub mod gate_handlers;
pub mod wallet_handlers;

// Re-export all public types
pub use types::*;

// Re-export all handler functions
pub use permission_handlers::{get_user_permissions, grant_manual_permission};
pub use gate_handlers::{
  create_nft_gate,
  create_token_gate,
  create_dao_proposal,
  get_nft_gates,
  get_token_gates,
  get_dao_proposals,
};
pub use wallet_handlers::{get_recent_wallets, search_wallets, get_tiers};
