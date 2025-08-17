// Template rendering for OIDC login pages

use askama::Template;
use serde::{Deserialize, Serialize};
use base64::Engine;
use crate::config::env::get_env_var;

/// Template for standard user login
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub error: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Template for admin login with enhanced security features
#[derive(Template)]
#[template(path = "admin_login.html")]
pub struct AdminLoginTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub error: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

// TODO: Implement registration templates once HTML files are created
// /// Template for user registration
// #[derive(Template)]
// #[template(path = "register.html")]
// pub struct RegistrationTemplate {
//     pub client_id: String,
//     pub redirect_uri: String,
//     pub state: String,
//     pub scope: String,
//     pub error: String,
//     pub code_challenge: Option<String>,
//     pub code_challenge_method: Option<String>,
// }

// /// Template for admin user registration
// #[derive(Template)]
// #[template(path = "admin_register.html")]
// pub struct AdminRegistrationTemplate {
//     pub client_id: String,
//     pub redirect_uri: String,
//     pub state: String,
//     pub scope: String,
//     pub error: String,
//     pub code_challenge: Option<String>,
//     pub code_challenge_method: Option<String>,
// }

/// Template for authentication errors
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error_message: String,
    pub error_details: String,
}

/// Template for logout success page
#[derive(Template)]
#[template(path = "logout_success.html")]
pub struct LogoutSuccessTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
}

/// Template for Firebase authentication page
#[derive(Template)]
#[template(path = "firebase_auth.html")]
pub struct FirebaseAuthTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub tenant_hint: Option<String>,
    pub domain_hint: String,
    pub firebase_config: FirebaseConfig,
}

/// Firebase configuration for the template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseConfig {
    pub api_key: String,
    pub auth_domain: String,
    pub project_id: String,
    pub storage_bucket: String,
    pub messaging_sender_id: String,
    pub app_id: String,
}

impl FirebaseConfig {
    /// Create Firebase config from environment variables
    pub fn from_env() -> Self {
        Self {
            api_key: get_env_var("FIREBASE_API_KEY")
                .unwrap_or_else(|_| "AIzaSyDtGcR8wF9f2M3VqQ7sN1xK9yP5tE8rU2wX".to_string()),
            auth_domain: get_env_var("FIREBASE_AUTH_DOMAIN")
                .unwrap_or_else(|_| "epsx-project.firebaseapp.com".to_string()),
            project_id: get_env_var("FIREBASE_PROJECT_ID")
                .unwrap_or_else(|_| "epsx-project".to_string()),
            storage_bucket: get_env_var("FIREBASE_STORAGE_BUCKET")
                .unwrap_or_else(|_| "epsx-project.appspot.com".to_string()),
            messaging_sender_id: get_env_var("FIREBASE_MESSAGING_SENDER_ID")
                .unwrap_or_else(|_| "123456789012".to_string()),
            app_id: get_env_var("FIREBASE_APP_ID")
                .unwrap_or_else(|_| "1:123456789012:web:abcdef123456789012345".to_string()),
        }
    }
}

/// Security features configuration for admin template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFeatures {
    pub threat_detection: bool,
    pub device_fingerprinting: bool,
    pub risk_scoring: bool,
    pub audit_logging: bool,
    pub session_monitoring: bool,
}

impl Default for SecurityFeatures {
    fn default() -> Self {
        Self {
            threat_detection: true,
            device_fingerprinting: true,
            risk_scoring: true,
            audit_logging: true,
            session_monitoring: true,
        }
    }
}

/// Template factory for creating login templates
pub struct TemplateFactory;

impl TemplateFactory {
    /// Create standard login template (without PKCE)
    pub fn create_login_template(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        error: String,
    ) -> LoginTemplate {
        LoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    /// Create standard login template with PKCE parameters
    pub fn create_login_template_with_pkce(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        error: String,
    ) -> LoginTemplate {
        LoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge,
            code_challenge_method,
        }
    }

    /// Create admin login template (without PKCE)
    pub fn create_admin_login_template(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        error: String,
    ) -> AdminLoginTemplate {
        AdminLoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    /// Create admin login template with PKCE parameters
    pub fn create_admin_login_template_with_pkce(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        error: String,
    ) -> AdminLoginTemplate {
        AdminLoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge,
            code_challenge_method,
        }
    }

    // TODO: Implement registration template methods once HTML files are created
    // /// Create user registration template with PKCE parameters
    // pub fn create_registration_template_with_pkce(
    //     client_id: String,
    //     redirect_uri: String,
    //     state: String,
    //     scope: String,
    //     code_challenge: Option<String>,
    //     code_challenge_method: Option<String>,
    //     error: String,
    // ) -> RegistrationTemplate {
    //     RegistrationTemplate {
    //         client_id,
    //         redirect_uri,
    //         state,
    //         scope,
    //         error,
    //         code_challenge,
    //         code_challenge_method,
    //     }
    // }

    // /// Create admin user registration template with PKCE parameters
    // pub fn create_admin_registration_template_with_pkce(
    //     client_id: String,
    //     redirect_uri: String,
    //     state: String,
    //     scope: String,
    //     code_challenge: Option<String>,
    //     code_challenge_method: Option<String>,
    //     error: String,
    // ) -> AdminRegistrationTemplate {
    //     AdminRegistrationTemplate {
    //         client_id,
    //         redirect_uri,
    //         state,
    //         scope,
    //         error,
    //         code_challenge,
    //         code_challenge_method,
    //     }
    // }

