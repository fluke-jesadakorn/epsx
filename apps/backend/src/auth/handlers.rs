use crate::auth::AuthService;
use axum::{
    extract::{ Json, State, Query },
    http::{ StatusCode, HeaderMap, HeaderValue, header },
    response::{ IntoResponse, Response },
};
use serde::{ Deserialize, Serialize };
use tracing::{ info, error, warn };
use std::{ collections::HashMap, sync::Mutex };
use lazy_static::lazy_static;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "email": "user@example.com",
    "password": "securepassword123"
}))]
pub struct EmailSignUpRequest {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "securepassword123", min_length = 8)]
    password: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "email": "user@example.com",
    "password": "securepassword123"
}))]
pub struct EmailSignInRequest {
    #[schema(example = "user@example.com")]
    email: String,
    #[schema(example = "securepassword123", min_length = 8)]
    password: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub enum ClientType {
    #[serde(rename = "web")]
    Web,
    #[serde(rename = "mobile")]
    Mobile,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(
    example = json!({
    "token": "eyJhbGciOiJSUzI1NiIsImtpZCI6Ij...",
    "user_id": "abc123",
    "email": "user@example.com",
    "role": "REGISTERED_USER",
    "token_balance": 0,
    "features": [],
    "permissions": [],
    "expires_in": 3600,
    "redirect_url": "http://localhost:3000/home"
})
)]
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
    expires_in: i64
) -> Vec<(HeaderValue, HeaderValue)> {
    let is_production = std::env
        ::var("ENV")
        .map(|v| v == "production")
        .unwrap_or(false);
    let secure_flag = if is_production { "Secure; " } else { "" };
    let same_site = if is_production { "SameSite=Lax" } else { "SameSite=Lax" };
    let domain = std::env::var("COOKIE_DOMAIN").unwrap_or_else(|_| String::from("localhost"));
    let cookie_options = format!(
        "Path=/; Domain={}; HttpOnly; {}{}; Max-Age={}",
        domain,
        secure_flag,
        same_site,
        expires_in
    );

    let create_cookie = |name: &str, value: String| {
        (
            HeaderValue::from_static("set-cookie"),
            HeaderValue::from_str(
                &format!("{}={}; {}", name, value, cookie_options)
            ).unwrap_or_else(|_| HeaderValue::from_static("")),
        )
    };

    vec![
        create_cookie("__session", token.to_string()),
        create_cookie("email", email.to_string()),
        create_cookie("role", role.to_string()),
        create_cookie("token_balance", token_balance.to_string()),
        create_cookie("features", serde_json::to_string(features).unwrap_or_default()),
        create_cookie("permissions", serde_json::to_string(permissions).unwrap_or_default())
    ]
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "redirect_url": "http://localhost:3000/home"
}))]
pub struct OAuthInitRequest {
    #[schema(example = "http://localhost:3000/home")]
    redirect_url: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "url": "https://accounts.google.com/o/oauth2/v2/auth?..."
}))]
pub struct OAuthUrlResponse {
    #[schema(example = "https://accounts.google.com/o/oauth2/v2/auth?...")]
    url: String,
}

// In-memory storage for OAuth state

lazy_static! {
    static ref OAUTH_STATES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "code": "4/0AeaYSH...",
    "state": "google_abc123"
}))]
pub struct OAuthCallbackRequest {
    #[schema(example = "4/0AeaYSH...")]
    code: String,
    #[schema(example = "google_abc123")]
    state: String,
}

/// Register a new user with email and password
///
/// Creates a new user account with the provided credentials.
/// Returns authentication tokens and user information.
#[utoipa::path(
    post,
    path = "/v1/auth/register",
    request_body = EmailSignUpRequest,
    responses(
        (
            status = 302,
            description = "Redirect to frontend with auth cookies set (web client)",
            body = AuthResponse,
        ),
        (
            status = 200,
            description = "Returns auth tokens directly (mobile client)",
            body = AuthResponse,
        ),
        (status = 400, description = "Invalid request or email already exists")
    ),
    tag = "Auth"
)]
pub async fn email_sign_up(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(request): Json<EmailSignUpRequest>
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
        .unwrap_or_else(||
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000"))
        );
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
                3600
            );

            for (_, value) in cookies {
                response = response.header(header::SET_COOKIE, value);
            }

            response = response.header(header::LOCATION, &redirect_url);
        }
        ClientType::Mobile => {}
    }

    Ok(response.body(serde_json::to_string(&auth_response).unwrap()).unwrap().into_response())
}

/// Authenticate user with email and password
///
/// Validates credentials and returns authentication tokens.
#[utoipa::path(
    post,
    path = "/v1/auth/login",
    request_body = EmailSignInRequest,
    responses(
        (
            status = 302,
            description = "Redirect to frontend with auth cookies set (web client)",
            body = AuthResponse,
        ),
        (
            status = 200,
            description = "Returns auth tokens directly (mobile client)",
            body = AuthResponse,
        ),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "Auth"
)]
pub async fn email_sign_in(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Json(request): Json<EmailSignInRequest>
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
        .unwrap_or_else(||
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000"))
        );
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
                3600
            );

            for (_, value) in cookies {
                response = response.header(header::SET_COOKIE, value);
            }

            response = response.header(header::LOCATION, &redirect_url);
        }
        ClientType::Mobile => {}
    }

    Ok(response.body(serde_json::to_string(&auth_response).unwrap()).unwrap().into_response())
}

