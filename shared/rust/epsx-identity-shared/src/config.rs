//! `crate::config::env` shim for `epsx-identity-shared`.
//!
//! The moved auth code uses `get_env_var` and `get_bsc_chain_id` only.

pub fn get_env_var(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

pub fn get_bsc_chain_id(blockchain_network: &str) -> u64 {
    match blockchain_network {
        "mainnet" => 56,
        "testnet" => 97,
        _ => 97,
    }
}
