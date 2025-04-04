pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod firebase;

use crate::config::Config;
use serde_json;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: String,
    pub email: Option<String>,
    pub token: String,
    pub roles: Vec<firebase::UserRole>,
}

pub use routes::auth_router;

#[derive(Clone)]
pub struct AuthService {
    firebase_service_account_path: String,
}

impl AuthService {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            firebase_service_account_path: config.firebase_service_account_path,
        })
    }

    pub async fn validate_session(&self, token: &str) -> Result<AuthUser, errors::AuthError> {
        let firebase_admin = firebase::FirebaseAdmin
            ::new(&self.firebase_service_account_path).await
            .map_err(|e| errors::AuthError::internal(&format!("Firebase init error: {}", e)))?;

        let firebase_user = firebase_admin
            .verify_token(token).await
            .map_err(|e| errors::AuthError::unauthorized(&format!("Invalid token: {}", e)))?;

        Ok(AuthUser {
            user_id: firebase_user.uid.to_owned(),
            email: firebase_user.email.clone(),
            token: token.to_owned(),
            roles: firebase_user.roles,
        })
    }

    async fn _set_user_roles(
        &self,
        uid: &str,
        roles: Vec<firebase::UserRole>
    ) -> Result<(), errors::AuthError> {
        let _firebase_admin = firebase::FirebaseAdmin
            ::new(&self.firebase_service_account_path).await
            .map_err(|e| errors::AuthError::internal(&format!("Firebase init error: {}", e)))?;

        // Convert roles to custom claims
        let claims =
            serde_json::json!({
            "roles": roles.iter().map(|r| format!("{:?}", r).to_lowercase()).collect::<Vec<_>>()
        });

        self._set_custom_claims(uid, claims).await?;
        Ok(())
    }

    async fn _get_user_roles(
        &self,
        uid: &str
    ) -> Result<Vec<firebase::UserRole>, errors::AuthError> {
        let _firebase_admin = firebase::FirebaseAdmin
            ::new(&self.firebase_service_account_path).await
            .map_err(|e| errors::AuthError::internal(&format!("Firebase init error: {}", e)))?;

        let claims = self._get_user_custom_claims(&uid).await?;

        let roles = claims
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str().and_then(|s| {
                            match s {
                                "admin" => Some(firebase::UserRole::Admin),
                                "user" => Some(firebase::UserRole::User),
                                _ => None,
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(|| vec![firebase::UserRole::User]);

        Ok(roles)
    }

    async fn _get_token_from_credentials(
        &self,
        _email: &str,
        _password: &str
    ) -> Result<String, errors::AuthError> {
        unimplemented!("Firebase get token from credentials not implemented")
    }

    async fn _generate_custom_token(&self, _uid: &str) -> Result<String, errors::AuthError> {
        unimplemented!("Firebase custom token generation not implemented")
    }

    async fn _set_custom_claims(
        &self,
        _uid: &str,
        _claims: serde_json::Value
    ) -> Result<(), errors::AuthError> {
        unimplemented!("Firebase set custom claims not implemented")
    }

    async fn _get_user_custom_claims(
        &self,
        _uid: &str
    ) -> Result<serde_json::Value, errors::AuthError> {
        unimplemented!("Firebase get user claims not implemented")
    }
}