/// Initialize Google OAuth flow
///
/// Returns URL to redirect user to for Google authentication.
#[utoipa::path(
    get,
    path = "/v1/auth/google/init",
    params(
        ("redirect_url" = Option<String>, Query, description = "Frontend URL to redirect to after auth")
    ),
    responses(
        (status = 200, description = "Returns Google auth URL", body = OAuthUrlResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
#[axum::debug_handler]
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
        .unwrap_or_else(||
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000"))
        );

    // Store the frontend destination URL for after authentication
    let frontend_destination = query.redirect_url.unwrap_or_else(||
        format!("{}/home", frontend_url)
    );

    // Generate auth URL and store PKCE verifier state
    let (auth_url, oauth_state) = {
        let oauth = auth_service.google_oauth
            .lock()
            .await;
        
        // Generate auth URL with PKCE using the configured redirect URI
        oauth.generate_auth_url(frontend_destination.clone()).await
    };

    // Store state and redirect URL mapping
    OAUTH_STATES.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(oauth_state.token.clone(), oauth_state.redirect_url);

    info!("Generated Google OAuth URL with state: {}", oauth_state.token);

    Ok(Json(OAuthUrlResponse { url: auth_url }))
}

/// Google OAuth callback handler
///
/// Handles the callback from Google OAuth flow, exchanges code for tokens,
/// and redirects to frontend with authentication cookies.
#[utoipa::path(
    get,
    path = "/v1/auth/google/callback",
    params(
        ("code" = String, Query, description = "Authorization code from Google"),
        ("state" = String, Query, description = "State parameter for CSRF protection")
    ),
    responses(
        (
            status = 302,
            description = "Redirects to frontend with auth cookies set",
            body = AuthResponse,
        ),
        (status = 400, description = "Invalid state parameter"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
#[axum::debug_handler]
pub async fn google_oauth_callback(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    Query(params): Query<OAuthCallbackRequest>
) -> Result<Response, StatusCode> {
    info!("Processing Google OAuth callback with code");

    // Verify state parameter to prevent CSRF
    if !params.state.starts_with("google_") {
        error!("Invalid state parameter in OAuth callback");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get a single lock on the GoogleOAuth instance for the entire callback
    let oauth = auth_service.google_oauth.lock().await;
    let auth_service_firebase = auth_service.firebase.clone();
    let auth_service_store = auth_service.clone();

    // Exchange code for tokens using the same instance that generated auth URL
    let code = params.code.clone();
    let oauth_tokens = oauth.exchange_code(&code).await
        .map_err(|e| {
            error!("Failed to exchange OAuth code: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // Get user info using the access token
    let access_token = oauth_tokens.access_token.clone();
    let user_info = oauth.get_user_info(&access_token).await
        .map_err(|e| {
            error!("Failed to get user info: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let expires_in = oauth_tokens.expires_in;
    let user_email = user_info.email.to_owned();
    let user_sub = user_info.sub.to_owned();

    // Create or update Firebase custom token
    let token = auth_service_firebase
        .create_custom_token(&user_sub, user_email.clone()).await
        .map_err(|e| {
            error!("Failed to create custom token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Store refresh token if available
    if let Some(ref refresh_token) = oauth_tokens.refresh_token {
        auth_service_store.store_refresh_token(&user_sub, &refresh_token).await.map_err(|e| {
            error!("Failed to store refresh token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    info!("Successfully authenticated Google user: {}", user_email);

    let frontend_url = headers
        .get("X-Frontend-URL")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(||
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| String::from("http://localhost:3000"))
        );

    // Get stored redirect URL from state
    let frontend_destination = OAUTH_STATES.lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .remove(&params.state[7..]) // Remove "google_" prefix
        .unwrap_or_else(|| format!("{}/home", frontend_url));

    // Create auth response
    let auth_response = AuthResponse {
        token: token.clone(),
        user_id: user_sub,
        email: Some(user_email.clone()),
        role: Some("REGISTERED_USER".to_string()),
        token_balance: Some(0),
        features: Some(vec![]),
        permissions: Some(vec![]),
        expires_in: expires_in.map(|e| e as i64).or(Some(3600)),
        redirect_url: frontend_destination.clone(),
    };

    // Set cookies and redirect to frontend
    let mut response = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::CONTENT_TYPE, "application/json");

    // Set auth cookies
    let cookies = set_auth_cookies(
        &token,
        &user_email,
        "REGISTERED_USER",
        0,
        &vec![],
        &vec![],
        expires_in.unwrap_or(3600) as i64
    );

    for (_, value) in cookies {
        response = response.header(header::SET_COOKIE, value);
    }

    // Redirect to frontend
    response = response.header(header::LOCATION, &frontend_destination);

    Ok(response.body(serde_json::to_string(&auth_response).unwrap()).unwrap().into_response())
}

/// Logout handler
///
/// Clears all authentication cookies and returns success message.
#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    responses((
        status = 200,
        description = "Successfully logged out",
        body = String,
        example = json!({"message": "Logged out successfully"}),
    )),
    tag = "Auth"
)]
pub async fn logout() -> Response {
    info!("Processing logout request");

    let cookie_options = "Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0";
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");

    // Clear all auth cookies by setting them to expire immediately
    let cookies = vec!["__session", "email", "role", "token_balance", "features", "permissions"];

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
