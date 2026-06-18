//! Admin shared layout helpers — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/shared/page-layout.tsx`,
//! which exports 7 layout/page-state helpers used across every admin
//! page:
//!
//! | Component | Use case |
//! | --- | --- |
//! | `PageLayout` | Container with consistent padding + max-width |
//! | `PageHeader` | Gradient-title page header (icon + title + actions) |
//! | `PageTabs` | Pill-style tab nav (gradient active state) |
//! | `PageSkeleton` | Loading skeleton (header / stats / tabs / rows) |
//! | `PageEmpty` | Empty-state card (icon + headline + CTA) |
//! | `PageError` | Error-state card (with retry CTA) |
//! | `PageAuthRequired` | Connect-wallet CTA |
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling (e.g. active tab state,
//! empty-list behavior, gradient selection).

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// PageLayout
// ============================================================================
//
// Outer container with responsive padding (p-3 sm:p-6 lg:p-8) and
// an optional `max_width` (default 7xl). The inner wrap uses
// `space-y-6` to gap sibling sections consistently.

/// Maximum width preset. Matches the source's 5 variants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageMaxWidth {
    Full,
    SevenXl,
    SixXl,
    FiveXl,
    FourXl,
}

impl Default for PageMaxWidth {
    fn default() -> Self { PageMaxWidth::SevenXl }
}

impl PageMaxWidth {
    pub fn class_name(&self) -> &'static str {
        match self {
            PageMaxWidth::Full => "",
            PageMaxWidth::SevenXl => "max-w-7xl mx-auto",
            PageMaxWidth::SixXl => "max-w-6xl mx-auto",
            PageMaxWidth::FiveXl => "max-w-5xl mx-auto",
            PageMaxWidth::FourXl => "max-w-4xl mx-auto",
        }
    }
}

/// Outer page container. Mirrors `PageLayout` from the source.
#[component]
pub fn PageLayout(
    class_name: Option<String>,
    max_width: Option<PageMaxWidth>,
    children: Element,
) -> Element {
    let mut outer = "p-3 sm:p-6 lg:p-8".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    let inner = max_width.unwrap_or_default().class_name();
    rsx! {
        div { class: "{outer}",
            div { class: "{inner} space-y-6", {children} }
        }
    }
}

// ============================================================================
// PageHeader
// ============================================================================
//
// Gradient-text title with optional icon/emoji + subtitle + actions.
// Mirrors the source's `PageHeader` (icon + gradient + actions
// justified end).

/// Gradient preset for `PageHeader` and `PageTabs` active states.
/// Matches the source's 7 presets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageGradient {
    Primary,
    Success,
    Info,
    Purple,
    Warning,
    Indigo,
    Default,
}

impl Default for PageGradient {
    fn default() -> Self { PageGradient::Default }
}

impl PageGradient {
    pub fn gradient_class(&self) -> &'static str {
        match self {
            PageGradient::Primary => "from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]",
            PageGradient::Success => "from-[#31d0aa] to-[#1fc7d4]",
            PageGradient::Info => "from-[#1fc7d4] to-[#7645d9]",
            PageGradient::Purple => "from-[#7645d9] to-[#ed4b9e]",
            PageGradient::Warning => "from-[#ffb237] to-[#ffb237]",
            PageGradient::Indigo => "from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]",
            PageGradient::Default => "from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]",
        }
    }

    pub fn icon_class(&self) -> &'static str {
        match self {
            PageGradient::Primary | PageGradient::Info | PageGradient::Default => "text-[#1fc7d4]",
            PageGradient::Success => "text-[#31d0aa]",
            PageGradient::Purple | PageGradient::Indigo => "text-[#7645d9]",
            PageGradient::Warning => "text-[#ffb237]",
        }
    }
}

