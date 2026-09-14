//! Type-safe routable route definitions and view wrappers for EPSX Frontend and Admin apps.

use crate::fullstack::shell::FrontendShell;
mod admin_aliases;
use admin_aliases::*;
use dioxus::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum FrontendRoute {
    #[layout(FrontendShell)]
    #[route("/")]
    HomeView {},

    #[route("/index")]
    HomeIndexView {},

    #[route("/auth?:..query")]
    AuthPageView { query: String },

    #[route("/dashboard")]
    DashboardView {},

    #[route("/profile?:..query")]
    ProfileView { query: String },

    #[route("/account")]
    AccountView {},

    #[route("/account/credits")]
    AccountCreditsView {},

    #[route("/account/payments?:..query")]
    PurchasesView { query: String },

    #[route("/account/payments/:order_id")]
    PurchaseDetailView { order_id: String },

    #[route("/analytics?:..query")]
    AnalyticsView { query: String },

    #[route("/chat?:..query")]
    ChatInboxView { query: String },

    #[route("/chat/history")]
    ChatHistoryView {},

    #[route("/chat/:id")]
    ChatConversationView { id: String },

    #[route("/contact")]
    ContactView {},

    #[route("/about")]
    AboutView {},

    #[route("/news?:..query")]
    NewsListView { query: String },

    #[route("/news/:slug")]
    NewsDetailView { slug: String },

    #[route("/notifications?:..query")]
    NotificationsView { query: String },

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

    #[route("/developer?:..query")]
    DeveloperView { query: String },

    #[route("/developer/usage?:..query")]
    DeveloperUsageView { query: String },

    #[route("/developer/docs")]
    DeveloperDocsView {},

    #[route("/manual")]
    ManualView {},

    #[route("/access-denied")]
    AccessDeniedView {},

    #[route("/offline?:..query")]
    OfflineView { query: String },

    #[route("/privacy")]
    PrivacyView {},

    #[route("/terms")]
    TermsView {},

    #[route("/:..route")]
    NotFoundView { route: Vec<String> },
}

#[component]
pub fn HomeIndexView() -> Element {
    rsx! { crate::pages::home::HydratedHome {} }
}

#[component]
pub fn HomeView() -> Element {
    rsx! { crate::pages::home::HydratedHome {} }
}

#[component]
pub fn AuthPageView(query: String) -> Element {
    // Routable decodes a catch-all query before passing this prop. Read the
    // original history URL so encoded '&' inside return_url stays nested.
    let route = use_context::<dioxus_router::RouterContext>().full_route_string();
    let query = route
        .split_once('?')
        .map(|(_, query)| query.split('#').next().unwrap_or(query))
        .unwrap_or(&query)
        .to_owned();
    rsx! { crate::fullstack::frontend_auth::HydratedAuth { query } }
}

#[component]
pub fn DashboardView() -> Element {
    rsx! { crate::pages::dashboard::HydratedDashboard {} }
}

#[component]
pub fn ProfileView(query: String) -> Element {
    rsx! { crate::pages::profile::HydratedProfile { query } }
}

#[component]
pub fn AccountView() -> Element {
    rsx! { crate::pages::account::hydrated::HydratedAccount {} }
}

#[component]
pub fn AccountCreditsView() -> Element {
    rsx! { crate::pages::account_credits::HydratedCredits {} }
}

#[component]
pub fn AnalyticsView(query: String) -> Element {
    rsx! { crate::fullstack::analytics::HydratedAnalytics { query } }
}

#[component]
pub fn ChatInboxView(query: String) -> Element {
    rsx! { crate::pages::chat::hydrated::HydratedChat { id: None, query } }
}
#[component]
pub fn ChatHistoryView() -> Element {
    rsx! { crate::pages::chat::hydrated::HydratedChat { id: None, query: String::new(), history: true } }
}
#[component]
pub fn ChatConversationView(id: String) -> Element {
    match id.parse::<uuid::Uuid>() {
        Ok(id) => {
            rsx! { crate::pages::chat::hydrated::HydratedChat { id: Some(id), query: String::new() } }
        }
        Err(_) => rsx! { crate::pages::not_found::HydratedNotFound {} },
    }
}

#[component]
pub fn ContactView() -> Element {
    rsx! { crate::pages::contact::HydratedContact {} }
}

#[component]
pub fn AboutView() -> Element {
    rsx! { crate::pages::about::HydratedAbout {} }
}

