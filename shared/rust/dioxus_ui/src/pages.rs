//! Page components — one rsx! function per Next.js route.

use dioxus::prelude::*;
use crate::auth::User;
use crate::auth::wallet_button::ConnectedWalletState;
use crate::i18n::t;

pub mod home;
pub mod auth_page;
pub mod dashboard;
pub mod profile;
pub mod account;
pub mod analytics;
pub mod chat;
pub mod contact;
pub mod about;
pub mod news;
pub mod notifications;
pub mod payment;
pub mod permissions;
pub mod plans;
pub mod portfolio;
pub mod portfolio_address;
pub mod developer;
pub mod manual;
pub mod news_detail;
pub mod chat_conversation;
pub mod access_denied;
pub mod not_found;
pub mod error_page;
pub mod offline;
pub mod privacy;
pub mod terms;
pub mod account_credits;
pub mod chat_history;
pub mod admin_pages;

pub use home::render as Home;
pub use auth_page::render as AuthPage;
pub use dashboard::render as Dashboard;
pub use profile::render as Profile;
pub use account::render as Account;
pub use analytics::render as Analytics;
pub use chat::render as ChatInbox;
pub use chat_conversation::render as ChatConversation;
pub use chat_history::render as ChatHistory;
pub use contact::render as Contact;
pub use about::render as About;
pub use news::render as NewsList;
pub use news_detail::render as NewsDetail;
pub use notifications::render as Notifications;
pub use payment::render as Payment;
pub use permissions::render as Permissions;
pub use plans::render as Plans;
pub use portfolio::render as Portfolio;
pub use developer::render_overview as Developer;
pub use developer::render_usage as DeveloperUsage;
pub use developer::render_docs as DeveloperDocs;
pub use manual::render as Manual;
pub use access_denied::render as AccessDeniedPage;
pub use not_found::render as NotFound;
pub use error_page::render as ErrorPage;
pub use offline::render as Offline;
pub use privacy::render as Privacy;
pub use terms::render as Terms;
pub use account_credits::render as AccountCredits;
pub use admin_pages::*;

/// Common page context passed to every page rsx! function.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PageContext {
    pub user: Option<User>,
    pub path: String,
    pub query: String,
    pub params: std::collections::HashMap<String, String>,
    pub api_url: String,
    pub demo_login_enabled: bool,
    /// Wave 3a Track B — server-side wallet state plumbed from the BFF.
    /// The BFF reads the `WalletInfo` cookie (or defaults) via
    /// `ConnectedWalletState::from_cookies(&headers)` and forwards the
    /// resulting state to the layout / connect-button cluster. Defaults
    /// to `Default::default()` so BFFs that don't yet plumb the cookie
    /// (admin, pay, preview) still compile.
    pub wallet: ConnectedWalletState,
}

impl PageContext {
    pub fn param(&self, key: &str) -> Option<&String> { self.params.get(key) }
    pub fn query_param(&self, key: &str) -> Option<String> {
        let key_eq = format!("{}=", key);
        for pair in self.query.split('&') {
            if let Some(rest) = pair.strip_prefix(&key_eq) {
                return Some(rest.to_string());
            }
        }
        None
    }
    pub fn is_authed(&self) -> bool { self.user.is_some() }
}

pub struct PageMeta {
    pub title: String,
    pub description: String,
    /// Optional class appended to the `<body>` element by the BFF
    /// page shell (see `epsx_templates::page_shell_with_body_class`).
    ///
    /// **Wave 38c T1** — changed from `String` to `Option<String>`
    /// so the body class can be scoped per-route. The Wave 38b T2
    /// change set a prod-EXACT body class on `PageMeta::admin()`
    /// ("h-screen bg-background text-foreground overflow-hidden
    /// font-sans") to make the 3 admin outliers' centered
    /// "Access Denied" panel render correctly. But that body class
    /// applied to all 22 admin routes and regressed 6 of them
    /// (~88.5% → ~75.5% match) because their header+sidebar flex
    /// layout depends on the default `min-h-screen` flow.
    ///
    /// Now `PageMeta::admin()` returns `body_class: None` (no
    /// override — the page shell falls through to the bare
    /// `min-h-screen` default that the 22 admin routes need). The
    /// 3 outliers use `PageMeta::admin_with_body_class(...)` to
    /// keep the prod-EXACT body class ONLY on those 3 routes.
    pub body_class: Option<String>,
    pub include_footer: bool,
    pub use_epsx_header: bool,
}

