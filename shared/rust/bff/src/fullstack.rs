//! Explicit registration prevents another application's shared server functions
//! (including migration pilots) from being exposed by this BFF.
use axum::Router;
use dioxus_server::{FullstackState, ServerFunction};
use epsx_dioxus_ui::fullstack::Surface;
use std::sync::{Arc, Mutex};

/// DX uses stable asset URLs while rebuilding. Do not let a browser or the dev
/// tunnel cache an older JS/WASM pair (including a temporary missing asset).
pub async fn dev_no_cache(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;
    if std::env::var_os("DIOXUS_DEVSERVER_PORT").is_some() {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
    response
}

/// Refuse the framework's silent SSR-only fallback when a native package is
/// missing its generated client bundle.
pub fn verify_public_assets() -> std::io::Result<std::path::PathBuf> {
    let public = std::env::var_os("DIOXUS_PUBLIC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .expect("executable path")
                .parent()
                .expect("executable directory")
                .join("public")
        });
    verify_assets_at(&public)?;
    Ok(public)
}

fn verify_assets_at(public: &std::path::Path) -> std::io::Result<()> {
    let index = std::fs::read_to_string(public.join("index.html"))?;
    if !index.contains("<script")
        || !index.contains("id=\"main\"")
        || index.contains("epsx_browser_runtime")
        || index.contains("epsx-browser-runtime")
    {
        return Err(std::io::Error::other(
            "Expected a Dioxus hydration index without the legacy UI runtime",
        ));
    }
    fn has_wasm(path: &std::path::Path) -> std::io::Result<bool> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "wasm")
            {
                return Ok(true);
            }
            if kind.is_dir() && has_wasm(&entry.path())? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    if !has_wasm(public)? {
        return Err(std::io::Error::other(
            "Missing Dioxus WebAssembly assets; build and package the app's public directory",
        ));
    }
    Ok(())
}

/// Request-local response effects, never serialized into hydration data. Unlike
/// FullstackContext::add_response_header this preserves repeated Set-Cookie.
#[derive(Clone, Default)]
pub struct ResponseHeaders(Arc<Mutex<axum::http::HeaderMap>>);

impl ResponseHeaders {
    pub fn append(&self, name: axum::http::header::HeaderName, value: axum::http::HeaderValue) {
        self.0
            .lock()
            .expect("response headers lock")
            .append(name, value);
    }
}

async fn response_effects(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let effects = ResponseHeaders::default();
    request.extensions_mut().insert(effects.clone());
    let mut response = next.run(request).await;
    for (name, value) in effects.0.lock().expect("response headers lock").iter() {
        response.headers_mut().append(name, value.clone());
    }
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}

pub fn server_functions(surface: Surface, state: FullstackState) -> Router {
    let mut router = Router::new();
    for function in ServerFunction::collect() {
        if surface.owns_server_path(function.path()) {
            let mut method = function.method_router();
            // JSON byte arrays need up to four encoded bytes per source byte.
            // Only file commands receive larger limits; native adapters still
            // enforce the original decoded file-size and MIME constraints.
            let upload_limit = match function.path() {
                "/_server/frontend/chat-change" => Some(20 * 1024 * 1024 + 16 * 1024),
                "/_server/admin/media_change" | "/_server/admin/news_change" => {
                    Some(100 * 1024 * 1024 + 16 * 1024)
                }
                _ => None,
            };
            if let Some(limit) = upload_limit {
                method = method.layer(axum::extract::DefaultBodyLimit::max(limit));
            }
            router = router.route(function.path(), method);
        }
    }
    router
        .with_state(state)
        .layer(axum::middleware::from_fn(response_effects))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[test]
    fn native_packages_require_hydration_assets() {
        let directory =
            std::env::temp_dir().join(format!("epsx-hydration-{}", rand::random::<u64>()));
        std::fs::create_dir(&directory).unwrap();
        let result = std::panic::catch_unwind(|| {
            assert!(verify_assets_at(&directory).is_err());
            std::fs::write(
                directory.join("index.html"),
                "<div id=\"main\"></div><script type=\"module\" src=\"/wasm/app.js\"></script>",
            )
            .unwrap();
            assert!(verify_assets_at(&directory).is_err());
            std::fs::write(directory.join("app.wasm"), b"\0asm").unwrap();
            assert!(verify_assets_at(&directory).is_ok());
            std::fs::write(directory.join("index.html"), "<div id=\"main\"></div><script src=\"epsx_browser_runtime_bootstrap.js\"></script>").unwrap();
            assert!(verify_assets_at(&directory).is_err());
        });
        std::fs::remove_dir_all(directory).unwrap();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
    }

    #[tokio::test]
    async fn response_effects_preserve_all_session_cookies() {
        let router = Router::new()
            .route(
                "/logout",
                axum::routing::post(
                    |axum::Extension(effects): axum::Extension<ResponseHeaders>| async move {
                        for cookie in [
                            "session=; Max-Age=0; HttpOnly",
                            "refresh=; Max-Age=0; HttpOnly",
                        ] {
                            effects.append(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
                        }
                        "signed out"
                    },
                ),
            )
            .layer(axum::middleware::from_fn(response_effects));
        let response = router
            .oneshot(Request::post("/logout").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.headers().get_all("set-cookie").iter().count(), 2);
        assert_eq!(response.headers()["cache-control"], "no-store");
    }

    #[tokio::test]
    async fn legacy_and_other_app_functions_are_not_mounted() {
        let functions = ServerFunction::collect();
        assert!(
            functions
                .iter()
                .any(|function| function.path() == "/_server/frontend/analytics"),
            "registered paths: {:?}",
            functions
                .iter()
                .map(|function| function.path())
                .collect::<Vec<_>>()
        );
        for surface in [Surface::Frontend, Surface::Admin, Surface::Pay] {
            let router = server_functions(surface, FullstackState::headless());
            for function in &functions {
                if surface.owns_server_path(function.path()) {
                    continue;
                }
                let response = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method(function.method())
                            .uri(function.path())
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(
                    response.status(),
                    StatusCode::NOT_FOUND,
                    "{:?}: {}",
                    surface,
                    function.path()
                );
            }
        }
    }
}
