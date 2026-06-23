//! EPSX Pay BFF (bff-pay).
//!
//! wave49(slice-2): Dioxus SSR rewrite. The previous version used
//! inline `r##"..."##` HTML strings for the /checkout, /success,
//! and /cancel pages. This rewrite composes the ported payment
//! components from `shared/rust/dioxus_ui::payment::*` via real
//! Dioxus `#[component]` functions in `crate::components::*`.
//!
//! Strategy:
//! - Axum handles the API routes (`/api/v1/pay/intent*`) and
//!   proxies them to the `epsx-pay-svc` backend via
//!   `ServiceClient`.
//! - The catch-all `pay_ssr_fallback` builds a `VirtualDom`
//!   from a `PageRouter` component that dispatches by URL
//!   path, then serializes via `dioxus_ssr::render_element`.
//! - The serialized body is wrapped in `page_shell_with_body_class`
//!   so the design system (Tailwind, lucide icons, FOUC prevention)
//!   matches `epsx.io` and `admin.epsx.io`.

use axum::{
    extract::Path as AxPath,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use dioxus::prelude::*;
use epsx_client::ServiceClient;
use epsx_templates::{page_shell_with_body_class, theme_toggle_button, logo};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

mod components;
mod state;

use components::{PayCancelScreen, PayCheckoutForm, PayEscrowStatus, PaySuccessScreen};
use state::payment_wizard_state::PaymentWizardState;

#[derive(Clone)]
struct AppState {
    pay: Arc<ServiceClient>,
    identity: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    api_url: String,
}

#[derive(Deserialize)]
struct PayIntentBody {
    amount: String,
    currency: String,
    description: Option<String>,
    order_id: Option<String>,
    success_url: Option<String>,
    cancel_url: Option<String>,
    metadata: Option<serde_json::Value>,
    payer: Option<String>,
    payee: Option<String>,
    merchant: Option<String>,
    chain_id: Option<String>,
    token: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-pay");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3002);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(30) };
    let state = AppState {
        pay: Arc::new(ServiceClient::new(cfg.clone())),
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/pay/intent", any(create_pay_intent))
        .route("/api/v1/pay/intent/{id}", any(get_pay_intent))
        .route("/api/v1/pay/intent/{id}/execute", any(execute_pay))
        .route("/api/v1/pay/intent/{id}/status", any(pay_status))
        // wave49(slice-3): pay_links + pay_history proxies.
        // Mirror the service's endpoint shape so the BFF
        // doesn't need its own type re-shaping.
        .route("/api/v1/pay/links", post(create_pay_link))
        .route("/api/v1/pay/links/{slug}", get(get_pay_link))
        .route("/api/v1/pay/links/{slug}/redeem", post(redeem_pay_link))
        .route("/api/v1/pay/history/{address}", get(get_pay_history))
        .fallback(pay_ssr_fallback)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Pay BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str {
    "ok"
}

async fn create_pay_intent(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<PayIntentBody>,
) -> Result<Response, StatusCode> {
    let chain_decimal = body.chain_id.as_deref()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .or_else(|| body.chain_id.as_deref().and_then(|s| s.parse().ok()))
        .unwrap_or(56);
    let payee = body.payee.clone()
        .or_else(|| body.merchant.clone())
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());
    let payer = body.payer.clone()
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());
    let token = body.token.clone()
        .or_else(|| Some(body.currency.clone()))
        .unwrap_or_else(|| "USDT".to_string());
    let res = state.pay.post_plain("/api/v1/pay/intents", &serde_json::json!({
        "amount": body.amount,
        "token": token,
        "chain_id": chain_decimal,
        "description": body.description,
        "payer": payer,
        "payee": payee,
    })).await;
    res.map(|v| Json(v).into_response()).map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn get_pay_intent(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/intents/{}", id);
    state.pay.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn execute_pay(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/intents/{}/execute", id);
    state.pay.post_plain(&path, &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn pay_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/intents/{}", id);
    state.pay.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

// ============================================================================
// wave49(slice-3): pay_links + pay_history proxy handlers
// ============================================================================

async fn create_pay_link(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.pay.post_plain("/api/v1/pay/links", &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn get_pay_link(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(slug): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/links/{}", slug);
    state.pay.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn redeem_pay_link(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(slug): AxPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/links/{}/redeem", slug);
    state.pay.post_plain(&path, &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn get_pay_history(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(address): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/pay/history/{}", address);
    state.pay.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

// ============================================================================
// SSR fallback — Dioxus VirtualDom render via dioxus_ssr::render_element
// ============================================================================

#[derive(Props, Clone, PartialEq)]
struct PageRouterProps {
    path: String,
    query: String,
}

#[component]
fn PageRouter(props: PageRouterProps) -> Element {
    let path = props.path.clone();
    let query = props.query.clone();

    // Route dispatch by path.
    if path == "/success" {
        // Pull intent id from query if present.
        let intent_id = parse_query_param(&query, "intent");
        return rsx! { PaySuccessScreen { intent_id: intent_id } };
    }
    if path == "/cancel" {
        return rsx! { PayCancelScreen {} };
    }
    if path.starts_with("/intent/") {
        // /intent/:id
        let id = path.trim_start_matches("/intent/").to_string();
        return rsx! { PayEscrowStatus { intent_id: id } };
    }
    if path.starts_with("/r/") {
        // /r/:slug — shareable payment link landing page.
        // For slice-3 we render a minimal placeholder; slice-3.5+
        // will resolve the slug via GET /api/v1/pay/links/:slug
        // and redirect to /pay?intent=<resolved.intent.id>.
        let slug = path.trim_start_matches("/r/").to_string();
        return rsx! { PayLinkLanding { slug } };
    }
    // Default: / and /checkout → PayCheckoutForm
    let state = PaymentWizardState::from_search(&query);
    rsx! { PayCheckoutForm { state: state } }
}

/// `/r/:slug` placeholder — slice-3 ships the route as a
/// stub. Slice-3.5+ will fetch `GET /api/v1/pay/links/:slug`,
/// extract `intent.id` from the response, and either
/// server-side redirect to `/pay?intent={id}` or render the
/// checkout directly.
#[component]
fn PayLinkLanding(slug: String) -> Element {
    rsx! {
        div { class: "pay-link-landing page-bg",
            section { class: "section",
                style: "display:flex;align-items:center;justify-content:center;min-height:60vh;text-align:center;",
                div {
                    h1 { class: "pay-link-landing-title",
                        style: "font-size:1.75rem;font-weight:800;margin-bottom:1rem;",
                        "Resolving payment link…"
                    }
                    p { class: "pay-link-landing-slug",
                        style: "font-family:monospace;color:var(--text-muted);margin-bottom:1.5rem;",
                        "{slug}"
                    }
                    p { class: "pay-link-landing-note",
                        style: "font-size:0.875rem;color:var(--text-subtle);",
                        "Slice-3 ships the route as a stub. Slice-3.5+ will resolve the slug server-side and redirect to the checkout."
                    }
                }
            }
        }
    }
}

fn parse_query_param(query: &str, key: &str) -> Option<String> {
    let needle = format!("{}=", key);
    for pair in query.trim_start_matches('?').split('&') {
        if let Some(rest) = pair.strip_prefix(&needle) {
            return Some(rest.to_string());
        }
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

async fn pay_ssr_fallback(uri: axum::http::Uri) -> Response {
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("").to_string();

    // Build the page VDom.
    let mut vdom = VirtualDom::new_with_props(
        PageRouter,
        PageRouterProps { path: path.clone(), query: query.clone() },
    );

    // Dioxus 0.7 SSR: rebuild_in_place walks the component tree
    // and resolves all `rsx!` blocks. Without this call,
    // `dioxus_ssr::render` panics with "tree has not been built".
    vdom.rebuild_in_place();

    // SSR render — `dioxus_ssr::render` returns the full HTML
    // body string. We rebuild the VDom each request (cheap;
    // ~1ms per page) so URL params + path dispatch stay pure.
    let body_html = dioxus_ssr::render(&vdom);

    // Choose title + body class per route.
    let (title, body_class) = if path == "/success" {
        ("Payment Successful", "page-bg")
    } else if path == "/cancel" {
        ("Payment Cancelled", "page-bg")
    } else if path.starts_with("/intent/") {
        ("Payment status", "page-bg")
    } else {
        ("EPSX Pay", "page-bg")
    };

    // Minimal nav — the checkout page already includes its own
    // shell via the design system. The nav here matches the
    // existing inline-HTML version (logo + theme toggle).
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

    let shell = page_shell_with_body_class(
        title,
        "EPSX Pay — Complete your payment on BSC",
        &nav,
        &body_html,
        false,
        body_class,
    );

    Html(shell).into_response()
}