impl PageMeta {
    pub fn marketing(title: &str) -> Self {
        Self {
            title: format!("{} — EPSX", title),
            description: "EPSX — Web3 commerce platform: visual page builder, on-chain payments, programmable subscriptions.".to_string(),
            body_class: Some("page-bg".to_string()),
            include_footer: true,
            use_epsx_header: true,
        }
    }
    pub fn app(title: &str) -> Self {
        Self {
            title: format!("{} — EPSX", title),
            description: "EPSX".to_string(),
            // Wave 49 T2 (Plan 13) — switch app pages to the
            // `page-bg-app` body class so they render with prod's
            // purple/magenta radial-glow background gradient
            // instead of plain dark. Sampled prod corners:
            // /account #13182b → #401c68 → #412148 (top-left →
            // center → bot-right) — purple/magenta hues. Without
            // this, dev renders plain dark (warm-neutral #171717)
            // which diverges from prod by 93%.
            body_class: Some("page-bg-app".to_string()),
            include_footer: false,
            use_epsx_header: false,
        }
    }
    /// Default admin meta — **no** body class override.
    ///
    /// Wave 38c T1 — reverts the Wave 38b T2 global body-class
    /// change. The 22 admin routes depend on the page shell's
    /// default `min-h-screen` flow for their header+sidebar
    /// layout. Setting `h-screen overflow-hidden` (the prod-EXACT
    /// body class) on the body element collapsed the document
    /// height and broke the flex-flow that positions the sidebar
    /// + main content.
    ///
    /// For the 3 outlier routes that DO need the prod-EXACT
    /// body class (`/access-denied`, `/unauthorized`,
    /// `/developer-portal/api-keys/create`), use
    /// [`PageMeta::admin_with_body_class`] instead.
    pub fn admin(title: &str) -> Self {
        Self {
            title: format!("{} — Admin", title),
            description: "EPSX Admin".to_string(),
            body_class: None,
            include_footer: false,
            use_epsx_header: false,
        }
    }
    /// Admin meta with an explicit body-class override.
    ///
    /// **Wave 38c T1** — for the 3 admin outlier routes that
    /// render the centered "Access Denied" panel (the panel needs
    /// `flex h-screen flex-col` on its outer wrapper which only
    /// positions correctly when the body itself is
    /// `h-screen overflow-hidden`). Pass the prod-EXACT body class
    /// string from `access_denied_panel::render` — kept narrow
    /// to ONLY the 3 outlier paths so the 22 other admin routes
    /// are not affected.
    ///
    /// Body class mirrors prod's
    /// `__variable_a460b5 h-screen bg-background text-foreground
    /// overflow-hidden font-sans`. The `__variable_a460b5` is the
    /// Next.js font-variable wrapper (CSS-var font family); the
    /// Tailwind v4 BFF dev uses `font-sans` directly.
    pub fn admin_with_body_class(title: &str, body_class: impl Into<String>) -> Self {
        Self {
            title: format!("{} — Admin", title),
            description: "EPSX Admin".to_string(),
            body_class: Some(body_class.into()),
            include_footer: false,
            use_epsx_header: false,
        }
    }
}

