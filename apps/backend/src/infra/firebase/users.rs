// Firebase User Management Operations
// Focused module handling user CRUD operations, profiles, and custom claims

use std::collections::HashMap;
use serde_json::{Value, json};
use chrono::Utc;
use tracing::{info, error};

use crate::config::env::get_env_var;
use super::types::{FirebaseAdmin, FirebaseUser, CreateUserRequest, UpdateUserRequest, GetUserResponse, FirebaseUserRecord, FirebaseErrorResponse};

impl FirebaseAdmin {
    /// Get user by Firebase UID (legacy method - may have authentication issues)
    pub async fn get_user(&self, firebase_uid: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/accounts:lookup?key={}",
                api_key
            );

            let payload = json!({
                "localId": [firebase_uid]
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                error!("Failed to get Firebase user {}: {}", firebase_uid, error_text);
                Err("Failed to get user".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // For Firebase Identity Toolkit, use API key as query parameter
        if let Ok(api_key) = get_env_var("FIREBASE_API_KEY") {
            let url = format!(
                "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:lookup?key={}",
                self.project_id, api_key
            );

            let payload = json!({
                "email": [email]
            });

            let response = self.client
                .post(&url)
                .json(&payload)
                .send()
                .await?;

            if response.status().is_success() {
                let user_response: GetUserResponse = response.json().await?;
                if let Some(users) = user_response.users {
                    if let Some(user_record) = users.first() {
                        return Ok(self.convert_user_record_to_firebase_user(user_record)?);
                    }
                }
                Err("User not found".into())
            } else {
                let error_text = response.text().await?;
                error!("Failed to get Firebase user by email {}: {}", email, error_text);
                Err("Failed to get user by email".into())
            }
        } else {
            Err("FIREBASE_API_KEY not configured".into())
        }
    }

    /// Create a new Firebase user (legacy method)
    pub async fn create_user(&self, email: Option<String>, password: Option<String>, display_name: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts",
            self.project_id
        );

        let create_request = CreateUserRequest {
            email,
            password,
            email_verified: Some(false),
            display_name,
            photo_url: None,
            disabled: Some(false),
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&create_request)
            .send()
            .await?;

        if response.status().is_success() {
            let user_response: serde_json::Value = response.json().await?;
            if let Some(local_id) = user_response["localId"].as_str() {
                info!("Firebase user created successfully: {}", local_id);
                return Ok(local_id.to_string());
            }
            Err("User creation response missing localId".into())
        } else {
            let error_text = response.text().await?;
            error!("Firebase user creation failed: {}", error_text);
            
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_EXISTS" => Err("Email already exists".into()),
                    "WEAK_PASSWORD" => Err("Password is too weak".into()),
                    _ => Err(format!("User creation failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("User creation failed".into())
            }
        }
    }

    /// Create a new Firebase user with email/password using Identity Toolkit API (preferred method)
    pub async fn create_user_with_password(&self, email: &str, password: &str, display_name: Option<String>) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        // Get Firebase API key from environment 
        let api_key = get_env_var("FIREBASE_API_KEY")
            .map_err(|_| "FIREBASE_API_KEY environment variable not set")?;
        
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/accounts:signUp?key={}",
            api_key
        );
        
        let mut signup_request = json!({
            "email": email,
            "password": password,
            "returnSecureToken": true
        });
        
        // Add display name if provided
        if let Some(name) = display_name {
            signup_request["displayName"] = json!(name);
        }
        
        info!("Creating Firebase user with email: {}", email);
        
        let response = self.client
            .post(&url)
            .json(&signup_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let signup_response: serde_json::Value = response.json().await?;
            
            // Extract user information from signup response
            let firebase_uid = signup_response["localId"]
                .as_str()
                .ok_or("Missing localId in Firebase signup response")?;
                
            let email_returned = signup_response["email"]
                .as_str()
                .ok_or("Missing email in Firebase signup response")?;
            
            let display_name = signup_response.get("displayName")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            // Create standard custom claims for all users
            let mut custom_claims = HashMap::new();
            custom_claims.insert("access_level".to_string(), Value::String("user".to_string()));
            custom_claims.insert("role".to_string(), Value::String("User".to_string()));
            
            // Create FirebaseUser from signup response
            Ok(FirebaseUser {
                uid: firebase_uid.to_string(),
                email: Some(email_returned.to_string()),
                email_verified: false, // New users need to verify email
                display_name,
                photo_url: None,
                phone_number: None,
                disabled: false,
                custom_claims,
                provider_data: vec![],
                created_at: Utc::now(),
                last_login_at: None, // Not logged in yet, just created
            })
        } else {
            let error_text = response.text().await?;
            error!("Firebase user creation failed for {}: {}", email, error_text);
            
            // Parse Firebase error for better error messages
            if let Ok(error_response) = serde_json::from_str::<FirebaseErrorResponse>(&error_text) {
                match error_response.error.message.as_str() {
                    "EMAIL_EXISTS" => Err("Email address is already in use".into()),
                    "WEAK_PASSWORD : Password should be at least 6 characters" => Err("Password must be at least 6 characters long".into()),
                    "INVALID_EMAIL" => Err("Invalid email address format".into()),
                    _ => Err(format!("User creation failed: {}", error_response.error.message).into()),
                }
            } else {
                Err("User creation failed".into())
            }
        }
    }

