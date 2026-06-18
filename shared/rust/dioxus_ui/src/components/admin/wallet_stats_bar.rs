//! Admin `WalletStatsBar` (renamed `AdminWalletStatsBar` to avoid
//! collision with the inline `fn WalletStatsBar` in
//! `pages::admin_pages::wallet_wallets`) — Wave 38a T1 admin wallet
//! domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/wallet/wallet-stats-bar.tsx`.
//! Renders 4 stat cards (Total / Active / Disabled / Subscribed) +
//! a platform-distribution panel.
//!
//! ## Visual layout
//!
//! - 4-column grid (`grid-cols-1 sm:grid-cols-2 lg:grid-cols-4`)
//!   of stat cards (icon + value + label + optional change pill).
//! - Below the grid, a `PlatformDistribution` panel with 4
//!   progress bars (analytics / pay / token / markets).
//!
//! ## Loading state
//!
//! When `is_loading = true`, a 5-column skeleton placeholder is
//! rendered instead of the real data (matches prod's
//! `StatsBarSkeleton`).
//!
//! ## Tests
//!
//! `test_admin_wallet_stats_bar_renders_4_cards` — the 4 stat
//! labels render.
//! `test_admin_wallet_stats_bar_renders_platform_distribution` —
//! the platform-distribution panel + 4 platform labels render.
//! `test_admin_wallet_stats_bar_loading_renders_skeleton` — the
//! skeleton emits 5 `animate-pulse` placeholders.
//! `test_admin_wallet_stats_bar_change_pill_green_for_positive` /
//! `_red_for_negative` — the change pill switches color based on
//! sign.

use dioxus::prelude::*;

/// Stats payload for `AdminWalletStatsBar`. Mirrors the prod
/// `WalletStats` type.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletStatsData {
    pub total: i64,
    pub active: i64,
    pub disabled: i64,
    pub subscribed: i64,
    pub changes: WalletStatsChanges,
    pub platform_distribution: WalletPlatformDistribution,
}

/// 7-day delta for each stat category.
#[derive(Clone, Debug, PartialEq)]
pub struct WalletStatsChanges {
    pub total: i64,
    pub active: i64,
    pub disabled: i64,
    pub subscribed: i64,
}

/// Per-platform counts. Matches the prod
/// `Record<Platform, number>` shape (analytics / pay / token /
/// markets).
#[derive(Clone, Debug, PartialEq)]
pub struct WalletPlatformDistribution {
    pub analytics: i64,
    pub pay: i64,
    pub token: i64,
    pub markets: i64,
}

impl WalletPlatformDistribution {
    pub fn total(&self) -> i64 {
        self.analytics + self.pay + self.token + self.markets
    }
}

/// Platform descriptor (label + emoji + color used for the
/// progress bar).
struct PlatformDescriptor {
    key: &'static str,
    label: &'static str,
    emoji: &'static str,
    color: &'static str,
}

const PLATFORMS: &[PlatformDescriptor] = &[
    PlatformDescriptor {
        key: "analytics",
        label: "Analytics",
        emoji: "📊",
        color: "bg-[#1fc7d4]",
    },
    PlatformDescriptor {
        key: "pay",
        label: "Pay",
        emoji: "💳",
        color: "bg-[#7645d9]",
    },
    PlatformDescriptor {
        key: "token",
        label: "Token",
        emoji: "🪙",
        color: "bg-[#ffb237]",
    },
    PlatformDescriptor {
        key: "markets",
        label: "Markets",
        emoji: "📈",
        color: "bg-[#31d0aa]",
    },
];

