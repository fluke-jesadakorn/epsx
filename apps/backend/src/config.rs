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
    pub frontend: FrontendConfig,
    pub external_services: ExternalServicesConfig,
    pub rate_limiting: RateLimitingConfig,
    pub network: NetworkConfig,
    pub business: BusinessConfig,
    pub branding: BrandingConfig,
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
    pub session_ttl_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct FrontendConfig {
    pub frontend_url: String,
    pub admin_frontend_url: String,
    pub production_frontend_url: String,
    pub production_admin_url: String,
}

#[derive(Debug, Clone)]
pub struct ExternalServicesConfig {
    pub tradingview: TradingViewConfig,
    pub qr_code: QrCodeConfig,
    pub firebase: FirebaseConfig,
}

#[derive(Debug, Clone)]
pub struct TradingViewConfig {
    pub origin_url: String,
    pub referer_url: String,
    pub scanner_api_url: String,
    pub websocket_timeout_seconds: u64,
    pub http_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct QrCodeConfig {
    pub api_base_url: String,
    pub default_size: String,
    pub max_size: u32,
}

#[derive(Debug, Clone)]
pub struct FirebaseConfig {
    pub auth_api_base_url: String,
    pub signin_endpoint: String,
    pub lookup_endpoint: String,
    pub signup_endpoint: String,
}

#[derive(Debug, Clone)]
pub struct RateLimitingConfig {
    pub default_per_minute: u32,
    pub default_per_hour: u32,
    pub default_per_day: u32,
    pub endpoint_specific: std::collections::HashMap<String, EndpointRateLimit>,
    pub time_windows: TimeWindowConfig,
}

#[derive(Debug, Clone)]
pub struct EndpointRateLimit {
    pub per_minute: u32,
    pub per_hour: u32,
    pub per_day: u32,
}

#[derive(Debug, Clone)]
pub struct TimeWindowConfig {
    pub minute_seconds: u64,
    pub hour_seconds: u64,
    pub day_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub default_host: String,
    pub bind_address: String,
    pub default_checkout_url_template: String,
    pub dashboard_base_url: String,
}

#[derive(Debug, Clone)]
pub struct BusinessConfig {
    pub supported_currencies: Vec<String>,
    pub default_currency: String,
    pub feature_expiration_warning_days: Vec<u32>,
    pub payment_confirmation_count: u32,
}

#[derive(Debug, Clone)]
pub struct BrandingConfig {
    pub company_name: String,
    pub platform_name: String,
    pub support_email: String,
    pub dashboard_url: String,
    pub welcome_message_template: String,
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
            frontend: FrontendConfig::from_env(),
            external_services: ExternalServicesConfig::from_env(),
            rate_limiting: RateLimitingConfig::from_env(),
            network: NetworkConfig::from_env(),
            business: BusinessConfig::from_env(),
            branding: BrandingConfig::from_env(),
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
            session_ttl_seconds: env::var("SESSION_TTL_SECONDS")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
        }
    }
}

