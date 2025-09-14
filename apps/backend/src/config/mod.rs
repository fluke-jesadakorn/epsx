// Simplified Configuration module for backend environment management
// Uses unified environment schema from env.rs

pub mod env;

// Re-export simplified items for backward compatibility
pub use env::{
    Config,
    init_config,
    get_env_var,
    is_production,
    is_development,
    get_log_level,
    get_database_url,
    get_jwt_secret,
    get_fallback_config,
    ValidationError,
    // Legacy config structs for backward compatibility
    ServerConfig,
    DatabaseConfig,
    AuthConfig,
    PaymentConfig,
    FirebaseConfig,
    EmailConfig,
    BrandingConfig,
    ExternalServicesConfig,
    TradingViewConfig,
    QrCodeConfig,
    RateLimitingConfig,
};