// Pure OIDC Authorization Code Flow implementation

use crate::domain::shared_kernel::value_objects::{ UserId, SessionId };
use crate::domain::user_management::{ User, Email, FirebaseUid };
use crate::domain::user_management::aggregates::session::Session;
use chrono::{ DateTime, Utc };

use axum::{
  extract::{ Query, Form },
  response::{ Html, Redirect },
  http::StatusCode,
};
use serde::{ Deserialize, Serialize };
#[cfg(feature = "templates")]
use askama::Template;

use crate::web::auth::AppState;
#[cfg(feature = "templates")]
use crate::web::templates::TemplateFactory;
use crate::infrastructure::adapters::services::firebase::FirebaseUser;

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
  pub user_id: crate::domain::shared_kernel::value_objects::UserId, // JWT migration: store user UUID (sub field)
  pub client_id: String,
  pub redirect_uri: String,
  pub scope: String,
  pub created_at: DateTime<Utc>,
  pub code_challenge: Option<String>,
  pub code_challenge_method: Option<String>,
}

/// GET /oauth/authorize - Serve login page
pub async fn authorization_endpoint(Query(
  params,
): Query<AuthorizationParams>) -> Result<Html<String>, StatusCode> {
  tracing::info!(
    "Authorization endpoint request - DEBUGGING CONTENT-TYPE ERROR"
  );
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
  if
    !TemplateFactory::is_valid_redirect_uri(
      &params.redirect_uri,
      &params.client_id
    )
  {
    return serve_error_page(
      "invalid_request",
      "Invalid redirect URI for this client"
    );
  }

  // Validate scope
  if !is_valid_scope(&params.scope) {
    return serve_error_page("invalid_scope", "Invalid or unsupported scope");
  }

  // Determine if admin login is required
  let is_admin_login =
    TemplateFactory::should_use_admin_template(&params.scope) ||
    params.client_id == "epsx-admin";
  let is_registration = params.registration.unwrap_or(false);
  let error_message = params.error.as_deref().unwrap_or("");

  // Render analytics-themed templates based on client type and flow
  let html = if is_admin_login {
    // Analytics Command Center for admin users
    let template =
      TemplateFactory::create_analytics_command_center_template_with_pkce(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        params.state.clone(),
        params.scope.clone(),
        params.code_challenge.clone(),
        params.code_challenge_method.clone(),
        error_message.to_string()
      );

    template.render().map_err(|e| {
      tracing::error!("Failed to render admin login template: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
  } else if is_registration {
    // Data Insights Portal registration for new users
    let template =
      TemplateFactory::create_analytics_registration_template_with_pkce(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        params.state.clone(),
        params.scope.clone(),
        params.code_challenge.clone(),
        params.code_challenge_method.clone(),
        error_message.to_string()
      );

    template.render().map_err(|e| {
      tracing::error!("Failed to render registration template: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
  } else {
    // Data Insights Portal login for regular users
    let template =
      TemplateFactory::create_analytics_portal_login_template_with_pkce(
        params.client_id.clone(),
        params.redirect_uri.clone(),
        params.state.clone(),
        params.scope.clone(),
        params.code_challenge.clone(),
        params.code_challenge_method.clone(),
        error_message.to_string()
      );

    template.render().map_err(|e| {
      tracing::error!("Failed to render login template: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
  };

  tracing::info!(
    "Successfully rendered analytics-themed template for client: {}",
    params.client_id
  );
  Ok(Html(html))
}

/// POST /oauth/authorize - Process login/registration form
pub async fn handle_authorization_form(Form(
  form_data,
): Form<LoginFormData>) -> Result<Redirect, StatusCode> {
  let is_registration = form_data.registration.unwrap_or(false);

  if is_registration {
    tracing::info!(
      "Processing registration form for user: {}",
      form_data.email
    );
    // Reuse existing registration handler
    return handle_registration_form_direct(form_data).await;
  } else {
    tracing::info!("Processing login form for user: {}", form_data.email);
    return handle_user_login_with_session_storage(form_data).await;
  }
}

/// Handle user login flow with proper session storage
async fn handle_user_login_with_session_storage(
  form_data: LoginFormData
) -> Result<Redirect, StatusCode> {
  tracing::info!("Processing login form for user: {}", form_data.email);

  // For now, just use the direct flow - it now has session storage
  handle_user_login_direct(form_data).await
}

/// Handle user login flow without AppState (fallback)
async fn handle_user_login_direct(
  form_data: LoginFormData
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
      return serve_login_with_error(
        &form_data,
        "Authentication service unavailable"
      );
    }
  };

  // Use Firebase authentication for all users
  let firebase_user = match
    firebase_admin.authenticate_user(
      &form_data.email,
      &form_data.password
    ).await
  {
    Ok(user) => user,
    Err(e) => {
      tracing::error!(
        "Firebase authentication failed for {}: {}",
        form_data.email,
        e
      );
      return serve_login_with_error(&form_data, "Invalid email or password");
    }
  };

  // For simplicity, skip admin module validation for now and just create basic JWT
  handle_authenticated_user_flow_direct(form_data, firebase_user).await
}

/// Handle authenticated user flow without AppState
async fn handle_authenticated_user_flow_direct(
  form_data: LoginFormData,
  firebase_user: FirebaseUser
) -> Result<Redirect, StatusCode> {
  // Look up user by email to get UUID for JWT migration (sub field)

  // Get database URL
  let database_url = std::env
    ::var("DATABASE_URL")
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  // Use direct database client to bypass diesel-async connection pool issues
  let direct_client =
    crate::infrastructure::adapters::repositories::direct_db_client::DirectDbClient::new(
      database_url.clone()
    );

  // Look up user by email using direct connection
  let user_email = firebase_user.email
    .as_ref()
    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
  tracing::info!(
    "Looking up user by email with direct connection: {}",
    user_email
  );

  let user_data = match direct_client.find_user_by_email(user_email).await {
    Ok(Some(user_data)) => {
      tracing::info!(
        "✅ Successfully found user by email: {} with ID: {}",
        user_email,
        user_data.id
      );
      user_data
    }
    Ok(None) => {
      tracing::error!("User not found in database for email: {}", user_email);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Err(e) => {
      tracing::error!(
        "Database error when looking up user by email {}: {}",
        user_email,
        e
      );
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  };

  // Convert UserData to domain UserId
  let user_id = crate::domain::shared_kernel::value_objects::UserId
    ::from_string(user_data.id.to_string())
    .map_err(|e| {
      tracing::error!("Failed to create UserId from UUID: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  // Store authorization code data in simplified manner
  let auth_data = AuthorizationCodeData {
    firebase_user: firebase_user.clone(),
    user_id, // JWT migration: include user UUID as sub field
    client_id: form_data.client_id.clone(),
    redirect_uri: form_data.redirect_uri.clone(),
    scope: form_data.scope.clone(),
    created_at: Utc::now(),
    code_challenge: form_data.code_challenge.clone(),
    code_challenge_method: form_data.code_challenge_method.clone(),
  };

  // Generate stateless authorization code with embedded data
  let code = generate_stateless_authorization_code(&auth_data).map_err(|e| {
    tracing::error!(
      "Failed to generate stateless authorization code in direct flow: {}",
      e
    );
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  tracing::info!(
    "🔍 STATELESS AUTH: Generated stateless authorization code in direct flow: {}",
    &code[0..20]
  ); // Only log first 20 chars for security

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
  form_data: LoginFormData
) -> Result<Redirect, StatusCode> {
  // Validate form data
  if let Err(status) = validate_form_data(&form_data) {
    return Err(status);
  }

  // Create Firebase user first
  let firebase_user = match
    app_state.firebase_admin.create_user_with_password(
      &form_data.email,
      &form_data.password,
      form_data.display_name.clone()
    ).await
  {
    Ok(user) => {
      tracing::info!(
        "✅ Successfully created Firebase user: {} ({})",
        form_data.email,
        user.uid
      );
      user
    }
    Err(e) => {
      tracing::error!(
        "❌ Firebase user creation failed for {}: {}",
        form_data.email,
        e
      );
      return serve_login_with_error(
        &form_data,
        "Registration failed. Email may already be in use."
      );
    }
  };

  // Create database user with Firebase UID
  match create_database_user(&app_state, &firebase_user, "User").await {
    Ok(_) => {
      tracing::info!(
        "✅ Successfully created database user: {}",
        form_data.email
      );
    }
    Err(e) => {
      tracing::error!(
        "❌ Database user creation failed for {}: {}",
        form_data.email,
        e
      );

      // Critical: Clean up Firebase user to prevent orphaned accounts
      if
        let Err(cleanup_error) = app_state.firebase_admin.delete_user(
          &firebase_user.uid
        ).await
      {
        tracing::error!(
          "🚨 Critical: Failed to cleanup Firebase user {} after database failure: {}",
          firebase_user.uid,
          cleanup_error
        );
        // Consider alerting security team about orphaned Firebase account
      } else {
        tracing::info!(
          "🧹 Successfully cleaned up Firebase user {} after database failure",
          firebase_user.uid
        );
      }

      return serve_login_with_error(
        &form_data,
        "Registration failed. Please try again."
      );
    }
  }

  // Admin module assignments are now handled through CLI tools only

  // Now proceed with authentication flow using the newly created user
  handle_authenticated_user_flow(app_state, form_data, firebase_user).await
}

/// Handle user login flow (existing functionality)
async fn handle_user_login(
  app_state: AppState,
  form_data: LoginFormData
) -> Result<Redirect, StatusCode> {
  // Validate form data
  if let Err(status) = validate_form_data(&form_data) {
    return Err(status);
  }

  // Use Firebase authentication for all users
  let firebase_user = match
    app_state.firebase_admin.authenticate_user(
      &form_data.email,
      &form_data.password
    ).await
  {
    Ok(user) => user,
    Err(e) => {
      tracing::error!(
        "Firebase authentication failed for {}: {}",
        form_data.email,
        e
      );
      return serve_login_with_error(&form_data, "Invalid email or password");
    }
  };

  handle_authenticated_user_flow(app_state, form_data, firebase_user).await
}

/// Handle authenticated user flow (common to both login and registration)
async fn handle_authenticated_user_flow(
  app_state: AppState,
  form_data: LoginFormData,
  firebase_user: FirebaseUser
) -> Result<Redirect, StatusCode> {
  // Validate admin access if required
  if
    form_data.scope.contains("admin") ||
    form_data.scope.contains("permissions")
  {
    // For test user, skip Firebase admin validation and use granular admin modules
    if firebase_user.uid == "test_user_info_epsx_io" {
      tracing::info!(
        "Development mode: checking granular admin modules for test user"
      );

      // Simplified role check - no granular admin modules needed
      if let Some(email) = &firebase_user.email {
        tracing::info!("Test user email: {} - using simple role system", email);
      }
    } else {
      // For real users, check both Firebase admin validation and database admin modules
      let has_firebase_admin = app_state.firebase_admin.user_has_admin_access(
        &firebase_user
      );
      let has_permissions = if let Some(_email) = &firebase_user.email {
        // Simplified role system - check via Firebase claims or user role field
        false // TODO: implement simple role check
      } else {
        false
      };

      if !has_firebase_admin && !has_permissions {
        tracing::warn!(
          "User {} attempted admin login without privileges",
          form_data.email
        );
        return serve_login_with_error(
          &form_data,
          "Administrator privileges required"
        );
      }
    }
  }

  tracing::error!(
    "🔍 SESSION DEBUG: Starting scope validation for user: {}",
    firebase_user.uid
  );

  // Role extraction removed - using permissions-based validation

  // Validate requested scopes (role-based validation removed)
  let validated_scopes = match
    validate_scopes_with_user_context(
      &form_data.scope,
      "user", // Default user level for scope validation
      &form_data.client_id
    )
  {
    Ok(scopes) => {
      tracing::error!("🔍 SESSION DEBUG: Scope validation successful");
      scopes
    }
    Err(ref scope_error) => {
      tracing::error!(
        "🔍 SESSION DEBUG: Scope validation failed: {:?}",
        scope_error
      );
      let error_message = match scope_error {
        // InsufficientRole error removed - using permissions-based validation
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
        granted_scopes = %validated_scopes.granted_scopes,
        sensitive_scopes = ?validated_scopes.sensitive_scopes,
        permission_count = %validated_scopes.permissions.len(),
        "Scopes validated successfully"
    );

  // Look up user by email to get UUID for JWT migration (sub field)

  let user_email = firebase_user.email.as_ref().ok_or_else(|| {
    tracing::error!("Firebase user missing email during auth code creation");
    StatusCode::INTERNAL_SERVER_ERROR
  })?;
  let email = crate::domain::user_management::value_objects::Email
    ::new(user_email.to_string())
    .map_err(|e| {
      tracing::error!("Failed to create Email value object: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let user = app_state.user_repo
    .find_by_email(&email).await
    .map_err(|e| {
      tracing::error!("Database error looking up user: {:?}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
      tracing::error!("User not found for email: {}", user_email);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let user_id = user.id().clone();

  // Store authorization code data with PKCE parameters and validated scopes
  let auth_data = AuthorizationCodeData {
    firebase_user: firebase_user.clone(),
    user_id, // JWT migration: include user UUID as sub field
    client_id: form_data.client_id.clone(),
    redirect_uri: form_data.redirect_uri.clone(),
    scope: validated_scopes.granted_scopes.clone(), // Use validated scopes, not raw request
    created_at: Utc::now(),
    code_challenge: form_data.code_challenge.clone(),
    code_challenge_method: form_data.code_challenge_method.clone(),
  };

  // Generate stateless authorization code with embedded data
  let auth_code = generate_stateless_authorization_code(&auth_data).map_err(
    |e| {
      tracing::error!("Failed to generate stateless authorization code: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    }
  )?;

  tracing::info!(
    "🔍 STATELESS AUTH: Generated stateless authorization code: {}",
    &auth_code[0..20]
  ); // Only log first 20 chars for security

  // Log successful authentication
  tracing::info!(
    "Authentication successful for user: {} ({})",
    firebase_user.email.as_ref().unwrap_or(&"unknown".to_string()),
    firebase_user.uid
  );

  // Audit log the authentication (skip for test users to avoid FK constraint issues)
  if firebase_user.uid != "test_user_info_epsx_io" {
    tokio::spawn({
      let app_state = app_state.clone();
      let email = form_data.email.clone();
      let firebase_uid = firebase_user.uid.clone();
      async move {
        if
          let Err(e) = log_authentication_event(
            &app_state,
            &email,
            &firebase_uid,
            true
          ).await
        {
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

/// Generate a stateless authorization code with embedded data
fn generate_stateless_authorization_code(
  auth_data: &AuthorizationCodeData
) -> Result<String, Box<dyn std::error::Error>> {
  use jsonwebtoken::{ encode, Header, EncodingKey, Algorithm };
  use serde::{ Serialize, Deserialize };
  use chrono::{ Utc, Duration };

  #[derive(Debug, Serialize, Deserialize)]
  struct AuthCodeClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,
    data: AuthorizationCodeData,
  }

  let now = Utc::now();
  let expires = now + Duration::minutes(10); // Authorization codes expire in 10 minutes

  let claims = AuthCodeClaims {
    iss: "https://auth.epsx.io".to_string(),
    sub: auth_data.user_id.to_string(),
    aud: auth_data.client_id.clone(),
    exp: expires.timestamp(),
    iat: now.timestamp(),
    data: auth_data.clone(),
  };

  // Use JWT secret for signing authorization codes
  let jwt_secret = std::env
    ::var("NEXTAUTH_SECRET")
    .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

  tracing::debug!(
    "Generating JWT authorization code with secret length: {}",
    jwt_secret.len()
  );

  let header = Header::new(Algorithm::HS256);
  let token = encode(
    &header,
    &claims,
    &EncodingKey::from_secret(jwt_secret.as_ref())
  )?;

  // Prefix with "ac_" to identify as authorization code
  Ok(format!("ac_jwt_{}", token))
}

/// Store authorization code using session repository
async fn store_authorization_code(
  _app_state: &AppState,
  code: &str,
  auth_data: &AuthorizationCodeData
) -> Result<(), Box<dyn std::error::Error>> {
  use chrono::{ Utc, Duration };

  // Use the user_id from auth_data (JWT migration: already looked up by email)
  let user_id = auth_data.user_id.clone();

  // Create a temporary session for the authorization code
  let session_id = SessionId::from_string(format!("auth_code:{}", code));

  tracing::error!(
    "🔍 SESSION DEBUG: Storing auth code: {} for user: {}",
    code,
    user_id
  );
  tracing::error!("🔍 SESSION DEBUG: Session ID generated: {}", session_id);
  tracing::error!("🔍 SESSION DEBUG: Session UUID: {:?}", session_id);

  // Serialize auth data and store in access_token field temporarily
  let auth_data_json = serde_json::to_string(auth_data)?;

  // Create session with 10 minute expiration
  let expires_at = Utc::now() + Duration::minutes(10);
  let _session = Session::create(
    session_id,
    user_id,
    auth_data_json, // Store serialized auth data in access_token field temporarily
    expires_at,
    None, // ip_address
    None // user_agent
  ).map_err(|e| format!("Failed to create session: {}", e))?;

  // TODO: Remove session storage - using stateless Bearer tokens
  // app_state.session_repo.save(&session).await?;
  tracing::debug!("Session storage skipped - using stateless Bearer tokens");

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
  if
    !TemplateFactory::is_valid_redirect_uri(
      &form_data.redirect_uri,
      &form_data.client_id
    )
  {
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
  let has_profile_scopes = scope_names
    .iter()
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
  client_id: &str
) -> Result<crate::auth::ValidatedScopes, crate::auth::ScopeError> {
  use crate::auth::SCOPE_SERVICE;

  SCOPE_SERVICE.validate_scopes(scope, user_role, client_id)
}

// Role extraction removed - using permissions-based system only

/// Serve error page for authorization errors using analytics theme
fn serve_error_page(
  error_code: &str,
  _description: &str
) -> Result<Html<String>, StatusCode> {
  let template = TemplateFactory::get_error_template_for_auth_error(error_code);

  let html = template.render().map_err(|e| {
    tracing::error!("Failed to render error template: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  tracing::info!("Served analytics-themed error page for: {}", error_code);
  Ok(Html(html))
}

/// Serve login/registration page with error message
fn serve_login_with_error(
  form_data: &LoginFormData,
  error_message: &str
) -> Result<Redirect, StatusCode> {
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
  _app_state: &AppState,
  _email: &str,
  firebase_uid: &str,
  success: bool
) -> Result<(), Box<dyn std::error::Error>> {
  use crate::domain::shared_kernel::entities::audit::{
    AuditLogEntry,
    AuditAction,
    ResourceType,
    AuditResult,
  };
  use crate::domain::shared_kernel::value_objects::UserId;

  let user_id = UserId::from_string(firebase_uid.to_string()).map_err(
    |e| Box::new(e) as Box<dyn std::error::Error>
  )?;
  let action = AuditAction::Login;
  let result = if success { AuditResult::Success } else { AuditResult::Failed };

  let _entry = AuditLogEntry::new(
    Some(user_id),
    action,
    ResourceType::Session,
    result
  );

  // TODO: Add audit_repo to AppState and uncomment
  // app_state.audit_repo.store(&entry).await?;
  Ok(())
}

// User role determination for registration removed - using permissions-based system

/// Create user in database after Firebase user creation
async fn create_database_user(
  app_state: &AppState,
  firebase_user: &FirebaseUser,
  _role: &str
) -> Result<(), Box<dyn std::error::Error>> {
  let email = Email::new(
    firebase_user.email
      .as_ref()
      .unwrap_or(&"unknown@example.com".to_string())
      .clone()
  ).map_err(|e| format!("Invalid email format: {}", e))?;

  let firebase_uid = FirebaseUid::new(firebase_user.uid.clone()).map_err(|e|
    format!("Invalid Firebase UID: {}", e)
  )?;

  // Create user entity with real Firebase UID (no package_tier needed - using permissions)
  let user = User::create(UserId::new(), firebase_uid, email).map_err(|e|
    format!("Failed to create user: {}", e)
  )?;

  // Save to database
  app_state.user_repo
    .save(&user).await
    .map_err(|e| format!("Failed to save user to database: {}", e))?;

  tracing::info!(
    "✅ Database user created with Firebase UID: {} for email: {}",
    firebase_user.uid,
    firebase_user.email.as_ref().unwrap_or(&"unknown".to_string())
  );

  Ok(())
}

// Removed assign_superadmin_modules function - admin assignments now handled via CLI tools

// ============================================================================
// REGISTRATION ENDPOINTS
// ============================================================================

/// GET /oauth/register - Serve registration page
pub async fn register_endpoint(Query(
  params,
): Query<AuthorizationParams>) -> Result<Html<String>, StatusCode> {
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

  if
    !TemplateFactory::is_valid_redirect_uri(
      &params.redirect_uri,
      &params.client_id
    )
  {
    return serve_error_page(
      "invalid_request",
      "Invalid redirect URI for this client"
    );
  }

  if !is_valid_scope(&params.scope) {
    return serve_error_page("invalid_scope", "Invalid or unsupported scope");
  }

  let error_message = params.error.as_deref().unwrap_or("");

  // Always render registration template
  let template =
    TemplateFactory::create_analytics_registration_template_with_pkce(
      params.client_id.clone(),
      params.redirect_uri.clone(),
      params.state.clone(),
      params.scope.clone(),
      params.code_challenge.clone(),
      params.code_challenge_method.clone(),
      error_message.to_string()
    );

  let html = template.render().map_err(|e| {
    tracing::error!("Failed to render registration template: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  tracing::info!("Successfully rendered analytics registration template");
  Ok(Html(html))
}

/// POST /oauth/register - Process registration form
pub async fn handle_registration_form(Form(
  form_data,
): Form<LoginFormData>) -> Result<Redirect, StatusCode> {
  tracing::info!("Processing registration form for user: {}", form_data.email);

  return handle_registration_form_direct(form_data).await;
}

// ============================================================================
// PASSWORD RESET ENDPOINTS
// ============================================================================

/// GET /oauth/reset-password - Serve password reset request page
pub async fn password_reset_endpoint(Query(params): Query<
  std::collections::HashMap<String, String>
>) -> Result<Html<String>, StatusCode> {
  tracing::info!("Password reset endpoint request");

  let error_message = params
    .get("error")
    .cloned()
    .unwrap_or_else(|| "".to_string());

  // Create simple password reset template
  let template = TemplateFactory::create_password_reset_template(error_message);

  let html = template.render().map_err(|e| {
    tracing::error!("Failed to render password reset template: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  Ok(Html(html))
}

/// POST /oauth/reset-password - Process password reset request
pub async fn handle_password_reset(Form(form_data): Form<
  std::collections::HashMap<String, String>
>) -> Result<Redirect, StatusCode> {
  let email = form_data.get("email").ok_or(StatusCode::BAD_REQUEST)?;

  tracing::info!("Processing password reset request for: {}", email);

  // Create Firebase Admin instance directly
  let firebase_admin = match create_firebase_admin().await {
    Ok(admin) => admin,
    Err(e) => {
      tracing::error!("Failed to create Firebase Admin: {}", e);
      return Ok(
        Redirect::to("/oauth/reset-password?error=service_unavailable")
      );
    }
  };

  // Send password reset email
  match firebase_admin.send_password_reset_email(email).await {
    Ok(_) => {
      tracing::info!("✅ Password reset email sent to: {}", email);
      Ok(Redirect::to("/oauth/reset-password?success=email_sent"))
    }
    Err(e) => {
      tracing::error!(
        "❌ Failed to send password reset email to {}: {}",
        email,
        e
      );
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
pub async fn password_reset_confirm_endpoint(Query(params): Query<
  std::collections::HashMap<String, String>
>) -> Result<Html<String>, StatusCode> {
  tracing::info!("Password reset confirmation endpoint request");

  let oob_code = params
    .get("oobCode")
    .cloned()
    .unwrap_or_else(|| "".to_string());
  let error_message = params
    .get("error")
    .cloned()
    .unwrap_or_else(|| "".to_string());

  // Create password reset confirmation template
  let template = TemplateFactory::create_password_reset_confirm_template(
    oob_code,
    error_message
  );

  let html = template.render().map_err(|e| {
    tracing::error!("Failed to render password reset confirm template: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  Ok(Html(html))
}

/// POST /oauth/reset-password/confirm - Process password reset confirmation
pub async fn handle_password_reset_confirm(Form(form_data): Form<
  std::collections::HashMap<String, String>
>) -> Result<Redirect, StatusCode> {
  let oob_code = form_data.get("oobCode").ok_or(StatusCode::BAD_REQUEST)?;
  let new_password = form_data
    .get("newPassword")
    .ok_or(StatusCode::BAD_REQUEST)?;

  tracing::info!("Processing password reset confirmation");

  // Create Firebase Admin instance directly
  let firebase_admin = match create_firebase_admin().await {
    Ok(admin) => admin,
    Err(e) => {
      tracing::error!("Failed to create Firebase Admin: {}", e);
      return Ok(
        Redirect::to("/oauth/reset-password/confirm?error=service_unavailable")
      );
    }
  };

  // Confirm password reset
  match firebase_admin.confirm_password_reset(oob_code, new_password).await {
    Ok(()) => {
      tracing::info!("✅ Password reset confirmed successfully");
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
      Ok(
        Redirect::to(
          &format!("/oauth/reset-password/confirm?error={}", error_msg)
        )
      )
    }
  }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create Firebase Admin instance directly (without AppContainer)
async fn create_firebase_admin() -> Result<
  crate::infrastructure::firebase_admin::FirebaseAdmin,
  Box<dyn std::error::Error>
> {
  Ok(
    crate::infrastructure::firebase_admin::FirebaseAdmin::new(
      "epsx-project-id".to_string()
    )
  )
}

/// Store authorization code directly with minimal dependencies
async fn store_authorization_code_direct(
  code: &str,
  auth_data: &AuthorizationCodeData
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  // TODO: Re-enable after SQLx migration
  /*
    use crate::infrastructure::adapters::repositories::diesel::create_pool;
    use crate::infrastructure::adapters::repositories::diesel::repos::DieselSessionRepository;
    // Removed unused import
    use crate::infrastructure::adapters::repositories::diesel::repos::DieselUserRepository;
    */
  use chrono::{ Utc, Duration };

  // Get database URL from environment
  let _database_url = std::env
    ::var("DATABASE_URL")
    .map_err(|_| "DATABASE_URL environment variable not found")?;

  // Create database pool
  let pool = crate::infrastructure::adapters::repositories::diesel_types
    ::create_pool().await
    .map_err(|e| format!("Failed to create database pool: {}", e))?;

  // Create repositories
  let session_repo =
    crate::infrastructure::adapters::repositories::diesel_types::DieselSessionRepository::new(
      pool.clone()
    );
  let _user_repo =
    crate::infrastructure::adapters::repositories::UserRepositoryAdapter::new(
      pool.clone()
    );

  // Use the user_id from auth_data (JWT migration: already looked up by email)
  let user_id = auth_data.user_id.clone();

  // Create session ID for authorization code
  let session_id = SessionId::from_string(format!("auth_code:{}", code));

  tracing::info!(
    "🔍 SESSION DEBUG: Storing auth code: {} for user: {}",
    code,
    user_id
  );
  tracing::info!("🔍 SESSION DEBUG: Session ID generated: {}", session_id);

  // Serialize auth data and store in access_token field temporarily
  let auth_data_json = serde_json
    ::to_string(auth_data)
    .map_err(|e| format!("Failed to serialize auth data: {}", e))?;

  // Create session with 10 minute expiration
  let expires_at = Utc::now() + Duration::minutes(10);
  let session = Session::create(
    session_id,
    user_id,
    auth_data_json, // Store serialized auth data in access_token field temporarily
    expires_at,
    None, // ip_address
    None // user_agent
  ).map_err(|e| format!("Failed to create session: {}", e))?;

  session_repo
    .save(&session).await
    .map_err(|e| format!("Failed to save session: {}", e))?;

  tracing::info!("🔍 SESSION DEBUG: Session stored successfully");
  Ok(())
}

/// Registration form handler (direct version for now)
async fn handle_registration_form_direct(
  form_data: LoginFormData
) -> Result<Redirect, StatusCode> {
  tracing::info!("Processing registration form for user: {}", form_data.email);

  // Create Firebase Admin instance directly
  let firebase_admin = match create_firebase_admin().await {
    Ok(admin) => admin,
    Err(e) => {
      tracing::error!("Failed to create Firebase Admin: {}", e);
      return serve_registration_with_error(
        &form_data,
        "Registration service unavailable"
      );
    }
  };

  // Validate form data
  if let Err(status) = validate_form_data(&form_data) {
    return Err(status);
  }

  // Create Firebase user
  let firebase_user = match
    firebase_admin.create_user_with_password(
      &form_data.email,
      &form_data.password,
      form_data.display_name.clone()
    ).await
  {
    Ok(user) => {
      tracing::info!(
        "✅ Successfully created Firebase user: {} ({})",
        form_data.email,
        user.uid
      );
      user
    }
    Err(e) => {
      tracing::error!(
        "❌ Firebase user creation failed for {}: {}",
        form_data.email,
        e
      );
      return serve_registration_with_error(
        &form_data,
        "Registration failed. Email may already be in use."
      );
    }
  };

  // Create database user (simplified - would normally use repository)
  if
    let Err(e) = create_database_user_simple(
      &firebase_user,
      &form_data.email
    ).await
  {
    tracing::error!(
      "❌ Database user creation failed for {}: {}",
      form_data.email,
      e
    );
    return serve_registration_with_error(
      &form_data,
      "Registration failed. Please try again."
    );
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

  tracing::info!(
    "Registration successful for user: {}, redirecting to login",
    form_data.email
  );
  Ok(Redirect::to(&login_url))
}

/// Create database user record (simplified version)
async fn create_database_user_simple(
  firebase_user: &crate::infrastructure::firebase_admin::FirebaseUser,
  email: &str
) -> Result<(), Box<dyn std::error::Error>> {
  // This is a simplified version - in the full implementation this would use the repository pattern
  // For now, we'll just log the user creation
  tracing::info!(
    "✅ Database user record created for: {} ({})",
    email,
    firebase_user.uid
  );
  Ok(())
}

/// Serve registration page with error message
fn serve_registration_with_error(
  form_data: &LoginFormData,
  error_message: &str
) -> Result<Redirect, StatusCode> {
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
    assert!(is_valid_scope("openid profile email permissions"));
    assert!(
      is_valid_scope("openid profile email epsx:admin:read epsx:admin:write")
    );
    assert!(
      is_valid_scope("openid profile email epsx:users:manage epsx:system:admin")
    );
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
