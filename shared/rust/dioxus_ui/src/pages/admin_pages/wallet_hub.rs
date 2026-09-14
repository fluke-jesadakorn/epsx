//! Shared production-shaped wrapper for every wallet-management workspace.
//!
//! The deployed admin application renders one parent hub above Wallets,
//! Access, Credits, Plans, and wallet detail pages.  Keep that information
//! architecture here while accepting only backend-authoritative aggregate
//! values.  Subscription-derived counts remain explicitly unavailable until
//! their owning service exposes a typed summary projection.

use dioxus::prelude::*;

use crate::components::admin::page_layout::{PageGradient, PageHeader, PageLayout, PageMaxWidth};
use crate::primitives::Icon;

use super::super::PageContext;
use super::wallet_wallets::{
    decode_admin_wallet_stats_projection, ADMIN_WALLET_STATS_DATA_PARAM, ADMIN_WALLET_STATS_READY,
    ADMIN_WALLET_STATS_STATE_PARAM,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct WalletHubMetrics {
    total: String,
    active: String,
    disabled: String,
}

fn metrics(ctx: &PageContext) -> WalletHubMetrics {
    let projection = (ctx
        .params
        .get(ADMIN_WALLET_STATS_STATE_PARAM)
        .map(String::as_str)
        == Some(ADMIN_WALLET_STATS_READY))
    .then(|| ctx.params.get(ADMIN_WALLET_STATS_DATA_PARAM))
    .flatten()
    .and_then(|raw| serde_json::from_str(raw).ok())
    .and_then(decode_admin_wallet_stats_projection);

    projection.map_or_else(
        || WalletHubMetrics {
            total: "Unavailable".to_string(),
            active: "Unavailable".to_string(),
            disabled: "Unavailable".to_string(),
        },
        |projection| WalletHubMetrics {
            total: format_count(projection.total_users),
            active: format_count(projection.active_users),
            disabled: format_count(projection.inactive_users),
        },
    )
}

/// Production-shaped wallet-management parent layout.
#[component]
pub fn WalletManagementHub(ctx: PageContext, children: Element) -> Element {
    let metrics = metrics(&ctx);

    rsx! {
        PageLayout {
            max_width: Some(PageMaxWidth::SevenXl),
            PageHeader {
                title: "Wallet Management Hub".to_string(),
                subtitle: Some("Unified management for EPSX ecosystem wallets, permissions, and subscriptions".to_string()),
                icon: Some("wallet".to_string()),
                gradient: Some(PageGradient::Info),
                centered: Some(false),
                extra_actions: None,
                class_name: None,
            }
            section {
                class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5",
                aria_label: "Wallet management metrics",
                WalletHubMetric {
                    label: "Total Wallets".to_string(),
                    value: metrics.total,
                    detail: "Registered records".to_string(),
                    icon: "wallet".to_string(),
                    color: "cyan".to_string(),
                }
                WalletHubMetric {
                    label: "Active Users".to_string(),
                    value: metrics.active,
                    detail: "Backend status".to_string(),
                    icon: "users".to_string(),
                    color: "green".to_string(),
                }
                WalletHubMetric {
                    label: "Disabled".to_string(),
                    value: metrics.disabled,
                    detail: "Attention".to_string(),
                    icon: "triangle-alert".to_string(),
                    color: "pink".to_string(),
                }
                WalletHubMetric {
                    label: "Subscribed".to_string(),
                    value: "Unavailable".to_string(),
                    detail: "Subscription-owned".to_string(),
                    icon: "layers".to_string(),
                    color: "purple".to_string(),
                }
                WalletHubMetric {
                    label: "Expiring".to_string(),
                    value: "Unavailable".to_string(),
                    detail: "Subscription-owned".to_string(),
                    icon: "clock".to_string(),
                    color: "amber".to_string(),
                }
            }
            div { class: "pb-12", {children} }
        }
    }
}

#[component]
fn WalletHubMetric(
    label: String,
    value: String,
    detail: String,
    icon: String,
    color: String,
) -> Element {
    let icon_color = match color.as_str() {
        "green" => "text-[#31d0aa]",
        "pink" => "text-[#ed4b9e]",
        "purple" => "text-[#7645d9]",
        "amber" => "text-[#ffb237]",
        _ => "text-[#1fc7d4]",
    };
    let border_style = match color.as_str() {
        "green" => "border-color: rgba(49, 208, 170, 0.18);",
        "pink" => "border-color: rgba(237, 75, 158, 0.18);",
        "purple" => "border-color: rgba(118, 69, 217, 0.2);",
        "amber" => "border-color: rgba(255, 178, 55, 0.2);",
        _ => "border-color: rgba(31, 199, 212, 0.18);",
    };

    rsx! {
        article { class: "rounded-2xl border border-border/30 bg-card p-0.5 shadow-xl", style: "{border_style}",
            div { class: "flex h-full min-h-36 flex-col justify-between rounded-2xl p-5",
                div { class: "flex h-9 w-9 items-center justify-center rounded-lg bg-muted/50 {icon_color}",
                    Icon { name: icon, size: Some(20) }
                }
                div { class: "mt-4 min-w-0",
                    p { class: "break-words text-xl font-bold tracking-tight text-card-foreground", "{value}" }
                    div { class: "mt-1 flex items-center justify-between gap-2",
                        span { class: "text-xs font-medium text-muted-foreground", "{label}" }
                        span { class: "rounded bg-muted/50 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground", "{detail}" }
                    }
                }
            }
        }
    }
}

fn format_count(value: i64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    let first_group = digits.len() % 3;
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && index % 3 == first_group {
            formatted.push(',');
        }
        formatted.push(char::from(byte));
    }
    formatted
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn hub_keeps_unowned_subscription_metrics_explicitly_unavailable() {
        let mut params = HashMap::new();
        params.insert(
            ADMIN_WALLET_STATS_STATE_PARAM.to_string(),
            ADMIN_WALLET_STATS_READY.to_string(),
        );
        params.insert(
            ADMIN_WALLET_STATS_DATA_PARAM.to_string(),
            serde_json::json!({
                "total_users": 1234,
                "active_users": 1200,
                "inactive_users": 34,
                "new_users_30_days": 10,
            })
            .to_string(),
        );
        let rendered = dioxus_ssr::render_element(rsx! {
            WalletManagementHub {
                ctx: PageContext { params, ..Default::default() },
                p { "Wallet child" }
            }
        });

        assert!(rendered.contains("Wallet Management Hub"));
        assert!(rendered.contains("1,234"));
        assert!(rendered.contains("1,200"));
        assert!(rendered.contains("Subscribed"));
        assert!(rendered.contains("Subscription-owned"));
        assert!(!rendered.contains("12.4%"));
    }
}