#[component]
pub fn NewsListView(query: String) -> Element {
    rsx! { crate::pages::news::hydrated::HydratedNews { query } }
}

#[component]
pub fn NewsDetailView(slug: String) -> Element {
    rsx! { crate::pages::news_detail::HydratedNewsDetail { slug } }
}

#[component]
pub fn NotificationsView(query: String) -> Element {
    rsx! { crate::pages::notifications::hydrated::HydratedNotifications { query } }
}

#[component]
pub fn PaymentView() -> Element {
    rsx! { crate::fullstack::frontend_payment::HydratedPayment { plan_id: None } }
}

#[component]
pub fn PaymentDynamicView(ptype: String, pid: String) -> Element {
    let plan_id = (ptype == "plan").then_some(pid.clone());
    rsx! { crate::fullstack::frontend_payment::HydratedPayment { key: "{ptype}/{pid}", plan_id } }
}

#[component]
pub fn PermissionsView() -> Element {
    rsx! { crate::pages::permissions::HydratedPermissions {} }
}

#[component]
pub fn PlansView() -> Element {
    rsx! { crate::pages::plans::HydratedPlans {} }
}

#[component]
pub fn PricingRedirectView() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace(FrontendRoute::PlansView {});
    });
    rsx! { Link { to: FrontendRoute::PlansView {}, "View plans" } }
}

#[component]
pub fn PortfolioView() -> Element {
    rsx! { crate::pages::portfolio::hydrated::HydratedPortfolio {} }
}

#[component]
pub fn PortfolioAddressView(address: String) -> Element {
    let _ = address;
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace(FrontendRoute::PortfolioView {});
    });
    rsx! { Link { to: FrontendRoute::PortfolioView {}, "Continue to saved companies" } }
}

#[component]
pub fn DeveloperView(query: String) -> Element {
    let _ = query;
    rsx! { DeveloperComingSoon {} }
}

#[component]
pub fn DeveloperUsageView(query: String) -> Element {
    let _ = query;
    rsx! { DeveloperComingSoon {} }
}

#[component]
pub fn DeveloperDocsView() -> Element {
    rsx! { DeveloperComingSoon {} }
}

// Temporary frontend placeholder; keep the developer implementation for a future release.
#[component]
fn DeveloperComingSoon() -> Element {
    rsx! {
        document::Title { "Developer API — Coming soon | EPSX" }
        section { class: "card card-glass fe-surface",
            div { class: "card-body space-y-5",
                h1 { class: "text-2xl font-semibold", "Developer API" }
                p { class: "text-lg", "Coming soon" }
                p { class: "fe-tone-muted", "API keys, usage, and documentation will be available in a future release." }
                crate::fullstack::shell::ShellLink { class: "fe-button", href: "/analytics", "Explore companies" }
            }
        }
    }
}

#[component]
pub fn ManualView() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace(FrontendRoute::AnalyticsView {
            query: String::new(),
        });
    });
    rsx! { Link { to: FrontendRoute::AnalyticsView { query: String::new() }, "Explore rankings" } }
}

#[component]
pub fn AccessDeniedView() -> Element {
    rsx! { crate::pages::access_denied::HydratedAccessDenied {} }
}

#[component]
pub fn OfflineView(query: String) -> Element {
    let _ = query;
    rsx! { crate::pages::offline::HydratedOffline {} }
}

#[component]
pub fn PrivacyView() -> Element {
    rsx! { crate::pages::privacy::HydratedPrivacy {} }
}

#[component]
pub fn TermsView() -> Element {
    rsx! { crate::pages::terms::HydratedTerms {} }
}

#[component]
pub fn NotFoundView(route: Vec<String>) -> Element {
    let _ = route;
    rsx! { crate::pages::not_found::HydratedNotFound {} }
}

