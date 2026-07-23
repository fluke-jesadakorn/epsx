//! `/manual` — static feature reference with a responsive category index.
//!
//! The accepted source is the pinned development implementation at
//! `apps/frontend/app/manual/{page.tsx,data.ts,screenshot-img.tsx}`. The
//! category and feature catalog below remains a verbatim content port. The
//! target adds semantic labels and responsive behavior while preserving the
//! source's dark layout, screenshot viewer, and page links.

use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use dioxus::prelude::*;

#[path = "manual_route_statuses.rs"]
mod manual_route_statuses;

#[cfg(test)]
use manual_route_statuses::RouteMigrationStatus;
use manual_route_statuses::{ManualRouteStatus, MANUAL_ROUTE_STATUSES};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ManualFeature {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    route: &'static str,
    screenshots: &'static [&'static str],
    category: &'static str,
}

const CATEGORIES: &[&str] = &[
    "Public",
    "Auth",
    "Dashboard",
    "Analytics",
    "Plans",
    "Portfolio",
    "Notifications",
    "Developer",
];

const FEATURES: &[ManualFeature] = &[
    // Public
    ManualFeature { id: "home", name: "Home", desc: "The landing page displays the hero section with platform tagline, primary navigation bar, and an overview of key features. Visitors see call-to-action buttons for signing up and exploring analytics.", route: "/", screenshots: &["home"], category: "Public" },
    ManualFeature { id: "about", name: "About", desc: "The about page presents platform background, mission statement, and team information. Sections describe the technology stack, partnerships, and the roadmap for upcoming features.", route: "/about", screenshots: &["about"], category: "Public" },
    ManualFeature { id: "terms", name: "Terms of Service", desc: "The legal terms page shows the full terms and conditions governing platform use. Users can read through sections covering account responsibilities, data usage, and dispute resolution.", route: "/terms", screenshots: &["terms"], category: "Public" },
    ManualFeature { id: "privacy", name: "Privacy Policy", desc: "The privacy policy page outlines how user data is collected, stored, and protected. Sections cover cookie usage, third-party integrations, and data retention periods.", route: "/privacy", screenshots: &["privacy"], category: "Public" },
    ManualFeature { id: "offline", name: "Offline", desc: "The PWA offline fallback page is shown when the user loses internet connectivity. It displays a friendly message indicating the app is offline and will reconnect automatically.", route: "/offline", screenshots: &["offline"], category: "Public" },
    ManualFeature { id: "access-denied", name: "Access Denied", desc: "This error page appears when a user attempts to access a route they lack permissions for. It shows the denied resource and suggests contacting an admin or upgrading their plan.", route: "/access-denied", screenshots: &["access-denied"], category: "Public" },
    // Auth
    ManualFeature { id: "auth", name: "Authentication", desc: "The Web3 authentication page presents wallet connection options via RainbowKit. Users can connect MetaMask, WalletConnect, or other providers, then sign a SIWE message to authenticate.", route: "/auth", screenshots: &["auth"], category: "Auth" },
    // Dashboard
    ManualFeature { id: "dashboard", name: "Dashboard", desc: "The main user dashboard displays portfolio summary stats, a watchlist of tracked stocks, and recent activity feed. Key metrics like total portfolio value and daily change are shown at the top.", route: "/dashboard", screenshots: &["dashboard"], category: "Dashboard" },
    ManualFeature { id: "account", name: "Account Overview", desc: "The account overview tab shows the user's current subscription plan, wallet address, and access level. Summary cards display plan expiration date, feature entitlements, and quick links to manage settings.", route: "/account", screenshots: &["account"], category: "Dashboard" },
    ManualFeature { id: "account-payments", name: "Payment History", desc: "User navigates to the Payments tab on the account page. The tab displays a chronological list of past transactions including amounts, dates, transaction hashes, and payment status badges.", route: "/account", screenshots: &["account-payments"], category: "Dashboard" },
    ManualFeature { id: "account-prefs", name: "Notification Preferences", desc: "User opens the Preferences tab and toggles notification settings. The UI shows switches for email alerts, push notifications, and in-app notifications, with the toggled switch reflecting the updated state.", route: "/account", screenshots: &["account-prefs"], category: "Dashboard" },
    ManualFeature { id: "account-credits", name: "Credits", desc: "The credits page displays the user's current credit balance, recent usage history, and purchase options. A usage chart shows credit consumption over time alongside available top-up packages.", route: "/account/credits", screenshots: &["account-credits"], category: "Dashboard" },
    ManualFeature { id: "profile", name: "Profile", desc: "The profile page shows the user's display name, connected wallet address, and account metadata. Read-only fields display registration date, last login, and current plan tier.", route: "/profile", screenshots: &["profile"], category: "Dashboard" },
    ManualFeature { id: "profile-edit", name: "Edit Profile", desc: "User clicks the Edit button on the profile page and types a new display name into the name input field. The form shows the editable field with the updated text and Save/Cancel action buttons.", route: "/profile", screenshots: &["profile-edit"], category: "Dashboard" },
    // Analytics
    ManualFeature { id: "analytics-default", name: "Stock Rankings", desc: "The default analytics view displays a paginated table of ranked stocks with columns for ticker, company name, price, daily change, volume, and composite score. Data loads with the default sort order.", route: "/analytics", screenshots: &["analytics-default"], category: "Analytics" },
    ManualFeature { id: "analytics-search", name: "Search Stocks", desc: "User types \"AAPL\" into the search input above the rankings table. The table filters in real-time to show only rows matching the query, displaying Apple Inc. and related tickers. The search input shows the active query text.", route: "/analytics", screenshots: &["analytics-search"], category: "Analytics" },
    ManualFeature { id: "analytics-filter-country", name: "Filter by Country", desc: "User clicks the Country filter button to open the country selection dropdown. The filter UI displays available country options, allowing the user to narrow rankings to stocks from a specific market.", route: "/analytics", screenshots: &["analytics-filter-country"], category: "Analytics" },
    ManualFeature { id: "analytics-filter-sector", name: "Filter by Sector", desc: "User clicks the Sector filter button to open the sector selection dropdown. Available sectors like Technology, Healthcare, and Finance are displayed, letting the user view rankings for a specific industry.", route: "/analytics", screenshots: &["analytics-filter-sector"], category: "Analytics" },
    ManualFeature { id: "analytics-sort", name: "Sort Column", desc: "User clicks a column header (e.g., Price or Change) to sort the rankings table. The column shows a sort direction indicator and the table rows reorder according to the selected column values.", route: "/analytics", screenshots: &["analytics-sort"], category: "Analytics" },
    ManualFeature { id: "analytics-pagination", name: "Pagination", desc: "User clicks the Next page button or page number in the pagination controls below the table. The table loads the next set of results and the pagination indicator updates to reflect the current page.", route: "/analytics", screenshots: &["analytics-pagination"], category: "Analytics" },
    // Plans
    ManualFeature { id: "plans", name: "Plans", desc: "The plans page presents available subscription tiers as side-by-side cards with pricing, feature lists, and comparison highlights. Each card shows the plan name, monthly price, and a Subscribe button.", route: "/plans", screenshots: &["plans"], category: "Plans" },
    ManualFeature { id: "payment", name: "Payment", desc: "The crypto payment checkout page shows the selected plan summary, total amount in USD and equivalent crypto, and wallet connection status. Users review the order before confirming the blockchain transaction.", route: "/payment", screenshots: &["payment"], category: "Plans" },
    ManualFeature { id: "payment-detail", name: "Payment Detail", desc: "The payment processing page for a specific plan and payment type displays transaction details, confirmation steps, and real-time status updates as the blockchain transaction is submitted and confirmed.", route: "/payment/[type]/[id]", screenshots: &["payment-detail"], category: "Plans" },
    // Portfolio
    ManualFeature { id: "portfolio", name: "Portfolio", desc: "The portfolio page displays the user's stock holdings in a table with columns for ticker, shares held, average cost, current value, and profit/loss. Summary cards at the top show total portfolio value and overall performance.", route: "/portfolio", screenshots: &["portfolio"], category: "Portfolio" },
    ManualFeature { id: "portfolio-search", name: "Search Portfolio", desc: "User types \"AAPL\" into the portfolio search input to filter their holdings. The table narrows to display only matching positions, showing the search query in the input and the filtered result count.", route: "/portfolio", screenshots: &["portfolio-search"], category: "Portfolio" },
    ManualFeature { id: "permissions", name: "Permissions", desc: "The permissions page lists the user's current feature entitlements granted by their subscription plan. Each permission shows the resource name, access level, and expiration date if applicable.", route: "/permissions", screenshots: &["permissions"], category: "Portfolio" },
    // Notifications
    ManualFeature { id: "notifications-default", name: "Notifications", desc: "The notification center displays all notifications in a chronological list with type icons, priority badges, timestamps, and read/unread indicators. Filter controls for type and priority appear above the list.", route: "/notifications", screenshots: &["notifications-default"], category: "Notifications" },
    ManualFeature { id: "notifications-filter-type", name: "Filter by Type", desc: "User clicks the Type filter and selects \"Security\" to narrow the notification list. Only security-related notifications are displayed, and the active filter chip shows the selected type.", route: "/notifications", screenshots: &["notifications-filter-type"], category: "Notifications" },
    ManualFeature { id: "notifications-filter-priority", name: "Filter by Priority", desc: "User clicks the Priority filter and selects \"High\" to show only urgent notifications. The list updates to display high-priority items, each marked with a colored priority badge.", route: "/notifications", screenshots: &["notifications-filter-priority"], category: "Notifications" },
    ManualFeature { id: "notifications-search", name: "Search Notifications", desc: "User types \"security\" into the notification search input. The list filters to show only notifications whose title or body contains the search term, with the query visible in the input field.", route: "/notifications", screenshots: &["notifications-search"], category: "Notifications" },
    ManualFeature { id: "notifications-empty", name: "Empty State", desc: "The notification center with no notifications displays an empty state illustration and message. This view appears when all notifications have been cleared or when using filters that match no results.", route: "/notifications", screenshots: &["notifications-empty"], category: "Notifications" },
    // Developer
    ManualFeature { id: "developer", name: "Developer Portal", desc: "The developer portal overview shows active API keys with their usage stats, rate limit status, and creation dates. Summary cards display total API calls, remaining quota, and quick links to documentation.", route: "/developer", screenshots: &["developer"], category: "Developer" },
    ManualFeature { id: "developer-create-key", name: "Create API Key", desc: "User clicks the Create button on the developer portal to open the API key creation dialog. The modal displays fields for key name, permission scopes, and expiration settings before generating a new key.", route: "/developer", screenshots: &["developer-create-key"], category: "Developer" },
    ManualFeature { id: "developer-docs", name: "API Documentation", desc: "The interactive API documentation page presents available endpoints grouped by module. Each endpoint card shows the HTTP method, path, description, request parameters, and expandable code samples.", route: "/developer/docs", screenshots: &["developer-docs"], category: "Developer" },
    ManualFeature { id: "developer-usage", name: "API Usage", desc: "The API usage page displays call volume charts over time, current rate limit consumption, and per-endpoint breakdown tables. Usage metrics include response times, error rates, and quota utilization.", route: "/developer/usage", screenshots: &["developer-usage"], category: "Developer" },
];

