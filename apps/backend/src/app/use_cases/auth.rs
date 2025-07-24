// Authentication use cases

use crate::dom::entities::{User, Session};
use crate::dom::values::{UserId, Email, SessId};
use crate::app::ports::repositories::{UserRepo, SessRepo};
use crate::app::dtos::auth::{LoginReq, LoginRes, LogoutReq, ValidateReq, RefreshReq, UserSession};
use std::sync::Arc;

pub struct AuthUC {
    user_repo: Arc<dyn UserRepo>,
    session_repo: Arc<dyn SessRepo>,
}

impl AuthUC {
    pub fn new(
        user_repo: Arc<dyn UserRepo>, 
        session_repo: Arc<dyn SessRepo>
    ) -> Self {
        Self { 
            user_repo,
            session_repo
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
        // Authenticate with email/password
        let email = Email::new(req.email)?;
        let user = self.authenticate_user(&email).await?
            .ok_or("User not found")?;
        
        // Basic password validation for development
        // TODO: Implement proper password hashing and verification
        if req.password.is_empty() {
            return Err("Invalid password".into());
        }
        
        // For development: accept any non-empty password for existing users
        tracing::info!("Password validation passed for user: {}", email.value());

        // Generate access token (stub implementation)
        let access_token = format!("token_{}", user.id().to_string());

        // Create session
        let session = self.create_session(user.id().clone(), access_token.clone()).await?;
        
        // Store session
        self.session_repo.save(&session).await?;

        Ok(LoginRes {
            user_id: user.id().clone(),
            role: user.role().clone(),
            access_token,
            expires_in: 86400, // 24 hours
            sess_id: session.id.value().to_string(),
        })
    }

    pub async fn logout(&self, req: LogoutReq) -> Result<(), Box<dyn std::error::Error>> {
        let session_id = SessId::from_str(&req.session_id)?;
        self.session_repo.delete(&session_id).await?;
        Ok(())
    }

    pub async fn validate(&self, req: ValidateReq) -> Result<UserSession, Box<dyn std::error::Error>> {
        // TODO: Implement JWT token validation
        let session_id = SessId::from_str(&req.sess_id)?;
        let session = self.session_repo.find_by_id(&session_id).await?;

        let user = self.user_repo.find_by_id(&session.user_id).await?;

        Ok(UserSession {
            user_id: user.id().clone(),
            role: user.role().clone(),
            permissions: vec!["read:own".to_string(), "write:own".to_string()], // Mock permissions
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        })
    }

    pub async fn refresh(&self, _req: RefreshReq) -> Result<LoginRes, Box<dyn std::error::Error>> {
        // Stub implementation - in real app would validate refresh token
        Err("Refresh not implemented".into())
    }
}