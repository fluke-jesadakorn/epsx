// Backend Environment Configuration with Validation
// All environment variable access must go through this module

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum EnvVarType {
    String,
    Url,
    Number,
    Boolean,
    Email,
    JwtSecret,
    DatabaseUrl,
    PrivateKey,
    Port,
    LogLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvCategory {
    Infrastructure,
    Database,
    Authentication,
    Security,
    Services,
    Payment,
    Logging,
    Testing,
    RateLimiting,
    External,
}

#[derive(Debug, Clone)]
pub struct EnvVarDefinition {
    pub objective: &'static str,
    pub required: bool,
    pub var_type: EnvVarType,
    pub category: EnvCategory,
    pub example: &'static str,
    pub default_value: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub variable: String,
    pub objective: String,
    pub reason: String,
    pub category: EnvCategory,
    pub suggestion: String,
    pub example: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Error,
    Warning,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {} ({})",
            match self.severity {
                ErrorSeverity::Error => "❌",
                ErrorSeverity::Warning => "⚠️",
            },
            self.variable,
            self.reason,
            self.objective
        )
    }
}

// Comprehensive Backend Environment Schema
lazy_static::lazy_static! {
    static ref BACKEND_ENV_SCHEMA: HashMap<&'static str, EnvVarDefinition> = {
        let mut schema = HashMap::new();

        // Server & Infrastructure Configuration
        schema.insert("PORT", EnvVarDefinition {
            objective: "Server port for backend API service",
            required: false,
            var_type: EnvVarType::Port,
            category: EnvCategory::Infrastructure,
            example: "8080",
            default_value: Some("8080"),
        });

        schema.insert("BACKEND_URL", EnvVarDefinition {
            objective: "Backend API URL for OIDC issuer and internal service communication",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "http://localhost:8080",
            default_value: Some("http://localhost:8080"),
        });

        schema.insert("HOST", EnvVarDefinition {
            objective: "Server host binding address for network interface",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "0.0.0.0",
            default_value: Some("127.0.0.1"),
        });

        schema.insert("BIND_ADDRESS", EnvVarDefinition {
            objective: "Server binding address for network socket configuration",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "0.0.0.0",
            default_value: Some("0.0.0.0"),
        });

        schema.insert("DEFAULT_HOST", EnvVarDefinition {
            objective: "Default host address fallback for server configuration",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "127.0.0.1",
            default_value: Some("127.0.0.1"),
        });

        schema.insert("FRONTEND_URL", EnvVarDefinition {
            objective: "Frontend application URL for CORS and redirect configuration",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "http://localhost:3000",
            default_value: Some("http://localhost:3000"),
        });

        schema.insert("ADMIN_FRONTEND_URL", EnvVarDefinition {
            objective: "Admin frontend URL for CORS and admin-specific redirects",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "http://localhost:3001",
            default_value: Some("http://localhost:3001"),
        });

        schema.insert("PRODUCTION_FRONTEND_URL", EnvVarDefinition {
            objective: "Production frontend URL for deployment environment",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "https://epsx.com",
            default_value: None,
        });

        schema.insert("PRODUCTION_ADMIN_URL", EnvVarDefinition {
            objective: "Production admin URL for deployment environment",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "https://admin.epsx.com",
            default_value: None,
        });

        schema.insert("RUST_ENV", EnvVarDefinition {
            objective: "Rust environment mode for application behavior",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "development",
            default_value: Some("development"),
        });

        schema.insert("NODE_ENV", EnvVarDefinition {
            objective: "Node.js environment compatibility for mixed stacks",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "development",
            default_value: Some("development"),
        });

        schema.insert("ENV", EnvVarDefinition {
            objective: "General environment identifier for application configuration",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Infrastructure,
            example: "development",
            default_value: Some("development"),
        });

        // Database Configuration
        schema.insert("DATABASE_URL", EnvVarDefinition {
            objective: "PostgreSQL connection string for all database operations",
            required: true,
            var_type: EnvVarType::DatabaseUrl,
            category: EnvCategory::Database,
            example: "postgresql://postgres:password@localhost:5432/epsx_db",
            default_value: None,
        });

        // Authentication & JWT Configuration
        schema.insert("NEXTAUTH_SECRET", EnvVarDefinition {
            objective: "JWT token signing secret (shared with frontend applications)",
            required: true,
            var_type: EnvVarType::JwtSecret,
            category: EnvCategory::Authentication,
            example: "epsx-shared-jwt-secret-2024-cross-app-authentication",
            default_value: None,
        });

        // Payment Configuration (MusePay)
        schema.insert("MUSEPAY_PARTNER_ID", EnvVarDefinition {
            objective: "MusePay partner identifier for payment processing integration",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Payment,
            example: "your-partner-id",
            default_value: None,
        });

        schema.insert("MUSEPAY_PRIVATE_KEY", EnvVarDefinition {
            objective: "MusePay private key for secure payment transaction signing",
            required: false,
            var_type: EnvVarType::PrivateKey,
            category: EnvCategory::Security,
            example: "-----BEGIN PRIVATE KEY-----\\nYour\\nPrivate\\nKey\\nHere\\n-----END PRIVATE KEY-----",
            default_value: None,
        });

        schema.insert("PAYMENT_WEBHOOK_URL", EnvVarDefinition {
            objective: "Payment webhook endpoint for payment status notifications",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Payment,
            example: "http://localhost:3000/api/v1/webhook/musepay",
            default_value: None,
        });

        // Firebase Configuration
        schema.insert("FIREBASE_TYPE", EnvVarDefinition {
            objective: "Firebase service account type for server-side authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "service_account",
            default_value: Some("service_account"),
        });

        schema.insert("FIREBASE_PROJECT_ID", EnvVarDefinition {
            objective: "Firebase project identifier for all Firebase services",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-project-id",
            default_value: None,
        });

        schema.insert("FIREBASE_API_KEY", EnvVarDefinition {
            objective: "Firebase API key for server-side Firebase operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-firebase-api-key",
            default_value: None,
        });

        schema.insert("FIREBASE_PRIVATE_KEY_ID", EnvVarDefinition {
            objective: "Firebase private key identifier for service account authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "your-private-key-id",
            default_value: None,
        });

        schema.insert("FIREBASE_PRIVATE_KEY", EnvVarDefinition {
            objective: "Firebase private key for secure server-side Firebase operations",
            required: false,
            var_type: EnvVarType::PrivateKey,
            category: EnvCategory::Security,
            example: "-----BEGIN PRIVATE KEY-----\\nYour\\nPrivate\\nKey\\nHere\\n-----END PRIVATE KEY-----",
            default_value: None,
        });

        schema.insert("FIREBASE_CLIENT_EMAIL", EnvVarDefinition {
            objective: "Firebase service account email for authentication",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Services,
            example: "your-service-account@your-project.iam.gserviceaccount.com",
            default_value: None,
        });

        schema.insert("FIREBASE_CLIENT_ID", EnvVarDefinition {
            objective: "Firebase client identifier for service account operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-client-id",
            default_value: None,
        });

        schema.insert("FIREBASE_AUTH_URI", EnvVarDefinition {
            objective: "Firebase OAuth authorization endpoint",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Services,
            example: "https://accounts.google.com/o/oauth2/auth",
            default_value: Some("https://accounts.google.com/o/oauth2/auth"),
        });

        schema.insert("FIREBASE_TOKEN_URI", EnvVarDefinition {
            objective: "Firebase OAuth token exchange endpoint",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Services,
            example: "https://oauth2.googleapis.com/token",
            default_value: Some("https://oauth2.googleapis.com/token"),
        });

        schema.insert("FIREBASE_AUTH_PROVIDER_CERT_URL", EnvVarDefinition {
            objective: "Firebase OAuth provider certificate URL for token verification",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Services,
            example: "https://www.googleapis.com/oauth2/v1/certs",
            default_value: Some("https://www.googleapis.com/oauth2/v1/certs"),
        });

        schema.insert("FIREBASE_CLIENT_CERT_URL", EnvVarDefinition {
            objective: "Firebase client certificate URL for service account verification",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Services,
            example: "https://www.googleapis.com/robot/v1/metadata/x509/your-service-account%40your-project.iam.gserviceaccount.com",
            default_value: None,
        });

        schema.insert("FIREBASE_UNIVERSE_DOMAIN", EnvVarDefinition {
            objective: "Firebase universe domain for service account operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "googleapis.com",
            default_value: Some("googleapis.com"),
        });

        schema.insert("FIREBASE_SERVICE_ACCOUNT_KEY", EnvVarDefinition {
            objective: "Firebase service account key JSON string for admin operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "{\"type\":\"service_account\",\"project_id\":\"your-project\"}",
            default_value: None,
        });

        schema.insert("FIREBASE_SERVICE_ACCOUNT_EMAIL", EnvVarDefinition {
            objective: "Firebase service account email for auth provider operations",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Services,
            example: "your-service-account@your-project.iam.gserviceaccount.com",
            default_value: None,
        });

        schema.insert("FIREBASE_AUTH_DOMAIN", EnvVarDefinition {
            objective: "Firebase auth domain for client-side Firebase authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-project.firebaseapp.com",
            default_value: None,
        });

        schema.insert("FIREBASE_STORAGE_BUCKET", EnvVarDefinition {
            objective: "Firebase storage bucket for file storage operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-project.appspot.com",
            default_value: None,
        });

        schema.insert("FIREBASE_MESSAGING_SENDER_ID", EnvVarDefinition {
            objective: "Firebase messaging sender ID for push notifications",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "123456789",
            default_value: None,
        });

        schema.insert("FIREBASE_APP_ID", EnvVarDefinition {
            objective: "Firebase app ID for client-side Firebase SDK initialization",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "1:123456789:web:abcdef123456",
            default_value: None,
        });

        schema.insert("FIREBASE_DOMAIN_HINT", EnvVarDefinition {
            objective: "Firebase domain hint for authentication flow optimization",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "your-domain.com",
            default_value: None,
        });

        // Cookie Security Configuration
        schema.insert("COOKIE_SIGNING_KEY", EnvVarDefinition {
            objective: "Cookie signing key for tamper-proof session cookies",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "your-signing-key-32-chars-minimum",
            default_value: None,
        });

        schema.insert("COOKIE_ENCRYPTION_KEY", EnvVarDefinition {
            objective: "Cookie encryption key for secure session data storage",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "your-encryption-key-64-chars-hex-minimum",
            default_value: None,
        });

        // Logging Configuration
        schema.insert("LOG_LEVEL", EnvVarDefinition {
            objective: "Application logging level for debug and monitoring",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Logging,
            example: "info",
            default_value: Some("info"),
        });

        schema.insert("RUST_LOG", EnvVarDefinition {
            objective: "Rust-specific logging configuration for tracing",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Logging,
            example: "info",
            default_value: Some("info"),
        });

        // OIDC Client Credentials
        schema.insert("OIDC_FRONTEND_CLIENT_ID", EnvVarDefinition {
            objective: "OIDC client identifier for frontend application authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "epsx-frontend",
            default_value: None,
        });

        schema.insert("OIDC_FRONTEND_CLIENT_SECRET", EnvVarDefinition {
            objective: "OIDC client secret for secure frontend authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "your-frontend-client-secret",
            default_value: None,
        });

        schema.insert("OIDC_ADMIN_CLIENT_ID", EnvVarDefinition {
            objective: "OIDC client identifier for admin application authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "epsx-admin",
            default_value: None,
        });

        schema.insert("OIDC_ADMIN_CLIENT_SECRET", EnvVarDefinition {
            objective: "OIDC client secret for secure admin authentication",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "your-admin-client-secret",
            default_value: None,
        });

        // OIDC Provider Configuration
        schema.insert("OIDC_ISSUER", EnvVarDefinition {
            objective: "OIDC provider issuer URL for token validation",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Authentication,
            example: "http://localhost:8080",
            default_value: Some("http://localhost:8080"),
        });

        schema.insert("OIDC_AUDIENCE", EnvVarDefinition {
            objective: "OIDC audience identifier for token validation",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "epsx-platform",
            default_value: Some("epsx-platform"),
        });

        schema.insert("OIDC_PRIVATE_KEY", EnvVarDefinition {
            objective: "RSA private key for OIDC token signing in production",
            required: false,
            var_type: EnvVarType::PrivateKey,
            category: EnvCategory::Security,
            example: "-----BEGIN PRIVATE KEY-----\\nMIIEvgIBADANBgkqhkiG9w0BA...\\n-----END PRIVATE KEY-----",
            default_value: None,
        });

        schema.insert("JWT_SECRET", EnvVarDefinition {
            objective: "JWT signing secret for development/legacy compatibility",
            required: false,
            var_type: EnvVarType::JwtSecret,
            category: EnvCategory::Authentication,
            example: "epsx-jwt-secret-2024-change-in-production",
            default_value: Some("epsx-jwt-secret-2024-change-in-production"),
        });

        schema.insert("OIDC_AUTHORIZATION_ENDPOINT", EnvVarDefinition {
            objective: "OIDC authorization endpoint for OAuth flows",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "/oauth/authorize",
            default_value: Some("/oauth/authorize"),
        });

        schema.insert("OIDC_TOKEN_ENDPOINT", EnvVarDefinition {
            objective: "OIDC token endpoint for OAuth token exchange",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "/oauth/token",
            default_value: Some("/oauth/token"),
        });

        schema.insert("OIDC_USERINFO_ENDPOINT", EnvVarDefinition {
            objective: "OIDC userinfo endpoint for user profile retrieval",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "/oauth/userinfo",
            default_value: Some("/oauth/userinfo"),
        });

        schema.insert("OIDC_JWKS_ENDPOINT", EnvVarDefinition {
            objective: "OIDC JWKS endpoint for public key retrieval",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Authentication,
            example: "/oauth/jwks",
            default_value: Some("/oauth/jwks"),
        });

        // Email & Notification Configuration
        schema.insert("EMAIL_FROM", EnvVarDefinition {
            objective: "Default from email address for system notifications",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Services,
            example: "noreply@epsx.com",
            default_value: Some("noreply@epsx.com"),
        });

        schema.insert("EMAIL_FROM_NAME", EnvVarDefinition {
            objective: "Default from name for system email notifications",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "EPSX Platform",
            default_value: Some("EPSX Platform"),
        });

        schema.insert("SENDGRID_API_KEY", EnvVarDefinition {
            objective: "SendGrid API key for email delivery service",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::External,
            example: "SG.your-sendgrid-api-key",
            default_value: None,
        });

        // Branding & Platform Configuration
        schema.insert("PLATFORM_NAME", EnvVarDefinition {
            objective: "Platform brand name for user-facing communications",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "EPSX",
            default_value: Some("EPSX"),
        });

        schema.insert("WELCOME_MESSAGE_TEMPLATE", EnvVarDefinition {
            objective: "Welcome message template for new user onboarding",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Services,
            example: "Welcome to {}!",
            default_value: Some("Welcome to {}!"),
        });

        schema.insert("DASHBOARD_URL", EnvVarDefinition {
            objective: "Dashboard URL for user navigation and email links",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "http://localhost:3000/dashboard",
            default_value: Some("http://localhost:3000/dashboard"),
        });

        schema.insert("SUPPORT_EMAIL", EnvVarDefinition {
            objective: "Support email address for user assistance",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Services,
            example: "support@epsx.com",
            default_value: Some("support@epsx.com"),
        });

        // Cache Configuration (Redis & In-Memory)
        schema.insert("REDIS_URL", EnvVarDefinition {
            objective: "Redis connection URL for caching and session storage",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Infrastructure,
            example: "redis://localhost:6379",
            default_value: None,
        });

        schema.insert("REDIS_POOL_SIZE", EnvVarDefinition {
            objective: "Redis connection pool size for concurrent operations",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Infrastructure,
            example: "10",
            default_value: Some("10"),
        });

        schema.insert("CACHE_TTL_SECONDS", EnvVarDefinition {
            objective: "Default cache time-to-live in seconds",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Infrastructure,
            example: "300",
            default_value: Some("300"),
        });

        schema.insert("CACHE_MAX_ENTRIES", EnvVarDefinition {
            objective: "Maximum number of entries in in-memory cache",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Infrastructure,
            example: "10000",
            default_value: Some("10000"),
        });

        schema.insert("CACHE_ENABLE_COMPRESSION", EnvVarDefinition {
            objective: "Enable compression for cached data to save memory",
            required: false,
            var_type: EnvVarType::Boolean,
            category: EnvCategory::Infrastructure,
            example: "false",
            default_value: Some("false"),
        });

        // Middleware Security Caching Configuration
        schema.insert("SECURITY_EVENT_CACHE_TTL", EnvVarDefinition {
            objective: "TTL for security event caching in Redis (seconds)",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Security,
            example: "86400",
            default_value: Some("86400"),
        });

        schema.insert("SESSION_VALIDATION_CACHE_TTL", EnvVarDefinition {
            objective: "TTL for session validation results in Redis (seconds)",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Security,
            example: "3600",
            default_value: Some("3600"),
        });

        schema.insert("PERMISSION_CACHE_TTL", EnvVarDefinition {
            objective: "TTL for user permission caching in Redis (seconds)",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Security,
            example: "300",
            default_value: Some("300"),
        });

        schema.insert("ADMIN_MODULE_CACHE_TTL", EnvVarDefinition {
            objective: "TTL for admin module assignment caching (seconds)",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Security,
            example: "1800",
            default_value: Some("1800"),
        });


        schema.insert("SECURITY_ALERT_WEBHOOK_URL", EnvVarDefinition {
            objective: "Webhook URL for critical security event notifications",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::Security,
            example: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL",
            default_value: None,
        });

        schema.insert("PERFORMANCE_MONITORING_ENABLED", EnvVarDefinition {
            objective: "Enable performance monitoring for middleware operations",
            required: false,
            var_type: EnvVarType::Boolean,
            category: EnvCategory::Infrastructure,
            example: "true",
            default_value: Some("true"),
        });

        schema.insert("MIDDLEWARE_EXECUTION_TIMEOUT_MS", EnvVarDefinition {
            objective: "Maximum execution time for middleware operations (milliseconds)",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::Infrastructure,
            example: "10000",
            default_value: Some("10000"),
        });

        // Rate Limiting Configuration
        schema.insert("RATE_LIMIT_DEFAULT_PER_MINUTE", EnvVarDefinition {
            objective: "Default rate limit per minute for API endpoints",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::RateLimiting,
            example: "60",
            default_value: Some("60"),
        });

        // TradingView External Service Configuration
        schema.insert("TRADINGVIEW_WEBSOCKET_URL", EnvVarDefinition {
            objective: "TradingView WebSocket URL for real-time market data",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::External,
            example: "wss://data.tradingview.com",
            default_value: Some("wss://data.tradingview.com"),
        });

        schema.insert("TRADINGVIEW_API_BASE_URL", EnvVarDefinition {
            objective: "TradingView API base URL for market data requests",
            required: false,
            var_type: EnvVarType::Url,
            category: EnvCategory::External,
            example: "https://api.tradingview.com",
            default_value: Some("https://api.tradingview.com"),
        });

        schema.insert("TRADINGVIEW_TIMEOUT_SECONDS", EnvVarDefinition {
            objective: "TradingView WebSocket connection timeout in seconds",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::External,
            example: "30",
            default_value: Some("30"),
        });

        schema.insert("TRADINGVIEW_HTTP_TIMEOUT_SECONDS", EnvVarDefinition {
            objective: "TradingView HTTP request timeout in seconds",
            required: false,
            var_type: EnvVarType::Number,
            category: EnvCategory::External,
            example: "30",
            default_value: Some("30"),
        });

        // Test User Configuration
        schema.insert("TEST_ADMIN_EMAIL", EnvVarDefinition {
            objective: "Test admin email for development environment testing",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Testing,
            example: "your-admin@example.com",
            default_value: None,
        });

        // IP Security Configuration
        schema.insert("SECURITY_IP_ALLOWLIST", EnvVarDefinition {
            objective: "Comma-separated list of allowed IP addresses/ranges for sensitive operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "127.0.0.1,10.0.0.0/8,192.168.0.0/16",
            default_value: Some(""),
        });

        schema.insert("SECURITY_ADMIN_IP_ALLOWLIST", EnvVarDefinition {
            objective: "Comma-separated list of IP addresses/ranges allowed for admin operations",
            required: false,
            var_type: EnvVarType::String,
            category: EnvCategory::Security,
            example: "127.0.0.1,10.0.0.0/8",
            default_value: Some(""),
        });

        schema
    };
}

