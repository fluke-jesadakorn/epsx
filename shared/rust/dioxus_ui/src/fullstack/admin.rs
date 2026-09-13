//! Typed admin analytics, authenticated by its own native BFF provider.
use super::analytics::{AnalyticsData, AnalyticsNavigation};
use super::LoadError;
use crate::pages::analytics::AnalyticsQueryState;
use dioxus::prelude::*;

/// No token, cookie, or upstream URL is serialized into the hydrated payload.
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AdminAnalyticsProvider(pub AdminAnalyticsProviderCallback);

#[server(prefix = "/_server/admin", endpoint = "analytics")]
pub async fn read_admin_analytics(
    query: AnalyticsQueryState,
) -> Result<Result<AnalyticsData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AdminAnalyticsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Analytics provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}

fn parse_query(query: &str) -> Result<AnalyticsQueryState, LoadError> {
    AnalyticsQueryState::from_normalized_query(query).map_err(|_| LoadError::InvalidQuery)
}

async fn load(query: &str) -> Result<AnalyticsData, LoadError> {
    read_admin_analytics(parse_query(query)?)
        .await
        .map_err(|_| LoadError::Unavailable)?
}

#[component]
pub fn HydratedAdminAnalytics(query: ReadSignal<String>) -> Element {
    // Only the initial read participates in SSR hydration. Later reads keep the
    // last successful data visible while the requested query is pending.
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move { load(&query).await }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut loaded_query = use_signal(|| initial_query);
    let mut retry = use_signal(|| 0_u64);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| AnalyticsNavigation(navigate));
    use_effect(move || {
        let requested = query();
        let retry_count = retry();
        if requested == *loaded_query.peek() && retry_count == 0 {
            return;
        }
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = load(&requested).await.and_then(|value| {
                value.rankings.as_ref().map_err(Clone::clone)?;
                Ok(value)
            });
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(value) => {
                    loaded_query.set(requested);
                    data.set(Some(value));
                }
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                    if data.peek().is_some() {
                        navigator.replace(format!("/analytics?{}", loaded_query.peek()));
                    }
                }
            }
        });
    });
    rsx! {
        AdminAnalyticsShell { authenticated: data().is_some(),
        section { "data-dioxus-analytics": "true", aria_busy: pending(),
            if pending() { p { role: "status", class: "fe-purchase-note", "Updating results…" } }
            if let Some(failure) = error() {
                div { role: "status", class: "fe-purchase-note",
                    p { "{failure.message()}" }
                    button { r#type: "button", class: "fe-button", onclick: move |_| { let value = *retry.peek() + 1; retry.set(value); }, "Try again" }
                }
            }
            if let Some(snapshot) = data() {
                match snapshot.rankings {
                    Ok(response) => rsx! { crate::pages::analytics::AnalyticsPage {
                        enterprise: false, signed_in: snapshot.signed_in,
                        response, filters: snapshot.filters.clone().ok(),
                        filters_state: (if snapshot.filters.is_ok() { "ready" } else { "unavailable" }).to_string(),
                        query: snapshot.query,
                        watchlist: snapshot.watchlist.clone().ok().flatten(),
                        watchlist_state: (if !snapshot.signed_in { "signed_out" } else if snapshot.watchlist.is_ok() { "ready" } else { "unavailable" }).to_string(),
                    } },
                    Err(failure) => rsx! { p { role: "status", "{failure.message()}" } },
                }
            }
        }
    }
    }
}

