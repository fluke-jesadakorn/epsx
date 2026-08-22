//! Server-rendered OpenAPI documentation routes.
//!
//! The reference is rendered from the same `utoipa` model as the JSON
//! endpoints. It intentionally loads no CDN library or browser script.

use axum::http::{HeaderMap, HeaderValue};
use axum::{
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::Value;
use utoipa::OpenApi;

use crate::config::env::get_env_var;
use crate::web::docs::openapi_admin::AdminApiDoc;
use crate::web::docs::openapi_user::UserApiDoc;

pub fn create_docs_routes() -> Router {
    Router::new()
        .route("/docs", get(docs_user_handler))
        .route("/api-docs/openapi.json", get(openapi_user_json_handler))
        .route("/admin/docs", get(docs_admin_handler))
        .route(
            "/admin/api-docs/openapi.json",
            get(openapi_admin_json_handler),
        )
}

pub async fn docs_user_handler() -> impl IntoResponse {
    create_openapi_html(
        "EPSX API Documentation",
        "EPSX Data Analytics Platform HTTP API",
        "/api-docs/openapi.json",
        UserApiDoc::openapi(),
        false,
    )
}

pub async fn docs_admin_handler() -> impl IntoResponse {
    create_openapi_html(
        "EPSX Admin API Documentation",
        "Administrative and service-management HTTP API",
        "/admin/api-docs/openapi.json",
        AdminApiDoc::openapi(),
        true,
    )
}

fn create_openapi_html(
    title: &str,
    description: &str,
    openapi_url: &str,
    spec: utoipa::openapi::OpenApi,
    is_admin: bool,
) -> impl IntoResponse {
    let operations = render_operations(&spec);
    let badge = if is_admin {
        r#"<span class="badge admin">Admin surface</span>"#
    } else {
        r#"<span class="badge">Public surface</span>"#
    };
    let title = escape_text(title);
    let description = escape_text(description);
    let openapi_url = escape_attr(openapi_url);
    let html = format!(
        r#"<!doctype html>
<html lang="en" class="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta name="description" content="{description}">
  <title>{title}</title>
  <style>
    :root {{ color-scheme: dark; font-family: ui-sans-serif, system-ui, sans-serif; background:#09090b; color:#f4f4f5; }}
    * {{ box-sizing:border-box; }} body {{ margin:0; min-height:100vh; background:linear-gradient(145deg,#09090b,#111827); }}
    header {{ padding:2.5rem max(1rem,calc((100vw - 76rem)/2)); border-bottom:1px solid #27272a; background:rgba(9,9,11,.92); }}
    h1 {{ margin:.5rem 0; font-size:clamp(1.8rem,4vw,3rem); }} h2,h3,p {{ margin-top:0; }}
    main {{ max-width:76rem; margin:0 auto; padding:2rem 1rem 5rem; }}
    .header-row,.operation-head,.meta {{ display:flex; align-items:center; gap:.75rem; flex-wrap:wrap; }}
    .badge,.method,.status {{ display:inline-flex; border-radius:999px; padding:.25rem .65rem; font-size:.75rem; font-weight:800; letter-spacing:.04em; }}
    .badge {{ background:#312e81; color:#c7d2fe; }} .badge.admin {{ background:#7f1d1d; color:#fecaca; }}
    .raw {{ color:#93c5fd; }} .intro {{ color:#a1a1aa; max-width:52rem; }}
    .operation {{ margin:0 0 1rem; padding:1.1rem; border:1px solid #27272a; border-radius:.9rem; background:rgba(24,24,27,.72); }}
    .path {{ font-family:ui-monospace,monospace; overflow-wrap:anywhere; }}
    .method {{ min-width:4.5rem; justify-content:center; background:#164e63; color:#a5f3fc; }}
    .method-post {{ background:#14532d; color:#bbf7d0; }} .method-put,.method-patch {{ background:#713f12; color:#fde68a; }} .method-delete {{ background:#7f1d1d; color:#fecaca; }}
    .summary {{ margin:.75rem 0 .35rem; font-weight:700; }} .description {{ color:#a1a1aa; white-space:pre-wrap; }}
    .status {{ border:1px solid #3f3f46; color:#d4d4d8; margin:.25rem .25rem 0 0; }}
    .empty {{ padding:2rem; border:1px dashed #3f3f46; border-radius:.9rem; color:#a1a1aa; }}
    a:focus-visible {{ outline:3px solid #38bdf8; outline-offset:3px; }}
  </style>
</head>
<body>
  <header>
    <div class="header-row">{badge}<span class="badge">SSR</span><span class="badge">OpenAPI</span></div>
    <h1>{title}</h1>
    <p class="intro">{description}</p>
    <a class="raw" href="{openapi_url}">Download the canonical OpenAPI JSON</a>
  </header>
  <main aria-label="API operations">{operations}</main>
</body>
</html>"#
    );

    let mut headers = HeaderMap::new();
    let frontend_url =
        get_env_var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let admin_url =
        get_env_var("ADMIN_FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3001".to_string());
    let csp = format!(
        "default-src 'none'; style-src 'unsafe-inline'; img-src 'self' data:; frame-ancestors 'self' {} {}",
        csp_origin(&frontend_url),
        csp_origin(&admin_url)
    );
    headers.insert("x-frame-options", HeaderValue::from_static("SAMEORIGIN"));
    if let Ok(value) = HeaderValue::from_str(&csp) {
        headers.insert("content-security-policy", value);
    }
    headers.insert(
        "content-type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    (headers, Html(html))
}

fn render_operations(spec: &utoipa::openapi::OpenApi) -> String {
    let Ok(value) = serde_json::to_value(spec) else {
        return r#"<p class="empty">The OpenAPI document could not be rendered.</p>"#.into();
    };
    let Some(paths) = value.get("paths").and_then(Value::as_object) else {
        return r#"<p class="empty">No API paths are published.</p>"#.into();
    };
    let mut rendered = String::new();
    for (path, item) in paths {
        let Some(item) = item.as_object() else {
            continue;
        };
        for method in ["get", "post", "put", "patch", "delete", "head", "options"] {
            let Some(operation) = item.get(method).and_then(Value::as_object) else {
                continue;
            };
            let summary = operation
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or("Documented operation");
            let description = operation
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let statuses = operation
                .get("responses")
                .and_then(Value::as_object)
                .map(|responses| {
                    responses
                        .keys()
                        .map(|status| {
                            format!(r#"<span class="status">{}</span>"#, escape_text(status))
                        })
                        .collect::<String>()
                })
                .unwrap_or_default();
            rendered.push_str(&format!(
                r#"<article class="operation"><div class="operation-head"><span class="method method-{method}">{method_upper}</span><code class="path">{path}</code></div><p class="summary">{summary}</p><p class="description">{description}</p><div class="meta" aria-label="Documented response statuses">{statuses}</div></article>"#,
                method = escape_attr(method),
                method_upper = escape_text(&method.to_ascii_uppercase()),
                path = escape_text(path),
                summary = escape_text(summary),
                description = escape_text(description),
            ));
        }
    }
    if rendered.is_empty() {
        r#"<p class="empty">No API operations are published.</p>"#.into()
    } else {
        rendered
    }
}

fn csp_origin(value: &str) -> &str {
    if value.starts_with("https://") || value.starts_with("http://localhost") {
        value
    } else {
        "'none'"
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value)
}

pub async fn openapi_user_json_handler() -> impl IntoResponse {
    let mut response = Json(crate::web::operation_registry::openapi_document()).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=60"),
    );
    response
}

pub async fn openapi_admin_json_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(AdminApiDoc::openapi())
}

pub use docs_user_handler as docs_scalar_handler;
pub use openapi_user_json_handler as openapi_json_handler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_escapes_operation_metadata_and_emits_no_script() {
        let html = render_operations(&UserApiDoc::openapi());
        assert!(html.contains("class=\"operation\"") || html.contains("No API operations"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("@scalar"));
    }

    #[test]
    fn html_escaping_is_context_safe() {
        assert_eq!(
            escape_text("</p><script>alert(1)</script>"),
            "&lt;/p&gt;&lt;script&gt;alert(1)&lt;/script&gt;"
        );
    }
}