// Validation functions
impl EnvVarType {
    fn validate(&self, value: &str) -> Result<(), String> {
        match self {
            EnvVarType::String => Ok(()),
            EnvVarType::Url => {
                if value.starts_with("http://") || value.starts_with("https://") {
                    Ok(())
                } else {
                    Err("Must be a valid URL starting with http:// or https://".to_string())
                }
            },
            EnvVarType::Number => {
                value.parse::<i32>()
                    .map(|_| ())
                    .map_err(|_| "Must be a valid number".to_string())
            },
            EnvVarType::Port => {
                match value.parse::<u16>() {
                    Ok(port) if port > 0 => Ok(()),
                    _ => Err("Must be a valid port number (1-65535)".to_string())
                }
            },
            EnvVarType::Boolean => {
                if ["true", "false"].contains(&value.to_lowercase().as_str()) {
                    Ok(())
                } else {
                    Err("Must be 'true' or 'false'".to_string())
                }
            },
            EnvVarType::Email => {
                if value.contains("@") && value.contains(".") {
                    Ok(())
                } else {
                    Err("Must be a valid email address".to_string())
                }
            },
            EnvVarType::JwtSecret => {
                if value.len() >= 32 {
                    Ok(())
                } else {
                    Err("JWT secret must be at least 32 characters for security".to_string())
                }
            },
            EnvVarType::DatabaseUrl => {
                if value.starts_with("postgresql://") || value.starts_with("postgres://") {
                    Ok(())
                } else {
                    Err("Must be a valid PostgreSQL connection string".to_string())
                }
            },
            EnvVarType::PrivateKey => {
                if value.contains("-----BEGIN PRIVATE KEY-----") {
                    Ok(())
                } else {
                    Err("Must be a valid private key in PEM format".to_string())
                }
            },
            EnvVarType::LogLevel => {
                if ["trace", "debug", "info", "warn", "error"].contains(&value.to_lowercase().as_str()) {
                    Ok(())
                } else {
                    Err("Must be a valid log level (trace, debug, info, warn, error)".to_string())
                }
            },
        }
    }
}

