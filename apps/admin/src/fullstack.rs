//! Scoped typed providers for Dioxus hydration. Never forward an unverified bearer.
use crate::{
    analytics_admin_adapter::{load_admin_analytics, AdminAnalyticsLoad},
    AppState,
};
use axum::{Extension, Router};
use dioxus_server::FullstackState;
use epsx_dioxus_ui::{
    fullstack::{
        admin::{
            AdminAnalyticsProvider, AdminAuditProvider, AdminDashboardProvider, AuditQuery,
            DashboardData,
        },
        analytics::AnalyticsData,
        LoadError, Surface,
    },
    pages::analytics::{AnalyticsFilters, AnalyticsQueryState, AnalyticsResponse},
};
use std::sync::Arc;

pub fn server_functions(state: AppState) -> Router<AppState> {
    let core_wallets = crate::core_wallets_fullstack::provider(state.clone());
    let core_credits = crate::core_credits_fullstack::provider(state.clone());
    let wallets = wallets_provider(state.clone());
    let news = news_provider(state.clone());
    let developer = crate::fullstack_developer::provider(state.clone());
    let notifications = crate::fullstack_notifications::provider(state.clone());
    let payments = crate::fullstack_payments::provider(state.clone());
    let media = media_provider(state.clone());
    let escrow = escrow_provider(state.clone());
    let chat = crate::fullstack_chat::provider(state.clone());
    let auth = auth_provider(state.clone());
    let orders = crate::fullstack_orders::provider(state.clone());
    let catalog = crate::fullstack_catalog::provider(state.clone());
    let settings = crate::fullstack_settings::provider(state.clone());
    let audit = audit_provider(state.clone());
    let dashboard = dashboard_provider(state.clone());
    let provider = AdminAnalyticsProvider(Arc::new(move |query, headers| {
        let state = state.clone();
        Box::pin(async move { load(&state, query, headers).await })
    }));
    epsx_bff::fullstack::server_functions(Surface::Admin, FullstackState::headless())
        .layer(Extension(provider))
        .layer(Extension(audit))
        .layer(Extension(dashboard))
        .layer(Extension(settings))
        .layer(Extension(catalog))
        .layer(Extension(orders))
        .layer(Extension(auth))
        .layer(Extension(chat))
        .layer(Extension(escrow))
        .layer(Extension(media))
        .layer(Extension(developer))
        .layer(Extension(notifications))
        .layer(Extension(payments))
        .layer(Extension(news))
        .layer(Extension(wallets))
        .layer(Extension(core_wallets))
        .layer(Extension(core_credits))
        .with_state(())
}

async fn load(
    state: &AppState,
    query: AnalyticsQueryState,
    headers: http::HeaderMap,
) -> Result<AnalyticsData, LoadError> {
    // Validate the wire query again server-side: UI controls aren't a boundary.
    let url = query.page_url(query.page, query.limit.unwrap_or(10));
    let normalized = url.split_once('?').map(|(_, query)| query).unwrap_or("");
    let normalized = ranking_analytics_query(normalized).map_err(|_| LoadError::InvalidQuery)?;
    let query = AnalyticsQueryState::from_normalized_query(&normalized)
        .map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    // Backend admin authorization must succeed before exposing ranking data.
    match load_admin_analytics(&state.identity, &context).await {
        AdminAnalyticsLoad::Ready(_) | AdminAnalyticsLoad::Empty(_) => {}
        AdminAnalyticsLoad::Unauthorized => return Err(LoadError::Unauthenticated),
        AdminAnalyticsLoad::Forbidden => return Err(LoadError::Forbidden),
        AdminAnalyticsLoad::Malformed => return Err(LoadError::Malformed),
        AdminAnalyticsLoad::Unavailable => return Err(LoadError::Unavailable),
    }
    let path = format!("/api/analytics/rankings?{normalized}");
    let (rankings, filters) = tokio::join!(
        state.analytics.get_with_ctx(&path, &context),
        state.analytics.get_plain("/api/analytics/filters"),
    );
    let rankings = rankings
        .map_err(|_| LoadError::Unavailable)
        .and_then(|value| {
            serde_json::from_value::<AnalyticsResponse>(value)
                .map_err(|_| LoadError::Malformed)?
                .validated()
                .map_err(|_| LoadError::Malformed)
        });
    let filters = filters
        .map_err(|_| LoadError::Unavailable)
        .and_then(|value| {
            serde_json::from_value::<AnalyticsFilters>(value)
                .map_err(|_| LoadError::Malformed)?
                .validated()
                .map_err(|_| LoadError::Malformed)
        });
    Ok(AnalyticsData {
        query,
        rankings,
        filters,
        watchlist: Ok(None),
        signed_in: true,
    })
}

