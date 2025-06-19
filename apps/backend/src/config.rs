use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub musepay_partner_id: String,
    pub musepay_private_key: String,
    pub musepay_api_url: String,
    pub firebase_type: String,
    pub firebase_project_id: String,
    pub firebase_private_key_id: String,
    pub firebase_private_key: String,
    pub firebase_client_email: String,
    pub firebase_client_id: String,
    pub firebase_auth_uri: String,
    pub firebase_token_uri: String,
    pub firebase_auth_provider_cert_url: String,
    pub firebase_client_cert_url: String,
    pub firebase_universe_domain: String,
    pub tradingview_auth_token: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();

        Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a number"),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            musepay_partner_id: env::var("MUSEPAY_PARTNER_ID")
                .expect("MUSEPAY_PARTNER_ID must be set"),
            musepay_private_key: env::var("MUSEPAY_PRIVATE_KEY")
                .expect("MUSEPAY_PRIVATE_KEY must be set"),
            musepay_api_url: env::var("MUSEPAY_API_URL")
                .unwrap_or_else(|_| "https://api.musepay.com".to_string()),
            firebase_type: env::var("FIREBASE_TYPE")
                .expect("FIREBASE_TYPE must be set"),
            firebase_project_id: env::var("FIREBASE_PROJECT_ID")
                .expect("FIREBASE_PROJECT_ID must be set"),
            firebase_private_key_id: env::var("FIREBASE_PRIVATE_KEY_ID")
                .expect("FIREBASE_PRIVATE_KEY_ID must be set"),
            firebase_private_key: env::var("FIREBASE_PRIVATE_KEY")
                .expect("FIREBASE_PRIVATE_KEY must be set"),
            firebase_client_email: env::var("FIREBASE_CLIENT_EMAIL")
                .expect("FIREBASE_CLIENT_EMAIL must be set"),
            firebase_client_id: env::var("FIREBASE_CLIENT_ID")
                .expect("FIREBASE_CLIENT_ID must be set"),
            firebase_auth_uri: env::var("FIREBASE_AUTH_URI")
                .expect("FIREBASE_AUTH_URI must be set"),
            firebase_token_uri: env::var("FIREBASE_TOKEN_URI")
                .expect("FIREBASE_TOKEN_URI must be set"),
            firebase_auth_provider_cert_url: env::var("FIREBASE_AUTH_PROVIDER_CERT_URL")
                .expect("FIREBASE_AUTH_PROVIDER_CERT_URL must be set"),
            firebase_client_cert_url: env::var("FIREBASE_CLIENT_CERT_URL")
                .expect("FIREBASE_CLIENT_CERT_URL must be set"),
            firebase_universe_domain: env::var("FIREBASE_UNIVERSE_DOMAIN")
                .expect("FIREBASE_UNIVERSE_DOMAIN must be set"),
            tradingview_auth_token: env::var("TRADINGVIEW_AUTH_TOKEN")
                .expect("TRADINGVIEW_AUTH_TOKEN must be set"),
        }
    }

    pub fn host(&self) -> String {
        env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
