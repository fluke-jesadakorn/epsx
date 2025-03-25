use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub musepay_partner_id: String,
    pub musepay_private_key: String,
    pub musepay_api_url: String,
    pub firebase_project_id: String,
    pub firebase_client_email: String,
    pub firebase_private_key: String,
    pub firebase_api_key: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Config {
            port: env
                ::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            frontend_url: env
                ::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            firebase_project_id: env::var("FIREBASE_PROJECT_ID").expect("FIREBASE_PROJECT_ID must be set"),
            firebase_client_email: env::var("FIREBASE_CLIENT_EMAIL").expect("FIREBASE_CLIENT_EMAIL must be set"),
            firebase_private_key: env::var("FIREBASE_PRIVATE_KEY").expect("FIREBASE_PRIVATE_KEY must be set"),
            firebase_api_key: env::var("FIREBASE_API_KEY").expect("FIREBASE_API_KEY must be set"),
            google_client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            google_client_secret: env
                ::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI must be set"),
            musepay_partner_id: env
                ::var("MUSEPAY_PARTNER_ID")
                .expect("MUSEPAY_PARTNER_ID must be set"),
            musepay_private_key: env
                ::var("MUSEPAY_PRIVATE_KEY")
                .expect("MUSEPAY_PRIVATE_KEY must be set"),
            musepay_api_url: env
                ::var("MUSEPAY_API_URL")
                .unwrap_or_else(|_| "https://api.musepay.io/v1/order/pay".to_string()),
        })
    }
}
