use axum::{
    extract::Path as AxPath,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use epsx_client::{RequestContext, ServiceClient};
use epsx_templates::{page_shell_with_body_class, theme_toggle_button, logo};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    identity: Arc<ServiceClient>,
    wallet: Arc<ServiceClient>,
    payment: Arc<ServiceClient>,
    subscription: Arc<ServiceClient>,
    content: Arc<ServiceClient>,
    notification: Arc<ServiceClient>,
    analytics: Arc<ServiceClient>,
    indexer: Arc<ServiceClient>,
    api_url: String,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-admin");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3001);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(30) };
    let state = AppState {
        identity: Arc::new(ServiceClient::new(cfg.clone())),
        wallet: Arc::new(ServiceClient::new(cfg.clone())),
        payment: Arc::new(ServiceClient::new(cfg.clone())),
        subscription: Arc::new(ServiceClient::new(cfg.clone())),
        content: Arc::new(ServiceClient::new(cfg.clone())),
        notification: Arc::new(ServiceClient::new(cfg.clone())),
        analytics: Arc::new(ServiceClient::new(cfg.clone())),
        indexer: Arc::new(ServiceClient::new(cfg.clone())),
        api_url,
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/auth/login", post(siwe_login))
        .route("/api/v1/auth/refresh", post(refresh_token))
        .route("/api/v1/users", any(list_users))
        .route("/api/v1/users/{id}", any(get_user))
        .route("/api/v1/payments", any(list_payments))
        .route("/api/v1/subscriptions", any(list_subscriptions))
        .route("/api/v1/pages", any(list_pages))
        .route("/api/v1/blocks", any(list_blocks))
        .route("/api/v1/themes", any(list_themes))
        .route("/api/v1/analytics/track", post(track_event))
        .route("/api/v1/analytics/revenue", get(revenue))
        .route("/api/v1/indexer/status/{chain}", any(chain_status))
        .route("/api/v1/notifications/templates", any(list_templates))
        .fallback(ssr_fallback)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Admin BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str { "ok" }

async fn siwe_login(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.identity.post_plain("/api/v1/identity/auth/siwe", &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn refresh_token(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.identity.post_plain("/api/v1/identity/auth/refresh", &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_users(
    axum::extract::State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Response, StatusCode> {
    let _ctx = RequestContext::from_headers(&headers);
    state.identity.get_plain("/api/v1/identity/users").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn get_user(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(id): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/identity/users/{}", id);
    state.identity.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_payments(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.payment.get_plain("/api/v1/payment/intents").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_subscriptions(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.subscription.get_plain("/api/v1/subscription/subscriptions").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_pages(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/pages").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_blocks(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/blocks").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_themes(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.content.get_plain("/api/v1/content/themes").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn track_event(
    axum::extract::State(state): axum::extract::State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    state.analytics.post_plain("/api/v1/analytics/track", &body).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn revenue(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.analytics.get_plain("/api/v1/analytics/revenue").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn chain_status(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(chain): AxPath<String>,
) -> Result<Response, StatusCode> {
    let path = format!("/api/v1/indexer/status/{}", chain);
    state.indexer.get_plain(&path).await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn list_templates(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Response, StatusCode> {
    state.notification.get_plain("/api/v1/notification/templates").await
        .map(|v| Json(v).into_response())
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn ssr_fallback(uri: axum::http::Uri) -> Response {
    let html = render_page(uri.path());
    Html(html).into_response()
}

fn render_page(path: &str) -> String {
    let title = if path.starts_with("/users") { "User Management" }
        else if path.starts_with("/payments") { "Payment Management" }
        else if path.starts_with("/subscriptions") { "Subscription Management" }
        else if path.starts_with("/content") { "Content Management" }
        else if path.starts_with("/analytics") { "Analytics Dashboard" }
        else if path.starts_with("/settings") { "System Settings" }
        else if path.starts_with("/audit") { "Audit Log" }
        else { "Admin Dashboard" };

    let active = |p: &str| -> &'static str {
        if path == p || (p != "/" && path.starts_with(p)) { "active" } else { "" }
    };

    let nav_links = [
        ("/", "gauge-high", "Dashboard"),
        ("/users", "users", "Users"),
        ("/payments", "credit-card", "Payments"),
        ("/subscriptions", "vault", "Subscriptions"),
        ("/content", "newspaper", "Content"),
        ("/analytics", "chart-line", "Analytics"),
        ("/settings", "gear", "Settings"),
        ("/audit", "list-check", "Audit Log"),
    ];
    let mut links_html = String::new();
    for (href, icon, label) in nav_links {
        let cls = active(href);
        links_html.push_str(&format!(
            r#"<a href="{href}" class="nav-link {cls}" style="width:100%;justify-content:flex-start;padding:0.625rem 0.75rem;">
              <i data-lucide="{icon}" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;"></i>
              <span>{label}</span>
            </a>"#,
        ));
    }

    let body = format!(
        r##"<div style="display:flex;min-height:calc(100vh - 3.5rem);">
  <aside class="admin-sidebar" style="width:16rem;background:var(--bg-secondary);border-right:1px solid var(--border);padding:1.5rem 1rem;display:flex;flex-direction:column;gap:1rem;">
    <div style="display:flex;align-items:center;gap:0.5rem;padding:0 0.5rem 0.5rem;border-bottom:1px solid var(--border);">
      {logo}
      <span class="badge badge-primary">Admin</span>
    </div>
    <nav style="display:flex;flex-direction:column;gap:0.25rem;">{links}</nav>
    <div style="margin-top:auto;padding:0.75rem;border-top:1px solid var(--border);font-size:0.75rem;color:var(--text-subtle);">
      <i data-lucide="shield" style="color:var(--epsx-orange);"></i> Secured by EPSX
    </div>
  </aside>
  <main style="flex:1;padding:2rem;overflow:auto;">
    <div style="display:flex;justify-content:space-between;align-items:flex-end;flex-wrap:wrap;gap:1rem;margin-bottom:2rem;">
      <div>
        <span class="badge-pill"><i data-lucide="shield" style="color:var(--epsx-orange);"></i> Admin</span>
        <h1 style="font-size:2rem;font-weight:800;margin-top:0.75rem;">{title}</h1>
      </div>
    </div>
    <div id="app" data-route="{path}">Loading...</div>
  </main>
</div>
<script>
const route = document.getElementById('app').dataset.route;
async function load() {{
  if (route === '/') {{
    document.getElementById('app').innerHTML = '<div class="skeleton" style="height:8rem;"></div>';
    const [u, s, p, rev] = await Promise.all([
      fetch('/api/v1/users').then(r => r.json()).catch(() => []),
      fetch('/api/v1/subscriptions').then(r => r.json()).catch(() => []),
      fetch('/api/v1/payments').then(r => r.json()).catch(() => []),
      fetch('/api/v1/analytics/revenue').then(r => r.json()).catch(() => ({{}})),
    ]);
    const users = u.users || u || [];
    const subs = s || [];
    const payments = p || [];
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1.5rem;">
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Total Users</span>
            <i data-lucide="users" style="color:var(--epsx-blue-start);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;">${{users.length}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Revenue</span>
            <i data-lucide="dollar-sign" style="color:var(--epsx-green);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div class="gradient-text" style="font-size:2rem;font-weight:800;">$0</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Active Subs</span>
            <i data-lucide="shield" style="color:var(--epsx-purple);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;color:var(--epsx-purple);">${{subs.length}}</div>
        </div>
        <div class="card-insight">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;">
            <span style="font-size:0.875rem;color:var(--text-muted);">Payments</span>
            <i data-lucide="credit-card" style="color:var(--epsx-orange);width:1.25rem;height:1.25rem;"></i>
          </div>
          <div style="font-size:2rem;font-weight:800;color:var(--epsx-orange);">${{payments.length}}</div>
        </div>
      </div>`;
  }} else if (route === '/users') {{
    const u = await fetch('/api/v1/users').then(r => r.json()).catch(() => []);
    const list = u.users || u || [];
    document.getElementById('app').innerHTML = `
      <div class="table-wrap">
        <table class="table">
          <thead><tr><th>Address</th><th>Roles</th><th>Created</th></tr></thead>
          <tbody>${{list.map(x => `<tr><td style="font-family:monospace;">${{(x.address||'').slice(0,10)}}...</td><td><span class="badge badge-primary">${{(x.roles||[]).join(', ') || 'user'}}</span></td><td style="color:var(--text-subtle);">${{(x.created_at||'').slice(0,10)}}</td></tr>`).join('') || '<tr><td colspan="3" style="text-align:center;color:var(--text-muted);">No users yet</td></tr>'}}</tbody>
        </table>
      </div>`;
  }} else if (route === '/payments') {{
    const p = await fetch('/api/v1/payments').then(r => r.json()).catch(() => []);
    const list = p || [];
    document.getElementById('app').innerHTML = `
      <div class="table-wrap">
        <table class="table">
          <thead><tr><th>Intent ID</th><th>Amount</th><th>Status</th><th>Created</th></tr></thead>
          <tbody>${{list.map(x => `<tr><td style="font-family:monospace;">${{(x.id||'').slice(0,10)}}...</td><td style="text-align:right;font-weight:600;">${{x.amount}} ${{x.currency||''}}</td><td><span class="badge badge-${{x.status==='confirmed'?'success':x.status==='failed'?'danger':'warning'}}">${{x.status||'pending'}}</span></td><td style="color:var(--text-subtle);">${{(x.created_at||'').slice(0,10)}}</td></tr>`).join('') || '<tr><td colspan="4" style="text-align:center;color:var(--text-muted);">No payments yet</td></tr>'}}</tbody>
        </table>
      </div>`;
  }} else if (route === '/subscriptions') {{
    const s = await fetch('/api/v1/subscriptions').then(r => r.json()).catch(() => []);
    const list = s || [];
    document.getElementById('app').innerHTML = `
      <div class="table-wrap">
        <table class="table">
          <thead><tr><th>Sub ID</th><th>Plan</th><th>Status</th></tr></thead>
          <tbody>${{list.map(x => `<tr><td style="font-family:monospace;">${{(x.id||'').slice(0,10)}}...</td><td style="font-family:monospace;">${{(x.plan_id||'').slice(0,10)}}</td><td><span class="badge badge-${{x.status==='active'?'active':x.status==='cancelled'?'danger':'warning'}}">${{x.status||'pending'}}</span></td></tr>`).join('') || '<tr><td colspan="3" style="text-align:center;color:var(--text-muted);">No subscriptions yet</td></tr>'}}</tbody>
        </table>
      </div>`;
  }} else if (route === '/content') {{
    const [p, b, t] = await Promise.all([
      fetch('/api/v1/pages').then(r => r.json()).catch(() => []),
      fetch('/api/v1/blocks').then(r => r.json()).catch(() => []),
      fetch('/api/v1/themes').then(r => r.json()).catch(() => []),
    ]);
    const pages = p || [];
    const blocks = b || [];
    const themes = t || [];
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;">
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Pages</span>
            <span class="badge badge-primary">${{pages.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{pages.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;">${{x.slug}}</li>`).join('') || '<li style="color:var(--text-muted);">No pages</li>'}}
          </ul>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Blocks</span>
            <span class="badge badge-info">${{blocks.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{blocks.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;display:flex;justify-content:space-between;"><span>${{x.name}}</span><span style="color:var(--text-subtle);">${{x.category}}</span></li>`).join('') || '<li style="color:var(--text-muted);">No blocks</li>'}}
          </ul>
        </section>
        <section class="card-insight">
          <h3 style="font-weight:600;margin-bottom:0.75rem;display:flex;justify-content:space-between;align-items:center;">
            <span>Themes</span>
            <span class="badge badge-purple">${{themes.length}}</span>
          </h3>
          <ul style="list-style:none;padding:0;margin:0;display:flex;flex-direction:column;gap:0.5rem;font-size:0.875rem;">
            ${{themes.map(x => `<li style="padding:0.5rem;background:var(--bg-secondary);border-radius:0.375rem;display:flex;justify-content:space-between;"><span>${{x.name}}</span>${{x.is_default ? '<span class="badge badge-primary" style="font-size:0.625rem;">default</span>' : ''}}</li>`).join('') || '<li style="color:var(--text-muted);">No themes</li>'}}
          </ul>
        </section>
      </div>`;
  }} else if (route === '/analytics') {{
    document.getElementById('app').innerHTML = `
      <div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(200px, 1fr));gap:1.5rem;">
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Page Views</div><div style="font-size:2rem;font-weight:800;">-</div></div>
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Signups</div><div style="font-size:2rem;font-weight:800;">-</div></div>
        <div class="card-insight"><div style="font-size:0.875rem;color:var(--text-muted);margin-bottom:0.5rem;">Revenue</div><div class="gradient-text" style="font-size:2rem;font-weight:800;">-</div></div>
      </div>
      <p style="color:var(--text-muted);font-size:0.875rem;margin-top:1rem;">Hook up analytics service to populate.</p>`;
  }} else if (route === '/settings') {{
    document.getElementById('app').innerHTML = `
      <div class="card-insight">
        <h3 style="font-weight:600;margin-bottom:1rem;">System</h3>
        <dl style="display:grid;gap:0.75rem;font-size:0.875rem;">
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Environment</dt><dd>production</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Region</dt><dd>local</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;border-bottom:1px solid var(--border);"><dt style="color:var(--text-muted);">Database</dt><dd>PostgreSQL</dd></div>
          <div style="display:flex;justify-content:space-between;padding:0.5rem 0;"><dt style="color:var(--text-muted);">Cache</dt><dd>Redis</dd></div>
        </dl>
      </div>`;
  }} else if (route === '/audit') {{
    document.getElementById('app').innerHTML = '<div class="card-insight" style="text-align:center;color:var(--text-muted);padding:3rem;">Audit log empty.</div>';
  }}
}}
load();
</script>"##,
        title = title,
        path = path,
        logo = logo("/", "sm"),
        links = links_html,
    );

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

    let full_title = format!("{} - EPSX Admin", title);
    page_shell_with_body_class(&full_title, "EPSX Admin Dashboard", &nav, &body, false, "page-bg")
}