// Environment validation function
pub fn validate_environment() -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for (key, definition) in BACKEND_ENV_SCHEMA.iter() {
        match env::var(key) {
            Ok(value) => {
                // Validate the value format
                if let Err(reason) = definition.var_type.validate(&value) {
                    errors.push(ValidationError {
                        variable: key.to_string(),
                        objective: definition.objective.to_string(),
                        reason,
                        category: definition.category.clone(),
                        suggestion: format!("Ensure {} matches the expected format", key),
                        example: definition.example.to_string(),
                        severity: ErrorSeverity::Error,
                    });
                }
            },
            Err(_) => {
                if definition.required {
                    errors.push(ValidationError {
                        variable: key.to_string(),
                        objective: definition.objective.to_string(),
                        reason: "Required environment variable is missing".to_string(),
                        category: definition.category.clone(),
                        suggestion: format!("Add {}={} to your .env file", key, definition.example),
                        example: definition.example.to_string(),
                        severity: ErrorSeverity::Error,
                    });
                } else if definition.default_value.is_none() {
                    // Optional variable without default - just a warning
                    errors.push(ValidationError {
                        variable: key.to_string(),
                        objective: definition.objective.to_string(),
                        reason: "Optional environment variable is not set".to_string(),
                        category: definition.category.clone(),
                        suggestion: format!("Consider adding {}={} for full functionality", key, definition.example),
                        example: definition.example.to_string(),
                        severity: ErrorSeverity::Warning,
                    });
                }
            }
        }
    }

    if errors.iter().any(|e| e.severity == ErrorSeverity::Error) {
        Err(errors)
    } else {
        // Only warnings - log them but don't fail
        for warning in errors {
            eprintln!("⚠️  {}", warning);
        }
        println!("✅ Backend environment validation passed (with warnings)");
        Ok(())
    }
}