/// Route-scoped rules provide the source colors and make its fixed desktop
/// sidebar usable at 390px without relying on a different Tailwind compiler.
const MANUAL_INLINE_CSS: &str = r#"
.manual-prod-layout { display: flex; min-width: 0; }
.manual-sidebar { background-color: rgba(17, 24, 39, 0.5) !important; }
.manual-sidebar-border { border-color: rgb(31, 41, 55) !important; }
.manual-sidebar-link { color: rgb(156, 163, 175) !important; }
.manual-sidebar-link:hover { background-color: rgb(31, 41, 55) !important; color: rgb(255, 255, 255) !important; }
.manual-category-h2 { border-color: rgb(31, 41, 55) !important; color: rgb(243, 244, 246) !important; }
.manual-prod-content { min-width: 0; }
.manual-prod-category { scroll-margin-top: 5rem; }
.manual-prod-feature-head { flex-wrap: wrap; }
.manual-prod-feature-route { overflow-wrap: anywhere; }
.manual-prod-feature-status { border: 1px solid currentColor; border-radius: 9999px; padding: 0.125rem 0.5rem; font-size: 0.75rem; font-weight: 600; line-height: 1rem; }
.manual-prod-feature-status[data-manual-route-status="aligned"] { color: rgb(134, 239, 172); }
.manual-prod-feature-status[data-manual-route-status="partial"] { color: rgb(253, 224, 71); }
.manual-prod-feature-status[data-manual-route-status="blocked"] { color: rgb(252, 165, 165); }
.manual-prod-feature-status[data-manual-route-status="unavailable"] { color: rgb(156, 163, 175); }
.manual-prod-screenshot-button { display: block; width: 100%; height: 100%; text-align: left; }
.manual-prod-screenshot-img { width: 100%; height: 100%; object-fit: cover; object-position: top; cursor: zoom-in; }
.manual-prod-screenshot-fallback { display: none; width: 100%; height: 100%; align-items: center; justify-content: center; color: rgb(107, 114, 128); }
.manual-prod-screenshot-button[data-image-error="true"] .manual-prod-screenshot-img { display: none; }
.manual-prod-screenshot-button[data-image-error="true"] .manual-prod-screenshot-fallback { display: flex; }
.manual-prod-dialog[hidden] { display: none; }
.manual-prod-dialog { position: fixed; inset: 0; z-index: 80; display: flex; align-items: center; justify-content: center; padding: 1rem; background: rgba(0, 0, 0, 0.86); }
.manual-prod-dialog-panel { position: relative; display: flex; max-width: min(72rem, 100%); max-height: 100%; flex-direction: column; gap: 0.75rem; }
.manual-prod-dialog-img { display: block; max-width: 100%; max-height: calc(100vh - 7rem); border-radius: 0.5rem; object-fit: contain; }
.manual-prod-dialog-close { align-self: flex-end; border-radius: 0.375rem; background: rgb(31, 41, 55); padding: 0.5rem 0.75rem; color: white; }
.manual-prod-dialog-caption { margin: 0; text-align: center; color: rgb(209, 213, 219); }
.manual-sidebar-link:focus-visible,
.manual-prod-feature-link:focus-visible,
.manual-prod-screenshot-button:focus-visible,
.manual-prod-dialog-close:focus-visible { outline: 3px solid rgb(96, 165, 250); outline-offset: 3px; }
@media (max-width: 640px) {
  .manual-prod-layout { display: block; }
  .manual-prod-sidebar { position: relative !important; top: auto !important; width: 100% !important; height: auto !important; border-right: 0 !important; border-bottom: 1px solid rgb(31, 41, 55); padding: 0.75rem !important; }
  .manual-prod-sidebar-title { margin-bottom: 0.5rem !important; }
  .manual-prod-sidebar-nav { flex-direction: row !important; overflow-x: auto; padding: 0.125rem 0.125rem 0.5rem; scrollbar-width: thin; }
  .manual-prod-sidebar-link { flex: 0 0 auto; }
  .manual-prod-content { padding: 1rem !important; }
}
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let mut meta = PageMeta::marketing("Manual");
    meta.title = "EPSX Manual - Feature Guide".to_string();
    meta.description = "Complete feature guide with screenshots for the EPSX platform".to_string();
    (
        meta,
        rsx! {
            MainLayout { ctx: ctx.clone(),
                style { "{MANUAL_INLINE_CSS}" }
                div { class: "manual-page-prod min-h-screen bg-gray-950 text-gray-100",
                    div { class: "manual-prod-layout flex",
                        ManualSidebar {}
                        ManualContent {}
                    }
                }
            }
        },
    )
}