// ============================================================================
// ADMIN ROUTES
// ============================================================================

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum AdminRoute {
    #[layout(crate::navigation::AdminRouteShell)]
    #[route("/wallet-management/wallets?:..query")]
    AdminWalletListView { query: String },
    #[route("/wallet-management/:address?:..query")]
    AdminWalletDetailView { address: String, query: String },
    #[route("/wallet-management/wallets/:address?:..query")]
    AdminWalletDetailAliasView { address: String, query: String },
    #[route("/wallet-management/wallets/:address/disable?:..query")]
    AdminWalletDisableView { address: String, query: String },
    #[route("/wallet-management/access?:..query")]
    AdminWalletAccessView { query: String },
    #[route("/wallet-management/credits?:..query")]
    AdminWalletCreditsView { query: String },
    #[route("/wallet-management/access/plans?:..query")]
    AdminWalletPlansView { query: String },
    #[route("/wallet-management/access/plans/:id?:..query")]
    AdminWalletPlanView { id: String, query: String },

    #[route("/pay/escrows?:..query")]
    AdminEscrowsView { query: String },
    #[route("/pay/escrows/:id?:..query")]
    AdminEscrowDetailView { id: String, query: String },
    #[route("/pay/merchant-escrows?:..query")]
    AdminMerchantEscrowsView { query: String },
    #[route("/pay/merchant-escrows/:id?:..query")]
    AdminMerchantEscrowDetailView { id: String, query: String },

    #[route("/payments/epsx?:..query")]
    AdminOrdersView { query: String },
    #[route("/payments/epsx/:id")]
    AdminOrderDetailView { id: String },

    #[route("/plans")]
    AdminCatalogView {},
    #[route("/plans/:id")]
    AdminCatalogDetailView { id: String },

    #[route("/")]
    AdminHomeView {},
    #[route("/index")]
    AdminIndexView {},

    #[route("/auth?:..query")]
    AdminAuthView { query: String },

    #[route("/dashboard")]
    AdminDashboardView {},

    #[route("/analytics?:..query")]
    AdminAnalyticsView { query: String },

    #[route("/users")]
    AdminUsersView {},

    #[route("/payments?:..query")]
    AdminPaymentsView { query: String },

    #[route("/notifications")]
    AdminNotificationsView {},
    #[route("/notifications/manage?:..query")]
    AdminNotificationsManageView { query: String },
    #[route("/notifications/create?:..query")]
    AdminNotificationsCreateView { query: String },

    #[route("/news?:..query")]
    AdminNewsView { query: String },
    #[route("/news/create")]
    AdminNewsCreateView {},
    #[route("/news/:id/edit")]
    AdminNewsEditView { id: String },

    #[route("/chat?:..query")]
    AdminChatView { query: String },
    #[route("/chat/:id?:..query")]
    AdminChatDetailView { id: String, query: String },

    #[route("/developer-portal?:..query")]
    AdminDeveloperPortalView { query: String },
    #[route("/developer-portal/api-keys/create")]
    AdminDeveloperCreateView {},

    #[route("/media?:..query")]
    AdminMediaView { query: String },

    #[route("/settings?:..query")]
    AdminSettingsView { query: String },

    #[route("/audit-log?:..query")]
    AdminAuditLogView { query: String },

    #[route("/access-denied?:..query")]
    AdminAccessDeniedView { query: String },

    #[route("/unauthorized")]
    AdminUnauthorizedView {},

    #[route("/:..route")]
    AdminNotFoundView { route: Vec<String> },
    // BEGIN GENERATED ADMIN PREFIX ROUTES
    #[route("/admin/wallet-management/wallets?:..query")]
    PrefixedAdminWalletListView { query: String },
    #[route("/admin/wallet-management/:address?:..query")]
    PrefixedAdminWalletDetailView { address: String, query: String },
    #[route("/admin/wallet-management/wallets/:address?:..query")]
    PrefixedAdminWalletDetailAliasView { address: String, query: String },
    #[route("/admin/wallet-management/wallets/:address/disable?:..query")]
    PrefixedAdminWalletDisableView { address: String, query: String },
    #[route("/admin/wallet-management/access?:..query")]
    PrefixedAdminWalletAccessView { query: String },
    #[route("/admin/wallet-management/credits?:..query")]
    PrefixedAdminWalletCreditsView { query: String },
    #[route("/admin/wallet-management/access/plans?:..query")]
    PrefixedAdminWalletPlansView { query: String },
    #[route("/admin/wallet-management/access/plans/:id?:..query")]
    PrefixedAdminWalletPlanView { id: String, query: String },
    #[route("/admin/pay/escrows?:..query")]
    PrefixedAdminEscrowsView { query: String },
    #[route("/admin/pay/escrows/:id?:..query")]
    PrefixedAdminEscrowDetailView { id: String, query: String },
    #[route("/admin/pay/merchant-escrows?:..query")]
    PrefixedAdminMerchantEscrowsView { query: String },
    #[route("/admin/pay/merchant-escrows/:id?:..query")]
    PrefixedAdminMerchantEscrowDetailView { id: String, query: String },
    #[route("/admin/payments/epsx?:..query")]
    PrefixedAdminOrdersView { query: String },
    #[route("/admin/payments/epsx/:id")]
    PrefixedAdminOrderDetailView { id: String },
    #[route("/admin/plans")]
    PrefixedAdminCatalogView {},
    #[route("/admin/plans/:id")]
    PrefixedAdminCatalogDetailView { id: String },
    #[route("/admin")]
    PrefixedAdminHomeView {},
    #[route("/admin/index")]
    PrefixedAdminIndexView {},
    #[route("/admin/auth?:..query")]
    PrefixedAdminAuthView { query: String },
    #[route("/admin/dashboard")]
    PrefixedAdminDashboardView {},
    #[route("/admin/analytics?:..query")]
    PrefixedAdminAnalyticsView { query: String },
    #[route("/admin/users")]
    PrefixedAdminUsersView {},
    #[route("/admin/payments?:..query")]
    PrefixedAdminPaymentsView { query: String },
    #[route("/admin/notifications")]
    PrefixedAdminNotificationsView {},
    #[route("/admin/notifications/manage?:..query")]
    PrefixedAdminNotificationsManageView { query: String },
    #[route("/admin/notifications/create?:..query")]
    PrefixedAdminNotificationsCreateView { query: String },
    #[route("/admin/news?:..query")]
    PrefixedAdminNewsView { query: String },
    #[route("/admin/news/create")]
    PrefixedAdminNewsCreateView {},
    #[route("/admin/news/:id/edit")]
    PrefixedAdminNewsEditView { id: String },
    #[route("/admin/chat?:..query")]
    PrefixedAdminChatView { query: String },
    #[route("/admin/chat/:id?:..query")]
    PrefixedAdminChatDetailView { id: String, query: String },
    #[route("/admin/developer-portal?:..query")]
    PrefixedAdminDeveloperPortalView { query: String },
    #[route("/admin/developer-portal/api-keys/create")]
    PrefixedAdminDeveloperCreateView {},
    #[route("/admin/media?:..query")]
    PrefixedAdminMediaView { query: String },
    #[route("/admin/settings?:..query")]
    PrefixedAdminSettingsView { query: String },
    #[route("/admin/audit-log?:..query")]
    PrefixedAdminAuditLogView { query: String },
    #[route("/admin/access-denied?:..query")]
    PrefixedAdminAccessDeniedView { query: String },
    #[route("/admin/unauthorized")]
    PrefixedAdminUnauthorizedView {},
    // END GENERATED ADMIN PREFIX ROUTES
}

