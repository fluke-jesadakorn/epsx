// Authentication use cases

use crate::dom::entities::{User, Session};
use crate::dom::values::{UserId, Email, SessId};
use crate::app::ports::repositories::{UserRepo, SessRepo};
use crate::app::ports::services::FbAuthSvc;
use crate::app::dtos::auth::{LoginReq, LoginRes, LogoutReq, ValidateReq, RefreshReq, UserSession};
use std::sync::Arc;

pub struct AuthUC {
    user_repo: Arc<dyn UserRepo>,
    session_repo: Arc<dyn SessRepo>,
    firebase_auth: Arc<dyn FbAuthSvc>,
}

impl AuthUC {
    pub fn new(
        user_repo: Arc<dyn UserRepo>, 
        session_repo: Arc<dyn SessRepo>,
        firebase_auth: Arc<dyn FbAuthSvc>
    ) -> Self {
        Self { 
            user_repo,
            session_repo,
            firebase_auth
        }
    }

    pub async fn authenticate_user(&self, email: &Email) -> Result<Option<User>, Box<dyn std::error::Error>> {
        self.user_repo.find_by_email(email).await.map_err(|e| e.into())
    }

    pub async fn create_session(&self, user_id: UserId) -> Result<Session, Box<dyn std::error::Error>> {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        Ok(Session::new(user_id, expires_at))
    }

    pub async fn login(&self, req: LoginReq) -> Result<LoginRes, Box<dyn std::error::Error>> {
        // Verify Firebase token
        let claims = self.firebase_auth.verify_token(&req.firebase_token).await
            .map_err(|e| format!("Firebase token verification failed: {:?}", e))?;

        // Find user by Firebase UID  
        let user_id = UserId::new(claims.uid);
        let user = self.user_repo.find_by_id(&user_id).await?;

        // Create session
        let session = self.create_session(user.id().clone()).await?;
        
        // Store session
        self.session_repo.save(&session).await?;

        // Generate access token (stub implementation)
        let access_token = format!("token_{}", session.id.value());

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
        // Verify token with Firebase
        let claims = self.firebase_auth.verify_token(&req.token).await
            .map_err(|e| format!("Token validation failed: {:?}", e))?;

        // Find user by Firebase UID
        let user_id = UserId::new(claims.uid);
        let user = self.user_repo.find_by_id(&user_id).await?;

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