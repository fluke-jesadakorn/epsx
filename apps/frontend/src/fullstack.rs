//! Native providers preserve BFF authentication, authorization and validation.
use crate::AppState;
use axum::{
    extract::{Request, State},
    response::{IntoResponse, Redirect, Response},
    Extension, Router,
};
use dioxus_server::FullstackState;
use epsx_dioxus_ui::fullstack::frontend_auth::AuthProvider;
use epsx_dioxus_ui::fullstack::frontend_payment::PaymentProvider;
use epsx_dioxus_ui::fullstack::shell::LogoutProvider;
use epsx_dioxus_ui::pages::account::{
    hydrated::{AccountMutationProvider, AccountProvider},
    push::PushProvider,
};
use epsx_dioxus_ui::pages::developer::hydrated::{
    DeveloperDocsProvider, DeveloperMutationProvider, DeveloperProvider,
};
use epsx_dioxus_ui::pages::home::HomeProvider;
use epsx_dioxus_ui::pages::notifications::hydrated::{
    NotificationMutationProvider, NotificationsProvider,
};
use epsx_dioxus_ui::pages::portfolio::hydrated::{WatchlistMutationProvider, WatchlistProvider};
use epsx_dioxus_ui::pages::{
    account_credits::CreditsProvider, news::hydrated::NewsProvider,
    news_detail::NewsDetailProvider, plans::PlansProvider, profile::ProfileProvider,
};
use epsx_dioxus_ui::{
    fullstack::{analytics::AnalyticsProvider, Surface},
    payment::purchases::PurchasesProvider,
};
use std::sync::Arc;

#[derive(Clone)]
struct Providers {
    chat: epsx_dioxus_ui::pages::chat::hydrated::ChatProvider,
    chat_mutations: epsx_dioxus_ui::pages::chat::hydrated::ChatMutationProvider,
    watchlist: WatchlistProvider,
    watchlist_mutations: WatchlistMutationProvider,
    payment: PaymentProvider,
    developer: DeveloperProvider,
    developer_mutations: DeveloperMutationProvider,
    developer_docs: DeveloperDocsProvider,
    auth: AuthProvider,
    home: HomeProvider,
    notifications: NotificationsProvider,
    notification_mutations: NotificationMutationProvider,
    account: AccountProvider,
    preferences: AccountMutationProvider,
    push: PushProvider,
    logout: LogoutProvider,
    analytics: AnalyticsProvider,
    purchases: PurchasesProvider,
    news: NewsProvider,
    news_detail: NewsDetailProvider,
    plans: PlansProvider,
    credits: CreditsProvider,
    profile: ProfileProvider,
}
impl Providers {
    fn attach(&self, router: Router) -> Router {
        router
            .layer(Extension(self.chat.clone()))
            .layer(Extension(self.chat_mutations.clone()))
            .layer(Extension(self.watchlist.clone()))
            .layer(Extension(self.watchlist_mutations.clone()))
            .layer(Extension(self.payment.clone()))
            .layer(Extension(self.developer.clone()))
            .layer(Extension(self.developer_mutations.clone()))
            .layer(Extension(self.developer_docs.clone()))
            .layer(Extension(self.auth.clone()))
            .layer(Extension(self.home.clone()))
            .layer(Extension(self.notifications.clone()))
            .layer(Extension(self.notification_mutations.clone()))
            .layer(Extension(self.account.clone()))
            .layer(Extension(self.preferences.clone()))
            .layer(Extension(self.push.clone()))
            .layer(Extension(self.logout.clone()))
            .layer(Extension(self.analytics.clone()))
            .layer(Extension(self.purchases.clone()))
            .layer(Extension(self.credits.clone()))
            .layer(Extension(self.profile.clone()))
            .layer(Extension(self.news.clone()))
            .layer(Extension(self.news_detail.clone()))
            .layer(Extension(self.plans.clone()))
    }
}