#[component]
pub fn PageHeader(
    title: String,
    subtitle: Option<String>,
    /// Icon name (lucide). Optional.
    icon: Option<String>,
    /// Emoji prefix. Optional.
    emoji: Option<String>,
    gradient: Option<PageGradient>,
    centered: Option<bool>,
    /// Extra action buttons. Rendered via `extra_actions` slot.
    extra_actions: Option<Element>,
    class_name: Option<String>,
) -> Element {
    let g = gradient.unwrap_or_default();
    let g_class = g.gradient_class();
    let icon_color = g.icon_class();
    let centered = centered.unwrap_or(false);
    let mut outer = "mb-6 sm:mb-8".to_string();
    if centered {
        outer.push_str(" text-center");
    }
    if let Some(c) = class_name.clone() {
        outer.push(' ');
        outer.push_str(&c);
    }
    let header_layout = if centered {
        "flex items-start gap-4 justify-center"
    } else if extra_actions.is_some() {
        "flex items-start gap-4 justify-between flex-col sm:flex-row sm:items-center"
    } else {
        "flex items-start gap-4 justify-between"
    };
    rsx! {
        div { class: "{outer}",
            div { class: "{header_layout}",
                div { class: if centered { "flex flex-col items-center" } else { "" },
                    h1 { class: if centered { "flex items-center gap-3 text-3xl sm:text-4xl lg:text-5xl font-bold justify-center" } else { "flex items-center gap-3 text-3xl sm:text-4xl lg:text-5xl font-bold" },
                        if let Some(icon_name) = icon.clone() {
                            span { class: "{icon_color}",
                                Icon { name: icon_name, size: Some(40), class_name: Some("w-8 h-8 sm:w-10 sm:h-10 lg:w-12 lg:h-12".to_string()) }
                            }
                        }
                        if let Some(em) = emoji.clone() {
                            span { class: "text-3xl sm:text-4xl", "{em}" }
                        }
                        span { class: "bg-gradient-to-r bg-clip-text text-transparent {g_class}",
                            "{title}"
                        }
                    }
                    if let Some(sub) = subtitle.clone() {
                        p { class: if centered { "text-sm sm:text-base lg:text-lg text-muted-foreground mt-2 max-w-2xl" } else { "text-sm sm:text-base lg:text-lg text-muted-foreground mt-2" },
                            "{sub}"
                        }
                    }
                }
                if let Some(actions) = extra_actions {
                    div { class: "flex items-center gap-2 shrink-0", {actions} }
                }
            }
        }
    }
}

// ============================================================================
// PageTabs
// ============================================================================
//
// Pill-style tab nav. Mirrors the source's `PageTabs`.

#[derive(Clone, Debug, PartialEq)]
pub struct PageTabItem {
    pub id: String,
    pub label: String,
    /// Optional emoji prefix when no icon is provided.
    pub prefix: Option<String>,
    /// Optional lucide icon name.
    pub icon: Option<String>,
    /// Optional gradient override for the active state.
    pub gradient: Option<PageGradient>,
}

