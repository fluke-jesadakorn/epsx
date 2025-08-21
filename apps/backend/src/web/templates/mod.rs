// Template rendering for OIDC login pages

use askama::Template;
use serde::{Deserialize, Serialize};
use base64::Engine;


/// Template for PancakeSwap-themed user login (OIDC)
#[derive(Template)]
#[template(path = "pancake/login.html")]
pub struct PancakeLoginTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub error: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Template for PancakeSwap-themed admin login (OIDC)
#[derive(Template)]
#[template(path = "pancake/admin_login.html")]
pub struct PancakeAdminLoginTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub error: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Template for PancakeSwap-themed user registration (OIDC)
#[derive(Template)]
#[template(path = "pancake/register.html")]
pub struct PancakeRegistrationTemplate {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub scope: String,
    pub error: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Template for authentication errors (PancakeSwap themed)
#[derive(Template)]
#[template(path = "pancake/error.html")]
pub struct ErrorTemplate {
    pub error_message: String,
    pub error_details: String,
}

/// Template for logout success page (PancakeSwap themed)
#[derive(Template)]
#[template(path = "pancake/error.html")]
pub struct LogoutSuccessTemplate {
    pub error_message: String,
    pub error_details: String,
}

/// Template for password reset request page (PancakeSwap themed)
#[derive(Template)]
#[template(path = "pancake/reset-password.html")]
pub struct PasswordResetTemplate {
    pub error: String,
    pub success: String,
}

/// Template for password reset confirmation page (PancakeSwap themed)
#[derive(Template)]
#[template(path = "pancake/reset-confirm.html")]
pub struct PasswordResetConfirmTemplate {
    pub oob_code: String,
    pub error: String,
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

    /// Create PancakeSwap-themed user login template with PKCE parameters
    pub fn create_pancake_login_template_with_pkce(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        error: String,
    ) -> PancakeLoginTemplate {
        PancakeLoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge,
            code_challenge_method,
        }
    }

    /// Create PancakeSwap-themed admin login template with PKCE parameters
    pub fn create_pancake_admin_login_template_with_pkce(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        error: String,
    ) -> PancakeAdminLoginTemplate {
        PancakeAdminLoginTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge,
            code_challenge_method,
        }
    }

    /// Create PancakeSwap-themed user registration template with PKCE parameters
    pub fn create_pancake_registration_template_with_pkce(
        client_id: String,
        redirect_uri: String,
        state: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
        error: String,
    ) -> PancakeRegistrationTemplate {
        PancakeRegistrationTemplate {
            client_id,
            redirect_uri,
            state,
            scope,
            error,
            code_challenge,
            code_challenge_method,
        }
    }

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
        success_message: String,
        success_details: String,
    ) -> LogoutSuccessTemplate {
        LogoutSuccessTemplate {
            error_message: success_message,
            error_details: success_details,
        }
    }

    /// Create password reset template
    pub fn create_password_reset_template(
        error: String,
    ) -> PasswordResetTemplate {
        PasswordResetTemplate {
            error,
            success: "".to_string(),
        }
    }

    /// Create password reset confirmation template
    pub fn create_password_reset_confirm_template(
        oob_code: String,
        error: String,
    ) -> PasswordResetConfirmTemplate {
        PasswordResetConfirmTemplate {
            oob_code,
            error,
        }
    }


    /// Determine if admin template should be used based on scope
    pub fn should_use_admin_template(scope: &str) -> bool {
        scope.contains("admin") || scope.contains("administrator") || scope.contains("admin_modules")
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
                // Development
                "http://localhost:3000/auth/callback",
                "http://localhost:3000/api/auth/callback/epsx-backend",
                // Production (.io domain)
                "https://epsx.io/auth/callback",
                "https://epsx.io/api/auth/callback/epsx-backend",
                // Legacy (.com domain - keep for backward compatibility)
                "https://app.epsx.com/auth/callback",
                "https://app.epsx.com/api/auth/callback/epsx-backend",
            ],
            "epsx-admin" => vec![
                // Development
                "http://localhost:3001/auth/callback",
                "http://localhost:3001/api/auth/callback/epsx-backend",
                // Production (.io domain)
                "https://admin.epsx.io/auth/callback",
                "https://admin.epsx.io/api/auth/callback/epsx-backend",
                // Legacy (.com domain - keep for backward compatibility)
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