fn providers(state: AppState) -> Providers {
    let chat_state = state.clone();
    let chat = epsx_dioxus_ui::pages::chat::hydrated::ChatProvider(Arc::new(move |id, headers| {
        let state = chat_state.clone();
        Box::pin(async move { crate::chat_fullstack::load(state, id, headers).await })
    }));
    let chat_mutation_state = state.clone();
    let chat_mutations = epsx_dioxus_ui::pages::chat::hydrated::ChatMutationProvider(Arc::new(
        move |command, headers| {
            let state = chat_mutation_state.clone();
            Box::pin(async move { crate::chat_fullstack::mutate(state, command, headers).await })
        },
    ));
    let watchlist_state = state.clone();
    let watchlist = WatchlistProvider(Arc::new(move |headers| {
        let state = watchlist_state.clone();
        Box::pin(async move { crate::watchlist_fullstack::load(state, headers).await })
    }));
    let watchlist_mutation_state = state.clone();
    let watchlist_mutations = WatchlistMutationProvider(Arc::new(move |command, headers| {
        let state = watchlist_mutation_state.clone();
        Box::pin(async move { crate::watchlist_fullstack::mutate(state, command, headers).await })
    }));
    let payment_state = state.clone();
    let payment_command_state = state.clone();
    let payment = PaymentProvider {
        read: Arc::new(move |id, headers| {
            let state = payment_state.clone();
            Box::pin(async move { crate::payment_fullstack::read(state, id, headers).await })
        }),
        command: Arc::new(move |command, headers| {
            let state = payment_command_state.clone();
            Box::pin(
                async move { crate::payment_fullstack::command(state, command, headers).await },
            )
        }),
    };
    let developer_state = state.clone();
    let developer = DeveloperProvider(Arc::new(move |days, headers| {
        let state = developer_state.clone();
        Box::pin(async move { crate::developer_fullstack::load(state, days, headers).await })
    }));
    let developer_mutation_state = state.clone();
    let developer_mutations = DeveloperMutationProvider(Arc::new(move |command, headers| {
        let state = developer_mutation_state.clone();
        Box::pin(async move { crate::developer_fullstack::mutate(state, command, headers).await })
    }));
    let developer_docs_state = state.clone();
    let developer_docs = DeveloperDocsProvider(Arc::new(move || {
        let state = developer_docs_state.clone();
        Box::pin(async move { crate::developer_fullstack::docs(state).await })
    }));
    let session_state = state.clone();
    let auth_state = state.clone();
    let auth = AuthProvider {
        session: Arc::new(move |headers| {
            let state = session_state.clone();
            Box::pin(async move { crate::auth_fullstack::session(state, headers).await })
        }),
        command: Arc::new(move |command, headers| {
            let state = auth_state.clone();
            Box::pin(async move { crate::auth_fullstack::command(state, command, headers).await })
        }),
    };
    let home_state = state.clone();
    let home = HomeProvider(Arc::new(move || {
        let state = home_state.clone();
        Box::pin(async move { crate::home_fullstack::load(state).await })
    }));
    let notifications_state = state.clone();
    let notifications = NotificationsProvider(Arc::new(move |query, headers| {
        let state = notifications_state.clone();
        Box::pin(async move { crate::notifications_fullstack::load(state, query, headers).await })
    }));
    let notification_mutations_state = state.clone();
    let notification_mutations = NotificationMutationProvider(Arc::new(move |command, headers| {
        let state = notification_mutations_state.clone();
        Box::pin(
            async move { crate::notifications_fullstack::mutate(state, command, headers).await },
        )
    }));
    let account_state = state.clone();
    let account = AccountProvider(Arc::new(move |headers| {
        let state = account_state.clone();
        Box::pin(async move { crate::account_fullstack::load(state, headers).await })
    }));
    let preferences_state = state.clone();
    let preferences = AccountMutationProvider(Arc::new(move |input, headers| {
        let state = preferences_state.clone();
        Box::pin(async move { crate::account_fullstack::save(state, input, headers).await })
    }));
    let push_state = state.clone();
    let push = PushProvider(Arc::new(move |command, headers| {
        let state = push_state.clone();
        Box::pin(async move { crate::push_fullstack::run(state, command, headers).await })
    }));
    let logout_state = state.clone();
    let logout = LogoutProvider(Arc::new(move |headers| {
        let state = logout_state.clone();
        Box::pin(async move { crate::account_fullstack::logout(state, headers).await })
    }));
    let analytics_state = state.clone();
    let analytics = AnalyticsProvider(Arc::new(move |query, headers| {
        let state = analytics_state.clone();
        Box::pin(async move { crate::ssr::load_fullstack_analytics(&state, query, headers).await })
    }));
    let purchase_state = state.clone();
    let purchases = PurchasesProvider(Arc::new(move |query, headers| {
        let state = purchase_state.clone();
        Box::pin(async move { crate::pay_orders::load_fullstack(state, query, headers).await })
    }));
    let news_state = state.clone();
    let news = NewsProvider(Arc::new(move |query| {
        let state = news_state.clone();
        Box::pin(async move { crate::news_fullstack::load(state, query).await })
    }));
    let detail_state = state.clone();
    let news_detail = NewsDetailProvider(Arc::new(move |slug| {
        let state = detail_state.clone();
        Box::pin(async move { crate::news_fullstack::detail(state, slug).await })
    }));
    let credits_state = state.clone();
    let credits = CreditsProvider(Arc::new(move |headers| {
        let state = credits_state.clone();
        Box::pin(async move { crate::credits_fullstack::load(state, headers).await })
    }));
    let profile_state = state.clone();
    let profile = ProfileProvider(Arc::new(move |headers| {
        let state = profile_state.clone();
        Box::pin(async move { crate::credits_fullstack::profile(state, headers).await })
    }));
    let plans = PlansProvider(Arc::new(move || {
        let state = state.clone();
        Box::pin(async move { crate::news_fullstack::plans(state).await })
    }));
    Providers {
        chat,
        chat_mutations,
        watchlist,
        watchlist_mutations,
        payment,
        developer,
        developer_mutations,
        developer_docs,
        auth,
        home,
        notifications,
        notification_mutations,
        account,
        preferences,
        push,
        logout,
        analytics,
        purchases,
        news,
        news_detail,
        plans,
        credits,
        profile,
    }
}

