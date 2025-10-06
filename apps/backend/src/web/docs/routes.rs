//! Documentation Routes
//!
//! Provides endpoints for serving OpenAPI documentation with Scalar, ReDoc, and landing page.

use axum::{ routing::get, response::{ Html, Json }, Router };
use utoipa::OpenApi;

use crate::web::docs::openapi::ApiDoc;

/// Create documentation routes
pub fn create_docs_routes() -> Router {
  Router::new()
    .route("/docs", get(docs_landing_handler))
    .route("/docs/scalar", get(docs_scalar_handler))
    .route("/docs/redoc", get(docs_redoc_handler))
    .route("/api-docs/openapi.json", get(openapi_json_handler))
}

/// Serve Documentation Landing Page at /docs
pub async fn docs_landing_handler() -> Html<String> {
  let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>EPSX API Documentation</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 16px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 900px;
            width: 100%;
            padding: 60px;
        }
        h1 {
            color: #667eea;
            font-size: 3em;
            margin-bottom: 10px;
            font-weight: 300;
        }
        .tagline {
            color: #666;
            font-size: 1.3em;
            margin-bottom: 40px;
        }
        .stats {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 20px;
            margin: 40px 0;
        }
        .stat {
            text-align: center;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }
        .stat-number {
            font-size: 2em;
            font-weight: bold;
            color: #667eea;
        }
        .stat-label {
            color: #666;
            margin-top: 5px;
        }
        .ui-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin: 40px 0;
        }
        .ui-card {
            border: 2px solid #e0e0e0;
            border-radius: 12px;
            padding: 30px;
            cursor: pointer;
            text-decoration: none;
            color: inherit;
            display: block;
        }
        .ui-card:hover {
            border-color: #667eea;
            box-shadow: 0 8px 24px rgba(102, 126, 234, 0.15);
        }
        .ui-card h2 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 1.8em;
        }
        .ui-card p {
            color: #666;
            line-height: 1.6;
            margin-bottom: 20px;
        }
        .ui-card .features {
            list-style: none;
            margin: 20px 0;
        }
        .ui-card .features li {
            padding: 8px 0;
            color: #444;
        }
        .ui-card .features li:before {
            content: "✓ ";
            color: #667eea;
            font-weight: bold;
            margin-right: 8px;
        }
        .cta-button {
            display: inline-block;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 12px 30px;
            border-radius: 6px;
            font-weight: 600;
            text-decoration: none;
            margin-top: 10px;
        }
        .cta-button:hover {
            opacity: 0.9;
        }
        .quick-links {
            margin-top: 40px;
            padding-top: 30px;
            border-top: 1px solid #e0e0e0;
        }
        .quick-links h3 {
            margin-bottom: 15px;
            color: #333;
        }
        .quick-links a {
            display: inline-block;
            margin: 5px 10px 5px 0;
            padding: 8px 16px;
            background: #f0f0f0;
            color: #667eea;
            text-decoration: none;
            border-radius: 4px;
            font-size: 0.9em;
        }
        .quick-links a:hover {
            background: #e0e0e0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🦄 EPSX API</h1>
        <p class="tagline">Web3-First Trading Analytics Platform</p>

        <div class="stats">
            <div class="stat">
                <div class="stat-number">154+</div>
                <div class="stat-label">Endpoints</div>
            </div>
            <div class="stat">
                <div class="stat-number">12</div>
                <div class="stat-label">Categories</div>
            </div>
            <div class="stat">
                <div class="stat-number">100%</div>
                <div class="stat-label">Documented</div>
            </div>
            <div class="stat">
                <div class="stat-number">Web3</div>
                <div class="stat-label">Native Auth</div>
            </div>
        </div>

        <div class="ui-grid">
            <a href="/docs/scalar" class="ui-card">
                <h2>⚡ Scalar</h2>
                <p>Modern, interactive API documentation with a clean interface perfect for development and testing.</p>
                <ul class="features">
                    <li>Interactive request testing</li>
                    <li>Code generation (10+ languages)</li>
                    <li>Real-time API calls</li>
                    <li>Modern purple theme</li>
                </ul>
                <span class="cta-button">Explore with Scalar →</span>
            </a>

            <a href="/docs/redoc" class="ui-card">
                <h2>📚 Redoc</h2>
                <p>Professional three-panel documentation ideal for comprehensive API reference and PDF exports.</p>
                <ul class="features">
                    <li>Three-panel layout</li>
                    <li>Enhanced navigation</li>
                    <li>PDF export capability</li>
                    <li>Advanced search</li>
                </ul>
                <span class="cta-button">Explore with Redoc →</span>
            </a>
        </div>

        <div class="quick-links">
            <h3>Quick Links</h3>
            <a href="/api-docs/openapi.json">📄 OpenAPI Spec (JSON)</a>
            <a href="/health">💚 Health Check</a>
            <a href="/api/auth/health">🔐 Auth Status</a>
            <a href="/api/permissions/system/health">🛡️ Permissions Health</a>
        </div>
    </div>
</body>
</html>
"#.to_string();

  Html(html)
}

/// Serve Scalar Interactive API Documentation at /docs/scalar
pub async fn docs_scalar_handler() -> Html<String> {
  let html =
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
"#.to_string();

  Html(html)
}

/// Serve OpenAPI JSON specification at /api-docs/openapi.json
pub async fn openapi_json_handler() -> Json<utoipa::openapi::OpenApi> {
  Json(ApiDoc::openapi())
}

/// Serve Redoc Interactive API Documentation at /docs/redoc
pub async fn docs_redoc_handler() -> Html<String> {
  let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>EPSX API Documentation - Redoc</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {
            margin: 0;
            padding: 0;
            font-family: 'Roboto', sans-serif;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            text-align: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .header h1 {
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }
        .header p {
            margin: 10px 0 0 0;
            opacity: 0.9;
            font-size: 1.1em;
        }
        #redoc-container {
            margin-top: 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>EPSX API Documentation</h1>
        <p>Professional three-panel API reference powered by Redoc</p>
    </div>
    <div id="redoc-container"></div>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
    <script>
        Redoc.init(
            '/api-docs/openapi.json',
            {
                scrollYOffset: 60,
                hideDownloadButton: false,
                disableSearch: false,
                expandResponses: '200,201',
                jsonSampleExpandLevel: 2,
                hideSingleRequestSampleTab: false,
                menuToggle: true,
                nativeScrollbars: false,
                pathInMiddlePanel: false,
                requiredPropsFirst: true,
                sortPropsAlphabetically: false,
                theme: {
                    colors: {
                        primary: {
                            main: '#667eea'
                        }
                    },
                    typography: {
                        fontSize: '15px',
                        fontFamily: '"Roboto", sans-serif',
                        headings: {
                            fontFamily: '"Roboto", sans-serif',
                            fontWeight: '400'
                        },
                        code: {
                            fontSize: '14px',
                            fontFamily: '"Courier New", monospace'
                        }
                    },
                    sidebar: {
                        backgroundColor: '#ffffff',
                        textColor: '#333333'
                    },
                    rightPanel: {
                        backgroundColor: '#263238',
                        textColor: '#ffffff'
                    }
                }
            },
            document.getElementById('redoc-container')
        );
    </script>
</body>
</html>
"#.to_string();

  Html(html)
}
