use dotenv::dotenv;
use std::env;
use crate::infra::db::postgres::DatabaseConfig;
use crate::infra::services::{PaymentGatewayConfig, MarketDataConfig};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub payment: PaymentServiceConfig,
    pub market_data: MarketDataServiceConfig,
    pub notification: NotificationConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub sendgrid_api_key: String,
    pub from_email: String,
    pub from_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PaymentServiceConfig {
    pub coinpayments: PaymentGatewayConfig,
    pub webhook_url: String,
    pub confirmation_count: u32,
}

#[derive(Debug, Clone)]
pub struct MarketDataServiceConfig {
    pub alpha_vantage: MarketDataConfig,
    pub cache_ttl_seconds: u64,
    pub fallback_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub database_enabled: bool,
    pub max_notifications_per_user: usize,
    pub cleanup_interval_hours: u64,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub firebase_project_id: String,
    pub firebase_service_key_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
            frontend_url: env::var("FRONTEND_URL")
                .expect("FRONTEND_URL environment variable is required"),
            database: DatabaseConfig::default(),
            email: EmailConfig::from_env(),
            payment: PaymentServiceConfig::from_env(),
            market_data: MarketDataServiceConfig::from_env(),
            notification: NotificationConfig::from_env(),
            auth: AuthConfig::from_env(),
        }
    }

    pub fn host(&self) -> String {
        env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string())
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl EmailConfig {
    pub fn from_env() -> Self {
        Self {
            sendgrid_api_key: env::var("SENDGRID_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            from_email: env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@epsx.com".to_string()),
            from_name: env::var("FROM_NAME")
                .unwrap_or_else(|_| "EPSX Trading Platform".to_string()),
            enabled: env::var("EMAIL_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

impl PaymentServiceConfig {
    pub fn from_env() -> Self {
        let coinpayments = PaymentGatewayConfig {
            api_key: env::var("COINPAYMENTS_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            api_secret: env::var("COINPAYMENTS_API_SECRET").ok(),
            base_url: env::var("COINPAYMENTS_API_URL")
                .unwrap_or_else(|_| "https://www.coinpayments.net/api.php".to_string()),
            webhook_secret: env::var("COINPAYMENTS_WEBHOOK_SECRET").ok(),
            supported_currencies: vec![
                crate::dom::values::Currency::BTC,
                crate::dom::values::Currency::ETH,
                crate::dom::values::Currency::USDT,
                crate::dom::values::Currency::BNB,
            ],
            network_configs: std::collections::HashMap::new(), // Initialize empty, populate as needed
        };

        Self {
            coinpayments,
            webhook_url: env::var("PAYMENT_WEBHOOK_URL")
                .expect("PAYMENT_WEBHOOK_URL environment variable is required"),
            confirmation_count: env::var("PAYMENT_CONFIRMATION_COUNT")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
        }
    }
}

impl MarketDataServiceConfig {
    pub fn from_env() -> Self {
        let alpha_vantage = MarketDataConfig {
            api_key: env::var("ALPHA_VANTAGE_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            base_url: env::var("ALPHA_VANTAGE_API_URL")
                .unwrap_or_else(|_| "https://www.alphavantage.co/query".to_string()),
            rate_limit: env::var("ALPHA_VANTAGE_RATE_LIMIT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            timeout_seconds: env::var("ALPHA_VANTAGE_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        };

        Self {
            alpha_vantage,
            cache_ttl_seconds: env::var("MARKET_DATA_CACHE_TTL")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            fallback_enabled: env::var("MARKET_DATA_FALLBACK_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

impl NotificationConfig {
    pub fn from_env() -> Self {
        Self {
            database_enabled: env::var("NOTIFICATION_DATABASE_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            max_notifications_per_user: env::var("MAX_NOTIFICATIONS_PER_USER")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            cleanup_interval_hours: env::var("NOTIFICATION_CLEANUP_INTERVAL_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
        }
    }
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string()),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            firebase_project_id: env::var("FIREBASE_PROJECT_ID")
                .unwrap_or_else(|_| "epsx-trading-platform".to_string()),
            firebase_service_key_path: env::var("FIREBASE_SERVICE_KEY_PATH").ok(),
        }
    }
}