/// The migrated route owns its chrome events; other destinations retain their
/// working document links until their typed page providers are ready.
#[component]
pub fn AdminAnalyticsShell(
    authenticated: bool,
    #[props(default = "/analytics".to_string())] current_path: String,
    #[props(default = "Analytics".to_string())] title: String,
    children: Element,
) -> Element {
    let navigator = use_navigator();
    let existing = try_consume_context::<AdminNavigation>();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| existing.unwrap_or(AdminNavigation(navigate)));
    let fallback_dark = use_signal(|| true);
    let mut dark = try_consume_context::<AdminTheme>()
        .map(|v| v.0)
        .unwrap_or(fallback_dark);
    let mut drawer = use_signal(|| false);
    let mut account = use_signal(|| false);
    rsx! {
        document::Title { "{title} | EPSX Admin" }
        document::Meta { name: "description", content: "EPSX administrator analytics workspace" }
        document::Link { rel: "stylesheet", href: "/public/dist/tailwind.css" }
        document::Link { rel: "stylesheet", href: "/_ui/admin.css" }
        div { onmounted: move |_| {spawn(async move {if let Ok(value)=document::eval("let theme=true; try {theme=localStorage.getItem('epsx-theme')!=='light';} catch (_) {} dioxus.send(theme);").recv::<bool>().await {dark.set(value);}});}, class: if dark() { "dark admin-app-shell flex h-screen w-full overflow-hidden bg-background text-foreground" } else { "admin-app-shell flex h-screen w-full overflow-hidden bg-background text-foreground" },
            div { class: if drawer() { "fixed inset-y-0 left-0 z-50 md:static" } else { "hidden md:block" },
                crate::layout::sidebar::AdminSidebar { current_path: current_path.clone(), is_authenticated: authenticated }
            }
            if drawer() {
                button { class: "fixed inset-0 z-40 bg-black/50 md:hidden", aria_label: "Close navigation", onclick: move |_| drawer.set(false) }
            }
            div { class: "flex flex-1 flex-col h-full min-w-0 overflow-hidden",
                header { class: "sticky top-0 z-30 border-b border-border/40 bg-card admin-header admin-header-chrome",
                    div { class: "flex h-16 w-full items-center justify-between px-6 gap-3",
                        div { class: "flex items-center gap-2",
                            button { class: "btn btn-ghost btn-icon md:hidden", aria_label: "Toggle navigation", onclick: move |_| drawer.toggle(), crate::primitives::Icon { name: "menu", size: 18 } }
                            crate::layout::Breadcrumb { current_path: current_path.clone() }
                        }
                        div { class: "flex items-center gap-3",
                            a { class: "btn btn-ghost btn-icon", href: "/notifications/manage", onclick: move |event| follow_admin_link(event,Some(AdminNavigation(navigate)),"/notifications/manage"), aria_label: "Notifications", crate::primitives::Icon { name: "bell", size: 18 } }
                            button { class: "btn btn-ghost btn-icon", aria_label: "Toggle theme", onclick: move |_| {dark.toggle();let value=dark();spawn(async move {let _=document::eval(if value {"try {localStorage.setItem('epsx-theme','dark')} catch (_) {}"}else{"try {localStorage.setItem('epsx-theme','light')} catch (_) {}"});});}, crate::primitives::Icon { name: if dark() { "sun" } else { "moon" }, size: 18 } }
                            div { class: "relative",
                                button { class: "btn btn-ghost", aria_expanded: account(), onclick: move |_| account.toggle(), "Your account" }
                                if account() {
                                    div { class: "absolute right-0 mt-2 rounded-xl border border-border bg-card p-3 shadow-xl z-50",
                                        a { class: "block p-2", href: "/settings", onclick: move |event| follow_admin_link(event,Some(AdminNavigation(navigate)),"/settings"), "Settings" }
                                        super::admin_auth::AdminLogoutButton { class: "block p-2" }
                                    }
                                }
                            }
                        }
                    }
                }
                main { id: "epsx-main-content", class: "flex-1 overflow-y-auto overflow-x-hidden", {children} }
                crate::layout::footer::AdminFooter {}
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct AdminTheme(pub Signal<bool>);

#[derive(Clone, Copy)]
pub struct AdminNavigation(pub EventHandler<String>);

pub fn follow_admin_link(event: MouseEvent, navigation: Option<AdminNavigation>, url: &str) {
    if let Some(navigation) = navigation {
        if event.modifiers().is_empty() {
            event.prevent_default();
            navigation.0.call(url.to_string());
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditQuery {
    pub category: Option<String>,
    pub cursor: Option<String>,
}

impl AuditQuery {
    pub fn parse(raw: &str) -> Result<Self, LoadError> {
        let mut result = Self::default();
        for (name, value) in url::form_urlencoded::parse(raw.as_bytes()) {
            match name.as_ref() {
                "category" if result.category.is_none() => {
                    result.category = Some(value.into_owned())
                }
                "cursor" if result.cursor.is_none() => result.cursor = Some(value.into_owned()),
                _ => return Err(LoadError::InvalidQuery),
            }
        }
        Ok(result)
    }
    pub fn encoded(&self) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if let Some(value) = &self.category {
            serializer.append_pair("category", value);
        }
        if let Some(value) = &self.cursor {
            serializer.append_pair("cursor", value);
        }
        serializer.finish()
    }
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AdminAuditProvider(pub AdminAuditProviderCallback);

#[server(prefix = "/_server/admin", endpoint = "audit")]
pub async fn read_admin_audit(
    query: AuditQuery,
) -> Result<Result<crate::pages::admin_pages::audit_log::AdminAuditList, LoadError>, ServerFnError>
{
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AdminAuditProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Audit provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}

#[component]
pub fn HydratedAdminAudit(query: ReadSignal<String>) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move { load_audit(&query).await }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone());
    let mut loaded_query = use_signal(|| initial_query);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(|| None::<LoadError>);
    let mut generation = use_signal(|| 0u64);
    let mut retry = use_signal(|| 0u64);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        if url == format!("/audit-log?{}", loaded_query.peek())
            || (url == "/audit-log" && loaded_query.peek().is_empty())
        {
            let next = *retry.peek() + 1;
            retry.set(next);
        } else {
            navigator.push(url);
        }
    });
    use_context_provider(|| AdminNavigation(navigate));
    use_effect(move || {
        let requested = query();
        let retry_count = retry();
        if requested == *loaded_query.peek() && retry_count == 0 {
            return;
        }
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = load_audit(&requested).await;
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(value) => {
                    loaded_query.set(requested);
                    data.set(Ok(value));
                }
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(Err(failure.clone()));
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    let location = AuditQuery::parse(&loaded_query()).unwrap_or_default();
    rsx! {
        AdminAnalyticsShell { authenticated: data().is_ok(), current_path: "/audit-log", title: "Audit log",
            section { aria_busy: pending(),
                if pending() { p { role: "status", "Updating audit records…" } }
                if let Some(failure) = error() {
                    div { role: "status", p { "{failure.message()}" }
                        button { class: "btn btn-primary", onclick: move |_| { let next = *retry.peek() + 1; retry.set(next); }, "Try again" }
                    }
                }
                crate::pages::admin_pages::audit_log::HydratedAuditBody { data: data(), category: location.category, cursor: location.cursor }
            }
        }
    }
}

async fn load_audit(
    raw: &str,
) -> Result<crate::pages::admin_pages::audit_log::AdminAuditList, LoadError> {
    read_admin_audit(AuditQuery::parse(raw)?)
        .await
        .map_err(|_| LoadError::Unavailable)?
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub user: crate::auth::User,
    pub user_status:
        Result<crate::pages::admin_pages::dashboard::AdminDashboardUserStatus, LoadError>,
    pub overview: Result<crate::pages::admin_pages::analytics::AdminAnalyticsSnapshot, LoadError>,
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AdminDashboardProvider(pub AdminDashboardProviderCallback);

#[server(prefix = "/_server/admin", endpoint = "dashboard")]
pub async fn read_admin_dashboard() -> Result<Result<DashboardData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AdminDashboardProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Dashboard provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(headers).await)
}

#[component]
pub fn HydratedAdminDashboard() -> Element {
    let initial = use_server_future(|| async {
        read_admin_dashboard()
            .await
            .map_err(|_| LoadError::Unavailable)
            .and_then(|value| value)
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut pending = use_signal(|| false);
    rsx! {
        AdminAnalyticsShell { authenticated: data().is_ok(), current_path: "/", title: "Command Center",
            section { aria_busy: pending(),
                button { class: "btn btn-sm btn-outline m-4", disabled: pending(), onclick: move |_| {
                    pending.set(true);
                    spawn(async move {
                        data.set(read_admin_dashboard().await.map_err(|_| LoadError::Unavailable).and_then(|value| value));
                        pending.set(false);
                    });
                }, if pending() { "Refreshing…" } else { "Refresh dashboard" } }
                match data() {
                    Ok(snapshot) => rsx! { crate::pages::admin_pages::dashboard::HydratedDashboardBody { data: snapshot } },
                    Err(failure) => rsx! { div { class: "p-6", role: "status", p { "{failure.message()}" } a { class: "btn btn-primary", href: "/auth", "Sign in" } } },
                }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type AdminAnalyticsProviderCallback = std::sync::Arc<
    dyn Fn(
            AnalyticsQueryState,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<AnalyticsData, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type AdminAuditProviderCallback = std::sync::Arc<
    dyn Fn(
            AuditQuery,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<
                            crate::pages::admin_pages::audit_log::AdminAuditList,
                            LoadError,
                        >,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type AdminDashboardProviderCallback = std::sync::Arc<
    dyn Fn(
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<DashboardData, LoadError>> + Send>,
        > + Send
        + Sync,
>;
