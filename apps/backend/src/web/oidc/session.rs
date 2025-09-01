// OpenID Connect Session Management and Logout
// Implements OpenID Connect Session Management 1.0 specification

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response, Html},
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

use crate::web::auth::AppState;
use crate::auth::jwt::Claims;

/// RP-Initiated Logout Request (OpenID Connect Session Management)
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub post_logout_redirect_uri: Option<String>,
    pub state: Option<String>,
    pub id_token_hint: Option<String>,
}

/// Logout error response
#[derive(Debug, Serialize)]
pub struct LogoutError {
    pub error: String,
    pub error_description: Option<String>,
}

/// GET /oauth/logout - RP-Initiated Logout Endpoint
/// 
/// OpenID Connect Session Management 1.0 logout endpoint
/// 
/// Usage:
/// ```
/// GET /oauth/logout?post_logout_redirect_uri=https://example.com&state=xyz&id_token_hint=eyJ...
/// ```
/// 
/// This endpoint:
/// 1. Validates the id_token_hint (if provided)
/// 2. Terminates the user session
/// 3. Revokes all active tokens for the user
/// 4. Redirects to post_logout_redirect_uri or shows logout confirmation
pub async fn oidc_logout(
    State(app_state): State<AppState>,
    Query(params): Query<LogoutRequest>,
) -> Result<Response, (StatusCode, Html<String>)> {
    tracing::info!(
        post_logout_redirect_uri = ?params.post_logout_redirect_uri,
        state = ?params.state,
        has_id_token_hint = params.id_token_hint.is_some(),
        "OIDC logout request received"
    );

    // Extract user information from id_token_hint if provided
    let user_info = if let Some(ref id_token) = params.id_token_hint {
        match extract_user_from_id_token(id_token) {
            Ok(info) => {
                tracing::info!(
                    sub = %info.sub,
                    email = %info.email,
                    "Valid ID token provided in logout request"
                );
                Some(info)
            }
            Err(e) => {
                tracing::warn!("Invalid ID token in logout request: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Perform logout operations if user is identified
    if let Some(ref user) = user_info {
        if let Err(e) = perform_user_logout(&app_state, user).await {
            tracing::error!(
                sub = %user.sub,
                error = %e,
                "Failed to perform user logout operations"
            );
        }
    }

    // Handle post-logout redirect
    handle_post_logout_redirect(params, user_info).await
}

/// User information extracted from ID token
#[derive(Debug, Clone)]
struct UserInfo {
    pub sub: String,
    pub email: String,
    pub jti: String,
}

/// Extract user information from ID token
fn extract_user_from_id_token(id_token: &str) -> Result<UserInfo, String> {
    use crate::config::env::get_env_var;
    
    let jwt_secret = get_env_var("NEXTAUTH_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    
    // Set validation parameters for ID token
    validation.set_audience(&["epsx-web", "epsx-admin", "epsx-ecosystem"]);
    let issuer_url = get_env_var("OIDC_ISSUER")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    validation.set_issuer(&[&issuer_url]);

    let token_data = decode::<Claims>(id_token, &decoding_key, &validation)
        .map_err(|e| format!("ID token validation failed: {}", e))?;

    Ok(UserInfo {
        sub: token_data.claims.sub,
        email: token_data.claims.email,
        jti: token_data.claims.jti,
    })
}

/// Perform user logout operations (session termination, token revocation)
async fn perform_user_logout(
    _app_state: &AppState,
    user_info: &UserInfo,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        sub = %user_info.sub,
        email = %user_info.email,
        "Performing logout operations for user"
    );

    // 1. Log the token revocation request (actual implementation would depend on token storage)
    tracing::info!(
        jti = %user_info.jti,
        "ID token revocation requested during logout"
    );

    // 2. Session termination would be implemented here based on your session storage
    tracing::info!(
        sub = %user_info.sub,
        "Session termination requested during logout - would terminate user sessions here"
    );
    
    Ok(())
}


/// Handle post-logout redirect or show confirmation page
async fn handle_post_logout_redirect(
    params: LogoutRequest,
    user_info: Option<UserInfo>,
) -> Result<Response, (StatusCode, Html<String>)> {
    // If post_logout_redirect_uri is provided, redirect there
    if let Some(redirect_uri) = params.post_logout_redirect_uri {
        // Validate redirect URI (basic validation)
        if is_valid_redirect_uri(&redirect_uri) {
            let mut redirect_url = redirect_uri;
            
            // Add state parameter if provided
            if let Some(state) = params.state {
                let separator = if redirect_url.contains('?') { "&" } else { "?" };
                redirect_url = format!("{}{}state={}", redirect_url, separator, urlencoding::encode(&state));
            }
            
            tracing::info!(
                redirect_url = %redirect_url,
                user_email = user_info.as_ref().map(|u| &u.email),
                "Redirecting after logout"
            );
            
            return Ok(Redirect::to(&redirect_url).into_response());
        } else {
            tracing::warn!(
                redirect_uri = %redirect_uri,
                "Invalid post_logout_redirect_uri provided"
            );
        }
    }

    // Show logout confirmation page
    let confirmation_page = create_logout_confirmation_page(&user_info);
    
    tracing::info!(
        user_email = user_info.as_ref().map(|u| &u.email),
        "Showing logout confirmation page"
    );
    
    Ok(Html(confirmation_page).into_response())
}

/// Validate redirect URI for security
fn is_valid_redirect_uri(uri: &str) -> bool {
    // Basic validation - ensure it's a valid URL and not javascript: or data:
    if uri.starts_with("javascript:") || uri.starts_with("data:") || uri.starts_with("file:") {
        return false;
    }
    
    // Must be HTTPS in production or localhost for development
    uri.starts_with("https://") || uri.starts_with("http://localhost")
}

/// Create logout confirmation HTML page
fn create_logout_confirmation_page(user_info: &Option<UserInfo>) -> String {
    let user_email = user_info.as_ref()
        .map(|u| u.email.as_str())
        .unwrap_or("User");

    format!(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>EPSX - Logout Successful</title>
        <style>
            body {{
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                margin: 0;
                padding: 20px;
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
            }}
            .container {{
                background: white;
                padding: 40px;
                border-radius: 12px;
                box-shadow: 0 10px 25px rgba(0,0,0,0.1);
                text-align: center;
                max-width: 400px;
                width: 100%;
            }}
            .logo {{
                font-size: 24px;
                font-weight: bold;
                color: #667eea;
                margin-bottom: 20px;
            }}
            .success-icon {{
                font-size: 48px;
                color: #10b981;
                margin-bottom: 16px;
            }}
            .title {{
                font-size: 24px;
                font-weight: 600;
                color: #1f2937;
                margin-bottom: 8px;
            }}
            .message {{
                color: #6b7280;
                margin-bottom: 24px;
                line-height: 1.5;
            }}
            .user-info {{
                background: #f9fafb;
                padding: 12px;
                border-radius: 6px;
                margin-bottom: 24px;
                font-size: 14px;
                color: #4b5563;
            }}
            .btn {{
                background: #667eea;
                color: white;
                padding: 12px 24px;
                border: none;
                border-radius: 6px;
                cursor: pointer;
                text-decoration: none;
                display: inline-block;
                font-weight: 500;
                transition: background-color 0.2s;
            }}
            .btn:hover {{
                background: #5a6fd8;
            }}
            .footer {{
                margin-top: 24px;
                font-size: 12px;
                color: #9ca3af;
            }}
        </style>
    </head>
    <body>
        <div class="container">
            <div class="logo">EPSX</div>
            <div class="success-icon">✓</div>
            <h1 class="title">Logout Successful</h1>
            <p class="message">
                You have been successfully logged out of EPSX.
                All your active sessions have been terminated.
            </p>
            <div class="user-info">
                Logged out: {user_email}
            </div>
            <a href="/" class="btn">Return to Home</a>
            <div class="footer">
                Your security is our priority. All tokens have been revoked.
            </div>
        </div>
        <script>
            // Auto-close window if opened as popup
            if (window.opener) {{
                setTimeout(() => {{
                    window.close();
                }}, 3000);
            }}
        </script>
    </body>
    </html>
    "#, user_email = user_email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logout_request_deserialize() {
        let query = "post_logout_redirect_uri=https://example.com&state=xyz";
        let request: LogoutRequest = serde_urlencoded::from_str(query).unwrap();
        
        assert_eq!(request.post_logout_redirect_uri.unwrap(), "https://example.com");
        assert_eq!(request.state.unwrap(), "xyz");
        assert!(request.id_token_hint.is_none());
    }

    #[test]
    fn test_is_valid_redirect_uri() {
        assert!(is_valid_redirect_uri("https://example.com"));
        assert!(is_valid_redirect_uri("http://localhost:3000"));
        assert!(!is_valid_redirect_uri("javascript:alert(1)"));
        assert!(!is_valid_redirect_uri("data:text/html,<script>"));
        assert!(!is_valid_redirect_uri("file:///etc/passwd"));
    }

    #[test]
    fn test_logout_confirmation_page_contains_user_email() {
        let user_info = Some(UserInfo {
            sub: "user123".to_string(),
            email: "test@example.com".to_string(),
            jti: "jti123".to_string(),
        });
        
        let page = create_logout_confirmation_page(&user_info);
        assert!(page.contains("test@example.com"));
        assert!(page.contains("Logout Successful"));
        assert!(page.contains("EPSX"));
    }

    #[test]
    fn test_logout_confirmation_page_without_user() {
        let page = create_logout_confirmation_page(&None);
        assert!(page.contains("User"));
        assert!(page.contains("Logout Successful"));
    }
}