/// Sticky left sidebar with 8 category anchor links — mirrors the
/// prod's `<aside className="sticky top-0 h-screen w-56 shrink-0
/// overflow-y-auto border-r border-gray-800 bg-gray-900/50 p-4">`.
#[component]
fn ManualSidebar() -> Element {
    rsx! {
        aside { class: "manual-prod-sidebar manual-sidebar manual-sidebar-border sticky top-0 h-screen w-56 shrink-0 overflow-y-auto border-r border-gray-800 bg-gray-900/50 p-4",
            p { class: "mb-4 text-lg font-semibold text-white manual-prod-sidebar-title", "Categories" }
            nav { class: "flex flex-col gap-1 manual-prod-sidebar-nav", "aria-label": "Manual categories",
                for cat in CATEGORIES.iter() {
                    a {
                        class: "manual-prod-sidebar-link manual-sidebar-link rounded px-3 py-1.5 text-sm text-gray-400 hover:bg-gray-800 hover:text-white transition-colors",
                        href: "#{cat_slug(cat)}",
                        "{cat}"
                    }
                }
            }
        }
    }
}

/// Content pane — preserves the prod styling while the shared page shell owns
/// the document's single `<main>` landmark.
#[component]
fn ManualContent() -> Element {
    rsx! {
        div { class: "manual-prod-content flex-1 p-8", id: "manual-content",
            div { class: "mx-auto max-w-6xl",
                h1 { class: "mb-2 text-3xl font-bold manual-prod-title", "EPSX Feature Manual" }
                p { class: "mb-8 text-gray-400 manual-prod-subtitle",
                    "Complete guide to all platform features. Screenshots auto-generated from E2E tests."
                }
                aside {
                    class: "mb-8 rounded-lg border border-amber-700/60 bg-amber-950/30 p-4 text-sm text-amber-100",
                    "aria-labelledby": "manual-migration-title",
                    "data-manual-migration-notice": "route-evidence",
                    h2 { class: "mb-1 font-semibold", id: "manual-migration-title", "Route migration status" }
                    p {
                        "This manual preserves the development target catalog and screenshots for migration reference. Descriptions show intended workflows; they do not confirm feature availability, live data, or enabled actions."
                    }
                    p { class: "mt-2",
                        "Badges report route-level migration readiness from the checked contract only. They are not feature, permission, plan, entitlement, authentication, or live runtime checks. Duplicate feature cards that share a route intentionally share its route migration status."
                    }
                }
                for cat in CATEGORIES.iter() {
                    ManualCategorySection { category: cat }
                }
            }
            ManualScreenshotDialog {}
        }
    }
}

