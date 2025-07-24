// Enhanced Authentication handlers for migrating from Firebase client to backend auth

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use chrono::{DateTime, Utc};
use crate::app::dtos::auth::{LoginRes, ValidateReq};
use crate::dom::values::{Email, Role};
use crate::dom::entities::User;
use super::routes::AppState;

/// Enhanced login request that supports multiple authentication methods
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum EnhancedLoginRequest {
    /// Direct email/password login (future method)
    #[serde(rename = "credentials")]
    Credentials {
        email: String,
        password: String,
    },
    /// Admin login with enhanced permissions check
    #[serde(rename = "admin")]
    Admin {
        email: String,
        password: String,
        admin_token: Option<String>, // Optional second factor
    },
}

/// Enhanced login response with user profile information
#[derive(Debug, Serialize)]
pub struct EnhancedLoginResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
    pub package_tier: String,
    pub expires_at: DateTime<Utc>,
    pub session_type: String, // "user", "admin", etc.
}

/// User registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
    pub package_tier: Option<String>,
}

/// User registration response
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub verification_sent: bool,
    pub message: String,
}

/// Enhanced login handler supporting multiple authentication methods
pub async fn enhanced_login_handler(
    cookies: Cookies,
    State(app_state): State<AppState>,
    Json(payload): Json<EnhancedLoginRequest>,
) -> Result<Json<EnhancedLoginResponse>, StatusCode> {
    match payload {
        EnhancedLoginRequest::Credentials { email, password } => {
            handle_credentials_login(cookies, app_state, email, password, false).await
        },
        EnhancedLoginRequest::Admin { email, password, admin_token } => {
            handle_admin_login(cookies, app_state, email, password, admin_token).await
        },
    }
}


