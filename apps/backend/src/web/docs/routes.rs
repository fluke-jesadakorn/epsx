//! Documentation Routes
//! 
//! Provides endpoints for serving OpenAPI documentation and ReDoc UI.

use axum::{
    routing::get,
    response::{Html, Json},
    Router,
};
use utoipa::OpenApi;

use crate::web::docs::openapi::ApiDoc;

/// Create documentation routes
pub fn create_docs_routes() -> Router {
    Router::new()
        .route("/docs", get(docs_handler))
        .route("/api-docs/openapi.json", get(openapi_json_handler))
}

/// Serve Scalar Interactive API Documentation at /docs
pub async fn docs_handler() -> Html<String> {
    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>EPSX API Documentation</title>
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
    </style>
</head>
<body>
    <div class="header">
        <h1>EPSX Trading Platform API</h1>
        <p>Interactive API documentation with real-time testing</p>
        <div class="feature-badges">
            <span class="badge">🚀 Interactive Testing</span>
            <span class="badge">🔐 Web3 Authentication</span>
            <span class="badge">📊 Real-time Analytics</span>
            <span class="badge">💼 Trading Platform</span>
        </div>
        <div style="margin-top: 15px; font-size: 0.9em; opacity: 0.9;">
            <strong>🦄 Web3 Authentication Flow:</strong>
            Generate Challenge → Sign with Wallet → Verify Signature → Get Bearer Token
        </div>
    </div>
    <div id="scalar-container"></div>
    <script id="api-reference" data-url="/api-docs/openapi.json"></script>
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
                url: '/api-docs/openapi.json'
            }},
            proxy: '/api-docs/openapi.json',
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
    
    Html(html)
}

/// Serve OpenAPI JSON specification at /api-docs/openapi.json
pub async fn openapi_json_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}