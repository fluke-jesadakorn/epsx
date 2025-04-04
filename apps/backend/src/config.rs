use serde::Deserialize;
use serde_json;
use std::env;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub musepay_partner_id: String,
    pub musepay_private_key: String,
    pub musepay_api_url: String,
    pub firebase_service_account_path: String
}

impl Config {
    pub fn load_firebase_config(path: &str) -> (String, String, String) {
        use tracing::debug;
        
        // Load Firebase config file once and extract all values
        let firebase_config = fs::read_to_string(path)
            .expect(&format!("Failed to read Firebase config at {}", path));
        let firebase_config: serde_json::Value = serde_json::from_str(&firebase_config)
            .expect(&format!("Failed to parse Firebase config at {}", path));
            
        // Extract Firebase values
        let project_id = firebase_config["project_id"]
            .as_str()
            .expect("Missing project_id in Firebase config")
            .to_string();
        let client_email = firebase_config["client_email"]
            .as_str()
            .expect("Missing client_email in Firebase config")
            .to_string();
        let private_key = firebase_config["private_key"]
            .as_str()
            .expect("Missing private_key in Firebase config")
            .to_string();

        debug!("Successfully loaded Firebase configuration");
        (project_id, client_email, private_key)
    }

    pub fn from_env() -> Result<Self, env::VarError> {
        if let Err(e) = dotenv::dotenv() {
            panic!("Failed to load .env file: {}", e);
        }
        use tracing::debug;
        debug!("Loaded environment variables: {:?}", env::vars().collect::<Vec<_>>());

        Ok(Config {
            port: env
                ::var("PORT")
                .unwrap_or_else(|_| "3002".to_string())
                .parse()
                .unwrap_or(3001),
            frontend_url: env
                ::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            musepay_partner_id: env
                ::var("MUSEPAY_PARTNER_ID")
                .expect("MUSEPAY_PARTNER_ID must be set"),
            musepay_private_key: env
                ::var("MUSEPAY_PRIVATE_KEY")
                .expect("MUSEPAY_PRIVATE_KEY must be set"),
            musepay_api_url: env
                ::var("MUSEPAY_API_URL")
                .unwrap_or_else(|_| "https://api.musepay.io/v1/order/pay".to_string()),
            firebase_service_account_path: env
                ::var("FIREBASE_SERVICE_ACCOUNT_PATH")
                .unwrap_or_else(|_| "secret/firebase-service-account.json".to_string()),
        })
    }
}
