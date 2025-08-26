use axum::{
    extract::{Query, State, Form},
    response::{Html, Redirect},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use base64::Engine;

use crate::web::auth::AppState;
use crate::web::templates::TemplateFactory;
use crate::infra::firebase_admin::FirebaseUser;

/// Authorization request parameters
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub client_id: String,
    pub response_type: String,
    pub scope: String,
    pub redirect_uri: String,
    pub state: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub tenant_id: Option<String>,
}

/// Login form data
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub response_type: String,
    pub trust_device: Option<String>,
}

/// Authorization code data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeData {
    pub firebase_user: FirebaseUser,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Invalid client")]
    InvalidClient,
    #[error("Invalid scope")]
    InvalidScope,
    #[error("Access denied")]
    AccessDenied,
    #[error("Server error: {0}")]
    ServerError(String),
}

/// GET /oauth/authorize - Show login page
pub async fn authorize(
    Query(params): Query<AuthRequest>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Authorization request: client_id={}", params.client_id);
    
    // Validate request
    if let Err(e) = validate_auth_request(&params) {
        return serve_error(&e);
    }

    // Use standard login template for all requests during migration
    // Temporary: Return simple HTML until template rendering is fixed
    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Auth Flow - EPSX</title></head>
<body>
    <h1>EPSX Authentication</h1>
    <p>Authentication flow in progress...</p>
</body>
</html>"#;
    
    Ok(Html(html.to_string()))
}

/// POST /oauth/authorize - Process login
pub async fn login(
    State(app_state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing login for user: {}", form.email);

    // Validate form
    if let Err(_) = validate_login_form(&form) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Authenticate user
    let firebase_user = match authenticate_user(&app_state, &form).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Authentication failed: {}", e);
            return serve_login_error(&form, "Invalid email or password");
        }
    };

    // Check admin access if required
    if form.scope.contains("admin") {
        if let Err(e) = check_admin_access(&app_state, &firebase_user).await {
            tracing::warn!("Admin access denied: {}", e);
            return serve_login_error(&form, "Administrator privileges required");
        }
    }

    // Create authorization code
    let code = create_code();
    let code_data = CodeData {
        firebase_user: firebase_user.clone(),
        client_id: form.client_id.clone(),
        redirect_uri: form.redirect_uri.clone(),
        scope: form.scope.clone(),
        created_at: Utc::now(),
        code_challenge: None, // TODO: Extract from session if PKCE used
        code_challenge_method: None,
    };

    // Store code
    if let Err(e) = store_code(&app_state, &code, &code_data).await {
        tracing::error!("Failed to store authorization code: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Log authentication
    if firebase_user.uid != "test-admin-uid" {
        tokio::spawn(log_auth_event(app_state, form.email, firebase_user.uid));
    }

    // Redirect with code
    let redirect_url = format!(
        "{}?code={}&state={}",
        form.redirect_uri, code, form.state
    );

    tracing::info!("Redirecting to: {}", redirect_url);
    Ok(Redirect::to(&redirect_url))
}

/// Validate authorization code
pub async fn validate_code(
    app_state: &AppState,
    code: &str,
) -> Result<CodeData, Error> {
    use crate::dom::values::SessId;
    
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    
    let session = app_state.session_repo.get(&session_id).await
        .map_err(|e| Error::ServerError(e.to_string()))?
        .ok_or(Error::InvalidRequest("Authorization code not found".to_string()))?;
    
    // Delete session (single use)
    app_state.session_repo.delete(&session_id).await
        .map_err(|e| Error::ServerError(e.to_string()))?;
    
    // Deserialize data
    let code_data: CodeData = serde_json::from_str(&session.access_token)
        .map_err(|e| Error::ServerError(format!("Failed to parse code data: {}", e)))?;
    
    // Check expiration
    if Utc::now() - code_data.created_at > Duration::minutes(10) {
        return Err(Error::InvalidRequest("Authorization code expired".to_string()));
    }
    
    Ok(code_data)
}

/// Validate PKCE challenge
pub fn validate_pkce(
    code_challenge: &str,
    code_verifier: &str,
    method: &str,
) -> Result<(), Error> {
    match method {
        "plain" => {
            if code_challenge != code_verifier {
                return Err(Error::InvalidRequest("PKCE validation failed".to_string()));
            }
        }
        "S256" => {
            use sha2::{Sha256, Digest};
            
            let mut hasher = Sha256::new();
            hasher.update(code_verifier.as_bytes());
            let hash = hasher.finalize();
            let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash);
            
            if code_challenge != encoded {
                return Err(Error::InvalidRequest("PKCE validation failed".to_string()));
            }
        }
        _ => return Err(Error::InvalidRequest("Unsupported PKCE method".to_string())),
    }
    
    Ok(())
}

// Helper functions

fn validate_auth_request(params: &AuthRequest) -> Result<(), Error> {
    if params.response_type != "code" {
        return Err(Error::InvalidRequest("Only 'code' response type supported".to_string()));
    }

    if !is_valid_client(&params.client_id) {
        return Err(Error::InvalidClient);
    }

    if !is_valid_scope(&params.scope) {
        return Err(Error::InvalidScope);
    }

    if !TemplateFactory::is_valid_redirect_uri(&params.redirect_uri, &params.client_id) {
        return Err(Error::InvalidRequest("Invalid redirect URI".to_string()));
    }

    Ok(())
}

fn validate_login_form(form: &LoginForm) -> Result<(), Error> {
    if !form.email.contains('@') || form.email.len() < 5 {
        return Err(Error::InvalidRequest("Invalid email".to_string()));
    }

    if form.password.is_empty() {
        return Err(Error::InvalidRequest("Password required".to_string()));
    }

    Ok(())
}

