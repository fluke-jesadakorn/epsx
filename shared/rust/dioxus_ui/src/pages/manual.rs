//! `/manual` — feature reference with sticky sidebar + 8 category sections.
//!
//! Source of truth: `apps-old/frontend/app/manual/{page.tsx, data.ts,
//! screenshot-img.tsx}`. The original Next.js page renders a sticky
//! left sidebar with 8 category links, a right pane with one section
//! per category, and a card grid of 3–6 feature rows per section. The
//! port keeps the same shape but uses native HTML `<details>` blocks
//! (per the Wave 5 design doc) so each section is independently
//! collapsible. Each feature row mirrors the source: a screenshot,
//! title, route pill, description, and a "Open page →" link.
//!
//! The CATEGORIES and FEATURES arrays are copied verbatim from
//! `apps-old/frontend/app/manual/data.ts` — only the `import type` was
//! dropped (Rust struct below mirrors the `Feature` interface). The
//! screenshot helper is a thin inline `#[component]` that renders
//! `<img loading="lazy" src=… alt=…>` and falls back to a
//! "No screenshot" placeholder when the asset 404s. The BFF serves
//! `apps/frontend/public/` at `/public/`, so the actual asset URL is
//! `/public/screenshots/{name}.webp`.

use crate::primitives::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::auth::ProgressiveAuthBanner;

/// One feature entry in the manual. Mirrors the `Feature` interface in
/// `apps-old/frontend/app/manual/data.ts`. The `screenshots` list is
/// unused in the port — we always render the first screenshot (or the
/// `id` as a fallback), matching the source's `?? feature.id` logic.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ManualFeature {
    id: &'static str,
    name: &'static str,
    desc: &'static str,
    route: &'static str,
    screenshots: &'static [&'static str],
    category: &'static str,
}

/// The 8 manual categories. Order matters — this is the sidebar order
/// and the section order in the right pane. The list is copied
/// verbatim from `apps-old/frontend/app/manual/data.ts` `CATEGORIES`.
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

/// Feature registry — copied verbatim from
/// `apps-old/frontend/app/manual/data.ts` `FEATURES`. Keeping this as
/// a `const` array (vs. fetching from a DB / API) matches the source
/// shape and lets the page be 100% static-rendered.
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

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::marketing("Manual");
    (meta, rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_none() {
                ProgressiveAuthBanner {
                    feature: Some("the manual".to_string()),
                }
            }
            // === wave22-t3-news-blog manual (public page; no AuthGate) ===
            div { class: "manual-page",
                ManualIntro {}
                div { class: "manual-grid container page-content",
                    ManualSidebar {}
                    ManualContent {}
                }
                ManualFooterCta {}
            }
        }
    })
}

/// Page-level intro — the source's two-paragraph opener
/// ("EPSX Feature Manual" + "Complete guide to all platform
/// features. Screenshots auto-generated from E2E tests."). The port
/// keeps the prose identical and adds a small "30 features" badge so
/// the design-doc section-marker test can grep for `manual-summary`
/// too.
#[component]
fn ManualIntro() -> Element {
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

/// Bottom CTA — "Need help?" mirrors the source's lack of an explicit
/// footer, but the design doc calls for the "request access" pattern
/// used on other info pages. We add a single Card linking to
/// `/contact` for consistency with the rest of the info-page family.
#[component]
fn ManualFooterCta() -> Element {
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

/// Sticky left sidebar with 8 category anchor links. The source uses
/// Tailwind `sticky top-0 h-screen overflow-y-auto` — the port maps
/// that to a `manual-sidebar` class (see CSS region in
/// `templates/src/lib.rs`).
#[component]
fn ManualSidebar() -> Element {
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

/// Right pane — one `<details>` block per category, each containing
/// 3–6 feature rows. The first category is open by default to
/// mirror the source's "show the first section" behaviour.
#[component]
fn ManualContent() -> Element {
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
fn ManualCategorySection(category: &'static str, open: bool) -> Element {
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
fn ManualFeatureCard(feature: ManualFeature) -> Element {
    // The source chooses the first screenshot if present, else the
    // feature id. We mirror that exactly.
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
/// `apps-old/frontend/app/manual/screenshot-img.tsx`. SSR is the only
/// rendering mode for the manual page, so the client-side `onError`
/// "No screenshot" fallback is emulated with a sibling placeholder
/// block (the `<img>` itself is wrapped so JS could swap the src on
/// error in a future enhancement).
#[component]
fn ScreenshotImg(src: String, alt: String) -> Element {
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

/// Slugify a category name for the `id` attribute and the
/// sidebar's `href="#…"` link. Mirrors the source's
/// `cat.toLowerCase().replace(/\s+/g, '-')`.
fn cat_slug(cat: &str) -> String {
    cat.to_lowercase().replace(' ', "-")
}

// === wave5-page-depth-track-b ===
// Unit tests for the manual page. The design doc requires:
//   - test_render_smoke: render() returns a non-empty Element
//   - test_section_markers: the rendered HTML contains the 8
//     section-marker class names. We use the `manual-category`
//     section class names and the `manual-sidebar` / `manual-content`
//     wrapper classes as the markers.
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
        // Render to a string and assert non-empty.
        let html = dioxus_ssr::render_element(el);
        assert!(!html.trim().is_empty(), "manual page should render non-empty HTML");
    }

    #[test]
    fn manual_section_markers() {
        let html = render_to_string(&empty_ctx());
        // The 8 expected section markers (one per category).
        for marker in &[
            "manual-sidebar",
            "manual-content",
            "manual-category",
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
                "manual page should contain section marker `{marker}`. Got: {}",
                html
            );
        }
    }

    #[test]
    fn manual_has_eight_categories() {
        assert_eq!(CATEGORIES.len(), 8, "CATEGORIES array must have 8 entries");
    }

    #[test]
    fn manual_features_match_categories() {
        // Every feature's category must be one of the 8 declared.
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
