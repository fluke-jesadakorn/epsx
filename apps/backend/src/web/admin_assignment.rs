use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::{Value, json};
use tracing::{info, warn, error};
use reqwest::Client;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use chrono::Utc;
use crate::config::env::get_env_var;

use crate::web::auth::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminAssignmentRequest {
    pub role: String,
    pub custom_claims: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct AdminAssignmentResponse {
    pub success: bool,
    pub message: String,
    pub user_id: String,
    pub assigned_role: String,
    pub custom_claims: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct GoogleOAuthClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: i64,
    exp: i64,
}

/// Generate Google OAuth2 access token using service account
async fn get_google_access_token() -> Result<String, Box<dyn std::error::Error>> {
    let client_email = get_env_var("FIREBASE_CLIENT_EMAIL")?;
    let private_key = get_env_var("FIREBASE_PRIVATE_KEY")?;
    
    // Clean up the private key (remove header/footer and normalize whitespace)
    let private_key = private_key
        .replace("-----BEGIN PRIVATE KEY-----", "")
        .replace("-----END PRIVATE KEY-----", "")
        .replace("\\n", "\n")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    
    let private_key_bytes = STANDARD.decode(&private_key)?;
    let private_key_pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----", 
        STANDARD.encode(&private_key_bytes)
            .chars()
            .collect::<Vec<char>>()
            .chunks(64)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    );
    
    let now = Utc::now().timestamp();
    let claims = GoogleOAuthClaims {
        iss: client_email,
        scope: "https://www.googleapis.com/auth/identitytoolkit".to_string(),
        aud: "https://oauth2.googleapis.com/token".to_string(),
        iat: now,
        exp: now + 3600, // 1 hour
    };
    
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
    let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)?;
    
    // Exchange JWT for access token
    let client = Client::new();
    let response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        let token_response: serde_json::Value = response.json().await?;
        if let Some(access_token) = token_response["access_token"].as_str() {
            Ok(access_token.to_string())
        } else {
            Err("No access token in response".into())
        }
    } else {
        let error_text = response.text().await?;
        Err(format!("Token exchange failed: {}", error_text).into())
    }
}

/// Set custom claims for a Firebase user using proper Admin SDK
async fn set_firebase_custom_claims(
    user_id: &str,
    custom_claims: &HashMap<String, Value>,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_id = get_env_var("FIREBASE_PROJECT_ID")?;
    let access_token = get_google_access_token().await?;
    
    let client = Client::new();
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
        project_id
    );
    
    let request_body = json!({
        "localId": user_id,
        "customClaims": serde_json::to_string(custom_claims)?
    });
    
    tracing::info!("Setting custom claims for user {} with access token", user_id);
    
    let response = client
        .post(&url)
        .bearer_auth(&access_token)
        .json(&request_body)
        .send()
        .await?;
    
    if response.status().is_success() {
        tracing::info!("Successfully set custom claims for user {}", user_id);
        Ok(())
    } else {
        let error_text = response.text().await?;
        tracing::error!("Failed to set custom claims: {}", error_text);
        Err(format!("Failed to set custom claims: {}", error_text).into())
    }
}

/// Assign admin role to a Firebase user
pub async fn assign_admin_role_handler(
    State(_app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<AdminAssignmentRequest>,
) -> Result<Json<AdminAssignmentResponse>, StatusCode> {
    tracing::info!("Admin assignment request for user {} with role {}", user_id, request.role);
    
    // Create admin custom claims - using structured permissions
    let mut custom_claims = HashMap::new();
    custom_claims.insert("admin".to_string(), Value::Bool(true));
    custom_claims.insert("access_level".to_string(), Value::String("full".to_string()));
    custom_claims.insert("permissions".to_string(), Value::Array(vec![
        Value::String("admin:*:*".to_string()),
        Value::String("epsx:*:*".to_string()),
        Value::String("system_admin".to_string()),
        Value::String("module_management".to_string()),
        Value::String("database_access".to_string()),
        Value::String("developer_portal".to_string()),
    ]));
    
    // Add any additional custom claims from request
    if let Some(additional_claims) = &request.custom_claims {
        for (key, value) in additional_claims {
            custom_claims.insert(key.clone(), value.clone());
        }
    }
    
    // Set the custom claims via Firebase Admin SDK
    match set_firebase_custom_claims(&user_id, &custom_claims).await {
        Ok(()) => {
            tracing::info!("Successfully assigned admin role to user {}", user_id);
            
            let response = AdminAssignmentResponse {
                success: true,
                message: "Admin role assigned successfully".to_string(),
                user_id: user_id.clone(),
                assigned_role: request.role,
                custom_claims,
            };
            
            Ok(Json(response))
        },
        Err(e) => {
            tracing::error!("Failed to assign admin role to user {}: {}", user_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user's current custom claims
pub async fn get_user_claims_handler(
    State(_app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<HashMap<String, Value>>, StatusCode> {
    let api_key = get_env_var("FIREBASE_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let client = Client::new();
    let url = format!(
        "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
        api_key
    );
    
    let request_body = json!({
        "localId": [user_id]
    });
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if response.status().is_success() {
        let user_response: serde_json::Value = response.json().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        if let Some(users) = user_response["users"].as_array() {
            if let Some(user) = users.first() {
                if let Some(custom_claims_str) = user["customClaims"].as_str() {
                    let custom_claims: HashMap<String, Value> = 
                        serde_json::from_str(custom_claims_str)
                            .unwrap_or_default();
                    return Ok(Json(custom_claims));
                }
            }
        }
        
        // Return empty claims if none found
        Ok(Json(HashMap::new()))
    } else {
        tracing::error!("Failed to get user claims for {}", user_id);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}