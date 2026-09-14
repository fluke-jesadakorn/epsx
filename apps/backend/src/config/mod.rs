// Simplified Configuration module for backend environment management
// Uses unified environment schema from env.rs

pub mod contracts;
pub mod env;

// Re-export simplified items
pub use env::{
    get_database_url, get_env_var, get_fallback_config, get_log_level, init_config, is_development,
    is_production, Config, ValidationError,
};

pub use contracts::{Chain, ChainContractConfig, ContractConfig, PAYMENT_EVENT_TOPIC};
