use axum::{
    extract::Path as AxPath,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use epsx_client::ServiceClient;
use epsx_templates::{page_shell_with_body_class, theme_toggle_button, logo};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    payment: Arc<ServiceClient>,
    identity: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    api_url: String,
}

#[derive(Deserialize)]
struct IntentBody {
    amount: String,
    currency: String,
    description: Option<String>,
    order_id: Option<String>,
    success_url: Option<String>,
    cancel_url: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-pay");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3002);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(30) };
    let state = AppState {
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/pay/intent", any(create_intent))
        .route("/api/v1/pay/intent/{id}", any(get_intent))
        .route("/api/v1/pay/intent/{id}/execute", any(execute_payment))
        .route("/api/v1/pay/intent/{id}/status", any(payment_status))
        .fallback(ssr_fallback)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Pay BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str {
    "ok"
}

async fn create_intent(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<IntentBody>,
) -> Result<Response, StatusCode> {
    let res = state.payment.post_plain("/api/v1/payment/intents", &serde_json::json!({
        "amount": body.amount,
        "currency": body.currency,
        "chain_id": "56",
        "description": body.description,
    })).await;
    res.map(|v| Json(v).into_response()).map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn get_intent(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/payment/intents/{}", id);
    state.payment.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn execute_payment(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/payment/intents/{}/execute", id);
    state.payment.post_plain(&path, &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn payment_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/payment/intents/{}", id);
    state.payment.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn ssr_fallback(uri: axum::http::Uri) -> Response {
    let path = uri.path();
    let html = render_page(path);
    Html(html).into_response()
}

fn render_page(path: &str) -> String {
    let (title, description, body) = if path == "/success" {
        ("Payment Successful", "Your payment was confirmed on BSC", success_body())
    } else if path == "/cancel" {
        ("Payment Cancelled", "Your payment was cancelled", cancel_body())
    } else {
        ("EPSX Pay", "Complete your payment on BSC", checkout_body())
    };

    let nav = format!(
        r#"<nav class="navbar"><div class="container-x flex items-center justify-between" style="height:3.5rem;">
  {logo}
  <div class="flex items-center gap-2">
    {toggle}
  </div>
</div></nav>"#,
        logo = logo("/", "sm"),
        toggle = theme_toggle_button(),
    );

    let shell = page_shell_with_body_class(title, description, &nav, &body, false, "page-bg");
    shell
}

fn success_body() -> String {
    r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;max-width:32rem;">
  <div style="width:5rem;height:5rem;border-radius:9999px;background:linear-gradient(135deg,#10b981,#34d399);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;box-shadow:0 20px 25px -5px rgba(16,185,129,0.5);">
    <i data-lucide="check" style="color:white;font-size:1.75rem;"></i>
  </div>
  <span class="badge badge-success" style="margin-bottom:1rem;">Confirmed</span>
  <h1 style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">Payment Successful</h1>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">Your payment has been confirmed on BSC. The recipient has been notified.</p>
  <div style="display:inline-flex;gap:0.75rem;flex-wrap:wrap;justify-content:center;">
    <a href="/" class="btn btn-gradient btn-lg"><i data-lucide="home"></i> Back to Home</a>
    <a href="/dashboard" class="btn btn-outline btn-lg"><i data-lucide="gauge"></i> View Dashboard</a>
  </div>
</div>
</section>"##.to_string()
}

fn cancel_body() -> String {
    r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;max-width:32rem;">
  <div style="width:5rem;height:5rem;border-radius:9999px;background:linear-gradient(135deg,#ef4444,#f87171);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;box-shadow:0 20px 25px -5px rgba(239,68,68,0.5);">
    <i data-lucide="x" style="color:white;font-size:1.75rem;"></i>
  </div>
  <span class="badge badge-danger" style="margin-bottom:1rem;">Cancelled</span>
  <h1 style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">Payment Cancelled</h1>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">Your payment was not completed. No funds have been transferred.</p>
  <div style="display:inline-flex;gap:0.75rem;flex-wrap:wrap;justify-content:center;">
    <a href="/" class="btn btn-gradient btn-lg"><i data-lucide="home"></i> Back to Home</a>
    <button onclick="history.back()" class="btn btn-outline btn-lg"><i data-lucide="arrow-left"></i> Try Again</button>
  </div>
</div>
</section>"##.to_string()
}

fn checkout_body() -> String {
    r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="width:100%;max-width:28rem;">
  <div class="card-insight" style="padding:2.5rem;">
    <div style="text-align:center;margin-bottom:2rem;">
      <span class="badge-pill"><i data-lucide="credit-card" style="color:var(--epsx-orange);width:1rem;height:1rem;"></i> EPSX Pay</span>
      <h1 style="font-size:1.75rem;font-weight:800;margin-top:0.75rem;">Complete Payment</h1>
      <p style="color:var(--text-muted);font-size:0.875rem;margin-top:0.25rem;">Powered by BSC</p>
    </div>

    <div style="background:var(--bg-secondary);border-radius:0.75rem;padding:1.25rem;margin-bottom:1rem;">
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.75rem;">
        <span style="font-size:0.875rem;color:var(--text-muted);">Amount</span>
        <span class="gradient-text" id="amount" style="font-size:1.5rem;font-weight:800;">0.00 USDT</span>
      </div>
      <div style="display:flex;justify-content:space-between;align-items:center;">
        <span style="font-size:0.875rem;color:var(--text-muted);">Chain</span>
        <span style="font-size:0.875rem;font-weight:500;"><i data-lucide="link" style="color:var(--epsx-orange);width:1rem;height:1rem;margin-right:0.25rem;"></i>BSC (BEP-20)</span>
      </div>
    </div>

    <button id="pay-btn" class="btn btn-gradient btn-block btn-lg" style="margin-bottom:0.75rem;">
      <i data-lucide="wallet"></i> Connect Wallet & Pay
    </button>
    <p style="font-size:0.75rem;color:var(--text-subtle);text-align:center;">
      <i data-lucide="shield" style="color:var(--epsx-green);"></i> Secured by EPSX escrow
    </p>
  </div>
</div>
</section>
<script>
const url = new URLSearchParams(location.search);
document.getElementById('amount').textContent = (url.get('amount') || '0.00') + ' ' + (url.get('currency') || 'USDT');
document.getElementById('pay-btn').onclick = async () => {
  try {
    const intent = await fetch('/api/v1/pay/intent', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        amount: url.get('amount') || '0',
        currency: url.get('currency') || 'USDT',
        description: url.get('description') || '',
      }),
    }).then(r => r.json());
    if (intent && intent.intent && intent.intent.id) location.href = '/checkout/' + intent.intent.id;
    else epsx.toast('Failed to create payment intent', 'error');
  } catch (e) {
    epsx.toast('Network error: ' + e.message, 'error');
  }
};
</script>"##.to_string()
}
