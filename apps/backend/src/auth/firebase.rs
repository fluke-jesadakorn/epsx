use jsonwebtoken::{ decode, DecodingKey, Validation, Algorithm };
use crate::{config::Config, db::{DB, models::User}};
use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use tracing::{debug, info};
use anyhow::Result;
use serde_json::Value;
use mongodb::bson::{doc, DateTime};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Admin,
    User,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUser {
    pub uid: String,
    pub email: Option<String>,
    pub roles: Vec<UserRole>,
    pub token: String,
}

#[derive(Debug, Deserialize)]
struct ServiceAccount {
    #[serde(rename = "private_key")]
    private_key_pem: String,
    _client_email: String,
    project_id: String,
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    sub: String,
    email: Option<String>,
    #[allow(dead_code)]
    claims: Option<Value>,
}

pub struct FirebaseAdmin {
    credentials: Arc<ServiceAccount>,
    validation: Validation,
    db: Arc<DB>,
}

impl FirebaseAdmin {
    pub async fn new(service_account_path: &str, db: Arc<DB>) -> Result<Self> {
        debug!("Initializing Firebase Admin with service account from: {}", service_account_path);

        let (project_id, _client_email, private_key_pem) =
            Config::load_firebase_config(service_account_path);

        let credentials = ServiceAccount {
            project_id,
            _client_email,
            private_key_pem,
        };

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&credentials.project_id]);
        validation.set_issuer(
            &[&format!("https://securetoken.google.com/{}", credentials.project_id)]
        );
        validation.validate_exp = true;
        validation.validate_nbf = false;

        Ok(Self {
            credentials: Arc::new(credentials),
            validation,
            db,
        })
    }

    pub async fn verify_token(&self, token: &str) -> Result<FirebaseUser> {
        debug!("Verifying Firebase token");

        let decoding_key = DecodingKey::from_rsa_pem(
            self.credentials.private_key_pem.as_bytes()
        ).map_err(|e| anyhow::anyhow!("Failed to create decoding key: {}", e))?;

        let token_data = decode::<TokenClaims>(token, &decoding_key, &self.validation).map_err(|e|
            anyhow::anyhow!("Failed to verify token: {}", e)
        )?;

        // Get or create user in MongoDB
        let user = match self.db.get_users()
            .find_one(doc! { "firebase_uid": &token_data.claims.sub }, None)
            .await? {
                Some(user) => {
                    // Update last login
                    self.db.get_users()
                        .update_one(
                            doc! { "firebase_uid": &token_data.claims.sub },
                            doc! { 
                                "$set": { 
                                    "metadata.last_login": DateTime::now(),
                                    "metadata.updated_at": DateTime::now()
                                }
                            },
                            None,
                        )
                        .await?;
                    user
                },
                None => {
                    // Create new user
                    let new_user = User::new(
                        token_data.claims.sub.clone(),
                        token_data.claims.email.clone().unwrap_or_default(),
                    );
                    self.db.get_users()
                        .insert_one(&new_user, None)
                        .await?;
                    info!("Created new user in MongoDB for Firebase user: {}", &token_data.claims.sub);
                    new_user
                }
            };

        Ok(FirebaseUser {
            uid: token_data.claims.sub,
            email: token_data.claims.email,
            roles: user.roles.iter()
                .map(|r| match r.as_str() {
                    "admin" => UserRole::Admin,
                    _ => UserRole::User,
                })
                .collect(),
            token: token.to_string(),
        })
    }

    pub async fn set_user_roles(&self, uid: &str, roles: Vec<String>) -> Result<()> {
        // Update roles in MongoDB
        self.db.get_users()
            .update_one(
                doc! { "firebase_uid": uid },
                doc! { 
                    "$set": { 
                        "roles": &roles,
                        "metadata.updated_at": DateTime::now()
                    }
                },
                None,
            )
            .await?;

        // Update Firebase custom claims
        self.sync_user_claims(uid).await?;

        Ok(())
    }

    pub async fn sync_user_claims(&self, uid: &str) -> Result<()> {
        // Get user from MongoDB
        let user = self.db.get_users()
            .find_one(doc! { "firebase_uid": uid }, None)
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Get user's current subscription
        let subscription_plan = user.subscription.as_ref().map(|s| s.plan_id.clone());

        // Construct claims
        let claims = doc! {
            "roles": &user.roles,
            "subscription": subscription_plan,
            "permissions": self.get_role_permissions(&user.roles).await?
        };

        // TODO: Implement Firebase custom claims update
        // This would require the Firebase Admin SDK's set_custom_user_claims functionality
        // For now, we'll just log the claims we would set
        debug!("Would set Firebase custom claims for user {}: {:?}", uid, claims);

        Ok(())
    }

    async fn get_role_permissions(&self, role_names: &[String]) -> Result<Vec<String>> {
        let mut permissions = Vec::new();

        for role_name in role_names {
            if let Some(role) = self.db.get_roles()
                .find_one(doc! { "name": role_name }, None)
                .await? 
            {
                permissions.extend(role.permissions);
            }
        }

        Ok(permissions.into_iter().collect())
    }

    pub async fn update_user_subscription(
        &self,
        uid: &str,
        plan_id: &str,
        end_date: DateTime,
    ) -> Result<()> {
        // Get associated role for the plan
        let role = match plan_id {
            "free" => "free",
            "personal" => "personal",
            "company" => "company",
            "api" => "api",
            _ => return Err(anyhow::anyhow!("Invalid plan ID")),
        };

        // Update subscription and roles in MongoDB
        self.db.get_users()
            .update_one(
                doc! { "firebase_uid": uid },
                doc! { 
                    "$set": { 
                        "subscription": {
                            "plan_id": plan_id,
                            "status": "active",
                            "start_date": DateTime::now(),
                            "end_date": end_date
                        },
                        "roles": [role],
                        "metadata.updated_at": DateTime::now()
                    }
                },
                None,
            )
            .await?;

        // Sync changes to Firebase claims
        self.sync_user_claims(uid).await?;

        Ok(())
    }
}
