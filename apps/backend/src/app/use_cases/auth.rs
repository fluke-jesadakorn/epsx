// Authentication use cases

use crate::dom::entities::{User, Session};
use crate::dom::values::{UserId, Email, SessId};
use crate::auth::roles::Role;
use crate::app::ports::repositories::{UserRepo, SessRepo};
use crate::app::dtos::auth::{LoginReq, LoginRes, LogoutReq, ValidateReq, RefreshReq, UserSession, AutoRegistrationRequest, RegistrationResponse, FeatureAssignmentResult};
use crate::dom::services::auto_assignment::{AutoAssignmentEngine, RegistrationContext};
use crate::infra::FirebaseAdmin;
use std::sync::Arc;

pub struct AuthUC {
    user_repo: Arc<dyn UserRepo>,
    session_repo: Arc<dyn SessRepo>,
    auto_assignment_engine: Option<Arc<AutoAssignmentEngine>>,
    firebase_admin: Arc<FirebaseAdmin>,
}

impl AuthUC {
    pub fn new(
        user_repo: Arc<dyn UserRepo>, 
        session_repo: Arc<dyn SessRepo>,
        firebase_admin: Arc<FirebaseAdmin>
    ) -> Self {
        Self { 
            user_repo,
            session_repo,
            auto_assignment_engine: None,
            firebase_admin,
        }
    }

    pub fn with_auto_assignment(mut self, engine: Arc<AutoAssignmentEngine>) -> Self {
        self.auto_assignment_engine = Some(engine);
        self
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
                // Auto-create user from Firebase
                let user = User::from_existing(
                    user_id.clone(),
                    firebase_uid.clone(),
                    email.clone(),
                    Role::Guest.to_string(),
                );
                self.user_repo.save(&user).await?;
                user
            }
        };

        // Generate JWT token using Firebase Admin
        let access_token = self.firebase_admin.generate_jwt_token(&firebase_uid).await?;

        // Create session
        let session = self.create_session(user.id().clone(), access_token.clone()).await?;
        
        // Store session
        self.session_repo.save(&session).await?;

        Ok(LoginRes {
            user_id: user.id().clone(),
            package_tier: user.package_tier().to_string(),
            admin_modules: user.admin_modules().clone(),
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

        // Simple role-based system - get user's role
        let user_role = user.role(); // Use the role field from the user
        let simple_permissions = self.get_simple_permissions(&user_role.to_string());

        Ok(UserSession {
            user_id: user.id().clone(),
            package_tier: user_role.to_string(), // Role instead of package tier
            admin_modules: vec![], // No admin modules in simple system
            permissions: simple_permissions,
            expires_at: session.expires_at,
        })
    }

    fn get_simple_permissions(&self, role: &str) -> Vec<String> {
        use crate::auth::roles::{Role, get_role_features};
        
        // Parse the role string and get associated features
        let user_role = match role {
            "admin" => Role::Admin,
            "user" => Role::User,
            "guest" => Role::Guest,
            _ => Role::Guest, // Default to guest for unknown roles
        };
        
        // Get all features that this role has access to
        get_role_features(&user_role)
    }

    pub async fn refresh(&self, _req: RefreshReq) -> Result<LoginRes, Box<dyn std::error::Error>> {
        // Stub implementation - in real app would validate refresh token
        Err("Refresh not implemented".into())
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
        let user = User::from_existing(
            user_id.clone(),
            firebase_uid.clone(),
            email,
            Role::Guest.to_string(), // Default role
        );
        
        // Save user
        self.user_repo.save(&user).await?;

        // Generate JWT access token
        let access_token = self.firebase_admin.generate_jwt_token(&firebase_uid).await?;
        let session = self.create_session(user_id.clone(), access_token.clone()).await?;
        self.session_repo.save(&session).await?;

        // If auto-assignment engine is available, process permission profile assignments
        let (features_unlocked, assignment_results, total_assigned) = if let Some(engine) = &self.auto_assignment_engine {
            let context = RegistrationContext {
                email: req.email.clone(),
                package_tier: crate::dom::services::auto_assignment::PackageTier::from(req.package_tier.as_str()),
                referral_code: req.referral_code,
                source: req.source,
                region: req.region,
                email_domain: extract_email_domain(&req.email),
                user_agent: None, // Would come from HTTP headers
                utm_source: req.utm_source,
                utm_campaign: req.utm_campaign,
            };

            match engine.process_registration(&user_id, &context).await {
                Ok(results) => {
                    let features: Vec<String> = results
                        .assignments
                        .iter()
                        .filter(|a| a.success)
                        .map(|a| a.feature_id.clone())
                        .collect();

                    let assignment_results: Vec<FeatureAssignmentResult> = results
                        .assignments
                        .into_iter()
                        .map(|a| FeatureAssignmentResult {
                            feature_id: a.feature_id,
                            profile_name: format!("Profile_{}", a.permission_profile_id), // Would fetch actual name
                            success: a.success,
                            reason: a.reason,
                            expires_at: a.expires_at,
                        })
                        .collect();

                    (features, assignment_results, results.total_assigned)
                }
                Err(e) => {
                    tracing::warn!("Auto-assignment failed for user {}: {}", user_id, e);
                    (vec![], vec![], 0)
                }
            }
        } else {
            (vec![], vec![], 0)
        };

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