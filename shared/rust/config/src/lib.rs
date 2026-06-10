use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub web3: Web3Config,
    pub auth: AuthConfig,
    pub content: ContentConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Config {
    pub bsc_rpc: String,
    pub bsc_testnet_rpc: String,
    pub paymaster_address: String,
    pub payment_escrow_address: String,
    pub subscription_vault_address: String,
    pub private_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub refresh_expires_in: u64,
    pub siwe_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    pub base_path: String,
    pub git_repo: String,
    pub preview_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_port: u16,
    pub jaeger_endpoint: Option<String>,
    pub loki_endpoint: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        let mut builder = config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(false))
            .add_source(config::Environment::with_prefix("EPSX").separator("__"));

        let config: AppConfig = builder.build()?.try_deserialize()?;

        Ok(config)
    }

    pub fn database_url(&self) -> String {
        self.database.url()
    }

    pub fn redis_url(&self) -> &str {
        &self.redis.url
    }
}