#[component]
fn ManualCategorySection(category: &'static str) -> Element {
    let features: Vec<&ManualFeature> =
        FEATURES.iter().filter(|f| f.category == category).collect();
    let id = cat_slug(category);
    let heading_id = format!("{id}-heading");
    rsx! {
        section { class: "manual-prod-category mb-12", id: "{id}", "aria-labelledby": "{heading_id}",
            h2 { class: "manual-prod-category-title manual-category-h2 mb-4 border-b border-gray-800 pb-2 text-xl font-semibold text-white", id: "{heading_id}",
                "{category}"
            }
            div { class: "grid gap-6 sm:grid-cols-2 lg:grid-cols-3 manual-prod-feature-grid",
                for f in features.iter() {
                    ManualFeatureCard { feature: (*f).clone() }
                }
            }
        }
    }
}

#[component]
fn ManualFeatureCard(feature: ManualFeature) -> Element {
    let screenshot_name = feature.screenshots.first().copied().unwrap_or(feature.id);
    let screenshot_src = format!("/public/screenshots/{screenshot_name}.webp");
    let heading_id = format!("manual-feature-{}", feature.id);
    let screenshot_label = format!("Open {} screenshot", feature.name);
    let dynamic_route = feature.route == "/payment/[type]/[id]";
    let route_status = lookup_manual_route_status(feature.route);
    let status_label = route_status
        .map(|status| status.status.label())
        .unwrap_or("Migration status unavailable");
    let status_token = route_status
        .map(|status| status.status.token())
        .unwrap_or("unavailable");
    let status_accessible_label = format!("Route status: {status_label}");
    let action_label = if dynamic_route {
        "Route template only"
    } else if route_status.is_some() {
        "View route"
    } else {
        "Route status unavailable"
    };
    let action_accessible_label = format!("{action_label}: {}", feature.name);
    rsx! {
        article { class: "manual-prod-feature group overflow-hidden rounded-lg border border-gray-800 bg-gray-900/60 transition-colors hover:border-gray-600", "aria-labelledby": "{heading_id}",
            div { class: "manual-prod-screenshot-wrap relative aspect-video w-full overflow-hidden bg-gray-800",
                button {
                    r#type: "button",
                    class: "manual-prod-screenshot-button",
                    "data-manual-screenshot": "true",
                    "data-screenshot-src": "{screenshot_src}",
                    "data-screenshot-alt": "{feature.name}",
                    "aria-label": "{screenshot_label}",
                    "aria-haspopup": "dialog",
                    img {
                        class: "manual-prod-screenshot-img",
                        src: "{screenshot_src}",
                        alt: "{feature.name}",
                        loading: "lazy",
                    }
                    span { class: "manual-prod-screenshot-fallback text-sm", "aria-hidden": "true", "No screenshot" }
                }
            }
            div { class: "p-4 manual-prod-feature-body",
                div { class: "mb-1 flex items-center gap-2 manual-prod-feature-head",
                    h3 { class: "font-medium text-white manual-prod-feature-name", id: "{heading_id}", "{feature.name}" }
                    span { class: "rounded bg-gray-800 px-1.5 py-0.5 text-xs text-gray-400 font-mono manual-prod-feature-route",
                        "{feature.route}"
                    }
                    span {
                        class: "manual-prod-feature-status",
                        "data-manual-route-status": "{status_token}",
                        "aria-label": "{status_accessible_label}",
                        "{status_label}"
                    }
                }
                p { class: "mb-2 text-sm text-gray-400 manual-prod-feature-desc", "{feature.desc}" }
                if dynamic_route || route_status.is_none() {
                    span {
                        class: "text-sm text-gray-400 manual-prod-feature-link",
                        "aria-disabled": "true",
                        "data-route-template": if dynamic_route { "true" } else { "false" },
                        "data-manual-route-action": "{status_token}",
                        "aria-label": "{action_accessible_label}",
                        "{action_label}"
                    }
                } else if let Some(status) = route_status {
                    a {
                        class: "text-sm text-blue-400 hover:text-blue-300 manual-prod-feature-link",
                        href: "{status.target_route}",
                        "aria-disabled": "false",
                        "data-route-template": "false",
                        "data-manual-route-action": "{status_token}",
                        "aria-label": "{action_accessible_label}",
                        "{action_label} →"
                    }
                }
            }
        }
    }
}

