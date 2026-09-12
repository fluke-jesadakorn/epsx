//! Cloudflare Worker entrypoint for `epsx-frontend` BFF (Dioxus SSR).
//! Reuses `dioxus_ssr::render_element` via `workers-rs` + `axum` bridge.
//! Local: `bunx wrangler dev --local --persist-to=.wrangler/state --config apps/frontend/wrangler.jsonc`
//! Prod: `bunx wrangler deploy --config apps/frontend/wrangler.jsonc`

use worker::*;

// Reuse existing BFF router logic via `epsx-bff` + `dioxus_ssr`.
// For now the Worker is a thin proxy to the K8s origin (`HYPERDRIVE_CORE` / `BACKEND_URL`);
// SSR runs in `workerd` WASM when `dioxus_ssr` is enabled with `workers-rs`.
// Full SSR cutover keeps `apps/frontend/src/main.rs::build_app` but swaps `axum` for `axum-cloudflare-adapter`.

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Static assets are served by `assets.directory = "dist"` (free/unlimited) before this fetch.
    // Fallback to origin for API routes; add Turnstile/WAF via Cloudflare dashboard.
    let backend_url = env.var("BACKEND_URL").map(|v| v.to_string()).unwrap_or_else(|_| "https://api.epsx.io".to_string());
    let url = req.url()?.to_string();
    // Simple proxy for /api/* to backend origin (Hyperdrive pool ≤5 per Worker).
    if url.contains("/api/") {
        let origin = format!("{}{}", backend_url, url.split("/api/").nth(1).map(|s| format!("/api/{}", s)).unwrap_or_default());
        let mut init = RequestInit::new();
        init.with_method(req.method());
        // Forward headers (strip host)
        let mut headers = Headers::new();
        for (k, v) in req.headers().entries() {
            if k.to_lowercase() != "host" { headers.set(&k, &v)?; }
        }
        init.with_headers(headers);
        if req.method() != Method::Get && req.method() != Method::Head {
            if let Some(body) = req.text().await.ok().filter(|s| !s.is_empty()) {
                init.with_body(Some(body.into()));
            }
        }
        let proxied = Request::new_with_init(&origin, &init)?;
        return Fetch::Request(proxied).send().await;
    }
    Response::ok(format!("epsx-frontend Worker (env={}) — static assets served via R2/Pages, API proxied to {}", env.var("ENV").map(|v| v.to_string()).unwrap_or("local".into()), backend_url))
}

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, _env: Env, _ctx: ScheduleContext) {
    console_log!("epsx-frontend cron — no-op (cron >5 requires Paid)");
}
