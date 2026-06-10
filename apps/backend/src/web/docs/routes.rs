//! Documentation Routes
//!
//! Provides endpoints for serving OpenAPI documentation with Scalar.
//! Separate documentation for users (/docs) and admins (/admin/docs).

use axum::{ routing::get, response::{ Html, Json, IntoResponse }, Router };
use axum::http::{HeaderMap, HeaderValue};
use utoipa::OpenApi;

use crate::web::docs::openapi_user::UserApiDoc;
use crate::web::docs::openapi_admin::AdminApiDoc;
use crate::config::env::get_env_var;

/// Create documentation routes for Scalar
/// - /docs - User-facing API documentation
/// - /admin/docs - Admin API documentation (includes all endpoints)
pub fn create_docs_routes() -> Router {
  Router::new()
    // User documentation
    .route("/docs", get(docs_user_handler))
    .route("/api-docs/openapi.json", get(openapi_user_json_handler))
    // Admin documentation
    .route("/admin/docs", get(docs_admin_handler))
    .route("/admin/api-docs/openapi.json", get(openapi_admin_json_handler))
}

/// Serve Scalar Interactive API Documentation for Users at /docs
/// This endpoint provides user-facing API documentation (excludes admin endpoints)
pub async fn docs_user_handler() -> impl IntoResponse {
  create_scalar_html(
    "EPSX API Documentation",
    "Interactive API documentation for EPSX Data Analytics Platform",
    "/api-docs/openapi.json",
    false // is_admin
  ).await
}

/// Serve Scalar Interactive API Documentation for Admins at /admin/docs
/// This endpoint provides complete API documentation including admin endpoints
pub async fn docs_admin_handler() -> impl IntoResponse {
  create_scalar_html(
    "EPSX Admin API Documentation",
    "Complete API documentation including admin management endpoints",
    "/admin/api-docs/openapi.json",
    true // is_admin
  ).await
}

/// Create Scalar HTML documentation page
async fn create_scalar_html(
  title: &str,
  description: &str,
  openapi_url: &str,
  is_admin: bool
) -> impl IntoResponse {
  let admin_badge = if is_admin {
    r#"<span class="badge admin-badge">🔐 Admin Panel</span>"#
  } else {
    ""
  };

  let admin_style = if is_admin {
    r#"
        .admin-badge {
            background: rgba(220, 38, 38, 0.3);
            border: 1px solid rgba(220, 38, 38, 0.5);
        }
        .header {
            background: linear-gradient(135deg, #dc2626 0%, #991b1b 100%) !important;
        }
    "#
  } else {
    ""
  };

  let html = format!(
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Roboto', sans-serif;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            text-align: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
            font-size: 1.1em;
        }}
        .feature-badges {{
            margin-top: 15px;
            display: flex;
            justify-content: center;
            gap: 10px;
            flex-wrap: wrap;
        }}
        .badge {{
            background: rgba(255, 255, 255, 0.2);
            padding: 5px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: 500;
        }}
        #scalar-container {{
            margin-top: 0;
            height: calc(100vh - 120px);
        }}
        /* Custom Scalar styling for EPSX branding */
        .scalar-app {{
            --scalar-color-1: #667eea;
            --scalar-color-2: #764ba2;
            --scalar-color-accent: #667eea;
            --scalar-background-1: #ffffff;
            --scalar-background-2: #f8fafc;
            --scalar-background-3: #f1f5f9;
            --scalar-border-color: #e2e8f0;
        }}
        {admin_style}
    </style>
