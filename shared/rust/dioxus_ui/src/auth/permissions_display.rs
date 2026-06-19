//! `PermissionsDisplay` — shows the user's permissions, tier, and
//! auth status.
//!
//! Port of `apps-old/frontend/components/auth/permissions-display.tsx`
//! (111 LoC). The TS source is a client component that renders one
//! of three variants: `compact` (tier badge + auth status), `card`
//! (full panel inside a Card), or `detailed` (large layout with
//! user status header + permissions card).
//!
//! The Dioxus port mirrors the three variants. Auth state is
//! provided through the `permissions_display::UserPermissions` data
//! prop (a snapshot of the user's tier / wallet / permissions
//! passed in by the page from the BFF's API response).

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub struct UserPermissionsSnapshot {
    pub wallet_address: Option<String>,
    pub tier: String,
    pub is_authenticated: bool,
    pub permissions: Vec<String>,
}

impl Default for UserPermissionsSnapshot {
    fn default() -> Self {
        Self {
            wallet_address: None,
            tier: "free".to_string(),
            is_authenticated: false,
            permissions: Vec::new(),
        }
    }
}

#[component]
pub fn PermissionsDisplay(
    /// Snapshot of the user's auth state. When
    /// `is_authenticated=false`, the component renders the
    /// "Connect Your Wallet" empty state.
    permissions: UserPermissionsSnapshot,
    /// `"compact"`, `"detailed"` (default), or `"card"`.
    #[props(default = "detailed".to_string())] variant: String,
    /// Whether to render the "Permissions" header on the
    /// card/detailed variant. Ignored in compact variant.
    #[props(default = true)] show_header: bool,
    /// Class names appended to the outer wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    if !permissions.is_authenticated {
        return rsx! {
            div { class: "permissions-display permissions-display-unauth card card-glass {cls}",
                div { class: "card-body",
                    div { class: "flex flex-col items-center gap-3 text-center",
                        div { class: "h-12 w-12 text-slate-400",
                            svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none",
                                stroke: "currentColor", stroke_width: "2", class: "h-12 w-12",
                                path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                            }
                        }
                        h3 { class: "font-semibold text-slate-900 dark:text-slate-100",
                            "Connect Your Wallet"
                        }
                        p { class: "text-sm text-slate-600 dark:text-slate-400",
                            "Connect your wallet to view your permissions and access levels."
                        }
                    }
                }
            }
        };
    }
    match variant.as_str() {
        "compact" => rsx! {
            div { class: "permissions-display permissions-display-compact flex items-center gap-2 {cls}",
                span { class: "permissions-tier-badge permissions-tier-{permissions.tier} inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-orange-500/10 text-orange-500",
                    "{permissions.tier}"
                }
                span { class: "permissions-auth-status permissions-auth-status-online inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-500",
                    "Online"
                }
            }
        },
        "card" => rsx! {
            div { class: "permissions-display permissions-display-card card card-glass {cls}",
                if show_header {
                    div { class: "card-header",
                        div { class: "card-title flex items-center gap-2",
                            span { class: "h-5 w-5 text-orange-500",
                                svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none",
                                    stroke: "currentColor", stroke_width: "2", class: "h-5 w-5",
                                    path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                                }
                            }
                            "Permissions"
                        }
                    }
                }
                div { class: "card-body space-y-6",
                    div { class: "flex items-center justify-between",
                        div { class: "permissions-wallet-display",
                            div { class: "text-slate-900 dark:text-slate-100 font-mono",
                                {permissions.wallet_address.clone().unwrap_or_default()}
                            }
                        }
                        span { class: "permissions-tier-badge permissions-tier-{permissions.tier} inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-orange-500/10 text-orange-500",
                            "{permissions.tier}"
                        }
                    }
                    span { class: "permissions-auth-status permissions-auth-status-online inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-500",
                        "Online"
                    }
                    div { class: "permissions-list",
                        ul { class: "space-y-1",
                            for p in permissions.permissions.iter().take(10) {
                                li { class: "permissions-list-item text-xs font-mono bg-background px-2 py-1 rounded",
                                    "{p}"
                                }
                            }
                        }
                    }
                }
            }
        },
        _ => rsx! {
            div { class: "permissions-display permissions-display-detailed space-y-6 {cls}",
                div { class: "permissions-status-header flex items-center justify-between p-6 bg-gradient-to-r from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 rounded-xl border",
                    div { class: "permissions-wallet-display text-xl font-mono",
                        {permissions.wallet_address.clone().unwrap_or_default()}
                    }
                    div { class: "flex items-center gap-3",
                        span { class: "permissions-tier-badge permissions-tier-{permissions.tier} inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-orange-500/10 text-orange-500",
                            "{permissions.tier}"
                        }
                        span { class: "permissions-auth-status permissions-auth-status-online inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-500",
                            "Online"
                        }
                    }
                }
                div { class: "permissions-card card card-glass",
                    if show_header {
                        div { class: "card-header",
                            div { class: "card-title flex items-center gap-2",
                                span { class: "h-5 w-5 text-orange-500",
                                    svg { xmlns: "http://www.w3.org/2000/svg", view_box: "0 0 24 24", fill: "none",
                                        stroke: "currentColor", stroke_width: "2", class: "h-5 w-5",
                                        path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
                                    }
                                }
                                "Your Permissions"
                            }
                        }
                    }
                    div { class: "card-body",
                        div { class: "permissions-list",
                            ul { class: "space-y-1",
                                for p in permissions.permissions.iter().take(50) {
                                    li { class: "permissions-list-item text-xs font-mono bg-background px-2 py-1 rounded",
                                        "{p}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_permissions_snapshot_default_is_free_unauth() {
        let p = UserPermissionsSnapshot::default();
        assert_eq!(p.tier, "free");
        assert!(!p.is_authenticated);
        assert!(p.permissions.is_empty());
        assert!(p.wallet_address.is_none());
    }

    #[test]
    fn permissions_display_signature_accepts_snapshot() {
        
    }
}