    /// Create error template
    pub fn create_error_template(
        error_message: String,
        error_details: String,
    ) -> ErrorTemplate {
        ErrorTemplate {
            error_message,
            error_details,
        }
    }

    /// Create logout success template
    pub fn create_logout_success_template(
        client_id: String,
        redirect_uri: String,
        state: String,
    ) -> LogoutSuccessTemplate {
        LogoutSuccessTemplate {
            client_id,
            redirect_uri,
            state,
        }
    }

    /// Create Firebase authentication template
    pub fn create_firebase_auth_template(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        tenant_hint: Option<String>,
    ) -> FirebaseAuthTemplate {
        let firebase_config = FirebaseConfig::from_env();
        let domain_hint = get_env_var("FIREBASE_DOMAIN_HINT")
            .unwrap_or_else(|_| "epsx.com".to_string());
            
        FirebaseAuthTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            tenant_hint,
            domain_hint,
            firebase_config,
        }
    }

    /// Determine if admin template should be used based on scope
    pub fn should_use_admin_template(scope: &str) -> bool {
        scope.contains("admin") || scope.contains("administrator")
    }

    /// Generate secure state parameter for templates
    pub fn generate_secure_state() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
    }

    /// Validate redirect URI for security
    pub fn is_valid_redirect_uri(redirect_uri: &str, client_id: &str) -> bool {
        let allowed_redirects = match client_id {
            "epsx-frontend" => vec![
                "http://localhost:3000/auth/callback",
                "http://localhost:3000/api/auth/callback/epsx-backend",
                "https://app.epsx.com/auth/callback",
                "https://app.epsx.com/api/auth/callback/epsx-backend",
            ],
            "epsx-admin" => vec![
                "http://localhost:3001/auth/callback",
                "http://localhost:3001/api/auth/callback/epsx-backend",
                "https://admin.epsx.com/auth/callback",
                "https://admin.epsx.com/api/auth/callback/epsx-backend",
            ],
            _ => vec![],
        };

        allowed_redirects.contains(&redirect_uri)
    }

    /// Get error template for common authentication errors
    pub fn get_error_template_for_auth_error(error: &str) -> ErrorTemplate {
        let (message, details) = match error {
            "invalid_client" => (
                "Invalid Client".to_string(),
                "The client identifier provided is invalid or not registered.".to_string(),
            ),
            "invalid_request" => (
                "Invalid Request".to_string(),
                "The request is missing required parameters or contains invalid values.".to_string(),
            ),
            "unauthorized_client" => (
                "Unauthorized Client".to_string(),
                "The client is not authorized to use this authentication flow.".to_string(),
            ),
            "unsupported_response_type" => (
                "Unsupported Response Type".to_string(),
                "The authorization server does not support this response type.".to_string(),
            ),
            "invalid_scope" => (
                "Invalid Scope".to_string(),
                "The requested scope is invalid, unknown, or malformed.".to_string(),
            ),
            "access_denied" => (
                "Access Denied".to_string(),
                "The user or authorization server denied the request.".to_string(),
            ),
            "authentication_failed" => (
                "Authentication Failed".to_string(),
                "Invalid email or password. Please check your credentials and try again.".to_string(),
            ),
            "admin_required" => (
                "Administrator Access Required".to_string(),
                "This resource requires administrator privileges. Please use an admin account.".to_string(),
            ),
            _ => (
                "Authentication Error".to_string(),
                "An unexpected error occurred during authentication. Please try again.".to_string(),
            ),
        };

        Self::create_error_template(message, details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_admin_template() {
        assert!(TemplateFactory::should_use_admin_template("openid profile email admin"));
        assert!(TemplateFactory::should_use_admin_template("administrator"));
        assert!(!TemplateFactory::should_use_admin_template("openid profile email"));
    }

    #[test]
    fn test_is_valid_redirect_uri() {
        assert!(TemplateFactory::is_valid_redirect_uri(
            "http://localhost:3000/auth/callback",
            "epsx-frontend"
        ));
        assert!(TemplateFactory::is_valid_redirect_uri(
            "http://localhost:3000/api/auth/callback/epsx-backend",
            "epsx-frontend"
        ));
        assert!(TemplateFactory::is_valid_redirect_uri(
            "http://localhost:3001/auth/callback",
            "epsx-admin"
        ));
        assert!(TemplateFactory::is_valid_redirect_uri(
            "http://localhost:3001/api/auth/callback/epsx-backend",
            "epsx-admin"
        ));
        assert!(!TemplateFactory::is_valid_redirect_uri(
            "http://malicious-site.com/callback",
            "epsx-frontend"
        ));
    }

    #[test]
    fn test_generate_secure_state() {
        let state1 = TemplateFactory::generate_secure_state();
        let state2 = TemplateFactory::generate_secure_state();
        
        assert_ne!(state1, state2);
        assert!(state1.len() > 40); // Base64 encoded 32 bytes should be longer
    }
}