// Environment getter functions
pub fn get_env_var(key: &str) -> Result<String, String> {
    match BACKEND_ENV_SCHEMA.get(key) {
        Some(definition) => {
            match env::var(key) {
                Ok(value) => Ok(value),
                Err(_) => {
                    if let Some(default) = definition.default_value {
                        Ok(default.to_string())
                    } else if definition.required {
                        Err(format!("Required environment variable {} is not set", key))
                    } else {
                        Ok(String::new())
                    }
                }
            }
        },
        None => Err(format!("Unknown environment variable: {}", key))
    }
}

// Typed configuration structs
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub bind_address: String,
    pub frontend_url: String,
    pub admin_frontend_url: String,
    pub environment: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret_main: String,
    pub jwt_secret: String,
    pub cookie_signing_key: Option<String>,
    pub cookie_encryption_key: Option<String>,
    pub firebase_project_id: Option<String>,
    pub backend_url: String,
    pub oidc_issuer: String,
}

#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub musepay_partner_id: Option<String>,
    pub musepay_private_key: Option<String>,
    pub webhook_url: Option<String>,
    pub supported_currencies: Vec<String>,
    pub default_currency: String,
    pub default_checkout_url_template: String,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub from_email: String,
    pub from_name: String,
    pub sendgrid_api_key: String,
}