#[component]
fn ManualScreenshotDialog() -> Element {
    rsx! {
        div {
            class: "manual-prod-dialog",
            hidden: true,
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "manual-screenshot-dialog-title",
            "data-manual-dialog": "true",
            div { class: "manual-prod-dialog-panel", "data-manual-dialog-panel": "true",
                button {
                    r#type: "button",
                    class: "manual-prod-dialog-close",
                    "data-manual-dialog-close": "true",
                    "aria-label": "Close screenshot",
                    "Close"
                }
                img {
                    class: "manual-prod-dialog-img",
                    "data-manual-dialog-image": "true",
                    alt: "",
                }
                p {
                    class: "manual-prod-dialog-caption",
                    id: "manual-screenshot-dialog-title",
                    "data-manual-dialog-title": "true",
                }
            }
        }
    }
}

fn cat_slug(cat: &str) -> String {
    cat.to_lowercase().replace(' ', "-")
}

fn normalize_manual_route(route: &str) -> &str {
    match route {
        "/payment/[type]/[id]" => "/payment/:type/:id",
        _ => route,
    }
}

fn lookup_manual_route_status(route: &str) -> Option<&'static ManualRouteStatus> {
    let normalized_route = normalize_manual_route(route);
    MANUAL_ROUTE_STATUSES
        .iter()
        .find(|status| status.target_route == normalized_route)
}

