pub mod handlers;
pub mod routes;

use axum::Router;
use crate::{config::Config, auth::AuthService};

pub fn payment_router(payment_service: PaymentService, auth_service: AuthService) -> Router {
    routes::router(payment_service, auth_service)
}

#[derive(Clone)]
pub struct PaymentService {
    musepay_client: MusePayClient,
}

impl PaymentService {
    pub fn new(config: Config) -> Result<Self, anyhow::Error> {
        let musepay_client = MusePayClient::new(
            &config.musepay_partner_id,
            &config.musepay_private_key,
            &config.musepay_api_url,
        )?;
        Ok(Self {
            musepay_client,
        })
    }
}

#[derive(Clone)]
struct MusePayClient {
    partner_id: String,
    private_key: String,
    api_url: String,
    client: reqwest::Client,
}

impl MusePayClient {
    pub fn new(partner_id: &str, private_key: &str, api_url: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            partner_id: partner_id.to_string(),
            private_key: private_key.to_string(),
            api_url: api_url.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn create_payment(
        &self,
        request_id: &str, 
        currency: &str,
        amount: &str,
        payment_method: &str,
        product_name: &str,
        email: &str,
        notify_url: Option<&str>
    ) -> Result<serde_json::Value, anyhow::Error> {
        // Build initial parameters
        let mut params = serde_json::Map::new();
        params.insert("partner_id".to_string(), serde_json::Value::String(self.partner_id.clone()));
        params.insert("sign_type".to_string(), serde_json::Value::String("RSA".to_string()));
        params.insert("timestamp".to_string(), serde_json::Value::String(chrono::Utc::now().timestamp_millis().to_string()));
        params.insert("nonce".to_string(), serde_json::Value::String(uuid::Uuid::new_v4().to_string()));
        params.insert("request_id".to_string(), serde_json::Value::String(request_id.to_string()));
        params.insert("currency".to_string(), serde_json::Value::String(currency.to_string()));
        params.insert("amount".to_string(), serde_json::Value::String(amount.to_string()));
        params.insert("payment_method".to_string(), serde_json::Value::String(payment_method.to_string()));
        params.insert("product_name".to_string(), serde_json::Value::String(product_name.to_string()));
        params.insert("email".to_string(), serde_json::Value::String(email.to_string()));
        
        if let Some(url) = notify_url {
            params.insert("notify_url".to_string(), serde_json::Value::String(url.to_string()));
        }

        // Create Value from Map for signing
        let params_value = serde_json::Value::Object(params.clone());
        
        // Generate signature
        let content = self.build_sign_content(&params_value)?;
        let signature = self.sign(&content)?;
        
        // Add signature to params
        params.insert("sign".to_string(), serde_json::Value::String(signature));

        // Make the request
        let response = self.client
            .post(&self.api_url)
            .json(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    fn build_sign_content(&self, params: &serde_json::Value) -> Result<String, anyhow::Error> {
        let mut map = params.as_object().unwrap().clone();
        map.remove("sign");
        
        let mut pairs: Vec<_> = map.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        
        let content = pairs.into_iter()
            .filter(|(_, v)| !v.is_null())
            .map(|(k, v)| format!("{}={}", k, v.as_str().unwrap_or_default()))
            .collect::<Vec<_>>()
            .join("&");
            
        Ok(content)
    }

    fn sign(&self, content: &str) -> Result<String, anyhow::Error> {
        use openssl::{hash::MessageDigest, pkey::PKey, rsa::Rsa, sign::Signer};
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

        // Parse private key
        let pem = format!("-----BEGIN PRIVATE KEY-----{}-----END PRIVATE KEY-----", self.private_key);
        let rsa = Rsa::private_key_from_pem(pem.as_bytes())?;
        let key = PKey::from_rsa(rsa)?;

        // Create signer with SHA256
        let mut signer = Signer::new(MessageDigest::sha256(), &key)?;

        // Sign the content
        signer.update(content.as_bytes())?;
        let signature = signer.sign_to_vec()?;
        
        // Convert to base64
        Ok(BASE64.encode(signature))
    }
}