#[component]
pub fn AdminHomeView() -> Element {
    rsx! { crate::fullstack::admin_system::HydratedAdminHome {} }
}

#[component]
pub fn AdminAuthView(query: String) -> Element {
    let route = use_context::<dioxus_router::RouterContext>().full_route_string();
    let query = route
        .split_once('?')
        .map(|(_, query)| query.split('#').next().unwrap_or(query))
        .unwrap_or(&query)
        .to_owned();
    rsx! { crate::fullstack::admin_auth::HydratedAdminAuth { query } }
}

#[component]
pub fn AdminDashboardView() -> Element {
    rsx! { crate::fullstack::admin::HydratedAdminDashboard {} }
}

#[component]
pub fn AdminAnalyticsView(query: String) -> Element {
    rsx! { crate::fullstack::admin::HydratedAdminAnalytics { query } }
}

#[component]
pub fn AdminUsersView() -> Element {
    rsx! { crate::fullstack::admin_system::HydratedAdminNotFound {} }
}

#[component]
pub fn AdminPaymentsView(query: String) -> Element {
    rsx! { crate::fullstack::admin_payments::HydratedAdminPayments { query } }
}

#[component]
pub fn AdminNotificationsView() -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace("/notifications/manage");
    });
    rsx! { Link { to: "/notifications/manage", "Manage notifications" } }
}

