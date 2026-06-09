use axum::{
    extract::{Path as AxPath, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use epsx_client::ServiceClient;
use epsx_templates::{page_shell_with_body_class, theme_toggle_button, logo};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    content: Arc<ServiceClient>,
    api_url: String,
}

#[derive(Deserialize)]
struct PreviewQuery {
    pub edit_session: Option<String>,
    pub user_id: Option<String>,
}

#[tokio::main]
async fn main() {
    epsx_observability::Observability::init("bff-preview");

    let api_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let port: u16 = std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3003);
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let cfg = epsx_client::ClientConfig { base_url: api_url.clone(), timeout: std::time::Duration::from_secs(30) };
    let state = AppState {
        content: Arc::new(ServiceClient::new(cfg)),
        api_url,
    };

    let app = Router::new()
        .route("/api/health", get(api_health))
        .route("/api/v1/preview/{slug}", get(preview_json))
        .route("/preview/{*slug}", get(preview_html))
        .route("/", get(preview_index))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("Preview BFF listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_health() -> &'static str { "ok" }

async fn preview_json(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(slug): AxPath<String>,
    Query(q): Query<PreviewQuery>,
) -> Result<Response, StatusCode> {
    let slug = if slug.is_empty() { "home".to_string() } else { slug };
    let path = format!("/api/v1/content/pages/{}", slug);
    let mut page = state.content.get_plain(&path).await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    if let Some(session) = q.edit_session {
        page["preview_session"] = serde_json::json!(session);
    }
    if let Some(user) = q.user_id {
        page["preview_user"] = serde_json::json!(user);
    }
    Ok(Json(page).into_response())
}

async fn preview_html(
    axum::extract::State(state): axum::extract::State<AppState>,
    AxPath(slug): AxPath<String>,
    Query(q): Query<PreviewQuery>,
) -> Result<Response, StatusCode> {
    let slug = if slug.is_empty() { "home".to_string() } else { slug };
    let path = format!("/api/v1/content/pages/{}", slug);
    let page = state.content.get_plain(&path).await.map_err(|e| {
        tracing::warn!("Failed to load page {}: {}", slug, e);
        StatusCode::NOT_FOUND
    })?;

    let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let blocks_str = page.get("blocks_json")
        .and_then(|v| v.as_str())
        .or_else(|| page.get("blocks").and_then(|v| v.as_str()))
        .unwrap_or("[]");
    let theme_id = page.get("theme_id").and_then(|v| v.as_str());

    let mut theme_css = String::new();
    if let Some(tid) = theme_id {
        let tpath = format!("/api/v1/content/themes/{}", tid);
        if let Ok(theme) = state.content.get_plain(&tpath).await {
            theme_css = theme_to_css(&theme);
        }
    }

    let body = render_blocks(blocks_str);
    let session = q.edit_session.as_deref().unwrap_or("");

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

    let main_body = format!(
        r##"<div style="position:fixed;top:3.5rem;left:0;right:0;background:rgba(245,158,11,0.15);border-bottom:1px solid rgba(245,158,11,0.3);padding:0.625rem 1rem;text-align:center;color:var(--epsx-amber);font-size:0.875rem;z-index:40;backdrop-filter:blur(8px);">
  <i class="fa-solid fa-eye"></i> PREVIEW MODE{session}
</div>
<div style="padding-top:6rem;">{body}</div>"##,
        session = if session.is_empty() { String::new() } else { format!(" — session {}", session) },
        body = body,
    );

    let full_title = format!("{} (Preview)", title);
    let shell = page_shell_with_body_class(&full_title, "Page preview", &nav, &main_body, false, "page-bg");
    let html = format!("{}\n<style>{}</style>", shell, theme_css);
    Ok(Html(html).into_response())
}

async fn preview_index() -> Response {
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
    let body = r##"<section class="section" style="display:flex;align-items:center;justify-content:center;min-height:80vh;">
<div style="text-align:center;max-width:32rem;">
  <div style="width:5rem;height:5rem;border-radius:9999px;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin:0 auto 1.5rem;box-shadow:var(--shadow-orange);">
    <i class="fa-solid fa-eye" style="color:white;font-size:1.5rem;"></i>
  </div>
  <span class="badge-pill"><i class="fa-solid fa-eye" style="color:var(--epsx-orange);"></i> Preview</span>
  <h1 style="font-size:2.5rem;font-weight:800;margin:1rem 0 1rem;">EPSX Preview</h1>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">Use <code class="badge badge-primary">/preview/&lt;slug&gt;</code> to preview a page.</p>
  <a href="/" class="btn btn-gradient btn-lg"><i class="fa-solid fa-house"></i> Back to Home</a>
</div>
</section>"##;
    let shell = page_shell_with_body_class("EPSX Preview", "Page preview server", &nav, body, false, "page-bg");
    Html(shell).into_response()
}

fn theme_to_css(theme: &serde_json::Value) -> String {
    let mut css = String::new();
    if let Some(colors) = theme.get("colors_json").and_then(|v| v.as_object())
        .or_else(|| theme.get("colors").and_then(|v| v.as_object())) {
        for (k, v) in colors {
            let v = v.as_str().unwrap_or("");
            css.push_str(&format!("--{}: {};", k, v));
        }
    }
    if !css.is_empty() { format!(":root{{{}}}", css) } else { String::new() }
}

fn render_blocks(blocks_json: &str) -> String {
    let blocks: Vec<serde_json::Value> = serde_json::from_str(blocks_json).unwrap_or_default();
    blocks.iter().map(|b| {
        let bt = b.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let props = b.get("props").cloned().unwrap_or(serde_json::json!({}));
        match bt {
            "hero" => format!(
                r##"<section class="section" style="text-align:center;min-height:60vh;display:flex;align-items:center;">
<div class="container-x">
  <h1 style="font-size:clamp(2.25rem, 5vw, 4.5rem);font-weight:800;line-height:1.1;margin-bottom:1rem;">{title}</h1>
  <p style="font-size:1.25rem;color:var(--text-muted);max-width:42rem;margin:0 auto 2rem;">{subtitle}</p>
  <a href="{cta_link}" class="btn btn-gradient btn-xl"><i class="fa-solid fa-arrow-right"></i> {cta_text}</a>
</div>
</section>"##,
                title = props.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                subtitle = props.get("subtitle").and_then(|v| v.as_str()).unwrap_or(""),
                cta_text = props.get("ctaText").and_then(|v| v.as_str()).unwrap_or("Get Started"),
                cta_link = props.get("ctaLink").and_then(|v| v.as_str()).unwrap_or("#"),
            ),
            "features" => {
                let items = props.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let cards = items.iter().map(|item| format!(
                    r##"<div class="card-insight">
  <div style="width:3rem;height:3rem;border-radius:0.75rem;background:var(--gradient-warm);display:flex;align-items:center;justify-content:center;margin-bottom:1rem;">
    <i class="fa-solid fa-bolt" style="color:white;"></i>
  </div>
  <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;">{title}</h3>
  <p style="color:var(--text-muted);">{description}</p>
</div>"##,
                    title = item.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    description = item.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                )).collect::<Vec<_>>().join("");
                format!(r##"<section class="section"><div class="container-x">
<h2 style="font-size:2.5rem;font-weight:800;text-align:center;margin-bottom:2rem;">{title}</h2>
<div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;">{cards}</div>
</div></section>"##,
                    title = props.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    cards = cards)
            }
            "pricing" => {
                let plans = props.get("plans").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let cards = plans.iter().map(|plan| {
                    let name = plan.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let price = plan.get("price").and_then(|v| v.as_str()).unwrap_or("");
                    let features_html = plan.get("features").and_then(|v| v.as_array()).cloned().unwrap_or_default()
                        .iter().map(|f| format!(r##"<li style="display:flex;gap:0.5rem;align-items:center;"><i class="fa-solid fa-check" style="color:var(--epsx-green);"></i>{}</li>"##, f.as_str().unwrap_or(""))).collect::<Vec<_>>().join("");
                    let cta = plan.get("ctaText").and_then(|v| v.as_str()).unwrap_or("Subscribe");
                    format!(r##"<div class="card-insight">
  <h3 style="font-size:1.5rem;font-weight:700;margin-bottom:0.5rem;">{name}</h3>
  <div class="gradient-text" style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">{price}</div>
  <ul style="list-style:none;padding:0;margin:0 0 1.5rem;display:grid;gap:0.5rem;">{features}</ul>
  <button class="btn btn-gradient btn-block">{cta}</button>
</div>"##, name = name, price = price, features = features_html, cta = cta)
                }).collect::<Vec<_>>().join("");
                format!(r##"<section class="section"><div class="container-x">
<h2 style="font-size:2.5rem;font-weight:800;text-align:center;margin-bottom:2rem;">{title}</h2>
<div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;max-width:72rem;margin:0 auto;">{cards}</div>
</div></section>"##,
                    title = props.get("title").and_then(|v| v.as_str()).unwrap_or("Pricing"),
                    cards = cards)
            }
            "testimonial" => format!(
                r##"<section class="section" style="text-align:center;">
<div class="container-x" style="max-width:48rem;">
  <i class="fa-solid fa-quote-left" style="font-size:2rem;color:var(--epsx-orange);margin-bottom:1rem;display:block;"></i>
  <blockquote style="font-size:1.5rem;font-style:italic;line-height:1.6;margin-bottom:1.5rem;">"{quote}"</blockquote>
  <div>
    <div style="font-weight:600;">{author}</div>
    <div style="color:var(--text-muted);font-size:0.875rem;">{role}</div>
  </div>
</div>
</section>"##,
                quote = props.get("quote").and_then(|v| v.as_str()).unwrap_or(""),
                author = props.get("author").and_then(|v| v.as_str()).unwrap_or(""),
                role = props.get("role").and_then(|v| v.as_str()).unwrap_or(""),
            ),
            "cta-banner" => format!(
                r##"<section class="section" style="text-align:center;background:var(--gradient-page);">
<div class="container-x" style="max-width:48rem;">
  <h2 style="font-size:2.5rem;font-weight:800;margin-bottom:1rem;">{title}</h2>
  <p style="color:var(--text-muted);font-size:1.125rem;margin-bottom:2rem;">{description}</p>
  <a href="{cta_link}" class="btn btn-gradient btn-xl"><i class="fa-solid fa-arrow-right"></i> {cta_text}</a>
</div>
</section>"##,
                title = props.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                description = props.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                cta_text = props.get("ctaText").and_then(|v| v.as_str()).unwrap_or("Get Started"),
                cta_link = props.get("ctaLink").and_then(|v| v.as_str()).unwrap_or("#"),
            ),
            "blog-list" => {
                let posts = props.get("posts").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let cards = posts.iter().map(|p| format!(
                    r##"<a href="/blog/{slug}" class="card-insight" style="text-decoration:none;color:inherit;display:block;">
  <h3 style="font-size:1.25rem;font-weight:700;margin-bottom:0.5rem;">{title}</h3>
  <p style="color:var(--text-muted);">{excerpt}</p>
</a>"##,
                    slug = p.get("slug").and_then(|v| v.as_str()).unwrap_or(""),
                    title = p.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    excerpt = p.get("excerpt").and_then(|v| v.as_str()).unwrap_or(""),
                )).collect::<Vec<_>>().join("");
                format!(r##"<section class="section"><div class="container-x">
<h2 style="font-size:2.5rem;font-weight:800;text-align:center;margin-bottom:2rem;">{title}</h2>
<div style="display:grid;grid-template-columns:repeat(auto-fit, minmax(280px, 1fr));gap:1.5rem;">{cards}</div>
</div></section>"##,
                    title = props.get("title").and_then(|v| v.as_str()).unwrap_or("Latest Posts"),
                    cards = cards)
            }
            "rich-text" => format!(
                r##"<section class="section"><div class="container-x" style="max-width:48rem;font-size:1.125rem;line-height:1.7;">{content}</div></section>"##,
                content = props.get("content").and_then(|v| v.as_str()).unwrap_or(""),
            ),
            "custom-html" => props.get("html").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            _ => format!("<!-- unknown block type: {} -->", bt),
        }
    }).collect()
}
