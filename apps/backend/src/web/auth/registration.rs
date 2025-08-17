// User Registration API Endpoint
// Integrates with Firebase Admin and database user creation

use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::infra::AppContainer;
use crate::dom::entities::User;
use crate::dom::values::{Email, Role};

/// Registration request from frontend
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub client_id: Option<String>,
}

/// Registration response to frontend
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub user_id: Option<String>,
    pub firebase_uid: Option<String>,
    pub admin_modules: Option<Vec<String>>,
}

/// Registration error response
#[derive(Debug, Serialize)]
pub struct RegisterErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<String>,
}

/// POST /api/auth/register - User Registration Endpoint
pub async fn register_user(
    State(container): State<Arc<AppContainer>>,
    Json(request): Json<RegisterRequest>,
) -> Result<ResponseJson<RegisterResponse>, (StatusCode, ResponseJson<RegisterErrorResponse>)> {
    tracing::info!("🚀 Registration request for email: {}", request.email);
    
    // Validate email format
    if !request.email.contains('@') || request.email.len() < 5 {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(RegisterErrorResponse {
                success: false,
                error: "invalid_email".to_string(),
                details: Some("Email format is invalid".to_string()),
            }),
        ));
    }
    
    // Validate password strength
    if request.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(RegisterErrorResponse {
                success: false,
                error: "weak_password".to_string(),
                details: Some("Password must be at least 8 characters".to_string()),
            }),
        ));
    }
    
    // Check if user already exists
    let email = Email::new(request.email.clone())
        .map_err(|e| {
            tracing::error!("❌ Email validation failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                ResponseJson(RegisterErrorResponse {
                    success: false,
                    error: "invalid_email".to_string(),
                    details: Some(e),
                }),
            )
        })?;
    
    if let Ok(Some(_)) = container.user_repo.find_by_email(&email).await {
        return Err((
            StatusCode::CONFLICT,
            ResponseJson(RegisterErrorResponse {
                success: false,
                error: "user_exists".to_string(),
                details: Some("User with this email already exists".to_string()),
            }),
        ));
    }
    
    tracing::info!("✅ Email {} is available for registration", request.email);
    
    // Create Firebase user
    let firebase_user = match container.firebase_admin
        .create_user_with_password(
            &request.email,
            &request.password,
            request.display_name.clone(),
        )
        .await
    {
        Ok(user) => {
            tracing::info!("✅ Firebase user created: {}", user.uid);
            user
        }
        Err(e) => {
            tracing::error!("❌ Firebase user creation failed: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(RegisterErrorResponse {
                    success: false,
                    error: "firebase_error".to_string(),
                    details: Some("Failed to create user account".to_string()),
                }),
            ));
        }
    };
    
    // Determine user role (SuperAdmin for info@epsx.io, otherwise User)
    let user_role = if request.email == "info@epsx.io" {
        Role::SuperAdmin
    } else {
        Role::User
    };
    
    // Create database user
    let user = User::new(
        firebase_user.uid.clone(),
        email,
        user_role.clone(),
    );
    
    if let Err(e) = container.user_repo.save(&user).await {
        tracing::error!("❌ Database user creation failed: {}", e);
        
        // TODO: Consider cleaning up Firebase user on database failure
        
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(RegisterErrorResponse {
                success: false,
                error: "database_error".to_string(),
                details: Some("Failed to save user profile".to_string()),
            }),
        ));
    }
    
    tracing::info!("✅ Database user created with ID: {}", user.id());
    
    // Assign admin modules if this is SuperAdmin
    let admin_modules = if request.email == "info@epsx.io" {
        match container.admin_module_service
            .assign_all_admin_modules(
                &firebase_user.uid,
                &firebase_user.uid, // Self-assigned during registration
                "SuperAdmin auto-assignment during registration"
            )
            .await
        {
            Ok(modules) => {
                tracing::info!("✅ Assigned {} admin modules to SuperAdmin", modules.len());
                Some(modules)
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to assign admin modules: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    // Log successful registration
    tracing::info!(
        "🎉 User registration completed successfully: {} ({})", 
        request.email, 
        firebase_user.uid
    );
    
    Ok(ResponseJson(RegisterResponse {
        success: true,
        message: "Registration successful".to_string(),
        user_id: Some(user.id().to_string()),
        firebase_uid: Some(firebase_user.uid),
        admin_modules,
    }))
}

/// POST /api/auth/check-email - Check if email is available for registration
pub async fn check_email_availability(
    State(container): State<Arc<AppContainer>>,
    Json(request): Json<CheckEmailRequest>,
) -> Result<ResponseJson<EmailAvailabilityResponse>, (StatusCode, ResponseJson<RegisterErrorResponse>)> {
    let email = Email::new(request.email.clone())
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                ResponseJson(RegisterErrorResponse {
                    success: false,
                    error: "invalid_email".to_string(),
                    details: Some(e),
                }),
            )
        })?;
    
    let is_available = match container.user_repo.find_by_email(&email).await {
        Ok(Some(_)) => false,
        Ok(None) => true,
        Err(_) => false, // Consider unavailable on error
    };
    
    Ok(ResponseJson(EmailAvailabilityResponse {
        email: request.email,
        available: is_available,
    }))
}

/// Email availability check request
#[derive(Debug, Deserialize)]
pub struct CheckEmailRequest {
    pub email: String,
}

/// Email availability response
#[derive(Debug, Serialize)]
pub struct EmailAvailabilityResponse {
    pub email: String,
    pub available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_request_validation() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            display_name: Some("Test User".to_string()),
            client_id: Some("epsx-frontend".to_string()),
        };
        
        assert!(request.email.contains('@'));
        assert!(request.password.len() >= 8);
    }
}