#[component]
pub fn PageTabs(
    tabs: Vec<PageTabItem>,
    active_tab: String,
    on_tab_change: EventHandler<String>,
    class_name: Option<String>,
) -> Element {
    let mut outer = "bg-card p-1.5 rounded-2xl border border-border/20 shadow-xl max-w-2xl mx-auto".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    rsx! {
        div { class: "{outer}",
            div { class: "relative flex gap-1 overflow-x-auto no-scrollbar justify-center",
                for tab in tabs.iter() {
                    {
                        let active = active_tab == tab.id;
                        let grad_cls = tab.gradient.unwrap_or_default().gradient_class();
                        let active_cls = format!("text-white shadow-lg shadow-purple-500/20 bg-gradient-to-r {grad_cls}");
                        let inactive_cls = "text-muted-foreground hover:text-foreground hover:bg-muted/30".to_string();
                        let btn_cls = if active {
                            format!("flex items-center justify-center gap-2 px-8 py-3 rounded-[28px] font-bold text-sm sm:text-base transition-all duration-300 active:scale-95 flex-1 min-w-[120px] {active_cls}")
                        } else {
                            format!("flex items-center justify-center gap-2 px-8 py-3 rounded-[28px] font-bold text-sm sm:text-base transition-all duration-300 active:scale-95 flex-1 min-w-[120px] {inactive_cls}")
                        };
                        let id_owned = tab.id.clone();
                        let label = tab.label.clone();
                        let icon = tab.icon.clone();
                        let prefix = tab.prefix.clone();
                        rsx! {
                            button {
                                key: "{tab.id}",
                                class: "{btn_cls}",
                                r#type: "button",
                                onclick: move |_| on_tab_change.call(id_owned.clone()),
                                if let Some(icon_name) = icon {
                                    Icon { name: icon_name, size: Some(20), class_name: Some("w-4 h-4 sm:w-5 sm:h-5".to_string()) }
                                } else if let Some(p) = prefix {
                                    span { "{p}" }
                                }
                                span { "{label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PageSkeleton
// ============================================================================
//
// Loading skeleton for header / stats / tabs / rows. Mirrors the
// source's `PageSkeleton`.

#[component]
pub fn PageSkeleton(
    /// Number of stat cards to show (0 = skip).
    stats: Option<u32>,
    /// Number of content rows to show (0 = skip).
    rows: Option<u32>,
    /// Show header skeleton (default true).
    show_header: Option<bool>,
    /// Show tabs skeleton (default false).
    show_tabs: Option<bool>,
    /// Number of tab placeholders when `show_tabs` is true.
    tab_count: Option<u32>,
    class_name: Option<String>,
) -> Element {
    let stats = stats.unwrap_or(4);
    let rows = rows.unwrap_or(6);
    let show_header = show_header.unwrap_or(true);
    let show_tabs = show_tabs.unwrap_or(false);
    let tab_count = tab_count.unwrap_or(4);
    let mut outer = "p-3 sm:p-6 lg:p-8".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    let tabs_grid = match tab_count {
        2 => "grid gap-2 grid-cols-2",
        3 => "grid gap-2 grid-cols-3",
        _ => "grid gap-2 grid-cols-4",
    };
    let stats_grid = if stats <= 2 {
        "grid gap-4 sm:gap-6 animate-pulse grid-cols-1 sm:grid-cols-2"
    } else if stats == 3 {
        "grid gap-4 sm:gap-6 animate-pulse grid-cols-1 sm:grid-cols-3"
    } else {
        "grid gap-4 sm:gap-6 animate-pulse grid-cols-2 lg:grid-cols-4"
    };
    rsx! {
        div { class: "{outer}",
            div { class: "max-w-7xl mx-auto space-y-6",
                if show_header {
                    div { class: "mb-6 sm:mb-8 animate-pulse",
                        div { class: "flex items-center gap-3 mb-2",
                            div { class: "w-10 h-10 sm:w-12 sm:h-12 bg-primary rounded-xl" }
                            div { class: "h-10 sm:h-12 bg-gradient-to-r from-primary to-secondary rounded-2xl w-48 sm:w-64" }
                        }
                        div { class: "h-4 sm:h-5 bg-muted rounded-full w-48 sm:w-72 mt-3" }
                    }
                }
                if show_tabs {
                    div { class: "rounded-2xl bg-card border border-border/20 p-2 animate-pulse",
                        div { class: "{tabs_grid}",
                            for i in 0..tab_count {
                                div { key: "tab-skel-{i}", class: "h-12 bg-muted/30 rounded-xl" }
                            }
                        }
                    }
                }
                if stats > 0 {
                    div { class: "{stats_grid}",
                        for i in 0..stats {
                            div { key: "stat-skel-{i}", class: "bg-card rounded-2xl p-4 sm:p-6 border border-border/20",
                                div { class: "flex items-center justify-between mb-3 sm:mb-4",
                                    div { class: "w-10 h-10 bg-gradient-to-br from-purple-500 to-orange-500 rounded-xl" }
                                    div { class: "w-12 h-4 bg-muted rounded-full" }
                                }
                                div { class: "space-y-2",
                                    div { class: "h-8 bg-gradient-to-r from-purple-500 to-orange-500 rounded-lg w-20" }
                                    div { class: "h-4 bg-muted rounded-full w-24" }
                                    div { class: "h-3 bg-muted rounded-full w-16" }
                                }
                                div { class: "sr-only", "stat-skel-{i}" }
                            }
                        }
                    }
                }
                if rows > 0 {
                    div { class: "bg-card rounded-2xl border border-border/20 overflow-hidden animate-pulse",
                        div { class: "p-4 sm:p-6 lg:p-8 space-y-4",
                            for i in 0..rows {
                                div { key: "row-skel-{i}", class: "flex items-center gap-4 p-3 sm:p-4 bg-muted/30 rounded-xl sm:rounded-2xl",
                                    div { class: "w-10 h-10 bg-gradient-to-br from-purple-500 to-orange-500 rounded-xl shrink-0" }
                                    div { class: "flex-1 space-y-2",
                                        div { class: "h-4 bg-muted/30 rounded-lg w-1/3" }
                                        div { class: "h-3 bg-muted/30 rounded-lg w-1/2" }
                                    }
                                    div { class: "h-8 w-20 bg-gradient-to-r from-purple-500 to-orange-500 rounded-full shrink-0" }
                                    div { class: "sr-only", "row-skel-{i}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// PageEmpty
// ============================================================================
//
// Empty-state card with icon (or emoji) + title + message + optional
// CTA. Mirrors the source's `PageEmpty`.

#[component]
pub fn PageEmpty(
    title: Option<String>,
    message: Option<String>,
    emoji: Option<String>,
    icon: Option<String>,
    /// Optional CTA slot.
    extra_action: Option<Element>,
    class_name: Option<String>,
) -> Element {
    let title = title.unwrap_or_else(|| "No data found".to_string());
    let message = message.unwrap_or_else(|| "There are no items to display".to_string());
    let emoji = emoji.unwrap_or_else(|| "\u{1F4ED}".to_string()); // 📭
    let mut outer = "flex flex-col items-center justify-center py-12 sm:py-16 text-center".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    rsx! {
        div { class: "{outer}",
            if let Some(icon_name) = icon {
                Icon { name: icon_name, size: Some(64), class_name: Some("w-16 h-16 text-muted-foreground/50 mb-4".to_string()) }
            } else {
                div { class: "text-5xl sm:text-6xl mb-4", "{emoji}" }
            }
            h3 { class: "text-lg sm:text-xl font-semibold text-foreground mb-2", "{title}" }
            p { class: "text-sm sm:text-base text-muted-foreground max-w-md", "{message}" }
            if let Some(action) = extra_action {
                div { class: "mt-6", {action} }
            }
        }
    }
}

// ============================================================================
// PageError
// ============================================================================
//
// Error-state card with optional retry CTA. Mirrors the source's
// `PageError`.

#[component]
pub fn PageError(
    title: Option<String>,
    message: Option<String>,
    /// Optional retry CTA slot.
    extra_action: Option<Element>,
    class_name: Option<String>,
) -> Element {
    let title = title.unwrap_or_else(|| "Something went wrong".to_string());
    let message = message.unwrap_or_else(|| "An error occurred while loading the data".to_string());
    let mut outer = "flex flex-col items-center justify-center py-12 sm:py-16 text-center".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    rsx! {
        div { class: "{outer}",
            div { class: "w-16 h-16 bg-red-500/10 rounded-2xl flex items-center justify-center mb-4",
                span { class: "text-3xl", "\u{26A0}\u{FE0F}" }
            }
            h3 { class: "text-lg sm:text-xl font-semibold text-foreground mb-2", "{title}" }
            p { class: "text-sm sm:text-base text-muted-foreground max-w-md mb-6", "{message}" }
            if let Some(action) = extra_action {
                {action}
            }
        }
    }
}

// ============================================================================
// PageAuthRequired
// ============================================================================
//
// Auth-required CTA with a connect-wallet button. Mirrors the
// source's `PageAuthRequired`.

#[component]
pub fn PageAuthRequired(
    title: Option<String>,
    message: Option<String>,
    /// Optional custom CTA slot. When `None`, renders the
    /// gradient Connect Wallet button.
    extra_action: Option<Element>,
    class_name: Option<String>,
) -> Element {
    let title = title.unwrap_or_else(|| "Authentication Required".to_string());
    let message = message.unwrap_or_else(|| "Please connect your wallet to access this page.".to_string());
    let mut outer = "flex flex-col items-center justify-center py-16 sm:py-24 text-center".to_string();
    if let Some(c) = class_name {
        outer.push(' ');
        outer.push_str(&c);
    }
    rsx! {
        div { class: "{outer}",
            div { class: "w-24 h-24 bg-gradient-to-br from-[#1fc7d4]/10 to-[#7645d9]/10 rounded-2xl flex items-center justify-center mb-8 border border-border/20 shadow-xl",
                span { class: "text-5xl", "\u{1F510}" }
            }
            h3 { class: "text-3xl sm:text-4xl font-bold bg-gradient-to-r from-[#1fc7d4] to-[#7645d9] bg-clip-text text-transparent mb-4 tracking-tight", "{title}" }
            p { class: "text-lg sm:text-xl text-muted-foreground max-w-md mb-10 font-medium", "{message}" }
            if let Some(action) = extra_action {
                {action}
            } else {
                a {
                    class: "px-10 py-5 bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white rounded-xl font-bold text-xl shadow-lg hover:opacity-90 hover:scale-[1.02] active:scale-95 transition-all",
                    href: "/auth",
                    "Connect Wallet"
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: render a harness fn through VirtualDom + dioxus_ssr.
    /// Matches the pattern in `feedback/admin_action_confirm.rs`.
    fn render_html(harness: fn() -> Element) -> String {
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn harness_page_layout_default() -> Element {
        rsx! { PageLayout { "child" } }
    }
    fn harness_page_layout_full() -> Element {
        rsx! { PageLayout { max_width: PageMaxWidth::Full, "child" } }
    }
    fn harness_page_header() -> Element {
        rsx! { PageHeader { title: "Payments Hub".to_string(), subtitle: Some("Manage payments".to_string()) } }
    }
    fn harness_page_header_emoji() -> Element {
        rsx! { PageHeader { title: "News".to_string(), emoji: Some("\u{1F4F0}".to_string()) } }
    }
    fn harness_page_tabs() -> Element {
        let tabs = vec![
            PageTabItem { id: "all".into(), label: "All".into(), prefix: None, icon: None, gradient: None },
            PageTabItem { id: "draft".into(), label: "Draft".into(), prefix: None, icon: None, gradient: None },
            PageTabItem { id: "published".into(), label: "Published".into(), prefix: None, icon: None, gradient: None },
        ];
        rsx! {
            PageTabs {
                tabs,
                active_tab: "draft".to_string(),
                on_tab_change: move |_id: String| {},
            }
        }
    }
    fn harness_page_skeleton() -> Element {
        rsx! { PageSkeleton { stats: Some(4), rows: Some(3), show_header: Some(true) } }
    }
    fn harness_page_skeleton_zero() -> Element {
        rsx! { PageSkeleton { stats: Some(0), rows: Some(0), show_header: Some(false) } }
    }
    fn harness_page_empty_default() -> Element {
        rsx! { PageEmpty { } }
    }
    fn harness_page_empty_custom() -> Element {
        rsx! { PageEmpty { title: Some("No payments".to_string()), message: Some("Try adjusting filters".to_string()), icon: Some("credit-card".to_string()) } }
    }
    fn harness_page_error() -> Element {
        rsx! { PageError { } }
    }
    fn harness_page_auth_required() -> Element {
        rsx! { PageAuthRequired { } }
    }

    /// `PageLayout` emits the responsive padding + max-width classes.
    #[test]
    fn page_layout_renders_with_default_max_width() {
        let html = render_html(harness_page_layout_default);
        assert!(html.contains("p-3 sm:p-6 lg:p-8"), "PageLayout must emit responsive padding. Got: {html}");
        assert!(html.contains("max-w-7xl mx-auto"), "PageLayout default must be max-w-7xl. Got: {html}");
        assert!(html.contains("space-y-6"), "PageLayout inner must space children. Got: {html}");
    }

    /// `PageLayout` with `Full` max-width omits the inner max-w-* class.
    #[test]
    fn page_layout_renders_full_width() {
        let html = render_html(harness_page_layout_full);
        assert!(!html.contains("max-w-7xl mx-auto"), "PageLayout Full must omit max-w-7xl. Got: {html}");
    }

    /// `PageHeader` emits the gradient text class + the title text.
    #[test]
    fn page_header_renders_gradient_title() {
        let html = render_html(harness_page_header);
        assert!(html.contains("Payments Hub"), "PageHeader must render the title text. Got: {html}");
        assert!(html.contains("Manage payments"), "PageHeader must render subtitle. Got: {html}");
        assert!(html.contains("bg-clip-text"), "PageHeader must use bg-clip-text for gradient. Got: {html}");
        assert!(html.contains("text-transparent"), "PageHeader gradient text must be transparent. Got: {html}");
    }

    /// `PageHeader` with emoji prefix renders the emoji.
    #[test]
    fn page_header_renders_emoji_prefix() {
        let html = render_html(harness_page_header_emoji);
        assert!(html.contains("\u{1F4F0}"), "PageHeader must render emoji prefix. Got: {html}");
    }

    /// `PageTabs` renders all tab labels and marks the active tab.
    #[test]
    fn page_tabs_renders_active_state() {
        let html = render_html(harness_page_tabs);
        assert!(html.contains("All"), "PageTabs must render first tab label. Got: {html}");
        assert!(html.contains("Draft"), "PageTabs must render second tab label. Got: {html}");
        assert!(html.contains("Published"), "PageTabs must render third tab label. Got: {html}");
        assert!(html.contains("shadow-purple-500/20"), "Active tab must have gradient shadow. Got: {html}");
    }

    /// `PageSkeleton` renders the expected number of stat cards + row
    /// placeholders.
    #[test]
    fn page_skeleton_renders_stats_and_rows() {
        let html = render_html(harness_page_skeleton);
        assert!(html.contains("animate-pulse"), "PageSkeleton must be animated. Got: {html}");
        for i in 0..4 {
            assert!(html.contains(&format!("stat-skel-{i}")), "PageSkeleton must render stat-skel-{i}. Got: {html}");
        }
        for i in 0..3 {
            assert!(html.contains(&format!("row-skel-{i}")), "PageSkeleton must render row-skel-{i}. Got: {html}");
        }
    }

    /// `PageSkeleton` with 0 stats and 0 rows renders an empty body.
    #[test]
    fn page_skeleton_renders_zero_stats() {
        let html = render_html(harness_page_skeleton_zero);
        assert!(!html.contains("stat-skel-"), "PageSkeleton with 0 stats must skip stat cards. Got: {html}");
    }

    /// `PageEmpty` renders the title + message + default emoji.
    #[test]
    fn page_empty_renders_default_emoji() {
        let html = render_html(harness_page_empty_default);
        assert!(html.contains("No data found"), "PageEmpty default title. Got: {html}");
        assert!(html.contains("There are no items to display"), "PageEmpty default message. Got: {html}");
        assert!(html.contains("\u{1F4ED}"), "PageEmpty default emoji. Got: {html}");
    }

    /// `PageEmpty` with custom title + icon shows them.
    #[test]
    fn page_empty_renders_custom_title_and_icon() {
        let html = render_html(harness_page_empty_custom);
        assert!(html.contains("No payments"), "PageEmpty custom title. Got: {html}");
        assert!(html.contains("Try adjusting filters"), "PageEmpty custom message. Got: {html}");
        assert!(html.contains("lucide-credit-card") || html.contains("credit-card"), "PageEmpty icon. Got: {html}");
    }

    /// `PageError` renders the default warning copy.
    #[test]
    fn page_error_renders_warning() {
        let html = render_html(harness_page_error);
        assert!(html.contains("Something went wrong"), "PageError default title. Got: {html}");
        assert!(html.contains("bg-red-500/10"), "PageError red bg. Got: {html}");
        assert!(html.contains("\u{26A0}\u{FE0F}"), "PageError warning emoji. Got: {html}");
    }

    /// `PageAuthRequired` renders the connect-wallet CTA.
    #[test]
    fn page_auth_required_renders_connect_cta() {
        let html = render_html(harness_page_auth_required);
        assert!(html.contains("Authentication Required"), "PageAuthRequired default title. Got: {html}");
        assert!(html.contains("Connect Wallet"), "PageAuthRequired CTA. Got: {html}");
        assert!(html.contains("\u{1F510}"), "PageAuthRequired lock emoji. Got: {html}");
    }
}
