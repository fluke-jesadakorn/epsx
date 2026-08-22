//! Static asset serving helpers.

use axum::{
    http::{HeaderName, HeaderValue},
    Router,
};
use std::path::Path;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

const WORKER_MODULE: &str = "epsx_service_worker_bootstrap.v3.js";
const LEGACY_WORKER_MODULE: &str = "epsx_service_worker_bootstrap.js";

pub fn static_assets_router(public_dir: &str) -> Router {
    Router::new().nest_service("/public", ServeDir::new(public_dir))
}

/// Serve generated browser runtime files while authorizing only the recovery
/// worker bootstrap to claim the root application scope.
pub fn browser_runtime_router<S>(runtime_dir: &str) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let worker = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("service-worker-allowed"),
            HeaderValue::from_static("/"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeFile::new(Path::new(runtime_dir).join(WORKER_MODULE)));
    let legacy_worker = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("service-worker-allowed"),
            HeaderValue::from_static("/"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeFile::new(
            Path::new(runtime_dir).join(LEGACY_WORKER_MODULE),
        ));
    let runtime = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeDir::new(runtime_dir));
    Router::new()
        .route_service(&format!("/runtime/{WORKER_MODULE}"), worker)
        .route_service(&format!("/runtime/{LEGACY_WORKER_MODULE}"), legacy_worker)
        .nest_service("/runtime", runtime)
}

#[cfg(test)]
mod tests {
    use super::{browser_runtime_router, LEGACY_WORKER_MODULE, WORKER_MODULE};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn only_worker_bootstrap_can_claim_the_root_scope() {
        let directory =
            std::env::temp_dir().join(format!("epsx-bff-browser-runtime-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(WORKER_MODULE), "worker").unwrap();
        std::fs::write(directory.join(LEGACY_WORKER_MODULE), "legacy-worker").unwrap();
        std::fs::write(directory.join("epsx_browser_runtime.js"), "runtime").unwrap();
        let app: Router = browser_runtime_router(directory.to_str().unwrap());

        let worker = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/runtime/{WORKER_MODULE}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(worker.status(), StatusCode::OK);
        assert_eq!(worker.headers()["service-worker-allowed"], "/");
        assert_eq!(worker.headers()["cache-control"], "no-cache");

        let legacy_worker = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/runtime/{LEGACY_WORKER_MODULE}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(legacy_worker.status(), StatusCode::OK);
        assert_eq!(legacy_worker.headers()["service-worker-allowed"], "/");
        assert_eq!(legacy_worker.headers()["cache-control"], "no-cache");

        let runtime = app
            .oneshot(
                Request::builder()
                    .uri("/runtime/epsx_browser_runtime.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(runtime.status(), StatusCode::OK);
        assert!(!runtime.headers().contains_key("service-worker-allowed"));
        assert_eq!(runtime.headers()["cache-control"], "no-cache");

        std::fs::remove_dir_all(directory).unwrap();
    }
}