/// All Admin UI routes share the same SSR and hydration component tree.
pub fn application(state: AppState) -> Router {
    let core_wallets = crate::core_wallets_fullstack::provider(state.clone());
    let core_credits = crate::core_credits_fullstack::provider(state.clone());
    let wallets = wallets_provider(state.clone());
    let news = news_provider(state.clone());
    let developer = crate::fullstack_developer::provider(state.clone());
    let notifications = crate::fullstack_notifications::provider(state.clone());
    let payments = crate::fullstack_payments::provider(state.clone());
    let media = media_provider(state.clone());
    let escrow = escrow_provider(state.clone());
    let chat = crate::fullstack_chat::provider(state.clone());
    let auth = auth_provider(state.clone());
    let orders = crate::fullstack_orders::provider(state.clone());
    let catalog = crate::fullstack_catalog::provider(state.clone());
    let settings = crate::fullstack_settings::provider(state.clone());
    let audit = audit_provider(state.clone());
    let dashboard = dashboard_provider(state.clone());
    let provider_state = state.clone();
    let provider = AdminAnalyticsProvider(Arc::new(move |query, headers| {
        let state = provider_state.clone();
        Box::pin(async move { load(&state, query, headers).await })
    }));
    let fullstack = FullstackState::new(
        dioxus_server::ServeConfig::new(),
        epsx_dioxus_ui::app::AdminRoot,
    );
    let assets = std::env::var("DIOXUS_PUBLIC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .expect("executable path")
                .parent()
                .expect("executable directory")
                .join("public")
        });
    crate::build_app(state)
        .route("/_ui/admin.css", axum::routing::get(styles))
        .layer(Extension(fullstack))
        .layer(Extension(provider))
        .layer(Extension(audit))
        .layer(Extension(dashboard))
        .layer(Extension(settings))
        .layer(Extension(catalog))
        .layer(Extension(orders))
        .layer(Extension(auth))
        .layer(Extension(chat))
        .layer(Extension(escrow))
        .layer(Extension(media))
        .layer(Extension(developer))
        .layer(Extension(notifications))
        .layer(Extension(payments))
        .layer(Extension(news))
        .layer(Extension(wallets))
        .layer(Extension(core_wallets))
        .layer(Extension(core_credits))
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new(assets.join("assets")),
        )
        .nest_service(
            "/wasm",
            tower_http::services::ServeDir::new(assets.join("wasm")),
        )
}