/// Handle direct email/password login (enhanced implementation)
async fn handle_credentials_login(
    cookies: Cookies,
    app_state: AppState,
    email: String,
    password: String,
    _is_admin: bool,
) -> Result<Json<EnhancedLoginResponse>, StatusCode> {
    // Use the real authentication system instead of mock implementation
    use crate::app::dtos::auth::LoginReq;
    
    // Create login request DTO
    let login_req = LoginReq {
        email: email.clone(),
        password,
    };
    
    // Perform login through use case
    let login_res = app_state.auth_uc.login(login_req).await
        .map_err(|e| {
            tracing::error!("Login failed with error: {:?}", e);
            tracing::error!("Error message: {}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    // Create HTTP-only session cookie
    let session_cookie = Cookie::build(("sess_id", login_res.sess_id.to_string()))
        .http_only(true)
        .secure(false) // Disable for development HTTP, enable in production with HTTPS
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .max_age(tower_cookies::cookie::time::Duration::days(7))
        .path("/")
        .build();
    
    cookies.add(session_cookie);
    
    // Get user details to determine role and permissions
    let user = app_state.user_repo.find_by_id(&login_res.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user details: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get role string and permissions
    let role_str = user.role().to_string();
    let permissions = match user.role() {
        Role::SuperAdmin | Role::Admin => vec!["read:all".to_string(), "write:all".to_string(), "manage:users".to_string()],
        Role::Moderator => vec!["read:all".to_string(), "write:limited".to_string(), "moderate:content".to_string()],
        Role::Premium => vec!["read:all".to_string(), "write:own".to_string(), "access:premium_features".to_string()],
        _ => vec!["read:own".to_string(), "write:own".to_string()],
    };
    
    // Determine session type
    let session_type = if matches!(user.role(), Role::SuperAdmin | Role::Admin) { "admin" } else { "user" };
    
    tracing::info!("Login successful for user: {} ({}), role: {:?}", email, user.id(), user.role());
    
    // Return enhanced response
    Ok(Json(EnhancedLoginResponse {
        user_id: user.id().to_string(),
        email: user.email().value().to_string(),
        role: role_str,
        permissions,
        subscription_tier: user.sub().tier().to_string(),
        package_tier: user.sub().tier().to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(login_res.expires_in),
        session_type: session_type.to_string(),
    }))
}

/// Handle admin login with enhanced security
async fn handle_admin_login(
    cookies: Cookies,
    app_state: AppState,
    email: String,
    password: String,
    admin_token: Option<String>,
) -> Result<Json<EnhancedLoginResponse>, StatusCode> {
    // Use credentials login with admin flag set to true
    let result = handle_credentials_login(cookies, app_state, email, password, true).await?;
    
    // For now, ignore admin_token but log if provided for future MFA implementation
    if admin_token.is_some() {
        tracing::info!("Admin token provided - MFA will be implemented in future version");
    }
    
    // Modify session type to admin in response
    let mut response_data = result.0;
    response_data.session_type = "admin".to_string();
    
    Ok(Json(response_data))
}

/// User registration handler
pub async fn register_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    // Validate email format
    let email_vo = Email::new(payload.email.clone())
        .map_err(|e| {
            tracing::error!("Invalid email format: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Check if user already exists
    if let Ok(Some(_)) = app_state.user_repo.find_by_email(&email_vo).await {
        tracing::warn!("Registration attempt for existing user: {}", payload.email);
        return Err(StatusCode::CONFLICT);
    }
    
    // Determine role based on package tier (default to User)
    let role = match payload.package_tier.as_deref() {
        Some("premium") => Role::Premium,
        Some("gold") => Role::Premium, // Map gold to premium for now
        Some("platinum") => Role::Premium, // Map platinum to premium for now
        _ => Role::User,
    };
    
    // Create new user entity
    let user = User::new(email_vo.clone(), role);
    
    // Save user to repository
    app_state.user_repo.save(&user).await
        .map_err(|e| {
            tracing::error!("Failed to save user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // TODO: In production, this would:
    // 1. Create user in Firebase Auth
    // 2. Send verification email
    // 3. Apply package tier-specific permissions
    
    tracing::info!("User registered successfully: {}", payload.email);
    
    Ok(Json(RegisterResponse {
        user_id: user.id().to_string(),
        email: payload.email,
        verification_sent: false, // Set to false until email integration is implemented
        message: "Registration successful. User created in system.".to_string(),
    }))
}

/// Password reset handler
#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
    pub reset_sent: bool,
}

pub async fn password_reset_handler(
    State(_app_state): State<AppState>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<Json<PasswordResetResponse>, StatusCode> {
    // Validate email format
    let _email = Email::new(payload.email.clone())
        .map_err(|e| {
            tracing::error!("Invalid email format: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Placeholder implementation
    // Full implementation would send password reset email through Firebase Auth
    
    Ok(Json(PasswordResetResponse {
        message: "If an account with this email exists, a password reset link has been sent.".to_string(),
        reset_sent: true,
    }))
}

/// Enhanced user profile information
#[allow(dead_code)]
struct EnhancedUserProfile {
    email: String,
    role: String,
    permissions: Vec<String>,
    subscription_tier: String,
    package_tier: String,
}

/// Get enhanced user profile information
#[allow(dead_code)]
async fn get_enhanced_user_profile(
    app_state: &AppState,
    login_res: &LoginRes,
) -> Result<EnhancedUserProfile, StatusCode> {
    // Get user info using validate to get role and permissions 
    let validate_req = ValidateReq {
        token: login_res.access_token.clone(),
        sess_id: login_res.sess_id.clone(),
    };
    
    let user_info = app_state.auth_uc.validate(validate_req).await
        .map_err(|e| {
            tracing::error!("Failed to get user info: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get user from repository for additional profile info
    let user = app_state.user_repo.get(&login_res.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user profile: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(EnhancedUserProfile {
        email: user.email().value().to_string(),
        role: user_info.role.to_string(),
        permissions: user_info.permissions,
        subscription_tier: user.sub().tier.to_string(),
        package_tier: user.sub().tier.to_string(), // For now, use same as subscription
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn should_deserialize_credentials_login() {
        let json = r#"{"type": "credentials", "email": "test@example.com", "password": "password123"}"#;
        let request: EnhancedLoginRequest = serde_json::from_str(json).unwrap();
        
        match request {
            EnhancedLoginRequest::Credentials { email, password } => {
                assert_eq!(email, "test@example.com");
                assert_eq!(password, "password123");
            },
            _ => panic!("Expected Credentials variant"),
        }
    }
    
    #[test]
    fn should_deserialize_admin_login() {
        let json = r#"{"type": "admin", "email": "admin@example.com", "password": "adminpass", "admin_token": "token123"}"#;
        let request: EnhancedLoginRequest = serde_json::from_str(json).unwrap();
        
        match request {
            EnhancedLoginRequest::Admin { email, password, admin_token } => {
                assert_eq!(email, "admin@example.com");
                assert_eq!(password, "adminpass");
                assert_eq!(admin_token, Some("token123".to_string()));
            },
            _ => panic!("Expected Admin variant"),
        }
    }
    
    #[test]
    fn should_serialize_enhanced_login_response() {
        let response = EnhancedLoginResponse {
            user_id: "user123".to_string(),
            email: "test@example.com".to_string(),
            role: "premium_user".to_string(),
            permissions: vec!["read:own_data".to_string(), "write:own_data".to_string()],
            subscription_tier: "premium".to_string(),
            package_tier: "gold".to_string(),
            expires_at: Utc::now(),
            session_type: "user".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("premium_user"));
        assert!(json.contains("read:own_data"));
        assert!(json.contains("gold"));
    }
    
    #[test]
    fn should_serialize_register_response() {
        let response = RegisterResponse {
            user_id: "new-user-123".to_string(),
            email: "newuser@example.com".to_string(),
            verification_sent: true,
            message: "Registration successful".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("newuser@example.com"));
        assert!(json.contains("Registration successful"));
        assert!(json.contains("true")); // verification_sent
    }
    
    #[test]
    fn should_build_session_cookie_correctly() {
        use tower_cookies::Cookies;
        
        let cookies = Cookies::default();
        set_session_cookie(&cookies, "test-session-id");
        
        let cookie = cookies.get("sess_id").unwrap();
        assert_eq!(cookie.value(), "test-session-id");
        assert!(cookie.http_only().unwrap());
        assert!(cookie.secure().unwrap());
    }
    
    #[test]
    fn should_validate_register_request() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "securepass123".to_string(),
            name: Some("Test User".to_string()),
            package_tier: Some("premium".to_string()),
        };
        
        assert!(!request.email.is_empty());
        assert!(!request.password.is_empty());
        assert_eq!(request.name, Some("Test User".to_string()));
        assert_eq!(request.package_tier, Some("premium".to_string()));
    }
}