/// Build the platform-distribution panel as inline content. Used
/// both inside `AdminWalletStatsBar` and as a public helper for
/// callers that want just the panel.
#[component]
pub fn PlatformDistributionPanel(
    distribution: WalletPlatformDistribution,
    total: i64,
) -> Element {
    rsx! {
        div { class: "rounded-xl bg-card border border-border/20 p-5 shadow-sm",
            h4 { class: "text-sm font-semibold text-foreground/80 mb-4",
                "Platform distribution"
            }
            div { class: "space-y-3",
                for p in PLATFORMS.iter() {
                    {
                        let count = match p.key {
                            "analytics" => distribution.analytics,
                            "pay" => distribution.pay,
                            "token" => distribution.token,
                            "markets" => distribution.markets,
                            _ => 0,
                        };
                        let percentage = if total > 0 { ((count as f64 / total as f64) * 100.0).round() as i64 } else { 0 };
                        let bar_color = p.color.to_string();
                        let emoji = p.emoji.to_string();
                        let label = p.label.to_string();
                        rsx! {
                            div {
                                div { class: "flex items-center justify-between text-sm mb-1",
                                    span { class: "flex items-center gap-2 text-muted-foreground",
                                        span { "{emoji}" }
                                        "{label}"
                                    }
                                    span { class: "font-medium text-foreground",
                                        "{count} ({percentage}%)"
                                    }
                                }
                                div { class: "h-2 bg-muted rounded-full overflow-hidden",
                                    div {
                                        class: "h-full rounded-full transition-all duration-500 {bar_color}",
                                        style: "width:{percentage}%;",
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Single stat card (icon + value + label + optional change pill).
#[component]
fn AdminStatCard(
    label: String,
    value: i64,
    change: i64,
    /// Icon glyph rendered on the left (emoji or HTML).
    icon: String,
    /// Optional 7-day delta label (defaults to "7d").
    change_label: Option<String>,
) -> Element {
    let has_positive = change > 0;
    let has_negative = change < 0;
    let pill_cls = if has_positive {
        "flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-[#31d0aa]/10 text-[#31d0aa]"
    } else if has_negative {
        "flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-destructive/10 text-destructive"
    } else {
        "flex items-center gap-1 text-xs font-medium px-2 py-1 rounded-full bg-muted text-muted-foreground"
    };
    let arrow = if has_positive { "▲" } else if has_negative { "▼" } else { "" };
    let change_label = change_label.unwrap_or_else(|| "7d".to_string());
    rsx! {
        div { class: "rounded-xl bg-card border border-border/20 p-5 shadow-sm",
            div { class: "flex items-center justify-between",
                div { class: "text-muted-foreground", "{icon}" }
                div { class: "{pill_cls}",
                    span { "{arrow}" }
                    span { "{change} ({change_label})" }
                }
            }
            div { class: "mt-3",
                p { class: "text-3xl font-bold text-foreground", "{value}" }
                p { class: "text-sm text-muted-foreground mt-1", "{label}" }
            }
        }
    }
}

/// Skeleton placeholder used while the stats payload is loading.
#[component]
fn AdminStatsBarSkeleton() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5 gap-4",
            for i in 0..5 {
                div {
                    key: "{i}",
                    class: "rounded-2xl bg-muted p-5 animate-pulse",
                    div { class: "flex items-center justify-between mb-3",
                        div { class: "h-6 w-6 rounded bg-muted/80" }
                        div { class: "h-5 w-16 rounded-full bg-muted/80" }
                    }
                    div { class: "h-8 w-20 rounded bg-muted/80 mb-2" }
                    div { class: "h-4 w-24 rounded bg-muted/80" }
                }
            }
        }
    }
}

/// Wallet stats dashboard bar. Renamed `AdminWalletStatsBar` to
/// avoid collision with the inline `fn WalletStatsBar` in
/// `wallet_wallets.rs`. Migration: delete the inline fn and `use
/// crate::components::admin::AdminWalletStatsBar`.
#[component]
pub fn AdminWalletStatsBar(
    stats: WalletStatsData,
    /// When true, render the skeleton placeholder instead of the
    /// real data.
    is_loading: Option<bool>,
    /// Optional extra classes appended to the outer wrapper.
    class_name: Option<String>,
) -> Element {
    let is_loading = is_loading.unwrap_or(false);
    let mut cls = "space-y-4".to_string();
    if let Some(extra) = class_name {
        if !extra.is_empty() {
            cls.push(' ');
            cls.push_str(&extra);
        }
    }
    if is_loading {
        return rsx! { AdminStatsBarSkeleton {} };
    }
    let total = stats.platform_distribution.total();
    let changes = stats.changes.clone();
    let distribution = stats.platform_distribution.clone();
    rsx! {
        div { class: "{cls}",
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4",
                AdminStatCard {
                    label: "Total wallets".to_string(),
                    value: stats.total,
                    change: changes.total,
                    icon: "👛".to_string(),
                }
                AdminStatCard {
                    label: "Active".to_string(),
                    value: stats.active,
                    change: changes.active,
                    icon: "👥".to_string(),
                }
                AdminStatCard {
                    label: "Disabled".to_string(),
                    value: stats.disabled,
                    change: changes.disabled,
                    icon: "⚠️".to_string(),
                }
                AdminStatCard {
                    label: "Subscribed".to_string(),
                    value: stats.subscribed,
                    change: changes.subscribed,
                    icon: "📦".to_string(),
                }
            }
            PlatformDistributionPanel {
                distribution: distribution,
                total: total,
            }
        }
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> WalletStatsData {
        WalletStatsData {
            total: 100,
            active: 80,
            disabled: 20,
            subscribed: 35,
            changes: WalletStatsChanges { total: 5, active: 3, disabled: -2, subscribed: 1 },
            platform_distribution: WalletPlatformDistribution {
                analytics: 40,
                pay: 30,
                token: 20,
                markets: 10,
            },
        }
    }

    /// The 4 stat labels render inside the bar.
    #[test]
    fn test_admin_wallet_stats_bar_renders_4_cards() {
        let el = rsx! {
            AdminWalletStatsBar {
                stats: sample_stats(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(html.contains("Total wallets"), "should render Total wallets. Got: {html}");
        assert!(html.contains("Active"), "should render Active label. Got: {html}");
        assert!(html.contains("Disabled"), "should render Disabled label. Got: {html}");
        assert!(html.contains("Subscribed"), "should render Subscribed label. Got: {html}");
        assert!(html.contains("100"), "should render total value. Got: {html}");
        assert!(html.contains("80"), "should render active value. Got: {html}");
        assert!(html.contains("20"), "should render disabled value. Got: {html}");
        assert!(html.contains("35"), "should render subscribed value. Got: {html}");
    }

    /// The platform-distribution panel renders the 4 platform
    /// labels.
    #[test]
    fn test_admin_wallet_stats_bar_renders_platform_distribution() {
        let el = rsx! {
            AdminWalletStatsBar {
                stats: sample_stats(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Platform distribution"),
            "should render the platform-distribution heading. Got: {html}"
        );
        assert!(html.contains("Analytics"), "should render Analytics. Got: {html}");
        assert!(html.contains("Pay"), "should render Pay. Got: {html}");
        assert!(html.contains("Token"), "should render Token. Got: {html}");
        assert!(html.contains("Markets"), "should render Markets. Got: {html}");
        // 40% analytics of 100 total.
        assert!(html.contains("40 (40%)"), "should render analytics percent. Got: {html}");
    }

    /// Loading state renders the 5-skeleton placeholder.
    #[test]
    fn test_admin_wallet_stats_bar_loading_renders_skeleton() {
        let el = rsx! {
            AdminWalletStatsBar {
                stats: sample_stats(),
                is_loading: Some(true),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("animate-pulse"),
            "Loading state should emit animate-pulse. Got: {html}"
        );
        // Skeleton should NOT render real labels.
        assert!(
            !html.contains("Total wallets"),
            "Skeleton should not render stat labels. Got: {html}"
        );
    }

    /// Positive change uses the green pill color.
    #[test]
    fn test_admin_wallet_stats_bar_change_pill_green_for_positive() {
        let el = rsx! {
            AdminStatCard {
                label: "Active".to_string(),
                value: 80,
                change: 3,
                icon: "👥".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-[#31d0aa]/10"),
            "Positive change should use the green pill. Got: {html}"
        );
        assert!(html.contains("3 (7d)"), "Should render change value. Got: {html}");
    }

    /// Negative change uses the destructive pill color.
    #[test]
    fn test_admin_wallet_stats_bar_change_pill_red_for_negative() {
        let el = rsx! {
            AdminStatCard {
                label: "Disabled".to_string(),
                value: 20,
                change: -2,
                icon: "⚠️".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-destructive/10"),
            "Negative change should use destructive pill. Got: {html}"
        );
    }

    /// Zero change uses the muted pill color.
    #[test]
    fn test_admin_wallet_stats_bar_change_pill_muted_for_zero() {
        let el = rsx! {
            AdminStatCard {
                label: "Total".to_string(),
                value: 100,
                change: 0,
                icon: "👛".to_string(),
            }
        };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("bg-muted"),
            "Zero change should use muted pill. Got: {html}"
        );
    }

    /// `WalletPlatformDistribution::total` sums all 4 platforms.
    #[test]
    fn test_platform_distribution_total() {
        let d = WalletPlatformDistribution { analytics: 10, pay: 20, token: 30, markets: 40 };
        assert_eq!(d.total(), 100);
    }
}
