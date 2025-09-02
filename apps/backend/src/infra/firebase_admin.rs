// Firebase Admin - Refactored with Focused Module Architecture
// Originally 1,596 lines - now split into 4 focused modules with domain separation

// Re-export all functionality from focused modules for backward compatibility
pub use super::firebase::*;

// Re-export core types that are used throughout the codebase
pub use super::firebase::types::{
    FirebaseAdmin, FirebaseUser, FirebasePublicKey, UserProvider,
    FcmMessage, FcmNotification, FcmResponse, DeviceToken, DevicePlatform,
    JWTClaims, AuthRequest, CreateUserRequest, UpdateUserRequest
};

// Test module for backward compatibility verification
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_backward_compatibility() {
        // Test that all original exports are still available
        let admin = FirebaseAdmin::create_test_client();
        
        // Test client functionality
        assert!(!admin.get_project_id().is_empty());
        assert!(!admin.get_auth_config_summary().is_empty());
        
        // Test IAM profile mapping with proper custom claims
        let mut test_claims = std::collections::HashMap::new();
        test_claims.insert("role".to_string(), serde_json::Value::String("User".to_string()));
        let profile = admin.get_iam_profile_from_custom_claims(&test_claims);
        assert_eq!(profile, "user-basic-001");
    }

    #[test]
    fn test_focused_modules_integration() {
        // Test that focused modules work together correctly
        let admin = FirebaseAdmin::create_test_client();

        // Test authentication module
        let validation_result = admin.validate_id_token_format("header.payload.signature");
        assert!(validation_result.is_ok());

        // Test user access checking with proper Firebase user structure
        let mut admin_claims = std::collections::HashMap::new();
        admin_claims.insert("admin".to_string(), serde_json::Value::Bool(true));
        admin_claims.insert("access_level".to_string(), serde_json::Value::String("admin".to_string()));
        
        let admin_user = FirebaseUser {
            uid: "test-admin-uid".to_string(),
            email: Some("admin@example.com".to_string()),
            email_verified: true,
            display_name: Some("Test Admin".to_string()),
            photo_url: None,
            phone_number: None,
            disabled: false,
            custom_claims: admin_claims,
            provider_data: vec![],
            created_at: chrono::Utc::now(),
            last_login_at: Some(chrono::Utc::now()),
        };
        
        assert!(admin.user_has_admin_access(&admin_user));
        assert_eq!(admin.get_admin_access_level(&admin_user), "admin");

        // Test tokens module
        let template_vars = HashMap::new();
        let (title, body) = admin.create_notification_template("account_security", &template_vars);
        assert_eq!(title, "Security Alert");
        assert!(body.contains("sign-in"));
    }

    #[test]
    fn test_fcm_types_available() {
        // Test that FCM types are still accessible
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

    #[test]
    fn test_jwt_claims_types() {
        let claims = JWTClaims {
            sub: "firebase-uid".to_string(),
            email: "user@example.com".to_string(),
            iat: 1234567890,
            exp: 1234567890 + 3600,
        };

        let json = serde_json::to_string(&claims).unwrap();
        assert!(json.contains("firebase-uid"));
        assert!(json.contains("user@example.com"));
    }

    #[tokio::test]
    async fn test_client_factory_methods() {
        // Test factory methods still work
        let admin1 = FirebaseAdmin::with_project_id("test-project".to_string()).await.unwrap();
        assert_eq!(admin1.get_project_id(), "test-project");

        let admin2 = FirebaseAdmin::create_test_client();
        assert_eq!(admin2.get_project_id(), "test-project");
    }
}