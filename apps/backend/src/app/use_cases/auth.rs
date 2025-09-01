// Authentication use cases

use crate::dom::entities::{User, Session};
use crate::dom::values::{UserId, Email, SessId};
// Removed role import - using permission-only system
use crate::app::ports::repositories::{UserRepository, SessionRepository};
use crate::app::dtos::auth::{LoginReq, LoginRes, LogoutReq, ValidateReq, RefreshReq, UserSession, AutoRegistrationRequest, RegistrationResponse};
use crate::app::services::PermissionApplicationService;
use crate::infra::FirebaseAdmin;
use crate::auth::{RefreshTokenService, DeviceInfo};
use std::sync::Arc;

pub struct AuthUC {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    firebase_admin: Arc<FirebaseAdmin>,
    permission_service: Arc<PermissionApplicationService>,
    refresh_token_service: Arc<RefreshTokenService>,
}

impl AuthUC {
    pub fn new(
        user_repo: Arc<dyn UserRepository>, 
        session_repo: Arc<dyn SessionRepository>,
        firebase_admin: Arc<FirebaseAdmin>,
        permission_service: Arc<PermissionApplicationService>,
        refresh_token_service: Arc<RefreshTokenService>,
    ) -> Self {
        Self { 
            user_repo,
            session_repo,
            firebase_admin,
            permission_service,
            refresh_token_service,
        }
    }


    pub async fn authenticate_user(&self, email: &Email) -> Result<Option<User>, Box<dyn std::error::Error>> {
        self.user_repo.find_by_email(email).await.map_err(|e| e.into())
    }

    pub async fn create_session(&self, user_id: UserId, access_token: String) -> Result<Session, Box<dyn std::error::Error>> {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        Ok(Session::new(user_id, access_token, expires_at))
    }

