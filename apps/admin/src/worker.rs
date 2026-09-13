//! Cloudflare Worker for `epsx-admin` BFF (same pattern as frontend).
use worker::*;
#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let backend_url = env.var("BACKEND_URL").map(|v| v.to_string()).unwrap_or_else(|_| "https://api.epsx.io".to_string());
    let url = req.url()?.to_string();
    if url.contains("/api/") {
        let origin = format!("{}{}", backend_url, url.split("/api/").nth(1).map(|s| format!("/api/{}", s)).unwrap_or_default());
        let mut init = RequestInit::new();
        init.with_method(req.method());
        let mut headers = Headers::new();
        for (k, v) in req.headers().entries() { if k.to_lowercase() != "host" { headers.set(&k, &v)?; } }
        init.with_headers(headers);
        if req.method() != Method::Get && req.method() != Method::Head {
            if let Some(body) = req.text().await.ok().filter(|s| !s.is_empty()) { init.with_body(Some(body.into())); }
        }
        let proxied = Request::new_with_init(&origin, &init)?;
        return Fetch::Request(proxied).send().await;
    }
    Response::ok(format!("epsx-admin Worker (env={})", env.var("ENV").map(|v| v.to_string()).unwrap_or("local".into())))
}
