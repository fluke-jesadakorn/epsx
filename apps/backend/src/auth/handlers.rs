use crate::auth::AuthService;
use axum::{ 
    extract::{ Json, State, Query }, 
    http::{ StatusCode, HeaderMap, HeaderValue, header },
    response::{ IntoResponse, Response },
};
use serde::{ Deserialize, Serialize };
use tracing::{ info, error, warn };
use std::{collections::HashMap, sync::Mutex};
use lazy_static::lazy_static;

#[derive(Debug, Deserialize)]
pub struct EmailSignUpRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailSignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ClientType {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "mobile")]
    Mobile,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    token: String,
    user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_balance: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<i64>,
    redirect_url: String,
}

fn set_auth_cookies(
    token: &str,
    email: &str,
    role: &str,
    token_balance: i32,
    features: &Vec<String>,
    permissions: &Vec<String>,
    expires_in: i64,
) -> Vec<(HeaderValue, HeaderValue)> {
    let cookie_options = format!(
        "Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
        expires_in
    );

    let create_cookie = |name: &str, value: String| {
        (
            HeaderValue::from_static("set-cookie"),
            HeaderValue::from_str(&format!("{}={}; {}", name, value, cookie_options))
                .unwrap_or_else(|_| HeaderValue::from_static(""))
        )
    };

    vec![
        create_cookie("__session", token.to_string()),
        create_cookie("email", email.to_string()),
        create_cookie("role", role.to_string()),
        create_cookie("token_balance", token_balance.to_string()),
        create_cookie("features", serde_json::to_string(features).unwrap_or_default()),
        create_cookie("permissions", serde_json::to_string(permissions).unwrap_or_default()),
    ]
}