async fn styles() -> impl axum::response::IntoResponse {
    // The hydrated shell owns theme state on its root, rather than mutating
    // the document element through a second browser runtime.
    let css = epsx_templates::DESIGN_SYSTEM_CSS
        .replace("html.dark", ":is(html.dark, .admin-app-shell.dark)");
    (
        [
            (http::header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (http::header::CACHE_CONTROL, "no-cache"),
        ],
        css,
    )
}

fn audit_provider(state: AppState) -> AdminAuditProvider {
    AdminAuditProvider(Arc::new(move |query, headers| {
        let state = state.clone();
        Box::pin(async move { load_audit(&state, query, headers).await })
    }))
}

async fn load_audit(
    state: &AppState,
    query: AuditQuery,
    headers: http::HeaderMap,
) -> Result<epsx_dioxus_ui::pages::admin_pages::audit_log::AdminAuditList, LoadError> {
    use crate::audit_log_adapter::{load_admin_audit, AdminAuditLoad, AdminAuditQuery};
    let query = AdminAuditQuery::from_raw(&query.encoded()).map_err(|_| LoadError::InvalidQuery)?;
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    match load_admin_audit(&state.analytics, &query, &context).await {
        AdminAuditLoad::Ready(value) | AdminAuditLoad::Empty(value) => Ok(value),
        AdminAuditLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminAuditLoad::Forbidden => Err(LoadError::Forbidden),
        AdminAuditLoad::Malformed => Err(LoadError::Malformed),
        AdminAuditLoad::Unavailable => Err(LoadError::Unavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn missing_session_cannot_read_either_provider() {
        let state = crate::routing_tests::test_state();
        assert_eq!(
            load(
                &state,
                AnalyticsQueryState {
                    page: 1,
                    ..Default::default()
                },
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
        assert_eq!(
            load_audit(&state, AuditQuery::default(), http::HeaderMap::new()).await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn malformed_audit_query_is_rejected_before_upstream_io() {
        let state = crate::routing_tests::test_state();
        let query = AuditQuery {
            category: Some("not-a-category".into()),
            cursor: None,
        };
        assert_eq!(
            load_audit(&state, query, http::HeaderMap::new()).await,
            Err(LoadError::InvalidQuery)
        );
    }
    #[tokio::test]
    async fn fullstack_audit_ssr_resolves_typed_provider_once_and_preserves_cursor() {
        use axum::{
            body::{to_bytes, Body},
            http::Request,
        };
        use epsx_dioxus_ui::pages::admin_pages::audit_log::{AdminAuditList, AdminAuditSummary};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tower::ServiceExt;
        let reads = Arc::new(AtomicUsize::new(0));
        let count = reads.clone();
        let provider = AdminAuditProvider(Arc::new(move |query, _| {
            count.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                assert_eq!(query.category.as_deref(), Some("system"));
                Ok(AdminAuditList {
                    items: vec![AdminAuditSummary {
                        id: "00000000-0000-0000-0000-000000000002".into(),
                        category: "system".into(),
                        action: "settings.updated".into(),
                        resource_type: "settings".into(),
                        effect: "success".into(),
                        occurred_at: "2026-07-22T12:00:00Z".into(),
                    }],
                    next_cursor: Some("next_page".into()),
                    has_more: true,
                })
            })
        }));
        let state = FullstackState::new(
            dioxus_server::ServeConfig::new(),
            epsx_dioxus_ui::app::AdminRoot,
        );
        let app = Router::new()
            .route(
                "/audit-log",
                axum::routing::get(FullstackState::render_handler),
            )
            .with_state(state)
            .layer(Extension(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/audit-log?category=system")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
        let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("settings.updated"), "{html}");
        assert!(html.contains("cursor=next_page"));
        if cfg!(debug_assertions) {
            assert!(
                html.contains("AdminAuditList"),
                "Server future hydration data missing"
            );
        }
        assert!(!html.contains("epsx_browser_runtime_bootstrap"));
        assert_eq!(reads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn both_admin_functions_use_explicit_scoped_paths() {
        let functions = dioxus_server::ServerFunction::collect();
        for path in ["/_server/admin/analytics", "/_server/admin/audit"] {
            assert!(
                functions.iter().any(|function| function.path() == path),
                "{path}"
            );
        }
    }
}

fn dashboard_provider(state: AppState) -> AdminDashboardProvider {
    AdminDashboardProvider(Arc::new(move |headers| {
        let state = state.clone();
        Box::pin(async move { load_dashboard(&state, headers).await })
    }))
}

async fn load_dashboard(
    state: &AppState,
    headers: http::HeaderMap,
) -> Result<DashboardData, LoadError> {
    use crate::dashboard_user_status_adapter::{
        load_admin_dashboard_user_status, AdminDashboardUserStatusLoad,
        AdminDashboardUserStatusQuery,
    };
    let Some((token, user)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let (user_status, overview) = tokio::join!(
        load_admin_dashboard_user_status(&state.identity, AdminDashboardUserStatusQuery, &context),
        load_admin_analytics(&state.identity, &context),
    );
    let user_status = match user_status {
        AdminDashboardUserStatusLoad::Ready(value) => Ok(value),
        AdminDashboardUserStatusLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminDashboardUserStatusLoad::Forbidden => Err(LoadError::Forbidden),
        AdminDashboardUserStatusLoad::Malformed => Err(LoadError::Malformed),
        AdminDashboardUserStatusLoad::Unavailable => Err(LoadError::Unavailable),
    };
    let overview = match overview {
        AdminAnalyticsLoad::Ready(value) | AdminAnalyticsLoad::Empty(value) => Ok(value),
        AdminAnalyticsLoad::Unauthorized => Err(LoadError::Unauthenticated),
        AdminAnalyticsLoad::Forbidden => Err(LoadError::Forbidden),
        AdminAnalyticsLoad::Malformed => Err(LoadError::Malformed),
        AdminAnalyticsLoad::Unavailable => Err(LoadError::Unavailable),
    };
    Ok(DashboardData {
        user: state.session().ui_user(user, None),
        user_status,
        overview,
    })
}

fn auth_provider(state: AppState) -> epsx_dioxus_ui::fullstack::admin_auth::AuthProvider {
    let session_state = state.clone();
    epsx_dioxus_ui::fullstack::admin_auth::AuthProvider {
        session: Arc::new(move |headers| {
            let state = session_state.clone();
            Box::pin(crate::auth_fullstack::session(state, headers))
        }),
        command: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(crate::auth_fullstack::command(state, command, headers))
        }),
    }
}

fn escrow_provider(state: AppState) -> epsx_dioxus_ui::fullstack::admin_escrow::EscrowProvider {
    let reads = state.clone();
    epsx_dioxus_ui::fullstack::admin_escrow::EscrowProvider {
        read: Arc::new(move |query, headers| {
            let state = reads.clone();
            Box::pin(crate::escrow_fullstack::read(state, query, headers))
        }),
        command: Arc::new(move |query, command, key, headers| {
            let state = state.clone();
            Box::pin(crate::escrow_fullstack::command(
                state, query, command, key, headers,
            ))
        }),
    }
}

fn media_provider(state: AppState) -> epsx_dioxus_ui::fullstack::admin_media::MediaProvider {
    let reads = state.clone();
    epsx_dioxus_ui::fullstack::admin_media::MediaProvider {
        read: Arc::new(move |bucket, headers| {
            let state = reads.clone();
            Box::pin(crate::media_fullstack::read(state, bucket, headers))
        }),
        mutate: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(crate::media_fullstack::mutate(state, command, headers))
        }),
    }
}

fn news_provider(state: AppState) -> epsx_dioxus_ui::fullstack::admin_news::NewsProvider {
    let reads = state.clone();
    epsx_dioxus_ui::fullstack::admin_news::NewsProvider {
        read: Arc::new(move |page, query, headers| {
            let state = reads.clone();
            Box::pin(crate::news_fullstack::read(state, page, query, headers))
        }),
        mutate: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(crate::news_fullstack::mutate(state, command, headers))
        }),
    }
}

fn wallets_provider(state: AppState) -> epsx_dioxus_ui::fullstack::admin_wallets::WalletProvider {
    let reads = state.clone();
    epsx_dioxus_ui::fullstack::admin_wallets::WalletProvider {
        read: Arc::new(move |query, headers| {
            let state = reads.clone();
            Box::pin(crate::wallets_fullstack::read(state, query, headers))
        }),
        command: Arc::new(move |command, key, headers| {
            let state = state.clone();
            Box::pin(crate::wallets_fullstack::command(
                state, command, key, headers,
            ))
        }),
    }
}

pub(crate) fn ranking_analytics_query(raw_query: &str) -> Result<String, ()> {
    if raw_query.is_empty() {
        return Ok(String::new());
    }
    let url =
        reqwest::Url::parse(&format!("https://admin.invalid/?{raw_query}")).map_err(|_| ())?;
    let mut seen = std::collections::HashSet::new();
    let mut normalized = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in url.query_pairs() {
        let key = key.as_ref();
        if !matches!(
            key,
            "page" | "limit" | "country" | "sector" | "sort_by" | "min_eps" | "min_growth"
        ) {
            continue;
        }
        if !seen.insert(key.to_string()) {
            return Err(());
        }
        match key {
            "page" => {
                let value = value.parse::<u32>().map_err(|_| ())?;
                if value == 0 || value > 1_000_000 {
                    return Err(());
                }
                normalized.append_pair(key, &value.to_string());
            }
            "limit" => {
                let value = value.parse::<u32>().map_err(|_| ())?;
                if value == 0 || value > 100 {
                    return Err(());
                }
                normalized.append_pair(key, &value.to_string());
            }
            "country" | "sector" => {
                if value.is_empty() {
                    continue;
                }
                if value.len() > 64 || value.chars().any(char::is_control) {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            "sort_by" => {
                if value.is_empty() || value.len() > 64 || value.chars().any(char::is_control) {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            "min_eps" | "min_growth" => {
                let number = value.parse::<f64>().map_err(|_| ())?;
                if !number.is_finite() {
                    return Err(());
                }
                normalized.append_pair(key, &value);
            }
            _ => unreachable!(),
        }
    }
    Ok(normalized.finish())
}

#[cfg(test)]
mod cutover_tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    #[tokio::test]
    async fn final_ui_dispatch_preserves_aliases_errors_and_redirects() {
        for (path, status, expected) in [
            ("/admin/settings", StatusCode::OK, "Settings"),
            (
                "/admin/unauthorized",
                StatusCode::FORBIDDEN,
                "Access denied",
            ),
            (
                "/missing-admin-fixture",
                StatusCode::NOT_FOUND,
                "Page not found",
            ),
        ] {
            let response = application(crate::routing_tests::test_state())
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), status, "{path}");
            let html = String::from_utf8(
                to_bytes(response.into_body(), 2_000_000)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            assert!(html.contains(expected), "{path}");
            assert!(!html.contains("epsx_browser_runtime_bootstrap"), "{path}");
        }
        let response = application(crate::routing_tests::test_state())
            .oneshot(
                Request::builder()
                    .uri("/admin/notifications")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers()[http::header::LOCATION],
            "/notifications/manage"
        );
    }
}