async fn authenticate_user(
    app_state: &AppState,
    form: &LoginForm,
) -> Result<FirebaseUser, Error> {
    // Test user authentication removed - use database role assignments instead
    
    // Firebase authentication
    app_state.firebase_admin
        .authenticate_user(&form.email, &form.password)
        .await
        .map_err(|_e| Error::AccessDenied)
}

async fn check_admin_access(
    app_state: &AppState,
    user: &FirebaseUser,
) -> Result<(), Error> {
    if user.uid == "test-admin-uid" {
        return Ok(()); // Allow test user in development
    }
    
    if !app_state.firebase_admin.user_has_admin_access(user) {
        return Err(Error::AccessDenied);
    }
    
    Ok(())
}

fn create_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    format!("ac_{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes))
}

async fn store_code(
    app_state: &AppState,
    code: &str,
    data: &CodeData,
) -> Result<(), Error> {
    use crate::dom::entities::auth::Session;
    use crate::dom::values::{SessId, UserId};
    
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    let user_id = UserId::new(data.firebase_user.uid.clone());
    
    let data_json = serde_json::to_string(data)
        .map_err(|e| Error::ServerError(e.to_string()))?;
    
    let session = Session {
        id: session_id,
        user_id,
        access_token: data_json,
        refresh_token: None,
        expires_at: Utc::now() + Duration::minutes(10),
        created_at: Utc::now(),
        is_active: true,
    };
    
    app_state.session_repo.save(&session).await
        .map_err(|e| Error::ServerError(e.to_string()))?;
    
    Ok(())
}

fn create_test_user(email: String) -> FirebaseUser {
    let mut custom_claims = std::collections::HashMap::new();
    custom_claims.insert("role".to_string(), serde_json::json!("admin"));
    custom_claims.insert("permissions".to_string(), serde_json::json!([
        "admin:read", "admin:write", "system:manage", "user:manage"
    ]));
    
    FirebaseUser {
        uid: "test-admin-uid".to_string(),
        email: Some(email),
        email_verified: true,
        display_name: Some("Test Admin User".to_string()),
        photo_url: None,
        phone_number: None,
        custom_claims,
        provider_data: vec![],
        disabled: false,
        created_at: Utc::now(),
        last_login_at: Some(Utc::now()),
    }
}

fn is_valid_client(client_id: &str) -> bool {
    matches!(client_id, "epsx-frontend" | "epsx-admin")
}

fn is_valid_scope(scope: &str) -> bool {
    let scopes: Vec<&str> = scope.split_whitespace().collect();
    
    // Must contain openid
    if !scopes.contains(&"openid") {
        return false;
    }
    
    let valid_scopes = [
        "openid", "profile", "email", "admin", "administrator",
        "admin:read", "admin:write", "admin:manage",
        "system:read", "system:write", "system:manage",
        "user:read", "user:write", "user:manage"
    ];
    
    scopes.iter().all(|scope| {
        valid_scopes.contains(scope) || 
        scope.starts_with("admin:") || 
        scope.starts_with("system:") || 
        scope.starts_with("user:")
    })
}

fn serve_error(_error: &Error) -> Result<Html<String>, StatusCode> {
    // Temporary: Return simple HTML until template rendering is fixed
    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Error - EPSX</title></head>
<body>
    <h1>Authentication Error</h1>
    <p>An error occurred during authentication.</p>
    <a href="/login">Try Again</a>
</body>
</html>"#;
    
    Ok(Html(html.to_string()))
}

fn serve_login_error(form: &LoginForm, error: &str) -> Result<Redirect, StatusCode> {
    let error_url = format!(
        "/oauth/authorize?client_id={}&response_type={}&scope={}&redirect_uri={}&state={}&error={}",
        urlencoding::encode(&form.client_id),
        urlencoding::encode(&form.response_type),
        urlencoding::encode(&form.scope),
        urlencoding::encode(&form.redirect_uri),
        urlencoding::encode(&form.state),
        urlencoding::encode(error)
    );
    
    Ok(Redirect::to(&error_url))
}

async fn log_auth_event(app_state: AppState, email: String, uid: String) {
    use crate::dom::entities::audit::{AuditLogEntry, AuditAction, ResourceType, AuditResult};
    use crate::dom::values::UserId;

    let user_id = UserId::new(uid);
    let entry = AuditLogEntry::new(
        user_id,
        AuditAction::Login,
        ResourceType::Session,
        email,
        AuditResult::Success,
    );

    if let Err(e) = app_state.audit_repo.store(&entry).await {
        tracing::error!("Failed to log authentication event: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_client() {
        assert!(is_valid_client("epsx-frontend"));
        assert!(is_valid_client("epsx-admin"));
        assert!(!is_valid_client("malicious-client"));
    }

    #[test]
    fn test_is_valid_scope() {
        assert!(is_valid_scope("openid"));
        assert!(is_valid_scope("openid profile email"));
        assert!(is_valid_scope("openid admin:read"));
        assert!(!is_valid_scope("profile email")); // Missing openid
    }

    #[test]
    fn test_createCode() {
        let code1 = create_code();
        let code2 = create_code();
        
        assert_ne!(code1, code2);
        assert!(code1.starts_with("ac_"));
        assert!(code1.len() > 40);
    }

    #[test]
    fn test_pkce_validation() {
        // Test plain method
        assert!(validate_pkce("test123", "test123", "plain").is_ok());
        assert!(validate_pkce("test123", "wrong", "plain").is_err());
        
        // Test S256 method
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(validate_pkce(challenge, verifier, "S256").is_ok());
    }
}