#[derive(Debug, Clone)]
pub struct BrandingConfig {
    pub platform_name: String,
    pub welcome_message_template: String,
    pub dashboard_url: String,
    pub support_email: String,
}

#[derive(Debug, Clone)]
pub struct ExternalServicesConfig {
    pub tradingview: TradingViewConfig,
    pub sendgrid_api_key: Option<String>,
    pub qr_code: QrCodeConfig,
}

#[derive(Debug, Clone)]
pub struct QrCodeConfig {
    pub enabled: bool,
    pub base_url: String,
    pub logo_url: Option<String>,
    pub api_base_url: String,
    pub default_size: u32,
}

#[derive(Debug, Clone)]
pub struct TradingViewConfig {
    pub websocket_url: String,
    pub api_base_url: String,
    pub timeout_seconds: u64,
    pub http_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimitingConfig {
    pub default_per_minute: u32,
    pub endpoint_specific: std::collections::HashMap<String, EndpointRateLimit>,
}

#[derive(Debug, Clone)]
pub struct EndpointRateLimit {
    pub per_minute: u32,
    pub burst: u32,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub redis_url: Option<String>,
    pub redis_pool_size: u32,
    pub default_ttl_seconds: i64,
    pub max_entries: Option<usize>,
    pub enable_compression: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub session_validation_cache_ttl: i64,
    pub permission_cache_ttl: i64,
    pub admin_module_cache_ttl: i64,
    pub performance_monitoring_enabled: bool,
    pub middleware_execution_timeout_ms: u64,
    pub ip_allowlist: Vec<String>,
    pub admin_ip_allowlist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FirebaseExtendedConfig {
    pub type_field: Option<String>,
    pub project_id: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: Option<String>,
    pub client_email: Option<String>,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
    pub auth_provider_cert_url: Option<String>,
    pub client_cert_url: Option<String>,
    pub universe_domain: Option<String>,
    pub api_key: Option<String>,
    pub service_account_key: Option<String>,
    pub service_account_email: Option<String>,
    pub auth_domain: Option<String>,
    pub storage_bucket: Option<String>,
    pub messaging_sender_id: Option<String>,
    pub app_id: Option<String>,
    pub domain_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub payment: PaymentConfig,
    pub email: EmailConfig,
    pub branding: BrandingConfig,
    pub external_services: ExternalServicesConfig,
    pub rate_limiting: RateLimitingConfig,
    pub cache: CacheConfig,
    pub security: SecurityConfig,
    pub firebase: FirebaseExtendedConfig,
}

// Load environment variables from .env file
pub fn load_env() {
    // Load app-specific variables only
    if let Err(e) = dotenv::dotenv() {
        eprintln!("Warning: Failed to load .env file: {}", e);
    }
}

// Configuration loader
pub fn load_validated_config() -> Result<ValidatedConfig, Vec<ValidationError>> {
    load_env();
    validate_environment()?;

    let server_config = ServerConfig {
        port: get_env_var("PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080),
        host: get_env_var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        bind_address: get_env_var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string()),
        frontend_url: get_env_var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
        admin_frontend_url: get_env_var("ADMIN_FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3001".to_string()),
        environment: get_env_var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
    };

    let database_config = DatabaseConfig {
        url: get_env_var("DATABASE_URL")
            .map_err(|e| vec![ValidationError {
                variable: "DATABASE_URL".to_string(),
                objective: "PostgreSQL connection for all database operations".to_string(),
                reason: e,
                category: EnvCategory::Database,
                suggestion: "Add DATABASE_URL=postgresql://postgres:password@localhost:5432/epsx_db to your .env file".to_string(),
                example: "postgresql://postgres:password@localhost:5432/epsx_db".to_string(),
                severity: ErrorSeverity::Error,
            }])?,
    };

    let auth_config = AuthConfig {
        jwt_secret_main: get_env_var("NEXTAUTH_SECRET")
            .map_err(|e| vec![ValidationError {
                variable: "NEXTAUTH_SECRET".to_string(),
                objective: "JWT token signing secret (shared with frontend applications)".to_string(),
                reason: e,
                category: EnvCategory::Authentication,
                suggestion: "Add NEXTAUTH_SECRET=epsx-shared-jwt-secret-2024-cross-app-authentication to your .env file".to_string(),
                example: "epsx-shared-jwt-secret-2024-cross-app-authentication".to_string(),
                severity: ErrorSeverity::Error,
            }])?,
        jwt_secret: get_env_var("NEXTAUTH_SECRET")
            .unwrap_or_else(|_| "default-jwt-secret".to_string()),
        cookie_signing_key: get_env_var("COOKIE_SIGNING_KEY").ok(),
        cookie_encryption_key: get_env_var("COOKIE_ENCRYPTION_KEY").ok(),
        firebase_project_id: get_env_var("FIREBASE_PROJECT_ID").ok(),
        backend_url: get_env_var("BACKEND_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        oidc_issuer: get_env_var("OIDC_ISSUER").unwrap_or_else(|_| "http://localhost:8080".to_string()),
    };

    let payment_config = PaymentConfig {
        musepay_partner_id: get_env_var("MUSEPAY_PARTNER_ID").ok(),
        musepay_private_key: get_env_var("MUSEPAY_PRIVATE_KEY").ok(),
        webhook_url: get_env_var("PAYMENT_WEBHOOK_URL").ok(),
        supported_currencies: vec!["USD".to_string(), "EUR".to_string(), "THB".to_string()],
        default_currency: "USD".to_string(),
        default_checkout_url_template: get_env_var("CHECKOUT_URL_TEMPLATE").unwrap_or_else(|_| "https://checkout.epsx.com/{}/pay".to_string()),
    };

    let email_config = EmailConfig {
        from_email: get_env_var("EMAIL_FROM").unwrap_or_else(|_| "noreply@epsx.com".to_string()),
        from_name: get_env_var("EMAIL_FROM_NAME").unwrap_or_else(|_| "EPSX Platform".to_string()),
        sendgrid_api_key: get_env_var("SENDGRID_API_KEY").unwrap_or_else(|_| "".to_string()),
    };

    let branding_config = BrandingConfig {
        platform_name: get_env_var("PLATFORM_NAME").unwrap_or_else(|_| "EPSX".to_string()),
        welcome_message_template: get_env_var("WELCOME_MESSAGE_TEMPLATE").unwrap_or_else(|_| "Welcome to {}!".to_string()),
        dashboard_url: get_env_var("DASHBOARD_URL").unwrap_or_else(|_| "http://localhost:3000/dashboard".to_string()),
        support_email: get_env_var("SUPPORT_EMAIL").unwrap_or_else(|_| "support@epsx.com".to_string()),
    };

    let tradingview_config = TradingViewConfig {
        websocket_url: get_env_var("TRADINGVIEW_WEBSOCKET_URL").unwrap_or_else(|_| "wss://data.tradingview.com".to_string()),
        api_base_url: get_env_var("TRADINGVIEW_API_BASE_URL").unwrap_or_else(|_| "https://api.tradingview.com".to_string()),
        timeout_seconds: get_env_var("TRADINGVIEW_TIMEOUT_SECONDS").unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
        http_timeout_seconds: get_env_var("TRADINGVIEW_HTTP_TIMEOUT_SECONDS").unwrap_or_else(|_| "30".to_string()).parse().unwrap_or(30),
    };

    let qr_code_config = QrCodeConfig {
        enabled: get_env_var("QR_CODE_ENABLED").unwrap_or_else(|_| "true".to_string()) == "true",
        base_url: get_env_var("QR_CODE_BASE_URL").unwrap_or_else(|_| "https://api.qrserver.com/v1/create-qr-code/".to_string()),
        logo_url: get_env_var("QR_CODE_LOGO_URL").ok(),
        api_base_url: get_env_var("QR_CODE_API_BASE_URL").unwrap_or_else(|_| "https://api.qrserver.com".to_string()),
        default_size: get_env_var("QR_CODE_DEFAULT_SIZE").unwrap_or_else(|_| "256".to_string()).parse().unwrap_or(256),
    };

    let external_services_config = ExternalServicesConfig {
        tradingview: tradingview_config,
        sendgrid_api_key: get_env_var("SENDGRID_API_KEY").ok(),
        qr_code: qr_code_config,
    };

    let rate_limiting_config = RateLimitingConfig {
        default_per_minute: get_env_var("RATE_LIMIT_DEFAULT_PER_MINUTE").unwrap_or_else(|_| "60".to_string()).parse().unwrap_or(60),
        endpoint_specific: std::collections::HashMap::new(), // Can be populated from env vars if needed
    };

    let cache_config = CacheConfig {
        redis_url: get_env_var("REDIS_URL").ok(),
        redis_pool_size: get_env_var("REDIS_POOL_SIZE").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10),
        default_ttl_seconds: get_env_var("CACHE_TTL_SECONDS").unwrap_or_else(|_| "300".to_string()).parse().unwrap_or(300),
        max_entries: get_env_var("CACHE_MAX_ENTRIES").ok().and_then(|s| s.parse().ok()),
        enable_compression: get_env_var("CACHE_ENABLE_COMPRESSION").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false),
    };