#[derive(Debug, Deserialize)]
pub struct OAuthInitRequest {
    redirect_url: Option<String>,
    oauth_redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthUrlResponse {
    url: String,
}

// In-memory storage for OAuth state

lazy_static! {
    static ref OAUTH_STATES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    code: String,
    state: String,
}

// Email sign up handler
pub async fn email_sign_up(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(request): Json<EmailSignUpRequest>,
) -> Result<Response, StatusCode> {
    info!("Processing sign up request for email: {}", request.email);

    // Sign up with email and password
    let firebase_response = auth_service.firebase
        .sign_up(&request.email, &request.password).await
        .map_err(|e| {
            error!("Sign up failed for email {}: {}", request.email, e);
            StatusCode::BAD_REQUEST
        })?;

    info!("User successfully registered with email: {}", request.email);

    let client_type = headers
        .get("X-Client-Type")
        .and_then(|value| value.to_str().ok())
        .and_then(|str| serde_json::from_str::<ClientType>(str).ok())
        .unwrap_or(ClientType::Web);

    let frontend_url = headers
        .get("X-Frontend-URL")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000")));
    let redirect_url = format!("{}/home", frontend_url);

    let token = firebase_response.id_token.clone();
    let user_id = firebase_response.local_id;
    let email = request.email.clone();

    let auth_response = AuthResponse {
        token: token.clone(),
        user_id,
        email: Some(email.clone()),
        role: Some("REGISTERED_USER".to_string()),
        token_balance: Some(0),
        features: Some(vec![]),
        permissions: Some(vec![]),
        expires_in: Some(3600),
        redirect_url: redirect_url.clone(),
    };

    let mut response = Response::builder()
        .status(match client_type {
            ClientType::Web => StatusCode::FOUND,
            ClientType::Mobile => StatusCode::OK,
        })
        .header(header::CONTENT_TYPE, "application/json");

    match client_type {
        ClientType::Web => {
            let cookies = set_auth_cookies(
                &token,
                &email,
                "REGISTERED_USER",
                0,
                &vec![],
                &vec![],
                3600,
            );
            
            for (_, value) in cookies {
                response = response.header(header::SET_COOKIE, value);
            }
            
            response = response.header(header::LOCATION, &redirect_url);
        }
        ClientType::Mobile => {}
    }

    Ok(response
        .body(serde_json::to_string(&auth_response).unwrap())
        .unwrap()
        .into_response())
}

// Email sign in handler
pub async fn email_sign_in(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(request): Json<EmailSignInRequest>,
) -> Result<Response, StatusCode> {
    info!("Processing sign in request for email: {}", request.email);

    // Sign in with email and password
    let firebase_response = auth_service.firebase
        .sign_in(&request.email, &request.password).await
        .map_err(|e| {
            warn!("Sign in failed for email {}: {}", request.email, e);
            StatusCode::UNAUTHORIZED
        })?;

    info!("User successfully signed in with email: {}", request.email);

    let client_type = headers
        .get("X-Client-Type")
        .and_then(|value| value.to_str().ok())
        .and_then(|str| serde_json::from_str::<ClientType>(str).ok())
        .unwrap_or(ClientType::Web);

    let frontend_url = headers
        .get("X-Frontend-URL")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000")));
    let redirect_url = format!("{}/home", frontend_url);

    let token = firebase_response.id_token.clone();
    let user_id = firebase_response.local_id;
    let email = request.email.clone();

    let auth_response = AuthResponse {
        token: token.clone(),
        user_id,
        email: Some(email.clone()),
        role: Some("REGISTERED_USER".to_string()),
        token_balance: Some(0),
        features: Some(vec![]),
        permissions: Some(vec![]),
        expires_in: Some(3600),
        redirect_url: redirect_url.clone(),
    };

    let mut response = Response::builder()
        .status(match client_type {
            ClientType::Web => StatusCode::FOUND,
            ClientType::Mobile => StatusCode::OK,
        })
        .header(header::CONTENT_TYPE, "application/json");

    match client_type {
        ClientType::Web => {
            let cookies = set_auth_cookies(
                &token,
                &email,
                "REGISTERED_USER",
                0,
                &vec![],
                &vec![],
                3600,
            );
            
            for (_, value) in cookies {
                response = response.header(header::SET_COOKIE, value);
            }
            
            response = response.header(header::LOCATION, &redirect_url);
        }
        ClientType::Mobile => {}
    }

    Ok(response
        .body(serde_json::to_string(&auth_response).unwrap())
        .unwrap()
        .into_response())
}

// Google OAuth initialization handler
pub async fn google_oauth_init(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Query(query): Query<OAuthInitRequest>
) -> Result<Json<OAuthUrlResponse>, StatusCode> {
    info!("Initializing Google OAuth flow");

    let frontend_url = headers
        .get("X-Frontend-URL")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000")));
    let redirect_url = query.redirect_url.unwrap_or_else(|| format!("{}/home", frontend_url));
    
    // Generate auth URL and store PKCE verifier state
    let (auth_url, oauth_state) = {
        let mut oauth = auth_service.google_oauth.lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        // Update OAuth redirect URI if provided
        if let Some(oauth_redirect_uri) = query.oauth_redirect_uri.clone() {
            oauth.update_redirect_uri(&oauth_redirect_uri)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
            
        let result = oauth.generate_auth_url(redirect_url.clone());
        drop(oauth); // Explicitly drop the guard
        result
    };

    // Store state and redirect URL mapping
    OAUTH_STATES.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(oauth_state.token.clone(), oauth_state.redirect_url);
    
    info!("Generated Google OAuth URL with state: {}", oauth_state.token);
    
    Ok(Json(OAuthUrlResponse { url: auth_url }))
}

#[axum::debug_handler]
pub async fn google_oauth_callback(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Query(params): Query<OAuthCallbackRequest>,
) -> Result<Response, StatusCode> {
    info!("Processing Google OAuth callback with code");
    
    // Verify state parameter to prevent CSRF
    if !params.state.starts_with("google_") {
        error!("Invalid state parameter in OAuth callback");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Clone components for each async operation
    let auth_service_google1 = auth_service.google_oauth.clone();
    let auth_service_google2 = auth_service.google_oauth.clone();
    let auth_service_firebase = auth_service.firebase.clone();
    let auth_service_store = auth_service.clone();
    
    // Exchange code for tokens
    let code = params.code.clone();
    let oauth_tokens = tokio::task::spawn_blocking(move || {
        let mut oauth = auth_service_google1.lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let rt = tokio::runtime::Handle::current();
        rt.block_on(oauth.exchange_code(&code))
            .map_err(|e| {
                error!("Failed to exchange OAuth code: {}", e);
                StatusCode::BAD_REQUEST
            })
    }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    // Get user info using the access token
    let access_token = oauth_tokens.access_token.clone();
    let user_info = tokio::task::spawn_blocking(move || {
        let oauth = auth_service_google2.lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let rt = tokio::runtime::Handle::current();
        rt.block_on(oauth.get_user_info(&access_token))
            .map_err(|e| {
            error!("Failed to get user info: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
            })
    }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    let expires_in = oauth_tokens.expires_in;
    let user_email = user_info.email.to_owned();
    let user_sub = user_info.sub.to_owned();

    // Create or update Firebase custom token
    let token = auth_service_firebase
        .create_custom_token(&user_sub, user_email.clone())
        .await
        .map_err(|e| {
            error!("Failed to create custom token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Store refresh token if available
    if let Some(ref refresh_token) = oauth_tokens.refresh_token {
        auth_service_store.store_refresh_token(&user_sub, &refresh_token).await
        .map_err(|e| {
            error!("Failed to store refresh token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    info!("Successfully authenticated Google user: {}", user_email);

    let client_type = headers
        .get("X-Client-Type")
        .and_then(|value| value.to_str().ok())
        .and_then(|str| serde_json::from_str::<ClientType>(str).ok())
        .unwrap_or(ClientType::Web);

    let frontend_url = headers
        .get("X-Frontend-URL")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000")));
    
    // Get stored redirect URL from state
    // Get stored redirect URL from state
    let stored_redirect_url = OAUTH_STATES.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&params.state[7..]) // Remove "google_" prefix
        .unwrap_or_else(|| format!("{}/home", frontend_url));

    let auth_response = AuthResponse {
        token: token.clone(),
        user_id: user_sub.clone(),
        email: Some(user_email.clone()),
        role: Some("REGISTERED_USER".to_string()),
        token_balance: Some(0),
        features: Some(vec![]),
        permissions: Some(vec![]),
        expires_in: expires_in.map(|e| e as i64).or(Some(3600)),
        redirect_url: stored_redirect_url.clone(),
    };

    let mut response = Response::builder()
        .status(match client_type {
            ClientType::Web => StatusCode::FOUND,
            ClientType::Mobile => StatusCode::OK,
        })
        .header(header::CONTENT_TYPE, "application/json");

    match client_type {
        ClientType::Web => {
            let cookies = set_auth_cookies(
                &token,
                &user_email,
                "REGISTERED_USER",
                0,
                &vec![],
                &vec![],
                expires_in.unwrap_or(3600) as i64,
            );
            
            for (_, value) in cookies {
                response = response.header(header::SET_COOKIE, value);
            }
            
            response = response.header(header::LOCATION, &stored_redirect_url);
        }
        ClientType::Mobile => {}
    }

    Ok(response
        .body(serde_json::to_string(&auth_response).unwrap())
        .unwrap()
        .into_response())
}

// Error response handler
pub async fn logout() -> Response {
    info!("Processing logout request");
    
    let cookie_options = "Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0";
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    // Clear all auth cookies by setting them to expire immediately
    let cookies = vec![
        "__session",
        "email",
        "role",
        "token_balance",
        "features",
        "permissions"
    ];
    
    for cookie_name in cookies {
        response = response.header(
            header::SET_COOKIE,
            format!("{}=; {}", cookie_name, cookie_options)
        );
    }
    
    response
        .body(serde_json::json!({ "message": "Logged out successfully" }).to_string())
        .unwrap()
        .into_response()
}

pub fn handle_auth_error(err: anyhow::Error) -> Response {
    error!("Authentication error occurred: {}", err);
    (StatusCode::INTERNAL_SERVER_ERROR, format!("Authentication error: {}", err)).into_response()
}
