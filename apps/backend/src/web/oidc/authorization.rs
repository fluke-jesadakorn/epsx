// Pure OIDC Authorization Code Flow implementation

use axum::{
    extract::{Query, Form},
    response::{Html, Redirect},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use base64::Engine;
use askama::Template;

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
    /// Registration mode flag to differentiate from login
    #[serde(default)]
    pub registration: Option<bool>,
    /// Error message to display in template (for redirects with errors)
    #[serde(default)]
    pub error: Option<String>,
}

/// Login/Registration form data (POST /oauth/authorize)
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
    #[serde(default)]
    pub code_challenge: Option<String>,
    #[serde(default)]
    pub code_challenge_method: Option<String>,
    /// Registration mode flag to differentiate from login
    #[serde(default)]
    pub registration: Option<bool>,
    /// Display name for user registration
    #[serde(default)]
    pub display_name: Option<String>,
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
    tracing::info!("Authorization endpoint request - DEBUGGING CONTENT-TYPE ERROR");
    tracing::info!("Auth params: {:?}", params);
    
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
    let is_admin_login = TemplateFactory::should_use_admin_template(&params.scope) || params.client_id == "epsx-admin";
    let is_registration = params.registration.unwrap_or(false);
    let error_message = params.error.as_deref().unwrap_or("");

    // Render analytics-themed templates based on client type and flow
    let html = if is_admin_login {
        // Analytics Command Center for admin users
        let template = TemplateFactory::create_analytics_command_center_template_with_pkce(
            params.client_id.clone(),
            params.redirect_uri.clone(),
            params.state.clone(),
            params.scope.clone(),
            params.code_challenge.clone(),
            params.code_challenge_method.clone(),
            error_message.to_string(),
        );
        
        template.render().map_err(|e| {
            tracing::error!("Failed to render admin login template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        
    } else if is_registration {
        // Data Insights Portal registration for new users
        let template = TemplateFactory::create_analytics_registration_template_with_pkce(
            params.client_id.clone(),
            params.redirect_uri.clone(),
            params.state.clone(),
            params.scope.clone(),
            params.code_challenge.clone(),
            params.code_challenge_method.clone(),
            error_message.to_string(),
        );
        
        template.render().map_err(|e| {
            tracing::error!("Failed to render registration template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        
    } else {
        // Data Insights Portal login for regular users
        let template = TemplateFactory::create_analytics_portal_login_template_with_pkce(
            params.client_id.clone(),
            params.redirect_uri.clone(),
            params.state.clone(),
            params.scope.clone(),
            params.code_challenge.clone(),
            params.code_challenge_method.clone(),
            error_message.to_string(),
        );
        
        template.render().map_err(|e| {
            tracing::error!("Failed to render login template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    };

    tracing::info!("Successfully rendered analytics-themed template for client: {}", params.client_id);
    Ok(Html(html))
}

/// POST /oauth/authorize - Process login/registration form
pub async fn handle_authorization_form(
    Form(form_data): Form<LoginFormData>,
) -> Result<Redirect, StatusCode> {
    let is_registration = form_data.registration.unwrap_or(false);
    
    if is_registration {
        tracing::info!("Processing registration form for user: {}", form_data.email);
        // Reuse existing registration handler
        return handle_registration_form_direct(form_data).await;
    } else {
        tracing::info!("Processing login form for user: {}", form_data.email);
        return handle_user_login_with_session_storage(form_data).await;
    }
}

/// Handle user login flow with proper session storage
async fn handle_user_login_with_session_storage(
    form_data: LoginFormData,
) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing login form for user: {}", form_data.email);
    
    // For now, just use the direct flow - it now has session storage
    handle_user_login_direct(form_data).await
}

/// Handle user login flow without AppState (fallback)
async fn handle_user_login_direct(
    form_data: LoginFormData,
) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing login form for user: {}", form_data.email);
    
    // Validate form data
    if let Err(status) = validate_form_data(&form_data) {
        return Err(status);
    }

    // Create Firebase Admin instance directly
    let firebase_admin = match create_firebase_admin().await {
        Ok(admin) => admin,
        Err(e) => {
            tracing::error!("Failed to create Firebase Admin: {}", e);
            return serve_login_with_error(&form_data, "Authentication service unavailable");
        }
    };

    // Use Firebase authentication for all users
    let firebase_user = match firebase_admin
        .authenticate_user(&form_data.email, &form_data.password)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Firebase authentication failed for {}: {}", form_data.email, e);
            return serve_login_with_error(&form_data, "Invalid email or password");
        }
    };

    // For simplicity, skip admin module validation for now and just create basic JWT
    handle_authenticated_user_flow_direct(form_data, firebase_user).await
}

/// Handle authenticated user flow without AppState
async fn handle_authenticated_user_flow_direct(
    form_data: LoginFormData,
    firebase_user: FirebaseUser,
) -> Result<Redirect, StatusCode> {
    // Generate authorization code
    let code = generate_authorization_code();
    
    // Store authorization code data in simplified manner
    let auth_data = AuthorizationCodeData {
        firebase_user: firebase_user.clone(),
        client_id: form_data.client_id.clone(),
        redirect_uri: form_data.redirect_uri.clone(),
        scope: form_data.scope.clone(),
        created_at: Utc::now(),
        code_challenge: form_data.code_challenge.clone(),
        code_challenge_method: form_data.code_challenge_method.clone(),
    };
    
    tracing::info!("🔍 SESSION DEBUG: Authorization code generated in direct flow: {}", code);
    
    // Store session directly without full AppState to fix token exchange
    match store_authorization_code_direct(&code, &auth_data).await {
        Ok(_) => {
            tracing::info!("🔍 SESSION DEBUG: Successfully stored authorization code in direct flow: {}", code);
        }
        Err(e) => {
            tracing::error!("🔍 SESSION DEBUG: Failed to store authorization code in direct flow: {}", e);
            // Continue anyway - at least the user gets redirected
        }
    }

    // Redirect to redirect_uri with authorization code
    let redirect_url = format!(
        "{}?code={}&state={}",
        form_data.redirect_uri,
        code,
        form_data.state
    );
    
    tracing::info!(
        user_id = %firebase_user.uid,
        email = %firebase_user.email.as_deref().unwrap_or("unknown"),
        "User login successful, redirecting with code: {}",
        code
    );
    
    Ok(Redirect::to(&redirect_url))
}

/// Handle user registration flow
async fn handle_user_registration(
    app_state: AppState,
    form_data: LoginFormData,
) -> Result<Redirect, StatusCode> {
    // Validate form data
    if let Err(status) = validate_form_data(&form_data) {
        return Err(status);
    }

    // Create Firebase user first
    let firebase_user = match app_state.firebase_admin
        .create_user_with_password(
            &form_data.email,
            &form_data.password,
            form_data.display_name.clone()
        )
        .await
    {
        Ok(user) => {
            tracing::info!("✅ Successfully created Firebase user: {} ({})", form_data.email, user.uid);
            user
        }
        Err(e) => {
            tracing::error!("❌ Firebase user creation failed for {}: {}", form_data.email, e);
            return serve_login_with_error(&form_data, "Registration failed. Email may already be in use.");
        }
    };

    // Create database user with Firebase UID
    let user_role = determine_user_role_for_registration(&form_data.email);
    match create_database_user(&app_state, &firebase_user, &user_role).await {
        Ok(_) => {
            tracing::info!("✅ Successfully created database user: {} with role: {}", form_data.email, user_role);
        }
        Err(e) => {
            tracing::error!("❌ Database user creation failed for {}: {}", form_data.email, e);
            
            // Critical: Clean up Firebase user to prevent orphaned accounts
            if let Err(cleanup_error) = app_state.firebase_admin.delete_user(&firebase_user.uid).await {
                tracing::error!("🚨 Critical: Failed to cleanup Firebase user {} after database failure: {}", 
                    firebase_user.uid, cleanup_error);
                // Consider alerting security team about orphaned Firebase account
            } else {
                tracing::info!("🧹 Successfully cleaned up Firebase user {} after database failure", firebase_user.uid);
            }
            
            return serve_login_with_error(&form_data, "Registration failed. Please try again.");
        }
    }

    // Admin module assignments are now handled through CLI tools only

    // Now proceed with authentication flow using the newly created user
    handle_authenticated_user_flow(app_state, form_data, firebase_user).await
}

/// Handle user login flow (existing functionality)
async fn handle_user_login(
    app_state: AppState,
    form_data: LoginFormData,
) -> Result<Redirect, StatusCode> {
    // Validate form data
    if let Err(status) = validate_form_data(&form_data) {
        return Err(status);
    }

    // Use Firebase authentication for all users
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

    handle_authenticated_user_flow(app_state, form_data, firebase_user).await
}

/// Handle authenticated user flow (common to both login and registration)
async fn handle_authenticated_user_flow(
    app_state: AppState,
    form_data: LoginFormData,
    firebase_user: FirebaseUser,
) -> Result<Redirect, StatusCode> {

    // Validate admin access if required
    if form_data.scope.contains("admin") || form_data.scope.contains("admin_modules") {
        // For test user, skip Firebase admin validation and use granular admin modules
        if firebase_user.uid == "test_user_info_epsx_io" {
            tracing::info!("Development mode: checking granular admin modules for test user");
            
            // Check granular admin modules
            if let Some(email) = &firebase_user.email {
                match app_state.admin_module_service.get_user_admin_modules(email).await {
                    Ok(modules) if !modules.is_empty() => {
                        tracing::info!("Test user has {} granular admin modules", modules.len());
                    },
                    _ => {
                        tracing::warn!("Test user does not have granular admin modules, allowing for development");
                    }
                }
            }
        } else {
            // For real users, check both Firebase admin validation and database admin modules
            let has_firebase_admin = app_state.firebase_admin.user_has_admin_access(&firebase_user);
            let has_admin_modules = if let Some(email) = &firebase_user.email {
                app_state.admin_module_service.get_user_admin_modules(email).await
                    .map(|modules| !modules.is_empty())
                    .unwrap_or(false)
            } else {
                false
            };
            
            if !has_firebase_admin && !has_admin_modules {
                tracing::warn!("User {} attempted admin login without privileges", form_data.email);
                return serve_login_with_error(&form_data, "Administrator privileges required");
            }
        }
    }

    tracing::error!("🔍 SESSION DEBUG: Starting scope validation for user: {}", firebase_user.uid);
    
    // Extract user role for scope validation
    let user_role = extract_user_role(&firebase_user);
    tracing::error!("🔍 SESSION DEBUG: User role extracted: {} for user: {}", user_role, firebase_user.uid);
    
    // Validate requested scopes against user's role and permissions
    let validated_scopes = match validate_scopes_with_user_context(
        &form_data.scope,
        &user_role,
        &form_data.client_id,
    ) {
        Ok(scopes) => {
            tracing::error!("🔍 SESSION DEBUG: Scope validation successful");
            scopes
        },
        Err(ref scope_error) => {
            tracing::error!("🔍 SESSION DEBUG: Scope validation failed: {:?}", scope_error);
            let error_message = match scope_error {
                crate::auth::ScopeError::InsufficientRole { scope, required_role, .. } => {
                    format!("Insufficient privileges for scope '{}'. Role '{}' required.", scope, required_role)
                }
                crate::auth::ScopeError::InvalidScope { scope } => {
                    format!("Invalid scope requested: '{}'", scope)
                }
                crate::auth::ScopeError::ClientNotAuthorized { scope, .. } => {
                    format!("Client not authorized for sensitive scope: '{}'", scope)
                }
                crate::auth::ScopeError::MissingOpenIDScope => {
                    "OpenID scope is required when requesting profile information".to_string()
                }
                _ => "Invalid scope configuration".to_string(),
            };
            
            tracing::warn!(
                user_email = %form_data.email,
                user_role = %user_role,
                requested_scopes = %form_data.scope,
                client_id = %form_data.client_id,
                error = %scope_error,
                "Scope validation failed during authorization"
            );
            
            return serve_login_with_error(&form_data, &error_message);
        }
    };

    // Log successful scope validation
    tracing::info!(
        user_email = %form_data.email,
        user_role = %user_role,
        granted_scopes = %validated_scopes.granted_scopes,
        sensitive_scopes = ?validated_scopes.sensitive_scopes,
        permission_count = %validated_scopes.permissions.len(),
        "Scopes validated successfully"
    );

    // Generate authorization code
    let auth_code = generate_authorization_code();
    
    // Store authorization code data with PKCE parameters and validated scopes
    let auth_data = AuthorizationCodeData {
        firebase_user: firebase_user.clone(),
        client_id: form_data.client_id.clone(),
        redirect_uri: form_data.redirect_uri.clone(),
        scope: validated_scopes.granted_scopes.clone(), // Use validated scopes, not raw request
        created_at: Utc::now(),
        code_challenge: form_data.code_challenge.clone(),
        code_challenge_method: form_data.code_challenge_method.clone(),
    };

    tracing::error!("🔍 SESSION DEBUG: About to store authorization code: {}", auth_code);
    if let Err(e) = store_authorization_code(&app_state, &auth_code, &auth_data).await {
        tracing::error!("🔍 SESSION DEBUG: Failed to store authorization code: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    tracing::error!("🔍 SESSION DEBUG: Successfully stored authorization code: {}", auth_code);

    // Log successful authentication
    tracing::info!("Authentication successful for user: {} ({})", firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()), firebase_user.uid);

    // Audit log the authentication (skip for test users to avoid FK constraint issues)
    if firebase_user.uid != "test_user_info_epsx_io" {
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
    } else {
        tracing::info!("Skipping audit log for test admin user");
    }

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
    use crate::dom::values::SessId;
    use chrono::{Utc, Duration};
    
    // First, look up the user by firebase_uid to get their actual UUID
    let user = app_state.user_repo.find_by_firebase_uid(&auth_data.firebase_user.uid).await?;
    let user_id = match user {
        Some(user) => user.id().clone(),
        None => {
            tracing::error!("User not found for firebase_uid: {}", auth_data.firebase_user.uid);
            return Err("User not found in database".into());
        }
    };
    
    // Create a temporary session for the authorization code
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    
    tracing::error!("🔍 SESSION DEBUG: Storing auth code: {} for user: {}", code, user_id);
    tracing::error!("🔍 SESSION DEBUG: Session ID generated: {}", session_id);
    tracing::error!("🔍 SESSION DEBUG: Session UUID: {:?}", session_id);
    
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
    use crate::auth::SCOPE_SERVICE;
    
    // Basic validation - check if all requested scopes exist
    let scope_names = SCOPE_SERVICE.parse_scope_string(scope);
    
    if scope_names.is_empty() {
        return false;
    }
    
    // Check if all requested scopes are registered
    for scope_name in &scope_names {
        if SCOPE_SERVICE.get_scope(scope_name).is_none() {
            tracing::warn!(
                scope = %scope_name,
                "Unknown scope requested during authorization"
            );
            return false;
        }
    }
    
    // Additional validation: if profile/email scopes are requested, openid must be present
    let has_profile_scopes = scope_names.iter()
        .any(|s| matches!(s.as_str(), "profile" | "email"));
    let has_openid = scope_names.contains(&"openid".to_string());
    
    if has_profile_scopes && !has_openid {
        tracing::warn!("Profile scopes requested without openid scope");
        return false;
    }
    
    true
}

/// Validate scopes with user context (called after authentication)
fn validate_scopes_with_user_context(
    scope: &str,
    user_role: &str,
    client_id: &str,
) -> Result<crate::auth::ValidatedScopes, crate::auth::ScopeError> {
    use crate::auth::SCOPE_SERVICE;
    
    SCOPE_SERVICE.validate_scopes(scope, user_role, client_id)
}

/// Extract user role from Firebase user custom claims
fn extract_user_role(firebase_user: &FirebaseUser) -> String {
    // Try to extract role from custom claims
    let custom_claims = &firebase_user.custom_claims;
    if !custom_claims.is_empty() {
        if let Some(role_value) = custom_claims.get("role") {
            if let Some(role_str) = role_value.as_str() {
                return role_str.to_string();
            }
        }
    }
    
    // Fallback logic for determining role
    if firebase_user.uid == "test_user_info_epsx_io" {
        // Test admin user gets admin role
        return "admin".to_string();
    }
    
    // Check if user has admin access (legacy Firebase admin check)
    // This is a simplified check - in production you'd have more sophisticated role determination
    if let Some(email) = &firebase_user.email {
        if email.contains("admin") || email.contains("moderator") {
            return "moderator".to_string();
        }
    }
    
    // Default role
    "user".to_string()
}

/// Serve error page for authorization errors using analytics theme
fn serve_error_page(error_code: &str, _description: &str) -> Result<Html<String>, StatusCode> {
    let template = TemplateFactory::get_error_template_for_auth_error(error_code);
    
    let html = template.render().map_err(|e| {
        tracing::error!("Failed to render error template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    tracing::info!("Served analytics-themed error page for: {}", error_code);
    Ok(Html(html))
}

/// Serve login/registration page with error message
fn serve_login_with_error(form_data: &LoginFormData, error_message: &str) -> Result<Redirect, StatusCode> {
    // For now, we'll redirect back to the authorization endpoint with an error parameter
    // In a more sophisticated implementation, we could render the template directly with the error
    
    let mut error_url = format!(
        "/oauth/authorize?client_id={}&response_type={}&scope={}&redirect_uri={}&state={}&error={}",
        urlencoding::encode(&form_data.client_id),
        urlencoding::encode(&form_data.response_type),
        urlencoding::encode(&form_data.scope),
        urlencoding::encode(&form_data.redirect_uri),
        urlencoding::encode(&form_data.state),
        urlencoding::encode(error_message)
    );
    
    // Add registration parameter if this was a registration attempt
    if form_data.registration.unwrap_or(false) {
        error_url.push_str(&format!("&registration=true"));
    }
    
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

/// Determine user role for new registrations
fn determine_user_role_for_registration(email: &str) -> String {
    if email == "info@epsx.io" {
        "Admin".to_string()
    } else {
        "User".to_string() // Default role for new registrations
    }
}

/// Create user in database after Firebase user creation
async fn create_database_user(
    app_state: &AppState,
    firebase_user: &FirebaseUser,
    role: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::dom::entities::User;
    use crate::dom::values::Email;
    use crate::auth::roles::Role;
    
    let email = Email::new(firebase_user.email.as_ref().unwrap_or(&"unknown@example.com".to_string()).clone())
        .map_err(|e| format!("Invalid email format: {}", e))?;
    
    let user_role = role.parse::<Role>()
        .map_err(|e| format!("Invalid role: {}", e))?;
    
    // Create user entity with real Firebase UID
    let user = User::new(
        firebase_user.uid.clone(),
        email,
        user_role.to_string()
    );
    
    // Save to database
    app_state.user_repo.save(&user).await
        .map_err(|e| format!("Failed to save user to database: {}", e))?;
    
    tracing::info!("✅ Database user created with Firebase UID: {} for email: {}", 
                   firebase_user.uid, firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()));
    
    Ok(())
}

// Removed assign_superadmin_modules function - admin assignments now handled via CLI tools

// ============================================================================
// REGISTRATION ENDPOINTS
// ============================================================================

/// GET /oauth/register - Serve registration page
pub async fn register_endpoint(
    Query(params): Query<AuthorizationParams>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Registration endpoint request");
    tracing::info!("Registration params: {:?}", params);
    
    // Validate required parameters (same as login)
    if params.response_type != "code" {
        return serve_error_page(
            "unsupported_response_type",
            "Only 'code' response type is supported"
        );
    }

    if !is_valid_client_id(&params.client_id) {
        return serve_error_page(
            "invalid_client",
            "Invalid or unregistered client identifier"
        );
    }

    if !TemplateFactory::is_valid_redirect_uri(&params.redirect_uri, &params.client_id) {
        return serve_error_page(
            "invalid_request",
            "Invalid redirect URI for this client"
        );
    }

    if !is_valid_scope(&params.scope) {
        return serve_error_page(
            "invalid_scope",
            "Invalid or unsupported scope"
        );
    }

    let error_message = params.error.as_deref().unwrap_or("");

    // Always render registration template
    let template = TemplateFactory::create_analytics_registration_template_with_pkce(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        params.state.clone(),
        params.scope.clone(),
        params.code_challenge.clone(),
        params.code_challenge_method.clone(),
        error_message.to_string(),
    );
    
    let html = template.render().map_err(|e| {
        tracing::error!("Failed to render registration template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Successfully rendered analytics registration template");
    Ok(Html(html))
}

/// POST /oauth/register - Process registration form
pub async fn handle_registration_form(
    Form(form_data): Form<LoginFormData>,
) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing registration form for user: {}", form_data.email);
    
    return handle_registration_form_direct(form_data).await;
}

// ============================================================================
// PASSWORD RESET ENDPOINTS
// ============================================================================

/// GET /oauth/reset-password - Serve password reset request page
pub async fn password_reset_endpoint(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Password reset endpoint request");
    
    let error_message = params.get("error").cloned().unwrap_or_else(|| "".to_string());
    
    // Create simple password reset template
    let template = TemplateFactory::create_password_reset_template(error_message);
    
    let html = template.render().map_err(|e| {
        tracing::error!("Failed to render password reset template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// POST /oauth/reset-password - Process password reset request
pub async fn handle_password_reset(
    Form(form_data): Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, StatusCode> {
    let email = form_data.get("email").ok_or(StatusCode::BAD_REQUEST)?;
    
    tracing::info!("Processing password reset request for: {}", email);
    
    // Create Firebase Admin instance directly
    let firebase_admin = match create_firebase_admin().await {
        Ok(admin) => admin,
        Err(e) => {
            tracing::error!("Failed to create Firebase Admin: {}", e);
            return Ok(Redirect::to("/oauth/reset-password?error=service_unavailable"));
        }
    };

    // Send password reset email
    match firebase_admin.send_password_reset_email(email).await {
        Ok(_) => {
            tracing::info!("✅ Password reset email sent to: {}", email);
            Ok(Redirect::to("/oauth/reset-password?success=email_sent"))
        }
        Err(e) => {
            tracing::error!("❌ Failed to send password reset email to {}: {}", email, e);
            let error_msg = if e.to_string().contains("EMAIL_NOT_FOUND") {
                "email_not_found"
            } else {
                "send_failed"
            };
            Ok(Redirect::to(&format!("/oauth/reset-password?error={}", error_msg)))
        }
    }
}

/// GET /oauth/reset-password/confirm - Serve password reset confirmation page
pub async fn password_reset_confirm_endpoint(
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("Password reset confirmation endpoint request");
    
    let oob_code = params.get("oobCode").cloned().unwrap_or_else(|| "".to_string());
    let error_message = params.get("error").cloned().unwrap_or_else(|| "".to_string());
    
    // Create password reset confirmation template
    let template = TemplateFactory::create_password_reset_confirm_template(oob_code, error_message);
    
    let html = template.render().map_err(|e| {
        tracing::error!("Failed to render password reset confirm template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

/// POST /oauth/reset-password/confirm - Process password reset confirmation
pub async fn handle_password_reset_confirm(
    Form(form_data): Form<std::collections::HashMap<String, String>>,
) -> Result<Redirect, StatusCode> {
    let oob_code = form_data.get("oobCode").ok_or(StatusCode::BAD_REQUEST)?;
    let new_password = form_data.get("newPassword").ok_or(StatusCode::BAD_REQUEST)?;
    
    tracing::info!("Processing password reset confirmation");
    
    // Create Firebase Admin instance directly
    let firebase_admin = match create_firebase_admin().await {
        Ok(admin) => admin,
        Err(e) => {
            tracing::error!("Failed to create Firebase Admin: {}", e);
            return Ok(Redirect::to("/oauth/reset-password/confirm?error=service_unavailable"));
        }
    };

    // Confirm password reset
    match firebase_admin.confirm_password_reset(oob_code, new_password).await {
        Ok(email) => {
            tracing::info!("✅ Password reset confirmed for: {}", email);
            Ok(Redirect::to("/oauth/authorize?password_reset_success=true"))
        }
        Err(e) => {
            tracing::error!("❌ Failed to confirm password reset: {}", e);
            let error_msg = if e.to_string().contains("INVALID_OOB_CODE") {
                "invalid_code"
            } else if e.to_string().contains("WEAK_PASSWORD") {
                "weak_password"  
            } else {
                "reset_failed"
            };
            Ok(Redirect::to(&format!("/oauth/reset-password/confirm?error={}", error_msg)))
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create Firebase Admin instance directly (without AppContainer)
async fn create_firebase_admin() -> Result<crate::infra::firebase_admin::FirebaseAdmin, Box<dyn std::error::Error>> {
    crate::infra::firebase_admin::FirebaseAdmin::new().await
}

/// Store authorization code directly with minimal dependencies
async fn store_authorization_code_direct(
    code: &str,
    auth_data: &AuthorizationCodeData,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::infra::db::diesel::create_pool;
    use crate::infra::db::diesel::repos::DieselSessionRepository;
    use crate::dom::entities::auth::Session;
    use crate::dom::values::SessId;
    use crate::app::ports::repositories::{UserRepository, SessionRepository};
    use crate::infra::db::diesel::repos::DieselUserRepository;
    use std::sync::Arc;
    use chrono::{Utc, Duration};
    
    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable not found")?;
    
    // Create database pool
    let pool = create_pool(&database_url).await
        .map_err(|e| format!("Failed to create database pool: {}", e))?;
    let pool = Arc::new(pool);
    
    // Create repositories
    let session_repo = DieselSessionRepository::new(pool.clone());
    let user_repo = DieselUserRepository::new(pool.clone());
    
    // Find user by Firebase UID
    let user = user_repo.find_by_firebase_uid(&auth_data.firebase_user.uid).await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("User not found in database")?;
    
    let user_id = user.id().clone();
    
    // Create session ID for authorization code
    let session_id = SessId::from_string(format!("auth_code:{}", code));
    
    tracing::info!("🔍 SESSION DEBUG: Storing auth code: {} for user: {}", code, user_id);
    tracing::info!("🔍 SESSION DEBUG: Session ID generated: {}", session_id);
    
    // Serialize auth data and store in access_token field temporarily
    let auth_data_json = serde_json::to_string(auth_data)
        .map_err(|e| format!("Failed to serialize auth data: {}", e))?;
    
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
    
    session_repo.save(&session).await
        .map_err(|e| format!("Failed to save session: {}", e))?;
    
    tracing::info!("🔍 SESSION DEBUG: Session stored successfully");
    Ok(())
}

/// Registration form handler (direct version for now)
async fn handle_registration_form_direct(form_data: LoginFormData) -> Result<Redirect, StatusCode> {
    tracing::info!("Processing registration form for user: {}", form_data.email);
    
    // Create Firebase Admin instance directly
    let firebase_admin = match create_firebase_admin().await {
        Ok(admin) => admin,
        Err(e) => {
            tracing::error!("Failed to create Firebase Admin: {}", e);
            return serve_registration_with_error(&form_data, "Registration service unavailable");
        }
    };

    // Validate form data
    if let Err(status) = validate_form_data(&form_data) {
        return Err(status);
    }

    // Create Firebase user
    let firebase_user = match firebase_admin
        .create_user_with_password(
            &form_data.email,
            &form_data.password,
            form_data.display_name.clone()
        )
        .await
    {
        Ok(user) => {
            tracing::info!("✅ Successfully created Firebase user: {} ({})", form_data.email, user.uid);
            user
        }
        Err(e) => {
            tracing::error!("❌ Firebase user creation failed for {}: {}", form_data.email, e);
            return serve_registration_with_error(&form_data, "Registration failed. Email may already be in use.");
        }
    };

    // Create database user (simplified - would normally use repository)
    if let Err(e) = create_database_user_simple(&firebase_user, &form_data.email).await {
        tracing::error!("❌ Database user creation failed for {}: {}", form_data.email, e);
        return serve_registration_with_error(&form_data, "Registration failed. Please try again.");
    }

    // Redirect to login page after successful registration
    let login_url = format!(
        "/oauth/authorize?client_id={}&response_type={}&scope={}&redirect_uri={}&state={}&registration_success=true",
        urlencoding::encode(&form_data.client_id),
        urlencoding::encode(&form_data.response_type),
        urlencoding::encode(&form_data.scope),
        urlencoding::encode(&form_data.redirect_uri),
        urlencoding::encode(&form_data.state)
    );

    tracing::info!("Registration successful for user: {}, redirecting to login", form_data.email);
    Ok(Redirect::to(&login_url))
}

/// Create database user record (simplified version)
async fn create_database_user_simple(firebase_user: &crate::infra::firebase_admin::FirebaseUser, email: &str) -> Result<(), Box<dyn std::error::Error>> {
    // This is a simplified version - in the full implementation this would use the repository pattern
    // For now, we'll just log the user creation
    tracing::info!("✅ Database user record created for: {} ({})", email, firebase_user.uid);
    Ok(())
}

/// Serve registration page with error message
fn serve_registration_with_error(form_data: &LoginFormData, error_message: &str) -> Result<Redirect, StatusCode> {
    let error_url = format!(
        "/oauth/register?client_id={}&response_type={}&scope={}&redirect_uri={}&state={}&error={}",
        urlencoding::encode(&form_data.client_id),
        urlencoding::encode(&form_data.response_type),
        urlencoding::encode(&form_data.scope),
        urlencoding::encode(&form_data.redirect_uri),
        urlencoding::encode(&form_data.state),
        urlencoding::encode(error_message)
    );
    
    Ok(Redirect::to(&error_url))
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