</head>
<body>
    <div class="header">
        <h1>{title}</h1>
        <p>{description}</p>
        <div class="feature-badges">
            {admin_badge}
            <span class="badge">🚀 Interactive Testing</span>
            <span class="badge">🔐 Web3 Authentication</span>
            <span class="badge">📊 Real-time Analytics</span>
            <span class="badge">💼 Data Analytics Platform</span>
        </div>
        <div style="margin-top: 15px; font-size: 0.9em; opacity: 0.9;">
            <strong>🦄 Web3 Authentication Flow:</strong>
            Generate Challenge → Sign with Wallet → Verify Signature → Get Bearer Token
        </div>
    </div>
    <div id="scalar-container"></div>
    <script id="api-reference" data-url="{openapi_url}"></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
    <script>
        // Configure Scalar with EPSX branding and interactive features
        const apiReference = document.getElementById('api-reference');
        
        // Enhanced configuration for interactive testing
        apiReference.dataset.configuration = JSON.stringify({{
            theme: 'purple',
            layout: 'modern',
            showSidebar: true,
            searchHotKey: 'k',
            darkMode: false,
            customCss: `
                .scalar-app {{
                    --scalar-color-1: #667eea;
                    --scalar-color-2: #764ba2;
                    --scalar-color-accent: #667eea;
                    --scalar-radius: 6px;
                    --scalar-font: 'Roboto', sans-serif;
                }}
            `,
            authentication: {{
                preferredSecurityScheme: 'bearerAuth',
                bearerAuth: {{
                    token: '',
                    description: 'Enter your Web3 authentication token obtained from /api/auth/web3/verify'
                }},
                apiKey: {{
                    token: '',
                    description: 'Alternative API key authentication'
                }}
            }},
            spec: {{
                url: '{openapi_url}'
            }},
            servers: [
                {{
                    url: 'http://localhost:8080',
                    description: 'Development Server'
                }},
                {{
                    url: 'https://api.epsx.io',
                    description: 'Production Server'
                }}
            ],
            defaultHttpClient: {{
                targetKey: 'javascript',
                clientKey: 'fetch'
            }},
            hiddenClients: [],
            examples: {{
                'ChallengeRequest': {{
                    wallet_address: '0x742d35Cc6634C0532925a3b8F39dBC0A31Da12345'
                }},
                'VerifyRequest': {{
                    wallet_address: '0x742d35Cc6634C0532925a3b8F39dBC0A31Da12345',
                    message: 'EPSX wants you to sign in with your Ethereum account...',
                    signature: '0x1a2b3c4d5e6f...'
                }}
            }},
            onSpecUpdate: function(spec) {{
                console.log('EPSX API Specification loaded:', spec.info);
                // Add Web3-specific guidance
                console.log('💡 Web3 Authentication Flow:');
                console.log('1. POST /api/auth/web3/challenge - Get SIWE challenge');
                console.log('2. Sign message with wallet (MetaMask, WalletConnect, etc.)');
                console.log('3. POST /api/auth/web3/verify - Submit signature for token');
                console.log('4. Use returned Bearer token for authenticated requests');
            }}
        }});
    </script>
</body>
</html>
"#
  );

  // Build headers that allow iframe embedding from allowed origins
  let mut headers = HeaderMap::new();
  
  // Get frontend URL from env, default to localhost for dev
  let frontend_url = get_env_var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
  let admin_url = get_env_var("ADMIN_FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());
  
  // Allow embedding from frontend origins
  let frame_ancestors = format!("frame-ancestors 'self' {} {}", frontend_url, admin_url);
  
  // Override X-Frame-Options to allow from same origin (modern browsers use frame-ancestors)
  headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
  
  // Set Content-Security-Policy with frame-ancestors allowing frontend
  if let Ok(csp_value) = HeaderValue::from_str(&frame_ancestors) {
    headers.insert("content-security-policy", csp_value);
  }
  
  // Set content type
  headers.insert("content-type", HeaderValue::from_static("text/html; charset=utf-8"));

  (headers, Html(html))
}

/// Serve User OpenAPI JSON specification at /api-docs/openapi.json
pub async fn openapi_user_json_handler() -> Json<utoipa::openapi::OpenApi> {
  Json(UserApiDoc::openapi())
}

/// Serve Admin OpenAPI JSON specification at /admin/api-docs/openapi.json
pub async fn openapi_admin_json_handler() -> Json<utoipa::openapi::OpenApi> {
  Json(AdminApiDoc::openapi())
}

// Keep backward compatibility exports
pub use docs_user_handler as docs_scalar_handler;
pub use openapi_user_json_handler as openapi_json_handler;