    let security_config = SecurityConfig {
        session_validation_cache_ttl: get_env_var("SESSION_VALIDATION_CACHE_TTL").unwrap_or_else(|_| "3600".to_string()).parse().unwrap_or(3600),
        permission_cache_ttl: get_env_var("PERMISSION_CACHE_TTL").unwrap_or_else(|_| "300".to_string()).parse().unwrap_or(300),
        admin_module_cache_ttl: get_env_var("ADMIN_MODULE_CACHE_TTL").unwrap_or_else(|_| "1800".to_string()).parse().unwrap_or(1800),
        performance_monitoring_enabled: get_env_var("PERFORMANCE_MONITORING_ENABLED").unwrap_or_else(|_| "true".to_string()).parse().unwrap_or(true),
        middleware_execution_timeout_ms: get_env_var("MIDDLEWARE_EXECUTION_TIMEOUT_MS").unwrap_or_else(|_| "10000".to_string()).parse().unwrap_or(10000),
        ip_allowlist: get_env_var("SECURITY_IP_ALLOWLIST")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        admin_ip_allowlist: get_env_var("SECURITY_ADMIN_IP_ALLOWLIST")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    };

    let firebase_config = FirebaseExtendedConfig {
        type_field: get_env_var("FIREBASE_TYPE").ok(),
        project_id: get_env_var("FIREBASE_PROJECT_ID").ok(),
        private_key_id: get_env_var("FIREBASE_PRIVATE_KEY_ID").ok(),
        private_key: get_env_var("FIREBASE_PRIVATE_KEY").ok(),
        client_email: get_env_var("FIREBASE_CLIENT_EMAIL").ok(),
        client_id: get_env_var("FIREBASE_CLIENT_ID").ok(),
        auth_uri: get_env_var("FIREBASE_AUTH_URI").ok(),
        token_uri: get_env_var("FIREBASE_TOKEN_URI").ok(),
        auth_provider_cert_url: get_env_var("FIREBASE_AUTH_PROVIDER_CERT_URL").ok(),
        client_cert_url: get_env_var("FIREBASE_CLIENT_CERT_URL").ok(),
        universe_domain: get_env_var("FIREBASE_UNIVERSE_DOMAIN").ok(),
        api_key: get_env_var("FIREBASE_API_KEY").ok(),
        service_account_key: get_env_var("FIREBASE_SERVICE_ACCOUNT_KEY").ok(),
        service_account_email: get_env_var("FIREBASE_SERVICE_ACCOUNT_EMAIL").ok(),
        auth_domain: get_env_var("FIREBASE_AUTH_DOMAIN").ok(),
        storage_bucket: get_env_var("FIREBASE_STORAGE_BUCKET").ok(),
        messaging_sender_id: get_env_var("FIREBASE_MESSAGING_SENDER_ID").ok(),
        app_id: get_env_var("FIREBASE_APP_ID").ok(),
        domain_hint: get_env_var("FIREBASE_DOMAIN_HINT").ok(),
    };