    pub async fn login(&self, req: LoginReq) -> Result<LoginRes, Box<dyn std::error::Error>> {
        // Authenticate with email/password via Firebase Admin SDK in backend
        let email = Email::new(req.email)?;
        
        // Verify user credentials with Firebase Admin (backend-only)
        let firebase_uid = self.authenticate_with_firebase(&email, &req.password).await?;
        
        // Find or create user by Firebase UID
        let user_id = UserId::new(firebase_uid.clone());
        let user = match self.user_repo.find_by_id(&user_id).await {
            Ok(user) => user,
            Err(_) => {
                // Auto-create user from Firebase (permissions handled separately)
                let user = User::from_existing(
                    user_id.clone(),
                    firebase_uid.clone(),
                    email.clone(),
                    // permissions parameter removed - now handled by separate table
                );
                
                // Save user first
                self.user_repo.save(&user).await?;
                
                // Set default permissions in separate table
                let default_permissions = vec!["epsx:analytics:view".to_string(), "epsx:user:read".to_string()];
                if let Err(e) = self.permission_service.set_user_permissions(user.id(), default_permissions).await {
                    tracing::error!("Failed to set default permissions for new user {}: {:?}", user.id(), e);
                    // Continue anyway since user was created successfully
                }
                user
            }
        };

        // Generate JWT token using Firebase Admin
        let access_token = self.firebase_admin.generate_jwt_token(&firebase_uid).await?;

        // Create session
        let session = self.create_session(user.id().clone(), access_token.clone()).await?;
        
        // Store session
        self.session_repo.save(&session).await?;

        // Fetch user permissions from separate table
        let user_firebase_uid = user.firebase_uid();
        let user_permissions = match self.permission_service.get_user_permissions(user_firebase_uid).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch user permissions for login response {}: {:?}", user.id(), e);
                vec![] // Return empty permissions on error
            }
        };

        Ok(LoginRes {
            user_id: user.id().clone(),
            package_tier: "user".to_string(), // Derived from permissions
            permissions: user_permissions,
            access_token,
            expires_in: 86400, // 24 hours
            sess_id: session.id.to_string(),
        })
    }

    async fn authenticate_with_firebase(&self, email: &Email, password: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Production Firebase authentication using Admin SDK
        match self.firebase_admin.authenticate_user(&email.to_string(), password).await {
            Ok(firebase_user) => {
                tracing::info!("Firebase authentication successful for: {}", email);
                Ok(firebase_user.uid)
            }
            Err(e) => {
                tracing::error!("Firebase authentication failed for {}: {}", email, e);
                Err(e)
            }
        }
    }

    pub async fn logout(&self, req: LogoutReq) -> Result<(), Box<dyn std::error::Error>> {
        let session_id = SessId::from_str(&req.session_id)?;
        self.session_repo.delete(&session_id).await?;
        Ok(())
    }

    pub async fn validate(&self, req: ValidateReq) -> Result<UserSession, Box<dyn std::error::Error>> {
        // Validate Firebase token and session
        let session_id = SessId::from_str(&req.sess_id)?;
        let session = self.session_repo.find_by_id(&session_id).await?;

        // Check if session is expired
        if session.expires_at < chrono::Utc::now() {
            return Err("Session expired".into());
        }

        let user = self.user_repo.find_by_id(&session.user_id).await?;

        // Permission-only system - permissions fetched from separate table
        let user_firebase_uid = user.firebase_uid();
        let user_permissions = match self.permission_service.get_user_permissions(user_firebase_uid).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch user permissions for session validation {}: {:?}", user.id(), e);
                vec![] // Return empty permissions on error
            }
        };

        Ok(UserSession {
            user_id: user.id().clone(),
            package_tier: "user".to_string(), // TODO: Derive from permissions 
            // Using structured permissions from separate table
            permissions: user_permissions,
            expires_at: session.expires_at,
        })
    }

    // Removed get_simple_permissions - using structured permissions from user entity

    pub async fn refresh(&self, req: RefreshReq) -> Result<LoginRes, Box<dyn std::error::Error>> {
        // Validate the refresh token
        let token_record = self.refresh_token_service.validate_token(&req.refresh_token).await
            .map_err(|e| format!("Invalid refresh token: {}", e))?;

        // Get user by ID
        let user_id = UserId::new(token_record.user_id.clone());
        let user = self.user_repo.find_by_id(&user_id).await
            .map_err(|e| format!("User not found: {}", e))?;

        // Rotate the refresh token (generate new one)
        let device_info = DeviceInfo {
            device_type: None,
            os: None, 
            browser: None,
            fingerprint: None,
            screen_resolution: None,
            timezone: None,
            language: None,
            platform: None,
            user_agent: None,
        };
        
        let _new_token_response = self.refresh_token_service.rotate_token(
            &req.refresh_token,
            Some(device_info),
            None, // IP address would come from request context
            None, // User agent would come from request headers
        ).await.map_err(|e| format!("Token rotation failed: {}", e))?;

        // Generate new JWT access token
        let access_token = self.firebase_admin.generate_jwt_token(user.firebase_uid()).await?;

        // Create new session
        let session = self.create_session(user.id().clone(), access_token.clone()).await?;
        self.session_repo.save(&session).await?;

        // Fetch user permissions
        let user_permissions = match self.permission_service.get_user_permissions(user.firebase_uid()).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch user permissions for refresh {}: {:?}", user.id(), e);
                vec![] // Return empty permissions on error
            }
        };

        Ok(LoginRes {
            user_id: user.id().clone(),
            package_tier: "user".to_string(),
            permissions: user_permissions,
            access_token,
            expires_in: 86400, // 24 hours
            sess_id: session.id.to_string(),
        })
    }

    /// Registration with automatic permission profile assignment
    pub async fn register_with_permission_profiles(&self, req: AutoRegistrationRequest) -> Result<RegistrationResponse, Box<dyn std::error::Error>> {
        // Create user in Firebase first
        let firebase_uid = self.firebase_admin.create_user(Some(req.email.clone()), Some(req.password.clone()), None).await?;
        let user_id = UserId::new(firebase_uid.clone());
        
        // Check if user already exists by Firebase UID (not email)
        if let Ok(_existing_user) = self.user_repo.find_by_id(&user_id).await {
            return Err("An account with this Firebase UID already exists".into());
        }

        // Use the actual email from registration request
        let email = Email::new(req.email.clone())?;
        
        // Set default permissions based on package tier
        let default_permissions = match req.package_tier.as_str() {
            "premium" => vec![
                "epsx:analytics:view".to_string(),
                "epsx:analytics:export".to_string(),
                "epsx:analytics:advanced".to_string(),
                "epsx:realtime:access".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string(),
                "epsx:billing:manage".to_string()
            ],
            "basic" => vec![
                "epsx:analytics:view".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string()
            ],
            _ => vec![
                "epsx:analytics:view".to_string(),
                "epsx:profile:manage".to_string(),
                "epsx:notifications:receive".to_string()
            ],
        };
        
        let user = User::from_existing(
            user_id.clone(),
            firebase_uid.clone(),
            email
        );
        
        // Save user
        self.user_repo.save(&user).await?;

        // Set default permissions in separate table
        if let Err(e) = self.permission_service.set_user_permissions(user.id(), default_permissions.clone()).await {
            tracing::error!("Failed to set default permissions for new user {}: {:?}", user.id(), e);
            // Continue anyway since user was created successfully
        }

        // Generate JWT access token
        let access_token = self.firebase_admin.generate_jwt_token(&firebase_uid).await?;
        let session = self.create_session(user_id.clone(), access_token.clone()).await?;
        self.session_repo.save(&session).await?;

        // Permission-based feature assignment - fetch from separate table
        let features_unlocked = match self.permission_service.get_user_permissions(user.firebase_uid()).await {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!("Failed to fetch permissions for registration response {}: {:?}", user.id(), e);
                default_permissions // Fallback to default permissions
            }
        };
        
        let assignment_results = vec![]; // Simplified - no complex assignment tracking
        let total_assigned = features_unlocked.len() as u32;

        tracing::info!(
            "User {} registered with {} features auto-assigned",
            user_id, total_assigned
        );

        Ok(RegistrationResponse {
            user_id,
            access_token,
            expires_in: 86400, // 24 hours
            features_unlocked,
            total_features_assigned: total_assigned,
            assignment_results,
        })
    }
}

/// Extract domain from email address
fn extract_email_domain(email: &str) -> String {
    email.split('@')
        .nth(1)
        .unwrap_or("unknown.com")
        .to_string()
}