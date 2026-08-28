//! Type-safe routable route definitions and view wrappers for EPSX Frontend and Admin apps.

use crate::pages::{self, PageContext};
use dioxus::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum FrontendRoute {
    #[route("/")]
    HomeView {},

    #[route("/auth")]
    AuthPageView {},

    #[route("/dashboard")]
    DashboardView {},

    #[route("/profile")]
    ProfileView {},

    #[route("/account")]
    AccountView {},

    #[route("/account/credits")]
    AccountCreditsView {},

    #[route("/analytics")]
    AnalyticsView {},

    #[route("/chat")]
    ChatInboxView {},

    #[route("/chat/history")]
    ChatHistoryView {},

    #[route("/chat/:id")]
    ChatConversationView { id: String },

    #[route("/contact")]
    ContactView {},

    #[route("/about")]
    AboutView {},

    #[route("/news")]
    NewsListView {},

    #[route("/news/:slug")]
    NewsDetailView { slug: String },

    #[route("/notifications")]
    NotificationsView {},

    #[route("/payment")]
    PaymentView {},

    #[route("/payment/:ptype/:pid")]
    PaymentDynamicView { ptype: String, pid: String },

    #[route("/permissions")]
    PermissionsView {},

    #[route("/plans")]
    PlansView {},

    #[redirect("/pricing", || FrontendRoute::PlansView {})]
    #[route("/pricing")]
    PricingRedirectView {},

    #[route("/portfolio")]
    PortfolioView {},

    #[route("/portfolio/:address")]
    PortfolioAddressView { address: String },

    #[route("/developer")]
    DeveloperView {},

    #[route("/developer/usage")]
    DeveloperUsageView {},

    #[route("/developer/docs")]
    DeveloperDocsView {},

    #[route("/manual")]
    ManualView {},

    #[route("/access-denied")]
    AccessDeniedView {},

    #[route("/offline")]
    OfflineView {},

    #[route("/privacy")]
    PrivacyView {},

    #[route("/terms")]
    TermsView {},

    #[route("/:..route")]
    NotFoundView { route: Vec<String> },
}

fn get_ctx() -> PageContext {
    try_consume_context::<PageContext>().unwrap_or_default()
}

#[component]
pub fn HomeView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AuthPageView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/auth".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn DashboardView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/dashboard".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ProfileView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/profile".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AccountView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/account".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AccountCreditsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/account/credits".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AnalyticsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/analytics".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ChatInboxView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/chat".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ChatHistoryView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/chat/history".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ChatConversationView(id: String) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/chat/{id}");
    ctx.params.insert("id".into(), id);
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ContactView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/contact".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AboutView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/about".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn NewsListView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/news".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn NewsDetailView(slug: String) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/news/{slug}");
    ctx.params.insert("slug".into(), slug);
    pages::render_page(&ctx, false).1
}

#[component]
pub fn NotificationsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/notifications".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PaymentView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/payment".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PaymentDynamicView(ptype: String, pid: String) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/payment/{ptype}/{pid}");
    ctx.params.insert("type".into(), ptype);
    ctx.params.insert("id".into(), pid);
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PermissionsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/permissions".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PlansView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/plans".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PricingRedirectView() -> Element {
    rsx! {
        document::Meta {
            http_equiv: "refresh",
            content: "0; url=/plans",
        }
    }
}

#[component]
pub fn PortfolioView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/portfolio".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PortfolioAddressView(address: String) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/portfolio/{address}");
    ctx.params.insert("address".into(), address);
    pages::render_page(&ctx, false).1
}

#[component]
pub fn DeveloperView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/developer".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn DeveloperUsageView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/developer/usage".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn DeveloperDocsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/developer/docs".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn ManualView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/manual".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn AccessDeniedView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/access-denied".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn OfflineView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/offline".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn PrivacyView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/privacy".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn TermsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/terms".to_string();
    pages::render_page(&ctx, false).1
}

#[component]
pub fn NotFoundView(route: Vec<String>) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/{}", route.join("/"));
    pages::render_page(&ctx, false).1
}

// ============================================================================
// ADMIN ROUTES
// ============================================================================

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum AdminRoute {
    #[route("/")]
    AdminHomeView {},

    #[route("/auth")]
    AdminAuthView {},

    #[route("/dashboard")]
    AdminDashboardView {},

    #[route("/analytics")]
    AdminAnalyticsView {},

    #[route("/users")]
    AdminUsersView {},

    #[route("/payments")]
    AdminPaymentsView {},

    #[route("/notifications")]
    AdminNotificationsView {},

    #[route("/news")]
    AdminNewsView {},

    #[route("/chat")]
    AdminChatView {},

    #[route("/developer-portal")]
    AdminDeveloperPortalView {},

    #[route("/media")]
    AdminMediaView {},

    #[route("/settings")]
    AdminSettingsView {},

    #[route("/audit-log")]
    AdminAuditLogView {},

    #[route("/access-denied")]
    AdminAccessDeniedView {},

    #[route("/unauthorized")]
    AdminUnauthorizedView {},

    #[route("/:..route")]
    AdminNotFoundView { route: Vec<String> },
}

#[component]
pub fn AdminHomeView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminAuthView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/auth".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminDashboardView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/dashboard".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminAnalyticsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/analytics".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminUsersView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/users".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminPaymentsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/payments".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminNotificationsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/notifications".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminNewsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/news".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminChatView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/chat".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminDeveloperPortalView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/developer-portal".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminMediaView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/media".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminSettingsView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/settings".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminAuditLogView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/audit-log".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminAccessDeniedView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/access-denied".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminUnauthorizedView() -> Element {
    let mut ctx = get_ctx();
    ctx.path = "/unauthorized".to_string();
    pages::render_page(&ctx, true).1
}

#[component]
pub fn AdminNotFoundView(route: Vec<String>) -> Element {
    let mut ctx = get_ctx();
    ctx.path = format!("/{}", route.join("/"));
    pages::render_page(&ctx, true).1
}
