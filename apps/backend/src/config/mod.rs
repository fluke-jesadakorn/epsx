// Configuration module for backend environment management

pub mod env;

// Re-export commonly used items
pub use env::{
    validate_environment,
    load_validated_config,
    get_env_var,
    is_production,
    is_development,
    get_log_level,
    ServerConfig,
    DatabaseConfig,
    AuthConfig,
    PaymentConfig,
    EmailConfig,
    BrandingConfig,
    ExternalServicesConfig,
    TradingViewConfig,
    RateLimitingConfig,
    EndpointRateLimit,
    ValidatedConfig,
    ValidationError,
    EnvCategory,
    ErrorSeverity
};

/// Main configuration struct that uses the new environment system
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub payment: PaymentConfig,
    pub email: EmailConfig,
    pub branding: BrandingConfig,
    pub external_services: ExternalServicesConfig,
    pub rate_limiting: RateLimitingConfig,
}

impl Config {
    /// Create config from validated environment variables
    pub fn from_env() -> Result<Self, Vec<ValidationError>> {
        let validated_config = load_validated_config()?;
        Ok(Self {
            server: validated_config.server,
            database: validated_config.database,
            auth: validated_config.auth,
            payment: validated_config.payment,
            email: validated_config.email,
            branding: validated_config.branding,
            external_services: validated_config.external_services,
            rate_limiting: validated_config.rate_limiting,
        })
    }

    /// Get database URL
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Get server port
    pub fn port(&self) -> u16 {
        self.server.port
    }

    /// Get JWT secret
    pub fn jwt_secret(&self) -> &str {
        &self.auth.jwt_secret
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        is_production()
    }

    /// Check if running in development mode  
    pub fn is_development(&self) -> bool {
        is_development()
    }
}