    Ok(ValidatedConfig {
        server: server_config,
        database: database_config,
        auth: auth_config,
        payment: payment_config,
        email: email_config,
        branding: branding_config,
        external_services: external_services_config,
        rate_limiting: rate_limiting_config,
        cache: cache_config,
        security: security_config,
        firebase: firebase_config,
    })
}

// Utility functions
pub fn is_production() -> bool {
    get_env_var("RUST_ENV").unwrap_or_default() == "production" ||
    get_env_var("ENV").unwrap_or_default() == "production"
}

pub fn is_development() -> bool {
    let env = get_env_var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    env == "development"
}

pub fn get_log_level() -> String {
    get_env_var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
}

// IP allowlist validation utilities
impl SecurityConfig {
    /// Check if an IP address is allowed based on the general allowlist
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        Self::check_ip_against_allowlist(ip, &self.ip_allowlist)
    }

    /// Check if an IP address is allowed for admin operations
    pub fn is_admin_ip_allowed(&self, ip: &str) -> bool {
        Self::check_ip_against_allowlist(ip, &self.admin_ip_allowlist)
    }

    /// Internal helper to check IP against a specific allowlist
    fn check_ip_against_allowlist(ip_str: &str, allowlist: &[String]) -> bool {
        // If allowlist is empty, allow all IPs
        if allowlist.is_empty() {
            return true;
        }

        // Parse the incoming IP address
        let ip = match ip_str.parse::<IpAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                tracing::warn!("Failed to parse IP address: {}", ip_str);
                return false;
            }
        };

        // Check against each entry in allowlist
        for allowed_entry in allowlist {
            if Self::ip_matches_entry(&ip, allowed_entry) {
                return true;
            }
        }

        false
    }

    /// Check if an IP matches a single allowlist entry (supports CIDR notation)
    fn ip_matches_entry(ip: &IpAddr, entry: &str) -> bool {
        let entry = entry.trim();

        // Handle CIDR notation (e.g., "192.168.0.0/16")
        if let Some((network_str, prefix_len_str)) = entry.split_once('/') {
            return Self::ip_matches_cidr(ip, network_str, prefix_len_str);
        }

        // Handle single IP addresses
        match entry.parse::<IpAddr>() {
            Ok(allowed_ip) => ip == &allowed_ip,
            Err(_) => {
                tracing::warn!("Invalid IP address in allowlist: {}", entry);
                false
            }
        }
    }

    /// Check if IP matches a CIDR range
    fn ip_matches_cidr(ip: &IpAddr, network_str: &str, prefix_len_str: &str) -> bool {
        let prefix_len = match prefix_len_str.parse::<u8>() {
            Ok(len) => len,
            Err(_) => {
                tracing::warn!("Invalid CIDR prefix length: {}", prefix_len_str);
                return false;
            }
        };

        let network = match network_str.parse::<IpAddr>() {
            Ok(addr) => addr,
            Err(_) => {
                tracing::warn!("Invalid network address in CIDR: {}", network_str);
                return false;
            }
        };

        match (ip, network) {
            (IpAddr::V4(ip4), IpAddr::V4(net4)) => {
                Self::ipv4_in_cidr(ip4, &net4, prefix_len)
            }
            (IpAddr::V6(ip6), IpAddr::V6(net6)) => {
                Self::ipv6_in_cidr(ip6, &net6, prefix_len)
            }
            _ => {
                // IP version mismatch
                false
            }
        }
    }

    /// Check if IPv4 address is in CIDR range
    fn ipv4_in_cidr(ip: &Ipv4Addr, network: &Ipv4Addr, prefix_len: u8) -> bool {
        if prefix_len > 32 {
            return false;
        }

        let ip_int = u32::from(*ip);
        let network_int = u32::from(*network);
        let mask = (!0u32) << (32 - prefix_len);

        (ip_int & mask) == (network_int & mask)
    }

    /// Check if IPv6 address is in CIDR range
    fn ipv6_in_cidr(ip: &Ipv6Addr, network: &Ipv6Addr, prefix_len: u8) -> bool {
        if prefix_len > 128 {
            return false;
        }

        let ip_bytes = ip.octets();
        let network_bytes = network.octets();

        let full_bytes = (prefix_len / 8) as usize;
        let remaining_bits = prefix_len % 8;

        // Check full bytes
        for i in 0..full_bytes {
            if ip_bytes[i] != network_bytes[i] {
                return false;
            }
        }

        // Check remaining bits if any
        if remaining_bits > 0 && full_bytes < 16 {
            let mask = 0xff << (8 - remaining_bits);
            if (ip_bytes[full_bytes] & mask) != (network_bytes[full_bytes] & mask) {
                return false;
            }
        }

        true
    }
}