pub fn server_functions(state: AppState) -> Router {
    providers(state).attach(epsx_bff::fullstack::server_functions(
        Surface::Frontend,
        FullstackState::headless(),
    ))
}

/// The native frontend and DX server share this typed SSR/hydration application.
/// Legacy HTML producers are retained only for regression fixtures.
pub fn application(state: AppState) -> Router {
    let providers = providers(state.clone());
    let fullstack = FullstackState::new(
        dioxus_server::ServeConfig::new(),
        epsx_dioxus_ui::app::FrontendRoot,
    );
    let pages = Router::new()
        .route("/", axum::routing::get(FullstackState::render_handler))
        .route("/chat", axum::routing::get(FullstackState::render_handler))
        .route("/auth", axum::routing::get(FullstackState::render_handler))
        .route("/offline", axum::routing::get(offline_page))
        .route("/index", axum::routing::get(FullstackState::render_handler))
        .route(
            "/analytics",
            axum::routing::get(FullstackState::render_handler),
        )
        .route("/news", axum::routing::get(FullstackState::render_handler))
        .route(
            "/news/{slug}",
            axum::routing::get(FullstackState::render_handler),
        )
        .route("/plans", axum::routing::get(FullstackState::render_handler))
        .route(
            "/developer",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/developer/usage",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/developer/docs",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/access-denied",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/dashboard",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/permissions",
            axum::routing::get(FullstackState::render_handler),
        )
        .route("/about", axum::routing::get(FullstackState::render_handler))
        .route(
            "/contact",
            axum::routing::get(FullstackState::render_handler),
        )
        .route(
            "/privacy",
            axum::routing::get(FullstackState::render_handler),
        )
        .route("/terms", axum::routing::get(FullstackState::render_handler))
        .fallback(frontend_fallback)
        .with_state(fullstack.clone())
        .merge(
            Router::new()
                .route("/chat/history", axum::routing::get(private_page))
                .route("/chat/{id}", axum::routing::get(private_page))
                .route("/portfolio", axum::routing::get(private_page))
                .route("/payment", axum::routing::get(private_page))
                .route("/payment/{ptype}/{id}", axum::routing::get(private_page))
                .route("/notifications", axum::routing::get(private_page))
                .route("/account", axum::routing::get(private_page))
                .route("/account/credits", axum::routing::get(private_page))
                .route("/profile", axum::routing::get(private_page))
                .route("/account/payments", axum::routing::get(private_page))
                .route("/account/payments/{id}", axum::routing::get(private_page))
                .with_state((state.clone(), fullstack)),
        )
        .layer(axum::middleware::from_fn(
            epsx_bff::middleware::security_headers,
        ))
        .layer(axum::middleware::map_response(
            |mut response: Response| async move {
                if !response.headers().contains_key("x-epsx-public-cache") {
                    response
                        .headers_mut()
                        .insert("cache-control", "no-store".parse().unwrap());
                }
                response
            },
        ));
    let assets = std::env::var("DIOXUS_PUBLIC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .expect("executable path")
                .parent()
                .expect("executable directory")
                .join("public")
        });
    crate::build_app_with_legacy_pages(state, false)
        .merge(providers.attach(pages))
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new(assets.join("assets")),
        )
        .nest_service(
            "/wasm",
            tower_http::services::ServeDir::new(assets.join("wasm")),
        )
}