// === wave5-page-depth-track-b tests + Wave 25 T2 prod markers ===
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::PageContext;

    fn empty_ctx() -> PageContext {
        PageContext {
            path: "/manual".to_string(),
            ..Default::default()
        }
    }

    fn render_to_string(ctx: &PageContext) -> String {
        let (_meta, el) = render(ctx);
        dioxus_ssr::render_element(el)
    }

    fn render_feature_to_string(feature: &ManualFeature) -> String {
        dioxus_ssr::render_element(rsx! {
            ManualFeatureCard { feature: feature.clone() }
        })
    }

    #[test]
    fn manual_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(
            !html.trim().is_empty(),
            "manual page should render non-empty HTML"
        );
        assert!(
            !html.contains("<main"),
            "manual page fragment must defer its sole main landmark to the shared shell"
        );
    }

    /// Wave 25 T2 — the manual page mirrors the prod Next.js page:
    /// - `bg-gray-950 text-gray-100` dark page background
    /// - sticky `bg-gray-900/50` left sidebar
    /// - `flex` two-column layout
    /// - sidebar links use `hover:bg-gray-800 hover:text-white`
    /// - category h2 has `border-b border-gray-800 pb-2`
    /// - feature cards have `aspect-video` screenshot wrapper
    #[test]
    fn manual_prod_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            "bg-gray-950",
            "text-gray-100",
            "manual-prod-sidebar",
            "sticky top-0 h-screen w-56",
            "bg-gray-900/50",
            "manual-prod-content",
            "hover:bg-gray-800",
            "border-b border-gray-800",
            "aspect-video",
            "data-manual-screenshot",
            "data-manual-dialog",
            "View route",
        ] {
            assert!(
                html.contains(marker),
                "manual page should contain prod marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn manual_section_markers() {
        let html = render_to_string(&empty_ctx());
        for marker in &[
            "manual-prod-sidebar",
            "manual-prod-content",
            "manual-prod-category",
            "public",
            "auth",
            "dashboard",
            "analytics",
            "plans",
            "portfolio",
            "notifications",
            "developer",
        ] {
            assert!(
                html.contains(marker),
                "manual page should contain section marker `{marker}`. Got: {html}"
            );
        }
    }

    #[test]
    fn manual_has_eight_categories() {
        assert_eq!(CATEGORIES.len(), 8, "CATEGORIES array must have 8 entries");
    }

    #[test]
    fn manual_matches_pinned_catalog_and_metadata() {
        let (meta, element) = render(&empty_ctx());
        let html = dioxus_ssr::render_element(element);
        assert_eq!(meta.title, "EPSX Manual - Feature Guide");
        assert_eq!(
            meta.description,
            "Complete feature guide with screenshots for the EPSX platform"
        );
        assert_eq!(FEATURES.len(), 35, "pinned source catalog has 35 features");
        assert_eq!(html.matches("data-manual-screenshot=\"true\"").count(), 35);
        assert_eq!(html.matches("data-route-template=").count(), 35);
        assert!(html.contains("Complete guide to all platform features"));
        assert!(html.contains("/public/screenshots/home.webp"));
        assert!(html.contains("/payment/[type]/[id]"));
    }

    #[test]
    fn manual_frames_badges_as_route_migration_evidence_not_feature_availability() {
        let html = render_to_string(&empty_ctx());
        assert!(html.contains("data-manual-migration-notice=\"route-evidence\""));
        assert!(html.contains("Route migration status"));
        assert!(html.contains("Descriptions show intended workflows"));
        assert!(html.contains("they do not confirm feature availability"));
        assert!(html.contains("route-level migration readiness from the checked contract only"));
        assert!(html.contains("Duplicate feature cards that share a route intentionally share"));
    }

    #[test]
    fn manual_catalog_fingerprint_matches_pinned_source() {
        fn feed(hash: &mut u64, value: &str) {
            for byte in value.bytes() {
                *hash ^= u64::from(byte);
                *hash = hash.wrapping_mul(0x100000001b3);
            }
            *hash ^= 0xff;
            *hash = hash.wrapping_mul(0x100000001b3);
        }

        let mut hash = 0xcbf29ce484222325;
        for category in CATEGORIES {
            feed(&mut hash, category);
        }
        for feature in FEATURES {
            for value in [
                feature.id,
                feature.name,
                feature.desc,
                feature.route,
                feature.screenshots[0],
                feature.category,
            ] {
                feed(&mut hash, value);
            }
        }
        assert_eq!(
            hash, 0x5a932c075bcaa698,
            "catalog fields must match development@373bd231 manual/data.ts"
        );
    }

    #[test]
    fn manual_has_accessible_landmarks_and_safe_route_controls() {
        let html = render_to_string(&empty_ctx());
        assert!(html.contains("aria-label=\"Manual categories\""));
        assert!(html.contains("id=\"manual-content\""));
        assert_eq!(
            html.matches("aria-labelledby=\"public-heading\"").count(),
            1
        );
        assert_eq!(html.matches("role=\"dialog\"").count(), 1);
        assert_eq!(
            html.matches("aria-haspopup=\"dialog\"").count(),
            FEATURES.len()
        );
        assert!(html.contains("data-route-template=\"true\""));
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn manual_features_match_categories() {
        for f in FEATURES.iter() {
            assert!(
                CATEGORIES.contains(&f.category),
                "feature `{}` has unknown category `{}`",
                f.id,
                f.category
            );
        }
    }

    #[test]
    fn every_pinned_feature_has_exactly_one_route_status_mapping() {
        assert_eq!(FEATURES.len(), 35, "pinned source catalog has 35 features");
        assert_eq!(
            FEATURES
                .iter()
                .map(|feature| normalize_manual_route(feature.route))
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            21,
            "pinned features have 21 unique normalized routes"
        );

        for feature in FEATURES {
            let normalized_route = normalize_manual_route(feature.route);
            let matches = MANUAL_ROUTE_STATUSES
                .iter()
                .filter(|status| status.target_route == normalized_route)
                .count();
            assert_eq!(
                matches, 1,
                "feature `{}` route `{}` must have exactly one presentation mapping",
                feature.id, feature.route
            );
        }
    }

    #[test]
    fn route_status_map_is_the_exact_unique_contract_path_set() {
        let expected_routes = [
            "/",
            "/about",
            "/news",
            "/news/:slug",
            "/auth",
            "/account",
            "/account/credits",
            "/profile",
            "/analytics",
            "/dashboard",
            "/portfolio",
            "/permissions",
            "/chat",
            "/chat/:id",
            "/chat/history",
            "/notifications",
            "/developer",
            "/developer/docs",
            "/developer/usage",
            "/manual",
            "/plans",
            "/payment",
            "/payment/:type/:id",
            "/contact",
            "/access-denied",
            "/offline",
            "/privacy",
            "/terms",
        ];
        assert_eq!(MANUAL_ROUTE_STATUSES.len(), expected_routes.len());
        for expected_route in expected_routes {
            assert_eq!(
                MANUAL_ROUTE_STATUSES
                    .iter()
                    .filter(|status| status.target_route == expected_route)
                    .count(),
                1,
                "contract route `{expected_route}` must occur exactly once"
            );
        }

        assert_eq!(
            normalize_manual_route("/payment/[type]/[id]"),
            "/payment/:type/:id"
        );
        assert_eq!(
            normalize_manual_route("/payment/[unknown]/[id]"),
            "/payment/[unknown]/[id]",
            "only the pinned payment template may be normalized"
        );
    }

    #[test]
    fn generated_route_statuses_expose_raw_contract_semantics() {
        let mut aligned = 0;
        let mut partial = 0;
        let mut blocked = 0;
        for status in MANUAL_ROUTE_STATUSES {
            match status.status {
                RouteMigrationStatus::Aligned => {
                    aligned += 1;
                    assert_eq!(status.status.label(), "Migration aligned");
                    assert_eq!(status.status.token(), "aligned");
                }
                RouteMigrationStatus::Partial => {
                    partial += 1;
                    assert_eq!(status.status.label(), "Migration partial");
                    assert_eq!(status.status.token(), "partial");
                }
                RouteMigrationStatus::Blocked => {
                    blocked += 1;
                    assert_eq!(status.status.label(), "Migration blocked");
                    assert_eq!(status.status.token(), "blocked");
                }
            }
        }
        assert_eq!((aligned, partial, blocked), (1, 10, 17));
    }

    #[test]
    fn duplicate_feature_routes_share_one_authoritative_status() {
        for feature in FEATURES {
            let status = lookup_manual_route_status(feature.route)
                .expect("all pinned features must have a status mapping");
            for duplicate in FEATURES
                .iter()
                .filter(|candidate| candidate.route == feature.route)
            {
                assert_eq!(
                    lookup_manual_route_status(duplicate.route),
                    Some(status),
                    "duplicate route `{}` disagrees between `{}` and `{}`",
                    feature.route,
                    feature.id,
                    duplicate.id
                );
            }
        }
    }

    #[test]
    fn rendered_cards_expose_accessible_route_status_labels() {
        let html = render_to_string(&empty_ctx());
        assert_eq!(
            html.matches("class=\"manual-prod-feature-status\"").count(),
            FEATURES.len(),
            "each feature card must expose one route-status badge"
        );
        assert_eq!(
            html.matches("aria-label=\"Route status:").count(),
            FEATURES.len(),
            "each route-status badge must have an explicit accessible label"
        );
        for (label, expected_count) in [
            ("Migration aligned", 1),
            ("Migration partial", 10),
            ("Migration blocked", 24),
        ] {
            assert_eq!(
                html.matches(&format!("aria-label=\"Route status: {label}\""))
                    .count(),
                expected_count,
                "unexpected rendered total for `{label}`"
            );
        }
        for label in [
            "Migration aligned",
            "Migration partial",
            "Migration blocked",
        ] {
            assert!(
                html.contains(&format!("Route status: {label}")),
                "rendered manual must include `{label}` status text"
            );
        }
    }

    #[test]
    fn concrete_actions_are_neutral_and_blocked_descriptions_remain_reference_copy() {
        for feature in FEATURES {
            let status = lookup_manual_route_status(feature.route)
                .expect("all pinned features must have a status mapping");
            let html = render_feature_to_string(feature);
            let action_start = html
                .find("manual-prod-feature-link")
                .expect("card must retain a route-status action");
            let action_tail = &html[action_start..];
            let action_end = ["</a>", "</span>"]
                .iter()
                .filter_map(|closing| action_tail.find(closing))
                .min()
                .map(|offset| action_start + offset)
                .expect("route-status action must close");
            let action = &html[action_start..action_end];
            let expected_action = if feature.route == "/payment/[type]/[id]" {
                "Route template only"
            } else {
                "View route"
            };
            assert!(
                action.contains(expected_action),
                "`{}` card must use its neutral route action",
                feature.id
            );
            assert!(
                !action.contains("Open page"),
                "`{}` card must not claim its page works",
                feature.id
            );
            assert!(
                action.contains(&format!(
                    "data-manual-route-action=\"{}\"",
                    status.status.token()
                )),
                "`{}` action must expose its raw migration status",
                feature.id,
            );
            if feature.route == "/payment/[type]/[id]" {
                assert!(
                    !action.contains("href="),
                    "the dynamic payment template status must be noninteractive"
                );
                assert!(action.contains("aria-disabled=\"true\""));
            } else {
                assert!(action.contains("href="));
                assert!(action.contains("aria-disabled=\"false\""));
            }
            if status.status == RouteMigrationStatus::Blocked {
                assert!(html.contains("manual-prod-feature-desc"));
                assert!(html.contains("Route status: Migration blocked"));
                assert!(!html.contains("Migration status unavailable"));
                assert!(!html.contains("Route status unavailable"));
                if feature.route != "/payment/[type]/[id]" {
                    assert!(
                        action.contains("View route"),
                        "blocked descriptions must not change the neutral route action"
                    );
                }
            }
        }

        let misleading_description = ManualFeature {
            id: "description-is-not-status",
            name: "Description Is Not Status",
            desc: "Available, aligned, live, and ready",
            route: "/",
            screenshots: &["home"],
            category: "Public",
        };
        let html = render_feature_to_string(&misleading_description);
        assert!(html.contains("Available, aligned, live, and ready"));
        assert!(html.contains("Route status: Migration blocked"));
    }

    #[test]
    fn unknown_or_malicious_routes_fail_closed() {
        for route in [
            "/unknown",
            "javascript:alert(1)",
            "/../../account",
            "/payment/[unknown]/[id]",
            "/payment/:type/:id?redirect=https://evil.example",
        ] {
            assert_eq!(
                lookup_manual_route_status(route),
                None,
                "unrecognized route `{route}` must not receive a target"
            );
        }

        let unknown = ManualFeature {
            id: "unknown",
            name: "Unknown",
            desc: "Untrusted test fixture",
            route: "javascript:alert(1)",
            screenshots: &["unknown"],
            category: "Public",
        };
        let html = render_feature_to_string(&unknown);
        assert!(html.contains("Route status: Migration status unavailable"));
        assert!(html.contains("Route status unavailable"));
        assert!(!html.contains("href="));
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(!html.contains("href=\"javascript:"));
        assert!(!html.contains("Open page"));

        let bracketed_unknown = ManualFeature {
            route: "/payment/[unknown]/[id]",
            ..unknown
        };
        let html = render_feature_to_string(&bracketed_unknown);
        assert!(html.contains("Migration status unavailable"));
        assert!(html.contains("Route status unavailable"));
        assert!(!html.contains("Route template only"));
    }
}
