use axum::{
    routing::{get, post},
    Router,
};

use crate::web::auth::routes::AppState;
use super::handlers::*;
use super::discovery::*;
use super::authorization::{authorization_endpoint, handle_authorization_form};
use super::token::oidc_token;

/// Create OIDC routes
pub fn oidc_routes() -> Router<AppState> {
    Router::new()
        // OIDC Discovery (standard and v2 paths)
        .route("/.well-known/openid-configuration", get(oidc_discovery))
        .route("/oauth/v2/.well-known/openid-configuration", get(oidc_discovery))
        
        // OIDC Core Endpoints (Pure Authorization Code Flow)
        .route("/oauth/authorize", get(authorization_endpoint).post(handle_authorization_form))
        .route("/oauth/token", post(oidc_token))
        .route("/oauth/userinfo", get(oidc_userinfo))
        .route("/oauth/jwks", get(jwks_endpoint))
        
        // Firebase authentication endpoint
        .route("/firebase-auth", get(firebase_auth_handler))
        
        // Additional endpoints for completeness
        .route("/oauth/revoke", post(oidc_revoke))
        .route("/oauth/introspect", post(oidc_introspect))
}

/// Firebase Authentication Handler
/// GET /firebase-auth
async fn firebase_auth_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Html<String>, axum::http::StatusCode> {
    tracing::info!("Firebase auth page requested");
    
    let client_id = params.get("client_id").cloned().unwrap_or_default();
    let redirect_uri = params.get("redirect_uri").cloned().unwrap_or_default();
    let state = params.get("state").cloned().unwrap_or_default();
    let scope = params.get("scope").cloned().unwrap_or_default();
    
    // Generate Firebase auth HTML page
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Firebase Authentication</title>
    <script src="https://www.gstatic.com/firebasejs/10.7.1/firebase-app-compat.js"></script>
    <script src="https://www.gstatic.com/firebasejs/10.7.1/firebase-auth-compat.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 400px;
            margin: 100px auto;
            padding: 20px;
            text-align: center;
        }}
        .auth-container {{
            border: 1px solid #ddd;
            border-radius: 8px;
            padding: 30px;
            background: #f9f9f9;
        }}
        .btn {{
            background: #4285f4;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
            margin: 10px;
        }}
        .btn:hover {{
            background: #357ae8;
        }}
        .error {{
            color: #d32f2f;
            margin: 10px 0;
        }}
        .loading {{
            color: #666;
            margin: 10px 0;
        }}
    </style>
</head>
<body>
    <div class="auth-container">
        <h1>Admin Authentication</h1>
        <p>Please sign in with your Google account to access the admin dashboard.</p>
        
        <div id="loading" class="loading" style="display: none;">
            Authenticating...
        </div>
        
        <div id="error" class="error" style="display: none;"></div>
        
        <button id="google-signin" class="btn">Sign in with Google</button>
        
        <div style="margin-top: 20px; font-size: 12px; color: #666;">
            Client: {client_id}<br>
            Scope: {scope}
        </div>
    </div>

    <script>
        // Firebase configuration - TODO: Move to environment variables
        const firebaseConfig = {{
            apiKey: "AIzaSyDtGcR8wF9f2M3VqQ7sN1xK9yP5tE8rU2wX",
            authDomain: "epsx-project.firebaseapp.com",
            projectId: "epsx-project",
            storageBucket: "epsx-project.appspot.com",
            messagingSenderId: "123456789012",
            appId: "1:123456789012:web:abcdef123456789012345"
        }};

        // Initialize Firebase
        firebase.initializeApp(firebaseConfig);
        
        const auth = firebase.auth();
        const provider = new firebase.auth.GoogleAuthProvider();
        
        // Add required scopes for admin access
        provider.addScope('email');
        provider.addScope('profile');
        
        // Set custom parameters
        provider.setCustomParameters({{
            'hd': 'epsx.com' // Optional: restrict to specific domain
        }});
        
        document.getElementById('google-signin').addEventListener('click', async () => {{
            const loadingEl = document.getElementById('loading');
            const errorEl = document.getElementById('error');
            const buttonEl = document.getElementById('google-signin');
            
            try {{
                loadingEl.style.display = 'block';
                errorEl.style.display = 'none';
                buttonEl.disabled = true;
                
                const result = await auth.signInWithPopup(provider);
                const user = result.user;
                const idToken = await user.getIdToken();
                
                console.log('Firebase authentication successful:', user.uid);
                
                // Exchange Firebase ID token for authorization code
                // In Firebase-native OIDC, we use the ID token directly as the "code"
                const authCode = idToken;
                
                // Redirect back to the original redirect URI with the authorization code
                const redirectUrl = new URL('{redirect_uri}');
                redirectUrl.searchParams.set('code', authCode);
                redirectUrl.searchParams.set('state', '{state}');
                
                console.log('Redirecting to:', redirectUrl.toString());
                window.location.href = redirectUrl.toString();
                
            }} catch (error) {{
                console.error('Firebase authentication failed:', error);
                errorEl.textContent = 'Authentication failed: ' + error.message;
                errorEl.style.display = 'block';
                buttonEl.disabled = false;
            }} finally {{
                loadingEl.style.display = 'none';
            }}
        }});
        
        // Handle already signed-in users
        auth.onAuthStateChanged(async (user) => {{
            if (user) {{
                console.log('User already signed in:', user.uid);
                // Could auto-proceed here if needed
            }}
        }});
    </script>
</body>
</html>"#, 
        client_id = client_id,
        scope = scope,
        redirect_uri = redirect_uri,
        state = state
    );
    
    Ok(axum::response::Html(html))
}

/// Additional OIDC handlers for completeness

/// OAuth Token Revocation Endpoint
/// POST /oauth/revoke
async fn oidc_revoke(
    axum::extract::State(_state): axum::extract::State<AppState>,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    tracing::info!("OIDC token revocation request");
    
    let _token = request.get("token")
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    
    // TODO: Implement token revocation using FirebaseSessionService
    // For now, return success
    tracing::info!("Token revocation not implemented yet");
    
    Ok(axum::http::StatusCode::OK)
}

/// OAuth Token Introspection Endpoint  
/// POST /oauth/introspect
async fn oidc_introspect(
    axum::extract::State(_state): axum::extract::State<AppState>,
    axum::extract::Form(request): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    tracing::info!("OIDC token introspection request");
    
    let _token = request.get("token")
        .ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    
    // TODO: Implement token introspection using FirebaseSessionService
    // For now, return inactive
    let response = serde_json::json!({
        "active": false
    });
    
    Ok(axum::response::Json(response))
}