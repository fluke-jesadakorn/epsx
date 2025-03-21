use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub mongodb_uri: String,
    pub firebase_project_id: String,
    pub firebase_client_email: String,
    pub firebase_private_key: String,
    pub firebase_api_key: String,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Config {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .unwrap_or(3001),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            mongodb_uri: env::var("MONGODB_URI").unwrap_or_default(),
            firebase_project_id: env::var("FIREBASE_PROJECT_ID").unwrap_or_default(),
            firebase_client_email: env::var("FIREBASE_CLIENT_EMAIL").unwrap_or_default(),
            firebase_private_key: env::var("FIREBASE_PRIVATE_KEY").unwrap_or_default(),
            firebase_api_key: env::var("FIREBASE_API_KEY").unwrap_or_default(),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            google_client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set"),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| format!("{}/login", env::var("FRONTEND_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string()))),
        })
    }
}
