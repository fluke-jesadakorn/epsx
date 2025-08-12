// Pure OIDC Authorization Code Flow implementation

use axum::{
    extract::{Query, State, Form},
    response::{Html, Redirect},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use askama::Template;
use chrono::{DateTime, Utc};
use base64::Engine;

use crate::web::auth::AppState;
use crate::web::templates::TemplateFactory;
use crate::infra::firebase_admin::FirebaseUser;

/// Authorization request parameters (GET /oauth/authorize)
#[derive(Debug, Deserialize)]
pub struct AuthorizationParams {
    pub client_id: String,
    pub response_type: String,
    pub scope: String,
    pub redirect_uri: String,
    pub state: String,
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_challenge_method: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
}

/// Login form data (POST /oauth/authorize)
#[derive(Debug, Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub response_type: String,
    #[serde(default)]
    pub trust_device: Option<String>,
}

/// Authorization code data stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCodeData {
    pub firebase_user: FirebaseUser,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// GET /oauth/authorize - Serve login page
pub async fn authorization_endpoint(
    Query(params): Query<AuthorizationParams>,
) -> Result<Html<String>, StatusCode> {
    // Validate required parameters
    if params.response_type != "code" {
        return serve_error_page(
            "unsupported_response_type",
            "Only 'code' response type is supported"
        );
    }

    // Validate client_id
    if !is_valid_client_id(&params.client_id) {
        return serve_error_page(
            "invalid_client",
            "Invalid or unregistered client identifier"
        );
    }

    // Validate redirect_uri
    if !TemplateFactory::is_valid_redirect_uri(&params.redirect_uri, &params.client_id) {
        return serve_error_page(
            "invalid_request",
            "Invalid redirect URI for this client"
        );
    }

    // Validate scope
    if !is_valid_scope(&params.scope) {
        return serve_error_page(
            "invalid_scope",
            "Invalid or unsupported scope"
        );
    }

    // Determine if admin login is required
    let is_admin_login = TemplateFactory::should_use_admin_template(&params.scope);

    // Render appropriate login template
    if is_admin_login {
        let template = TemplateFactory::create_admin_login_template(
            params.client_id,
            params.redirect_uri,
            params.state,
            params.scope,
            String::new(), // No error on initial load
        );
        
        template.render()
            .map(Html)
            .map_err(|e| {
                tracing::error!("Failed to render admin login template: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    } else {
        let template = TemplateFactory::create_login_template(
            params.client_id,
            params.redirect_uri,
            params.state,
            params.scope,
            String::new(), // No error on initial load
        );
        
        template.render()
            .map(Html)
            .map_err(|e| {
                tracing::error!("Failed to render login template: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })
    }
}

/// POST /oauth/authorize - Process login form
pub async fn handle_authorization_form(
    State(app_state): State<AppState>,
    Form(form_data): Form<LoginFormData>,
) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing login form for user: {}", form_data.email);

    // Validate form data
    if let Err(status) = validate_form_data(&form_data) {
        return Err(status);
    }

    // Authenticate with Firebase
    let firebase_user = match app_state.firebase_admin
        .authenticate_user(&form_data.email, &form_data.password)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Firebase authentication failed for {}: {}", form_data.email, e);
            return serve_login_with_error(&form_data, "Invalid email or password");
        }
    };

    // Validate admin access if required
    if form_data.scope.contains("admin") {
        if !app_state.firebase_admin.user_has_admin_access(&firebase_user) {
            tracing::warn!("User {} attempted admin login without privileges", form_data.email);
            return serve_login_with_error(&form_data, "Administrator privileges required");
        }
    }

    // Generate authorization code
    let auth_code = generate_authorization_code();
    
    // Store authorization code data
    let auth_data = AuthorizationCodeData {
        firebase_user: firebase_user.clone(),
        client_id: form_data.client_id.clone(),
        redirect_uri: form_data.redirect_uri.clone(),
        scope: form_data.scope.clone(),
        created_at: Utc::now(),
        code_challenge: None, // TODO: Extract from session if PKCE was used
        code_challenge_method: None,
    };

    if let Err(e) = store_authorization_code(&app_state, &auth_code, &auth_data).await {
        tracing::error!("Failed to store authorization code: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Log successful authentication
    tracing::info!("Authentication successful for user: {} ({})", firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()), firebase_user.uid);

    // Audit log the authentication
    tokio::spawn({
        let app_state = app_state.clone();
        let email = form_data.email.clone();
        let firebase_uid = firebase_user.uid.clone();
        async move {
            if let Err(e) = log_authentication_event(&app_state, &email, &firebase_uid, true).await {
                tracing::error!("Failed to log authentication event: {}", e);
            }
        }
    });

    // Redirect back to client with authorization code
    let redirect_url = format!(
        "{}?code={}&state={}",
        form_data.redirect_uri,
        auth_code,
        form_data.state
    );

    tracing::info!("Redirecting to: {}", redirect_url);
    Ok(Redirect::to(&redirect_url))
}

/// Generate a secure authorization code
fn generate_authorization_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let random_bytes: [u8; 32] = rng.gen();
    format!("ac_{}", base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes))
}

/// Store authorization code using session repository
async fn store_authorization_code(
    app_state: &AppState,
    code: &str,
    auth_data: &AuthorizationCodeData,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::dom::entities::auth::Session;
    use crate::dom::values::{SessId, UserId};
    use chrono::{Utc, Duration};
    
    // Create a temporary session for the authorization code
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    let user_id = UserId::new(auth_data.firebase_user.uid.clone());
    
    // Serialize auth data and store in access_token field temporarily
    let auth_data_json = serde_json::to_string(auth_data)?;
    
    // Create session with 10 minute expiration
    let expires_at = Utc::now() + Duration::minutes(10);
    let session = Session {
        id: session_id,
        user_id,
        access_token: auth_data_json, // Store serialized auth data here
        refresh_token: None,
        expires_at,
        created_at: Utc::now(),
        is_active: true,
    };
    
    app_state.session_repo.save(&session).await?;
    
    tracing::debug!("Authorization code stored: {}", code);
    Ok(())
}

/// Validate authorization form data
fn validate_form_data(form_data: &LoginFormData) -> Result<(), StatusCode> {
    // Validate email format
    if !form_data.email.contains('@') || form_data.email.len() < 5 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate password
    if form_data.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate client_id
    if !is_valid_client_id(&form_data.client_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate redirect_uri
    if !TemplateFactory::is_valid_redirect_uri(&form_data.redirect_uri, &form_data.client_id) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate scope
    if !is_valid_scope(&form_data.scope) {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

/// Validate client ID
fn is_valid_client_id(client_id: &str) -> bool {
    matches!(client_id, "epsx-frontend" | "epsx-admin")
}

/// Validate scope
fn is_valid_scope(scope: &str) -> bool {
    let required_scopes = ["openid"];
    let valid_scope_patterns = [
        "openid", "profile", "email", 
        "admin", "administrator",
        "admin:read", "admin:write", "admin:manage",
        "system:read", "system:write", "system:manage",
        "user:read", "user:write", "user:manage"
    ];
    
    let scopes: Vec<&str> = scope.split_whitespace().collect();
    
    // Must contain required scopes
    for required in &required_scopes {
        if !scopes.contains(required) {
            return false;
        }
    }
    
    // All scopes must be valid or follow admin/system/user pattern
    for scope in &scopes {
        if !valid_scope_patterns.contains(scope) {
            // Check if it follows a valid pattern like "admin:*", "system:*", "user:*"
            if !(scope.starts_with("admin:") || scope.starts_with("system:") || scope.starts_with("user:")) {
                return false;
            }
        }
    }
    
    true
}

/// Serve error page for authorization errors
fn serve_error_page(error_code: &str, _description: &str) -> Result<Html<String>, StatusCode> {
    let template = TemplateFactory::get_error_template_for_auth_error(error_code);
    
    template.render()
        .map(Html)
        .map_err(|e| {
            tracing::error!("Failed to render error template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Serve login page with error message
fn serve_login_with_error(form_data: &LoginFormData, error_message: &str) -> Result<Redirect, StatusCode> {
    // For now, we'll redirect back to the authorization endpoint with an error parameter
    // In a more sophisticated implementation, we could render the template directly with the error
    
    let error_url = format!(
        "/oauth/authorize?client_id={}&response_type={}&scope={}&redirect_uri={}&state={}&error={}",
        urlencoding::encode(&form_data.client_id),
        urlencoding::encode(&form_data.response_type),
        urlencoding::encode(&form_data.scope),
        urlencoding::encode(&form_data.redirect_uri),
        urlencoding::encode(&form_data.state),
        urlencoding::encode(error_message)
    );
    
    Ok(Redirect::to(&error_url))
}

/// Log authentication event for audit purposes
async fn log_authentication_event(
    app_state: &AppState,
    email: &str,
    firebase_uid: &str,
    success: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::dom::entities::audit::{AuditLogEntry, AuditAction, ResourceType, AuditResult};
    use crate::dom::values::UserId;

    let user_id = UserId::new(firebase_uid.to_string());
    let action = if success { AuditAction::Login } else { AuditAction::LoginFailed };
    let result = if success { AuditResult::Success } else { AuditResult::Failure };

    let entry = AuditLogEntry::new(
        user_id,
        action,
        ResourceType::Session,
        email.to_string(),
        result,
    );

    app_state.audit_repo.store(&entry).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_client_id() {
        assert!(is_valid_client_id("epsx-frontend"));
        assert!(is_valid_client_id("epsx-admin"));
        assert!(!is_valid_client_id("malicious-client"));
        assert!(!is_valid_client_id(""));
    }

    #[test]
    fn test_is_valid_scope() {
        assert!(is_valid_scope("openid"));
        assert!(is_valid_scope("openid profile email"));
        assert!(is_valid_scope("openid profile email admin"));
        assert!(is_valid_scope("openid profile email admin:read admin:write"));
        assert!(is_valid_scope("openid profile email admin:read admin:write system:manage"));
        assert!(!is_valid_scope("profile email")); // Missing openid
        assert!(!is_valid_scope("openid invalid_scope"));
        assert!(!is_valid_scope("openid malicious:inject"));
        assert!(!is_valid_scope(""));
    }

    #[test]
    fn test_generate_authorization_code() {
        let code1 = generate_authorization_code();
        let code2 = generate_authorization_code();
        
        assert_ne!(code1, code2);
        assert!(code1.starts_with("ac_"));
        assert!(code1.len() > 40);
    }
}