#[component]
pub fn AdminNewsView(query: String) -> Element {
    rsx! { crate::fullstack::admin_news::HydratedAdminNews {  page: crate::fullstack::admin_news::NewsPage::List, query } }
}
#[component]
pub fn AdminNewsCreateView() -> Element {
    rsx! { crate::fullstack::admin_news::HydratedAdminNews {  page: crate::fullstack::admin_news::NewsPage::Create, query: String::new() } }
}
#[component]
pub fn AdminNewsEditView(id: String) -> Element {
    if uuid::Uuid::parse_str(&id).is_err() {
        return rsx! {crate::fullstack::admin_system::HydratedAdminNotFound{}};
    }
    rsx! { crate::fullstack::admin_news::HydratedAdminNews { key: "{id}", page: crate::fullstack::admin_news::NewsPage::Edit(id), query: String::new() } }
}

#[component]
pub fn AdminChatView(query: String) -> Element {
    let malformed_original = use_server_cached(|| {
        #[cfg(feature = "server")]
        {
            dioxus_fullstack::FullstackContext::current()
                .is_some_and(|context| context.parts_mut().uri.path().ends_with('/'))
        }
        #[cfg(not(feature = "server"))]
        {
            false
        }
    });
    if malformed_original {
        return rsx! {crate::fullstack::admin_system::HydratedAdminNotFound{}};
    }
    rsx! { crate::fullstack::admin_chat::HydratedAdminChat { id: None, query } }
}
#[component]
pub fn AdminChatDetailView(id: String, query: String) -> Element {
    if uuid::Uuid::parse_str(&id).is_err() {
        return rsx! {crate::fullstack::admin_system::HydratedAdminNotFound{}};
    }
    rsx! { crate::fullstack::admin_chat::HydratedAdminChat { key: "{id}", id: Some(id), query } }
}

#[component]
pub fn AdminDeveloperPortalView(query: String) -> Element {
    rsx! { crate::fullstack::admin_developer::HydratedAdminDeveloper {  query, create: false } }
}
#[component]
pub fn AdminDeveloperCreateView() -> Element {
    rsx! { crate::fullstack::admin_developer::HydratedAdminDeveloper {  query: String::new(), create: true } }
}

#[component]
pub fn AdminMediaView(query: String) -> Element {
    rsx! { crate::fullstack::admin_media::HydratedAdminMedia { query } }
}

#[component]
pub fn AdminSettingsView(query: String) -> Element {
    rsx! { crate::fullstack::admin_settings::HydratedAdminSettings { query } }
}

#[component]
pub fn AdminAuditLogView(query: String) -> Element {
    rsx! { crate::fullstack::admin::HydratedAdminAudit { query } }
}

#[component]
pub fn AdminAccessDeniedView(query: String) -> Element {
    rsx! { crate::fullstack::admin_system::HydratedAdminDenied { query, unauthorized: false } }
}

#[component]
pub fn AdminUnauthorizedView() -> Element {
    rsx! { crate::fullstack::admin_system::HydratedAdminDenied { query: String::new(), unauthorized: true } }
}

#[component]
pub fn AdminNotFoundView(route: Vec<String>) -> Element {
    let _ = route;
    rsx! { crate::fullstack::admin_system::HydratedAdminNotFound {} }
}

pub fn purchase_route_query(
    path: &str,
    query: &str,
) -> Result<crate::payment::purchases::PurchaseQuery, crate::fullstack::LoadError> {
    use crate::{fullstack::LoadError, payment::purchases::PurchaseQuery};
    let order_id = match path.strip_prefix("/account/payments/") {
        Some(value) => Some(uuid::Uuid::parse_str(value).map_err(|_| LoadError::InvalidQuery)?),
        None if path == "/account/payments" => None,
        _ => return Err(LoadError::InvalidQuery),
    };
    let mut offset = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key != "offset" || offset.is_some() {
            return Err(LoadError::InvalidQuery);
        }
        offset = Some(value.parse::<u64>().map_err(|_| LoadError::InvalidQuery)?);
    }
    let query = PurchaseQuery {
        order_id,
        offset: offset.unwrap_or(0),
    };
    query.validate()?;
    Ok(query)
}

#[component]
pub fn PurchasesView(query: String) -> Element {
    match purchase_route_query("/account/payments", &query) {
        Ok(query) => rsx! { crate::payment::purchases::HydratedPurchases { query } },
        Err(error) => {
            rsx! { crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  } }
        }
    }
}

#[component]
pub fn PurchaseDetailView(order_id: String) -> Element {
    match purchase_route_query(&format!("/account/payments/{order_id}"), "") {
        Ok(query) => rsx! { crate::payment::purchases::HydratedPurchases { query } },
        Err(error) => {
            rsx! { crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  } }
        }
    }
}

