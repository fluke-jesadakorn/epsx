//! `/manual` — feature reference with sticky sidebar + 8 category sections.
//!
//! Wave 25 T2 — ported from
//! `apps-old/frontend/app/manual/{page.tsx, data.ts, screenshot-img.tsx}`
//! to match prod's `bg-gray-950 text-gray-100` dark layout + sticky
//! left sidebar (`bg-gray-900/50`).
//!
//! The source uses `flex` + `aside className="sticky top-0 h-screen
//! w-56 shrink-0 overflow-y-auto border-r border-gray-800
//! bg-gray-900/50 p-4"` + `main className="flex-1 p-8"`. The T2 port
//! matches that layout exactly. Sidebar links use
//! `text-gray-400 hover:bg-gray-800 hover:text-white`. Category
//! `h2` headings have `border-b border-gray-800 pb-2`. Feature cards
//! have `aspect-video` screenshots (we keep the `<img>` fallback for
//! SSR).
//!
//! The CATEGORIES + FEATURES arrays are copied verbatim from
//! `apps-old/frontend/app/manual/data.ts`.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;

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

/// Wave 27 T1 — inline CSS rules for Tailwind v2 CDN arbitrary-value
/// classes that render with slight color differences vs prod's v3+
/// PostCSS pipeline. Force the v3-style colors on the sidebar +
/// category headings so the dev capture pixel-matches prod.
const MANUAL_INLINE_CSS: &str = r#"
.manual-sidebar { background-color: rgba(17, 24, 39, 0.5) !important; }
.manual-sidebar-border { border-color: rgb(31, 41, 55) !important; }
.manual-sidebar-link { color: rgb(156, 163, 175) !important; }
.manual-sidebar-link:hover { background-color: rgb(31, 41, 55) !important; color: rgb(255, 255, 255) !important; }
.manual-category-h2 { border-color: rgb(31, 41, 55) !important; color: rgb(243, 244, 246) !important; }
"#;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Manual");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            // Wave 27 T1 — inject inline CSS so Tailwind v2 CDN renders
            // v3-style colors on the sidebar + category h2.
            style { "{MANUAL_INLINE_CSS}" }
            // Wave 25 T2 — match prod's `bg-gray-950 text-gray-100`
            // + flex layout with sticky sidebar. Prod does NOT show
            // ProgressiveAuthBanner on /manual.
            div { class: "manual-page-prod min-h-screen bg-gray-950 text-gray-100",
                div { class: "flex",
                    ManualSidebar {}
                    ManualContent {}
                }
            }
        }
    })
}

/// Sticky left sidebar with 8 category anchor links — mirrors the
/// prod's `<aside className="sticky top-0 h-screen w-56 shrink-0
/// overflow-y-auto border-r border-gray-800 bg-gray-900/50 p-4">`.
#[component]
fn ManualSidebar() -> Element {
    rsx! {
        aside { class: "manual-prod-sidebar manual-sidebar manual-sidebar-border sticky top-0 h-screen w-56 shrink-0 overflow-y-auto border-r border-gray-800 bg-gray-900/50 p-4",
            h2 { class: "mb-4 text-lg font-semibold text-white manual-prod-sidebar-title", "Categories" }
            nav { class: "flex flex-col gap-1 manual-prod-sidebar-nav",
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

/// Main content pane — matches prod's `<main className="flex-1 p-8">`
/// + `<div className="mx-auto max-w-6xl">`.
#[component]
fn ManualContent() -> Element {
    rsx! {
        main { class: "manual-prod-content flex-1 p-8",
            div { class: "mx-auto max-w-6xl",
                h1 { class: "mb-2 text-3xl font-bold manual-prod-title", "EPSX Feature Manual" }
                p { class: "mb-8 text-gray-400 manual-prod-subtitle",
                    "Complete guide to all platform features. Screenshots auto-generated from E2E tests."
                }
                for cat in CATEGORIES.iter() {
                    ManualCategorySection { category: cat }
                }
            }
        }
    }
}

#[component]
fn ManualCategorySection(category: &'static str) -> Element {
    let features: Vec<&ManualFeature> = FEATURES.iter().filter(|f| f.category == category).collect();
    let id = cat_slug(category);
    rsx! {
        section { class: "manual-prod-category mb-12", id: "{id}",
            h2 { class: "manual-prod-category-title manual-category-h2 mb-4 border-b border-gray-800 pb-2 text-xl font-semibold text-white",
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
    let route_href = if feature.route.contains('[') { "#".to_string() } else { feature.route.to_string() };
    rsx! {
        // === wave25-t2 manual-prod feature-card ===
        div { class: "manual-prod-feature group overflow-hidden rounded-lg border border-gray-800 bg-gray-900/60 transition-colors hover:border-gray-600",
            div { class: "manual-prod-screenshot-wrap relative aspect-video w-full overflow-hidden bg-gray-800",
                img {
                    class: "manual-prod-screenshot-img",
                    src: "{screenshot_src}",
                    alt: "{feature.name}",
                    loading: "lazy",
                }
            }
            div { class: "p-4 manual-prod-feature-body",
                div { class: "mb-1 flex items-center gap-2 manual-prod-feature-head",
                    h3 { class: "font-medium text-white manual-prod-feature-name", "{feature.name}" }
                    span { class: "rounded bg-gray-800 px-1.5 py-0.5 text-xs text-gray-400 font-mono manual-prod-feature-route",
                        "{feature.route}"
                    }
                }
                p { class: "mb-2 text-sm text-gray-400 manual-prod-feature-desc", "{feature.desc}" }
                a {
                    class: "text-sm text-blue-400 hover:text-blue-300 manual-prod-feature-link",
                    href: "{route_href}",
                    "Open page →"
                }
            }
        }
    }
}

fn cat_slug(cat: &str) -> String {
    cat.to_lowercase().replace(' ', "-")
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

    #[test]
    fn manual_renders_smoke() {
        let ctx = empty_ctx();
        let (_meta, el) = render(&ctx);
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "manual page should render non-empty HTML");
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
            "Open page",
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
}