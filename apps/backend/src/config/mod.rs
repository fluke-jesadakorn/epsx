// Simplified Configuration module for backend environment management
// Uses unified environment schema from env.rs

pub mod env;
pub mod contracts;

// Re-export simplified items
pub use env::{
    Config,
    init_config,
    get_env_var,
    is_production,
    is_development,
    get_log_level,
    get_database_url,
    get_fallback_config,
    ValidationError,
};

pub use contracts::{
    Chain,
    ChainContractConfig,
    ContractConfig,
    PAYMENT_EVENT_TOPIC,
};