impl FrontendConfig {
    pub fn from_env() -> Self {
        Self {
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            admin_frontend_url: env::var("ADMIN_FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            production_frontend_url: env::var("PRODUCTION_FRONTEND_URL")
                .unwrap_or_else(|_| "https://epsx.com".to_string()),
            production_admin_url: env::var("PRODUCTION_ADMIN_URL")
                .unwrap_or_else(|_| "https://admin.epsx.com".to_string()),
        }
    }
}

impl ExternalServicesConfig {
    pub fn from_env() -> Self {
        Self {
            tradingview: TradingViewConfig::from_env(),
            qr_code: QrCodeConfig::from_env(),
            firebase: FirebaseConfig::from_env(),
        }
    }
}

impl TradingViewConfig {
    pub fn from_env() -> Self {
        Self {
            origin_url: env::var("TRADINGVIEW_ORIGIN_URL")
                .unwrap_or_else(|_| "https://www.tradingview.com".to_string()),
            referer_url: env::var("TRADINGVIEW_REFERER_URL")
                .unwrap_or_else(|_| "https://www.tradingview.com/".to_string()),
            scanner_api_url: env::var("TRADINGVIEW_SCANNER_API_URL")
                .unwrap_or_else(|_| "https://scanner.tradingview.com/global/scan".to_string()),
            websocket_timeout_seconds: env::var("TRADINGVIEW_WEBSOCKET_TIMEOUT")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            http_timeout_seconds: env::var("TRADINGVIEW_HTTP_TIMEOUT")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

impl QrCodeConfig {
    pub fn from_env() -> Self {
        Self {
            api_base_url: env::var("QR_CODE_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.qrserver.com/v1/create-qr-code/".to_string()),
            default_size: env::var("QR_CODE_DEFAULT_SIZE")
                .unwrap_or_else(|_| "200x200".to_string()),
            max_size: env::var("QR_CODE_MAX_SIZE")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
        }
    }
}

impl FirebaseConfig {
    pub fn from_env() -> Self {
        Self {
            auth_api_base_url: env::var("FIREBASE_AUTH_API_BASE_URL")
                .unwrap_or_else(|_| "https://identitytoolkit.googleapis.com/v1/accounts".to_string()),
            signin_endpoint: env::var("FIREBASE_SIGNIN_ENDPOINT")
                .unwrap_or_else(|_| "signInWithPassword".to_string()),
            lookup_endpoint: env::var("FIREBASE_LOOKUP_ENDPOINT")
                .unwrap_or_else(|_| "lookup".to_string()),
            signup_endpoint: env::var("FIREBASE_SIGNUP_ENDPOINT")
                .unwrap_or_else(|_| "signUp".to_string()),
        }
    }
}

impl RateLimitingConfig {
    pub fn from_env() -> Self {
        let mut endpoint_specific = std::collections::HashMap::new();
        
        // Load endpoint-specific rate limits from environment
        endpoint_specific.insert("/api/auth/login".to_string(), EndpointRateLimit {
            per_minute: env::var("RATE_LIMIT_LOGIN_PER_MINUTE")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            per_hour: env::var("RATE_LIMIT_LOGIN_PER_HOUR")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            per_day: env::var("RATE_LIMIT_LOGIN_PER_DAY")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        });
        
        endpoint_specific.insert("/api/payments/create".to_string(), EndpointRateLimit {
            per_minute: env::var("RATE_LIMIT_PAYMENT_PER_MINUTE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            per_hour: env::var("RATE_LIMIT_PAYMENT_PER_HOUR")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
            per_day: env::var("RATE_LIMIT_PAYMENT_PER_DAY")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
        });
        
        Self {
            default_per_minute: env::var("DEFAULT_RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            default_per_hour: env::var("DEFAULT_RATE_LIMIT_PER_HOUR")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            default_per_day: env::var("DEFAULT_RATE_LIMIT_PER_DAY")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .unwrap_or(10000),
            endpoint_specific,
            time_windows: TimeWindowConfig {
                minute_seconds: 60,
                hour_seconds: 3600,
                day_seconds: 86400,
            },
        }
    }
}

impl NetworkConfig {
    pub fn from_env() -> Self {
        Self {
            default_host: env::var("DEFAULT_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            bind_address: env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            default_checkout_url_template: env::var("CHECKOUT_URL_TEMPLATE")
                .unwrap_or_else(|_| "https://checkout.example.com/pay/{}".to_string()),
            dashboard_base_url: env::var("DASHBOARD_BASE_URL")
                .unwrap_or_else(|_| "https://epsx.com/dashboard".to_string()),
        }
    }
}

impl BusinessConfig {
    pub fn from_env() -> Self {
        let default_currencies = vec![
            "USD".to_string(), "EUR".to_string(), "GBP".to_string(), 
            "JPY".to_string(), "AUD".to_string(), "CAD".to_string(),
            "CHF".to_string(), "CNY".to_string(), "SEK".to_string(), "NZD".to_string()
        ];
        
        let supported_currencies = env::var("SUPPORTED_CURRENCIES")
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
            .unwrap_or(default_currencies);
            
        let warning_days_str = env::var("FEATURE_EXPIRATION_WARNING_DAYS")
            .unwrap_or_else(|_| "30,7,3,1".to_string());
        let warning_days = warning_days_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        
        Self {
            supported_currencies,
            default_currency: env::var("DEFAULT_CURRENCY")
                .unwrap_or_else(|_| "USD".to_string()),
            feature_expiration_warning_days: warning_days,
            payment_confirmation_count: env::var("PAYMENT_CONFIRMATION_COUNT")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
        }
    }
}

impl BrandingConfig {
    pub fn from_env() -> Self {
        Self {
            company_name: env::var("COMPANY_NAME")
                .unwrap_or_else(|_| "EPSX".to_string()),
            platform_name: env::var("PLATFORM_NAME")
                .unwrap_or_else(|_| "EPSX Trading Platform".to_string()),
            support_email: env::var("SUPPORT_EMAIL")
                .unwrap_or_else(|_| "support@epsx.com".to_string()),
            dashboard_url: env::var("DASHBOARD_URL")
                .unwrap_or_else(|_| "https://epsx.com/dashboard".to_string()),
            welcome_message_template: env::var("WELCOME_MESSAGE_TEMPLATE")
                .unwrap_or_else(|_| "Welcome to {}, {}!".to_string()),
        }
    }
}
