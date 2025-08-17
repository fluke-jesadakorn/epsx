// Backend Environment Configuration with Validation
// All environment variable access must go through this module

use std::collections::HashMap;
use std::env;
use std::fmt;

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

        // Test User Configuration
        schema.insert("TEST_ADMIN_EMAIL", EnvVarDefinition {
            objective: "Test admin email for development environment testing",
            required: false,
            var_type: EnvVarType::Email,
            category: EnvCategory::Testing,
            example: "your-admin@example.com",
            default_value: None,
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
    pub nextauth_secret: String,
    pub jwt_secret: String,
    pub cookie_signing_key: Option<String>,
    pub cookie_encryption_key: Option<String>,
    pub firebase_project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub musepay_partner_id: Option<String>,
    pub musepay_private_key: Option<String>,
    pub webhook_url: Option<String>,
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
pub struct ValidatedConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub payment: PaymentConfig,
    pub email: EmailConfig,
    pub branding: BrandingConfig,
    pub external_services: ExternalServicesConfig,
    pub rate_limiting: RateLimitingConfig,
}

// Configuration loader
pub fn load_validated_config() -> Result<ValidatedConfig, Vec<ValidationError>> {
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
        nextauth_secret: get_env_var("NEXTAUTH_SECRET")
            .map_err(|e| vec![ValidationError {
                variable: "NEXTAUTH_SECRET".to_string(),
                objective: "JWT token signing secret (shared with frontend applications)".to_string(),
                reason: e,
                category: EnvCategory::Authentication,
                suggestion: "Add NEXTAUTH_SECRET=epsx-shared-jwt-secret-2024-cross-app-authentication to your .env file".to_string(),
                example: "epsx-shared-jwt-secret-2024-cross-app-authentication".to_string(),
                severity: ErrorSeverity::Error,
            }])?,
        jwt_secret: get_env_var("NEXTAUTH_SECRET").unwrap_or_else(|_| "default-jwt-secret".to_string()),
        cookie_signing_key: get_env_var("COOKIE_SIGNING_KEY").ok(),
        cookie_encryption_key: get_env_var("COOKIE_ENCRYPTION_KEY").ok(),
        firebase_project_id: get_env_var("FIREBASE_PROJECT_ID").ok(),
    };

    let payment_config = PaymentConfig {
        musepay_partner_id: get_env_var("MUSEPAY_PARTNER_ID").ok(),
        musepay_private_key: get_env_var("MUSEPAY_PRIVATE_KEY").ok(),
        webhook_url: get_env_var("PAYMENT_WEBHOOK_URL").ok(),
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

    let external_services_config = ExternalServicesConfig {
        tradingview: tradingview_config,
    };

    let rate_limiting_config = RateLimitingConfig {
        default_per_minute: get_env_var("RATE_LIMIT_DEFAULT_PER_MINUTE").unwrap_or_else(|_| "60".to_string()).parse().unwrap_or(60),
        endpoint_specific: std::collections::HashMap::new(), // Can be populated from env vars if needed
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