async fn private_page(
    State((state, fullstack)): State<(AppState, FullstackState)>,
    request: Request,
) -> Response {
    let path = request.uri().path();
    if path
        .strip_prefix("/account/payments/")
        .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
    {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    if path.starts_with("/account/payments")
        && epsx_dioxus_ui::routes::purchase_route_query(path, request.uri().query().unwrap_or(""))
            .is_err()
    {
        return axum::http::StatusCode::BAD_REQUEST.into_response();
    }
    if state
        .session()
        .verified_access_token(request.headers())
        .await
        .is_none()
    {
        let next = url::form_urlencoded::byte_serialize(request.uri().to_string().as_bytes())
            .collect::<String>();
        return Redirect::to(&format!("/auth?return_url={next}")).into_response();
    }
    let mut response = FullstackState::render_handler(State(fullstack), request).await;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}

async fn offline_page(State(fullstack): State<FullstackState>, mut request: Request) -> Response {
    // This is the sole public document cached by the recovery worker. Strip
    // credentials as defense in depth; the offline shell also skips providers.
    request.headers_mut().remove(axum::http::header::COOKIE);
    request
        .headers_mut()
        .remove(axum::http::header::AUTHORIZATION);
    let mut response = FullstackState::render_handler(State(fullstack), request).await;
    response
        .headers_mut()
        .insert("cache-control", "public, max-age=300".parse().unwrap());
    response
        .headers_mut()
        .insert("x-epsx-public-cache", "offline-shell-v1".parse().unwrap());
    response.headers_mut().remove(axum::http::header::VARY);
    response
}

/// The live frontend has no legacy HTML renderer fallback.
async fn frontend_fallback(State(fullstack): State<FullstackState>, request: Request) -> Response {
    use axum::http::{Method, StatusCode};
    let path = request.uri().path();
    if crate::is_api_path(path) {
        return crate::api_not_found_response();
    }
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if path == "/pricing" {
        let suffix = request
            .uri()
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        return Redirect::temporary(&format!("/plans{suffix}")).into_response();
    }
    if path == "/manual" {
        return Redirect::temporary("/analytics").into_response();
    }
    if path
        .strip_prefix("/portfolio/")
        .is_some_and(|address| !address.is_empty() && !address.contains('/'))
    {
        return Redirect::temporary("/portfolio").into_response();
    }
    FullstackState::render_handler(State(fullstack), request).await
}
