//! Sub-components extracted from `pages/manual.rs` during
//! Wave 6C Track E (1:1 user-side component parity).
//!
//! Seven named sub-components: `ManualIntro`, `ManualFooterCta`,
//! `ManualSidebar`, `ManualContent`, `ManualCategorySection`,
//! `ManualFeatureCard`, `ScreenshotImg`. Also the `ManualFeature`
//! data type and the `CATEGORIES` + `FEATURES` const arrays (made
//! `pub`).

use crate::primitives::*;
use crate::pages::PageContext;

use dioxus::prelude::*;

/// One feature entry in the manual. Mirrors the `Feature` interface
/// in `apps-old/frontend/app/manual/data.ts`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualFeature {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub route: &'static str,
    pub screenshots: &'static [&'static str],
    pub category: &'static str,
}

/// The 8 manual categories (in sidebar/section order).
pub const CATEGORIES: &[&str] = &[
    "Public",
    "Auth",
    "Dashboard",
    "Analytics",
    "Plans",
    "Portfolio",
    "Notifications",
    "Developer",
];

/// Feature registry — copied verbatim from
/// `apps-old/frontend/app/manual/data.ts` `FEATURES`.
pub const FEATURES: &[ManualFeature] = &[
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

/// Page-level intro — the source's two-paragraph opener.
#[component]
pub fn ManualIntro() -> Element {
    let feature_count = FEATURES.len();
    rsx! {
        section { class: "manual-summary",
            h1 { class: "manual-summary-title", "EPSX Feature Manual" }
            p { class: "manual-summary-subtitle text-muted-foreground",
                "Complete guide to all platform features. Screenshots auto-generated from E2E tests."
            }
            div { class: "manual-summary-meta",
                span { class: "manual-summary-count", "{feature_count} features" }
                span { class: "manual-summary-sep", "·" }
                span { class: "manual-summary-categories", "{CATEGORIES.len()} categories" }
            }
        }
    }
}

/// Bottom CTA — "Need help?" links to `/contact`.
#[component]
pub fn ManualFooterCta() -> Element {
    rsx! {
        section { class: "manual-cta",
            div { class: "card card-glass manual-cta-card",
                div { class: "card-body text-center",
                    h3 { class: "manual-cta-title", "Need help?" }
                    p { class: "manual-cta-subtitle text-muted-foreground",
                        "Can't find a feature? Reach out and we'll add a section."
                    }
                    a { class: "btn btn-primary", href: "/contact", "Contact us" }
                }
            }
        }
    }
}

/// Sticky left sidebar with 8 category anchor links.
#[component]
pub fn ManualSidebar() -> Element {
    rsx! {
        aside { class: "manual-sidebar",
            div { class: "card card-glass manual-sidebar-card",
                div { class: "card-header",
                    h3 { class: "card-title", "Categories" }
                }
                div { class: "card-body",
                    nav { class: "manual-nav",
                        for cat in CATEGORIES.iter() {
                            a {
                                class: "manual-nav-link",
                                href: "#{cat_slug(cat)}",
                                "{cat}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Right pane — one `<details>` block per category.
#[component]
pub fn ManualContent() -> Element {
    rsx! {
        div { class: "manual-content",
            for (idx, cat) in CATEGORIES.iter().enumerate() {
                ManualCategorySection {
                    category: cat,
                    open: idx == 0,
                }
            }
        }
    }
}

#[component]
pub fn ManualCategorySection(category: &'static str, open: bool) -> Element {
    let features: Vec<&ManualFeature> = FEATURES.iter().filter(|f| f.category == category).collect();
    let id = cat_slug(category);
    rsx! {
        section { class: "manual-category", id: "{id}",
            details { class: "manual-category-details", open: open,
                summary { class: "manual-category-summary",
                    h2 { class: "manual-category-title", "{category}" }
                    span { class: "manual-category-count", "{features.len()} features" }
                }
                div { class: "manual-feature-grid",
                    for f in features.iter() {
                        ManualFeatureCard { feature: (*f).clone() }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ManualFeatureCard(feature: ManualFeature) -> Element {
    let screenshot_name = feature.screenshots.first().copied().unwrap_or(feature.id);
    let screenshot_src = format!("/public/screenshots/{screenshot_name}.webp");
    let route_href = if feature.route.contains('[') { "#" } else { feature.route };
    rsx! {
        article { class: "card card-glass manual-feature-card",
            div { class: "manual-feature-screenshot",
                ScreenshotImg { src: screenshot_src, alt: feature.name }
            }
            div { class: "card-body manual-feature-body",
                div { class: "manual-feature-head",
                    h3 { class: "manual-feature-name", "{feature.name}" }
                    span { class: "manual-feature-route", "{feature.route}" }
                }
                p { class: "manual-feature-desc text-muted-foreground text-sm", "{feature.desc}" }
                a { class: "manual-feature-link", href: "{route_href}", "Open page →" }
            }
        }
    }
}

/// Inline `<img>` helper, ported from
/// `apps-old/frontend/app/manual/screenshot-img.tsx`.
#[component]
pub fn ScreenshotImg(src: String, alt: String) -> Element {
    rsx! {
        div { class: "screenshot-img-wrap",
            img {
                class: "screenshot-img",
                src: "{src}",
                alt: "{alt}",
                loading: "lazy",
            }
            div { class: "screenshot-img-fallback", "No screenshot" }
        }
    }
}

/// Slugify a category name for the `id` attribute and the sidebar's
/// `href="#…"` link.
pub fn cat_slug(cat: &str) -> String {
    cat.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wave 6C Track E — `test_render_smoke` for the extracted
    /// manual sub-components.
    #[test]
    fn manual_subcomponents_render_smoke() {
        // ManualIntro
        let el = rsx! { ManualIntro {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("manual-summary"), "ManualIntro missing section-marker");
        assert!(html.contains("EPSX Feature Manual"));

        // ManualFooterCta
        let el = rsx! { ManualFooterCta {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("manual-cta"));
        assert!(html.contains("Contact us"));

        // ManualSidebar
        let el = rsx! { ManualSidebar {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("manual-sidebar"));
        assert!(html.contains("Categories"));

        // ManualContent
        let el = rsx! { ManualContent {} };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("manual-content"));
        assert!(html.contains("Public"));
        assert!(html.contains("Developer"));

        // ScreenshotImg
        let el = rsx! { ScreenshotImg { src: "/test.webp".to_string(), alt: "test".to_string() } };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("screenshot-img"));
    }
}