#[component]
pub fn AdminCatalogView() -> Element {
    rsx! { crate::fullstack::admin_catalog::HydratedCatalog { id: None } }
}
#[component]
pub fn AdminCatalogDetailView(id: String) -> Element {
    rsx! { crate::fullstack::admin_catalog::HydratedCatalog { key: "{id}", id: Some(id.clone()) } }
}

#[component]
pub fn AdminOrdersView(query: String) -> Element {
    match purchase_route_query("/account/payments", &query) {
        Ok(query) => rsx! { crate::fullstack::admin_orders::HydratedAdminOrders { query } },
        Err(error) => {
            rsx! { crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  } }
        }
    }
}
#[component]
pub fn AdminOrderDetailView(id: String) -> Element {
    match purchase_route_query(&format!("/account/payments/{id}"), "") {
        Ok(query) => {
            rsx! { crate::fullstack::admin_orders::HydratedAdminOrders { key: "{id}", query } }
        }
        Err(error) => {
            rsx! { crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  } }
        }
    }
}

#[component]
pub fn AdminEscrowsView(query: String) -> Element {
    rsx! { crate::fullstack::admin_escrow::HydratedAdminEscrows { key: "native/{query}", merchant: false, id: None, query } }
}
#[component]
pub fn AdminEscrowDetailView(id: String, query: String) -> Element {
    rsx! { crate::fullstack::admin_escrow::HydratedAdminEscrows { key: "native/{id}/{query}", merchant: false, id: Some(id), query } }
}
#[component]
pub fn AdminMerchantEscrowsView(query: String) -> Element {
    rsx! { crate::fullstack::admin_escrow::HydratedAdminEscrows { key: "merchant/{query}", merchant: true, id: None, query } }
}
#[component]
pub fn AdminMerchantEscrowDetailView(id: String, query: String) -> Element {
    rsx! { crate::fullstack::admin_escrow::HydratedAdminEscrows { key: "merchant/{id}/{query}", merchant: true, id: Some(id), query } }
}

#[component]
pub fn AdminWalletListView(query: String) -> Element {
    rsx! { crate::fullstack::core_wallets::CoreWallets { key: "AdminWalletListView/{query}", address: None, query } }
}

#[component]
pub fn AdminWalletDetailView(address: String, query: String) -> Element {
    rsx! { crate::fullstack::core_wallets::CoreWallets { key: "AdminWalletDetailView/{address}/{query}", address: Some(address), query } }
}

#[component]
pub fn AdminWalletDetailAliasView(address: String, query: String) -> Element {
    rsx! { crate::fullstack::core_wallets::CoreWallets { key: "AdminWalletDetailAliasView/{address}/{query}", address: Some(address), query } }
}

#[component]
pub fn AdminWalletDisableView(address: String, query: String) -> Element {
    rsx! { crate::fullstack::core_wallets::CoreWallets { key: "AdminWalletDisableView/{address}/{query}", address: Some(address), query } }
}

#[component]
pub fn AdminWalletAccessView(query: String) -> Element {
    rsx! { crate::fullstack::core_wallets::CoreWallets { key: "AdminWalletAccessView/{query}", address: None, query } }
}

#[component]
pub fn AdminWalletCreditsView(query: String) -> Element {
    rsx! { crate::fullstack::core_credits::CoreCredits { key: "credits/{query}", query } }
}

#[component]
pub fn AdminWalletPlansView(query: String) -> Element {
    rsx! { crate::fullstack::admin_catalog::HydratedCatalog { key: "plans/{query}", id: None } }
}

#[component]
pub fn AdminWalletPlanView(id: String, query: String) -> Element {
    rsx! { crate::fullstack::admin_catalog::HydratedCatalog { key: "plans/{id}/{query}", id: Some(id) } }
}

#[component]
pub fn AdminIndexView() -> Element {
    rsx! { crate::fullstack::admin_system::HydratedAdminHome {} }
}

#[component]
pub fn AdminNotificationsManageView(query: String) -> Element {
    rsx! { crate::fullstack::admin_notifications::HydratedAdminNotifications { query, create: false } }
}
#[component]
pub fn AdminNotificationsCreateView(query: String) -> Element {
    rsx! { crate::fullstack::admin_notifications::HydratedAdminNotifications { query, create: true } }
}
