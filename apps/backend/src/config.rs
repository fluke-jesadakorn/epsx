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
    pub firebase_service_account_path: String,
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
            firebase_service_account_path: env::var("FIREBASE_SERVICE_ACCOUNT_PATH")
                .expect("FIREBASE_SERVICE_ACCOUNT_PATH must be set"),
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
