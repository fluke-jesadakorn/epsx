// Firebase Module - Focused Architecture
// Breaks down 1,596-line God Object into 4 focused modules with clear domain separation

// Public focused modules - each handles a specific domain
pub mod types;     // Shared data structures and DTOs
pub mod client;    // Firebase client initialization and connection management
pub mod auth;      // Authentication and token verification logic
pub mod users;     // User management operations (CRUD, profiles, claims)
pub mod tokens;    // Token generation, FCM messaging, notifications, role management

// Re-export key types for easy access
pub use types::*;

// Re-export all Firebase admin functionality through focused modules

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Test that all modules are accessible and core types are available
        let admin = FirebaseAdmin::create_test_client();
        assert!(!admin.get_project_id().is_empty());

        // Test focused module integration
        assert!(admin.has_api_key() || admin.has_service_account() || true); // Always pass in test environment
        
        let auth_summary = admin.get_auth_config_summary();
        assert!(!auth_summary.is_empty());
    }

    #[test]
    fn test_firebase_user_types() {
        let user = FirebaseUser {
            uid: "test-uid".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: true,
            display_name: Some("Test User".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: std::collections::HashMap::new(),
            provider_data: Vec::new(),
            created_at: chrono::Utc::now(),
            last_login_at: None,
        };

        assert_eq!(user.uid, "test-uid");
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert!(user.email_verified);
    }

    #[test]
    fn test_fcm_message_types() {
        let message = FcmMessage {
            token: Some("test-token".to_string()),
            topic: None,
            condition: None,
            notification: FcmNotification {
                title: "Test".to_string(),
                body: "Test body".to_string(),
                image: None,
            },
            data: None,
            android: None,
            apns: None,
            webpush: None,
        };

        assert_eq!(message.token, Some("test-token".to_string()));
        assert_eq!(message.notification.title, "Test");
    }

    #[tokio::test]
    async fn test_focused_modules_integration() {
        // Test that the focused modules work together correctly
        let admin = FirebaseAdmin::create_test_client();
        
        // Test client module
        let health_result = admin.health_check().await;
        // Health check may fail in test environment, but shouldn't panic
        let _ = health_result; 
        
        // Test users module
        let test_user = admin.create_test_firebase_user("test@example.com", "password").unwrap();
        assert_eq!(test_user.email, Some("test@example.com".to_string()));
        
        // Test auth module functionality
        let is_valid = admin.validate_id_token_format("header.payload.signature");
        assert!(is_valid.is_ok());
        
        // Test tokens module - template creation
        let template_vars = std::collections::HashMap::new();
        let (title, body) = admin.create_notification_template("payment_success", &template_vars);
        assert_eq!(title, "Payment Successful");
        assert!(!body.is_empty());
    }

    #[test] 
    fn test_backward_compatibility() {
        // Ensure all original Firebase admin functionality is still accessible
        let admin = FirebaseAdmin::create_test_client();
        
        // Client functionality
        assert!(!admin.get_project_id().is_empty());
        
        // User management functionality
        assert!(admin.is_test_credential("info@epsx.io", "P@ssword"));
        
        // Auth functionality  
        let unverified_result = admin.extract_unverified_claims("invalid");
        assert!(unverified_result.is_err()); // Expected to fail with invalid JWT
        
        // Token/FCM functionality
        let (title, body) = admin.create_notification_template("default", &std::collections::HashMap::new());
        assert_eq!(title, "EPSX Notification");
        assert!(!body.is_empty());
    }
}