    /// Update Firebase user
    pub async fn update_user(&self, firebase_uid: &str, email: Option<String>, display_name: Option<String>, disabled: Option<bool>) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
            self.project_id
        );

        let update_request = UpdateUserRequest {
            local_id: firebase_uid.to_string(),
            email,
            email_verified: None,
            display_name,
            photo_url: None,
            disabled,
            custom_claims: None,
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&update_request)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Firebase user {} updated successfully", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("Firebase user update failed for {}: {}", firebase_uid, error_text);
            Err("User update failed".into())
        }
    }

    /// Delete Firebase user
    pub async fn delete_user(&self, firebase_uid: &str) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:delete",
            self.project_id
        );

        let payload = json!({
            "localId": firebase_uid
        });

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Firebase user {} deleted successfully", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("Firebase user deletion failed for {}: {}", firebase_uid, error_text);
            Err("User deletion failed".into())
        }
    }

    /// Set custom claims for Firebase user (for role management)
    pub async fn set_custom_claims(&self, firebase_uid: &str, custom_claims: HashMap<String, Value>) -> Result<(), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:update",
            self.project_id
        );

        let claims_json = serde_json::to_string(&custom_claims)?;
        let update_request = UpdateUserRequest {
            local_id: firebase_uid.to_string(),
            email: None,
            email_verified: None,
            display_name: None,
            photo_url: None,
            disabled: None,
            custom_claims: Some(claims_json),
        };

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&update_request)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Custom claims set successfully for Firebase user {}", firebase_uid);
            Ok(())
        } else {
            let error_text = response.text().await?;
            error!("Failed to set custom claims for {}: {}", firebase_uid, error_text);
            Err("Setting custom claims failed".into())
        }
    }

    /// List Firebase users (with pagination)
    pub async fn list_users(&self, max_results: Option<u32>, page_token: Option<String>) -> Result<(Vec<FirebaseUser>, Option<String>), Box<dyn std::error::Error>> {
        let access_token = self.get_access_token().await?;
        let url = format!(
            "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:batchGet",
            self.project_id
        );

        let mut payload = json!({
            "maxResults": max_results.unwrap_or(100)
        });

        if let Some(token) = page_token {
            payload["nextPageToken"] = json!(token);
        }

        let response = self.client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            let list_response: serde_json::Value = response.json().await?;
            
            let mut users = Vec::new();
            if let Some(users_array) = list_response["users"].as_array() {
                for user_value in users_array {
                    if let Ok(user_record) = serde_json::from_value::<FirebaseUserRecord>(user_value.clone()) {
                        if let Ok(firebase_user) = self.convert_user_record_to_firebase_user(&user_record) {
                            users.push(firebase_user);
                        }
                    }
                }
            }

            let next_page_token = list_response.get("nextPageToken")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Ok((users, next_page_token))
        } else {
            let error_text = response.text().await?;
            error!("Failed to list Firebase users: {}", error_text);
            Err("Failed to list users".into())
        }
    }

    /// Check if we're in development environment
    pub fn is_development_environment(&self) -> bool {
        get_env_var("RUST_ENV").unwrap_or_else(|_| "production".to_string()) == "development"
    }
    
    /// Check if credentials match test/development credentials
    pub fn is_test_credential(&self, email: &str, password: &str) -> bool {
        let test_credentials = vec![
            ("info@epsx.io", "P@ssword"),
            ("admin@epsx.io", "admin123"),
            ("test@epsx.io", "test123"),
        ];
        
        test_credentials.iter().any(|(test_email, test_password)| {
            email == *test_email && password == *test_password
        })
    }
    
    /// Create test Firebase user for development
    pub fn create_test_firebase_user(&self, email: &str, _password: &str) -> Result<FirebaseUser, Box<dyn std::error::Error>> {
        let mut custom_claims = HashMap::new();
        
        // Set role and permissions based on email
        match email {
            "info@epsx.io" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("super_admin".to_string()));
                custom_claims.insert("role".to_string(), Value::String("SuperAdmin".to_string()));
            },
            "admin@epsx.io" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("admin".to_string()));
                custom_claims.insert("role".to_string(), Value::String("Admin".to_string()));
            },
            _ => {
                custom_claims.insert("access_level".to_string(), Value::String("user".to_string()));
                custom_claims.insert("role".to_string(), Value::String("User".to_string()));
            }
        }
        
        // Generate a consistent but unique UID for test users
        let test_uid = format!("test_user_{}", email.replace("@", "_").replace(".", "_"));
        
        Ok(FirebaseUser {
            uid: test_uid,
            email: Some(email.to_string()),
            email_verified: true,
            display_name: Some(email.split('@').next().unwrap_or("Test User").to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims,
            provider_data: vec![],
            created_at: Utc::now(),
            last_login_at: Some(Utc::now()),
        })
    }

    /// Get IAM profile from custom claims
    pub fn get_iam_profile_from_custom_claims(&self, custom_claims: &HashMap<String, Value>) -> String {
        custom_claims.get("role")
            .and_then(|v| v.as_str())
            .map(|role| match role {
                "SuperAdmin" => "admin-full-004",
                "Admin" => "moderator-standard-003", 
                "Premium" => "user-premium-002",
                _ => "user-basic-001",
            })
            .unwrap_or("user-basic-001")
            .to_string()
    }

    /// Check if user has admin access
    pub fn user_has_admin_access(&self, firebase_user: &FirebaseUser) -> bool {
        firebase_user.custom_claims.get("admin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get admin access level
    pub fn get_admin_access_level(&self, firebase_user: &FirebaseUser) -> String {
        firebase_user.custom_claims.get("access_level")
            .and_then(|v| v.as_str())
            .unwrap_or("user")
            .to_string()
    }

    /// Set user role (convenience method)
    pub async fn set_user_role(&self, firebase_uid: &str, role: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut custom_claims = HashMap::new();
        
        match role {
            "SuperAdmin" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("super_admin".to_string()));
                custom_claims.insert("role".to_string(), Value::String("SuperAdmin".to_string()));
            },
            "Admin" => {
                custom_claims.insert("admin".to_string(), Value::Bool(true));
                custom_claims.insert("access_level".to_string(), Value::String("admin".to_string()));
                custom_claims.insert("role".to_string(), Value::String("Admin".to_string()));
            },
            "Premium" => {
                custom_claims.insert("access_level".to_string(), Value::String("premium".to_string()));
                custom_claims.insert("role".to_string(), Value::String("Premium".to_string()));
            },
            _ => {
                custom_claims.insert("access_level".to_string(), Value::String("user".to_string()));
                custom_claims.insert("role".to_string(), Value::String("User".to_string()));
            }
        }

        self.set_custom_claims(firebase_uid, custom_claims).await
    }

    /// Batch update users (for admin operations)
    pub async fn batch_update_users(&self, updates: Vec<(String, HashMap<String, Value>)>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut successful_updates = Vec::new();
        
        for (firebase_uid, custom_claims) in updates {
            match self.set_custom_claims(&firebase_uid, custom_claims).await {
                Ok(()) => {
                    successful_updates.push(firebase_uid.clone());
                    info!("Successfully updated user: {}", firebase_uid);
                },
                Err(e) => {
                    error!("Failed to update user {}: {}", firebase_uid, e);
                }
            }
        }
        
        Ok(successful_updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_credential_validation() {
        let admin = FirebaseAdmin::create_test_client();
        
        assert!(admin.is_test_credential("info@epsx.io", "P@ssword"));
        assert!(admin.is_test_credential("admin@epsx.io", "admin123"));
        assert!(!admin.is_test_credential("user@example.com", "wrong_password"));
    }

    #[test]
    fn test_create_test_firebase_user() {
        let admin = FirebaseAdmin::create_test_client();
        
        let user = admin.create_test_firebase_user("info@epsx.io", "P@ssword").unwrap();
        assert_eq!(user.email, Some("info@epsx.io".to_string()));
        assert!(admin.user_has_admin_access(&user));
        assert_eq!(admin.get_admin_access_level(&user), "super_admin");
    }

    #[test]
    fn test_iam_profile_from_custom_claims() {
        let admin = FirebaseAdmin::create_test_client();
        
        let mut claims = HashMap::new();
        claims.insert("role".to_string(), Value::String("SuperAdmin".to_string()));
        let profile = admin.get_iam_profile_from_custom_claims(&claims);
        assert_eq!(profile, "admin-full-004");

        claims.insert("role".to_string(), Value::String("Premium".to_string()));
        let profile = admin.get_iam_profile_from_custom_claims(&claims);
        assert_eq!(profile, "user-premium-002");
    }

    #[test]
    fn test_user_admin_access() {
        let admin = FirebaseAdmin::create_test_client();
        
        let super_admin_user = admin.create_test_firebase_user("info@epsx.io", "P@ssword").unwrap();
        assert!(admin.user_has_admin_access(&super_admin_user));
        assert_eq!(admin.get_admin_access_level(&super_admin_user), "super_admin");

        let regular_user = admin.create_test_firebase_user("user@example.com", "password").unwrap();
        assert!(!admin.user_has_admin_access(&regular_user));
        assert_eq!(admin.get_admin_access_level(&regular_user), "user");
    }

    #[test]
    fn test_development_environment_check() {
        let admin = FirebaseAdmin::create_test_client();
        // This will depend on the actual environment variable
        let _is_dev = admin.is_development_environment();
        // Just ensure it doesn't panic
    }
}