pub fn render_page(ctx: &PageContext, is_admin: bool) -> (PageMeta, Element) {
    let p = ctx.path.as_str();
    if is_admin { return admin_pages::dispatch(ctx); }
    match p {
        "/" | "/index" => home::render(ctx),
        "/auth" => auth_page::render(ctx),
        "/dashboard" => dashboard::render(ctx),
        "/profile" => profile::render(ctx),
        "/account" => account::render(ctx),
        "/account/credits" => account_credits::render(ctx),
        "/analytics" => analytics::render(ctx),
        "/chat" => chat::render(ctx),
        "/chat/history" => chat_history::render(ctx),
        "/contact" => contact::render(ctx),
        "/about" => about::render(ctx),
        "/news" => news::render(ctx),
        "/notifications" => notifications::render(ctx),
        "/payment" => payment::render(ctx),
        "/permissions" => permissions::render(ctx),
        "/plans" => plans::render(ctx),
        "/portfolio" => portfolio::render(ctx),
        "/developer" => developer::render_overview(ctx),
        "/developer/usage" => developer::render_usage(ctx),
        "/developer/docs" => developer::render_docs(ctx),
        "/manual" => manual::render(ctx),
        "/access-denied" => access_denied::render(ctx),
        "/offline" => offline::render(ctx),
        "/privacy" => privacy::render(ctx),
        "/terms" => terms::render(ctx),
        _ => {
            if p.starts_with("/portfolio/") {
                // T2: per-address portfolio route. Mirrors the OLD
                // prod 307-to-/portfolio behaviour via inline
                // meta-refresh (see portfolio_address.rs).
                let addr = p.trim_start_matches("/portfolio/")
                    .trim_end_matches('/').to_string();
                let mut c = ctx.clone();
                c.params.insert("address".into(), addr);
                portfolio_address::render(&c)
            } else if p.starts_with("/chat/") {
                let id = p.trim_start_matches("/chat/").trim_end_matches('/').to_string();
                let mut c = ctx.clone();
                c.params.insert("id".into(), id);
                chat_conversation::render(&c)
            } else if p.starts_with("/news/") {
                let slug = p.trim_start_matches("/news/").trim_end_matches('/').to_string();
                let mut c = ctx.clone();
                c.params.insert("slug".into(), slug);
                news_detail::render(&c)
            } else if p.starts_with("/payment/") {
                let rest = p.trim_start_matches("/payment/").trim_end_matches('/');
                let mut parts = rest.splitn(2, '/');
                let ptype = parts.next().unwrap_or("").to_string();
                let pid = parts.next().unwrap_or("").to_string();
                let mut c = ctx.clone();
                c.params.insert("type".into(), ptype);
                c.params.insert("id".into(), pid);
                payment::render_dynamic(&c)
            } else {
                not_found::render(ctx)
            }
        }
    }
}

pub fn page_title_for(p: &str) -> String {
    match p {
        "/" => t("nav.home"),
        "/auth" => t("nav.auth"),
        "/dashboard" => t("nav.dashboard"),
        "/profile" => t("nav.profile"),
        "/account" | "/account/credits" => t("nav.account"),
        "/analytics" => t("nav.analytics"),
        "/chat" | "/chat/history" => t("nav.chat"),
        "/contact" => t("nav.contact"),
        "/about" => t("nav.about"),
        "/news" => t("nav.news"),
        "/notifications" => t("nav.notifications"),
        "/payment" | "/payment/[type]/[id]" => t("nav.payment"),
        "/permissions" => t("nav.permissions"),
        "/plans" => t("nav.plans"),
        "/pricing" => t("nav.plans"),
        "/portfolio" => t("nav.portfolio"),
        "/developer" | "/developer/usage" | "/developer/docs" => t("nav.developer"),
        "/manual" => t("nav.manual"),
        "/access-denied" => "Access denied".to_string(),
        "/offline" => "Offline".to_string(),
        "/privacy" => "Privacy".to_string(),
        "/terms" => "Terms".to_string(),
        _ => "EPSX".to_string(),
    }
}
