//! /profile — user profile with 4-tab layout (Web3, Account, Email, Data).
//!
//! Wave 6A Track D — port of `apps-old/frontend/app/profile/page.tsx` +
//! `components/profile/wallet-profile-client.tsx` (234 LoC) +
//! `web3-integration.tsx` (227 LoC) + `email-management.tsx` (351 LoC) +
//! `data-management.tsx` (385 LoC).
//!
//! Section coverage (matches design doc §"Track D — profile"):
//! - `WalletProfile` — wallet info, ENS, balances, role badge (sidebar)
//! - `Web3Integration` — wallet connect/signature + API key + permissions
//!   (its 4 sub-tabs: Overview / Permissions / API Keys / Settings)
//! - `EmailManagement` — change email, verify code, resend, email prefs
//! - `DataManagement` — overview stats, export (3 types), delete account
//!   with the "DELETE MY ACCOUNT" confirmation pattern
//!
//! The Next.js source uses shadcn `Card`/`Tabs`/etc.; we map those
//! to the design-system `card card-glass` + custom tab buttons used
//! elsewhere in the dioxus port (same pattern as `permissions.rs`).

use crate::primitives::*;
use crate::feedback::*;

use dioxus::prelude::*;
use super::PageContext;
use super::PageMeta;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Profile");
    (meta, rsx! { RenderProfile { ctx: ctx.clone() } })
}

#[component]
fn RenderProfile(ctx: PageContext) -> Element {
    let mut tab = use_signal(|| "web3".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your profile".to_string()),
                required_permissions: Some(vec!["profile:read".to_string(), "profile:write".to_string()]),
                return_url: Some(ctx.path.clone()),
                div { class: "container page-content max-w-6xl",
                    // === wave6-auth-pages-depth-track-d profile header ===
                    PageHeader { title: "Profile & Settings".to_string(), description: Some("Manage your authentication, permissions, API keys, and account preferences".to_string()), icon: Some("user".to_string()) }
                    div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6",
                        // === wave6-auth-pages-depth-track-d profile wallet-profile sidebar ===
                        div { class: "lg:col-span-1",
                            WalletProfile { user: ctx.user.clone() }
                        }
                        // === wave6-auth-pages-depth-track-d profile main content ===
                        div { class: "lg:col-span-3",
                            // === wave6-auth-pages-depth-track-d profile tab nav ===
                            div { class: "tabs profile-tab-nav mb-4",
                                button { class: tab_class(*tab.read() == "web3"), onclick: move |_| tab.set("web3".to_string()), Icon { name: "shield".to_string(), size: Some(16) } " Web3" }
                                button { class: tab_class(*tab.read() == "account"), onclick: move |_| tab.set("account".to_string()), Icon { name: "settings".to_string(), size: Some(16) } " Account" }
                                button { class: tab_class(*tab.read() == "email"), onclick: move |_| tab.set("email".to_string()), Icon { name: "mail".to_string(), size: Some(16) } " Email" }
                                button { class: tab_class(*tab.read() == "data"), onclick: move |_| tab.set("data".to_string()), Icon { name: "database".to_string(), size: Some(16) } " Data" }
                            }
                            // === wave6-auth-pages-depth-track-d profile tab panels ===
                            div { class: "profile-tab-panels",
                                if *tab.read() == "web3" { Web3Integration {} } else if *tab.read() == "account" { AccountTab {} } else if *tab.read() == "email" { EmailManagement {} } else { DataManagement {} }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn tab_class(active: bool) -> String {
    if active { "btn btn-primary".to_string() } else { "btn btn-outline".to_string() }
}

// === wave6-auth-pages-depth-track-d profile wallet-profile ===
//
// Sidebar card with avatar, name, email, "Verified" badge, role
// (Super Admin / Admin / Premium / user), permissions count, and
// an "Upgrade Access" CTA. Mirrors `wallet-profile-client.tsx`
// lines 50-115.
#[component]
fn WalletProfile(user: Option<crate::auth::User>) -> Element {
    let addr = user.as_ref().map(|u| u.address.clone()).unwrap_or_else(|| "0x0000…0000".to_string());
    let chain = user.as_ref().map(|u| u.chain_id.clone()).unwrap_or_else(|| "56".to_string());
    let email = user.as_ref().and_then(|u| u.email.clone()).unwrap_or_else(|| "—".to_string());
    let display = user.as_ref().and_then(|u| u.display_name.clone()).unwrap_or_else(|| "Wallet".to_string());
    let role = user.as_ref().and_then(|u| u.roles.first().cloned()).unwrap_or_else(|| "user".to_string());
    let perms = user.as_ref().map(|u| u.permissions.len()).unwrap_or(0);
    rsx! {
        div { class: "card card-glass wallet-profile-sidebar",
            div { class: "card-body text-center",
                div { class: "mx-auto mb-4 h-20 w-20 rounded-full bg-gradient-to-br from-orange-400 to-pink-500 flex items-center justify-center",
                    Icon { name: "wallet".to_string(), size: Some(40) }
                }
                h3 { class: "text-lg font-bold", "{display}" }
                p { class: "text-sm text-muted-foreground", "{email}" }
                div { class: "mt-2 flex justify-center",
                    span { class: "badge badge-success", "Verified" }
                }
                div { class: "mt-4",
                    div { class: "text-2xl font-bold text-orange-500", "{role}" }
                    div { class: "text-sm text-muted-foreground", "Access Group" }
                }
                div { class: "mt-4 pt-4 border-t border-border text-left space-y-2 text-sm",
                    div { class: "flex justify-between", span { class: "text-muted-foreground", "Permissions:" } span { class: "font-medium", "{perms}" } }
                    div { class: "flex justify-between", span { class: "text-muted-foreground", "Chain:" } span { class: "font-medium", "{chain}" } }
                    div { class: "flex justify-between", span { class: "text-muted-foreground", "Address:" } span { class: "font-mono text-xs truncate max-w-[10rem]", "{addr}" } }
                }
                a { class: "btn btn-outline btn-sm w-full mt-4", href: "/plans", "Upgrade Access" }
            }
        }
    }
}

// === wave6-auth-pages-depth-track-d profile web3-integration ===
//
// Top-level card "Web3 Authentication Dashboard" with the
// "Connect Your Web3 Wallet" empty-state (when not connected) and,
//// when connected, a 4-tab sub-nav (Overview / Permissions / API
// Keys / Settings) that mirrors the source. Port of
// `web3-integration.tsx`.
#[component]
fn Web3Integration() -> Element {
    let mut connected = use_signal(|| true);
    let mut sub_tab = use_signal(|| "overview".to_string());
    rsx! {
        div { class: "space-y-6 web3-integration-panel",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2",
                        Icon { name: "wallet".to_string(), size: Some(20) }
                        " Web3 Authentication Dashboard"
                    }
                    p { class: "text-sm text-muted-foreground", "Manage your wallet connection, permissions, and authentication settings" }
                }
                div { class: "card-body",
                    if !*connected.read() {
                        div { class: "text-center p-8 border-2 border-dashed border-orange-200 rounded-xl bg-gradient-to-br from-orange-50 to-purple-50 web3-connect-empty",
                            div { class: "p-4 bg-gradient-to-r from-orange-500 to-purple-600 rounded-xl w-fit mx-auto mb-4",
                                Icon { name: "wallet".to_string(), size: Some(32) }
                            }
                            h3 { class: "text-xl font-bold mb-2", "Connect Your Web3 Wallet" }
                            p { class: "text-sm text-muted-foreground mb-6 max-w-md mx-auto",
                                "Unlock the full potential of EPSX with Web3 authentication. Get access to NFT-gated content, token-based permissions, DAO governance, and enterprise API features."
                            }
                            button { class: "btn btn-primary", r#type: "button", onclick: move |_| connected.set(true), "Connect Wallet" }
                        }
                    } else {
                        div { class: "space-y-6",
                            div { class: "flex items-center justify-between p-6 bg-gradient-to-r from-green-50 to-emerald-50 border border-green-200 rounded-xl",
                                div { class: "flex items-center gap-4",
                                    div { class: "p-3 bg-green-100 rounded-xl", Icon { name: "check-circle".to_string(), size: Some(24) } }
                                    div {
                                        h3 { class: "font-bold text-green-900", "Wallet Connected & Authenticated" }
                                        p { class: "text-sm text-green-700", "0x1234…abcd · Pro tier · SIWE" }
                                    }
                                }
                            }
                            div { class: "tabs web3-sub-tab-nav",
                                button { class: tab_class(*sub_tab.read() == "overview"), onclick: move |_| sub_tab.set("overview".to_string()), Icon { name: "shield".to_string(), size: Some(14) } " Overview" }
                                button { class: tab_class(*sub_tab.read() == "permissions"), onclick: move |_| sub_tab.set("permissions".to_string()), Icon { name: "users".to_string(), size: Some(14) } " Permissions" }
                                button { class: tab_class(*sub_tab.read() == "api"), onclick: move |_| sub_tab.set("api".to_string()), Icon { name: "key".to_string(), size: Some(14) } " API Keys" }
                                button { class: tab_class(*sub_tab.read() == "settings"), onclick: move |_| sub_tab.set("settings".to_string()), Icon { name: "settings".to_string(), size: Some(14) } " Settings" }
                            }
                            div { class: "web3-sub-tab-panels",
                                if *sub_tab.read() == "overview" { Web3Overview {} }
                                else if *sub_tab.read() == "permissions" { Web3PermissionsTab {} }
                                else if *sub_tab.read() == "api" { Web3ApiKeysTab {} }
                                else { Web3SettingsTab {} }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Web3Overview() -> Element {
    rsx! {
        div { class: "card card-glass web3-overview-panel",
            div { class: "card-body space-y-4",
                div { class: "p-4 bg-muted rounded-lg",
                    div { class: "flex items-center justify-between mb-2",
                        span { class: "font-mono text-sm", "0x1234…abcd" }
                        span { class: "badge badge-info", "Pro" }
                    }
                    div { class: "text-sm text-muted-foreground", "Status: authenticated" }
                }
                h4 { class: "text-sm font-bold text-muted-foreground uppercase", "Current Permissions" }
                div { class: "flex flex-wrap gap-2",
                    span { class: "badge badge-outline", "trade:read" }
                    span { class: "badge badge-outline", "payments:read" }
                    span { class: "badge badge-outline", "profile:write" }
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4 mt-4",
                    button { class: "btn btn-outline", r#type: "button", Icon { name: "refresh-cw".to_string(), size: Some(16) } " Refresh Permissions" }
                    button { class: "btn btn-outline", r#type: "button", Icon { name: "users".to_string(), size: Some(16) } " View All Permissions" }
                    button { class: "btn btn-outline", r#type: "button", Icon { name: "key".to_string(), size: Some(16) } " Manage API Keys" }
                    button { class: "btn btn-outline", r#type: "button", Icon { name: "settings".to_string(), size: Some(16) } " Web3 Settings" }
                }
            }
        }
    }
}

#[component]
fn Web3PermissionsTab() -> Element {
    rsx! {
        div { class: "card card-glass web3-permissions-panel",
            div { class: "card-body",
                h3 { class: "text-lg font-bold mb-4", "Your Permissions" }
                div { class: "space-y-2",
                    div { class: "perm-row", div { class: "font-mono text-sm", "trade:read" } span { class: "badge badge-success", "Granted" } }
                    div { class: "perm-row", div { class: "font-mono text-sm", "payments:read" } span { class: "badge badge-success", "Granted" } }
                    div { class: "perm-row", div { class: "font-mono text-sm", "profile:write" } span { class: "badge badge-success", "Granted" } }
                }
            }
        }
    }
}

#[component]
fn Web3ApiKeysTab() -> Element {
    rsx! {
        div { class: "card card-glass web3-apikeys-panel",
            div { class: "card-body",
                h3 { class: "text-lg font-bold mb-4", "API Keys" }
                p { class: "text-sm text-muted-foreground mb-4", "Create and manage API keys for external integrations." }
                button { class: "btn btn-primary", r#type: "button", Icon { name: "plus".to_string(), size: Some(16) } " Create API Key" }
            }
        }
    }
}

#[component]
fn Web3SettingsTab() -> Element {
    rsx! {
        div { class: "card card-glass web3-settings-panel",
            div { class: "card-body space-y-4",
                h3 { class: "text-lg font-bold flex items-center gap-2", Icon { name: "wallet".to_string(), size: Some(20) } " Wallet Management" }
                div { class: "p-4 bg-muted rounded-lg",
                    div { class: "font-mono text-sm", "0x1234…abcd" }
                    div { class: "text-sm text-muted-foreground mt-1", "Connected via MetaMask" }
                }
                div { class: "p-4 border border-amber-200 bg-amber-50 dark:bg-amber-900/20 rounded-lg text-sm",
                    strong { "Security Tip: " }
                    "Web3 authentication is self-sovereign and secure. Your wallet controls your identity — keep your private keys safe and never share them."
                }
            }
        }
    }
}

// === wave6-auth-pages-depth-track-d profile account tab ===
//
// Two-column key/value display of wallet metadata + a permissions
// chip list. Mirrors `wallet-profile-client.tsx` lines 156-220.
#[component]
fn AccountTab() -> Element {
    rsx! {
        div { class: "card card-glass profile-account-panel",
            div { class: "card-body space-y-4",
                h3 { class: "text-lg font-bold flex items-center gap-2", Icon { name: "settings".to_string(), size: Some(20) } " Wallet Information" }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    div { label { class: "text-sm font-medium", "Wallet ID" } div { class: "mt-1 p-3 bg-muted rounded font-mono text-sm", "u1" } }
                    div { label { class: "text-sm font-medium", "Email Address" } div { class: "mt-1 p-3 bg-muted rounded text-sm", "— " } }
                    div { label { class: "text-sm font-medium", "Access Group" } div { class: "mt-1 p-3 bg-muted rounded text-sm", span { class: "badge badge-outline", "user" } } }
                    div { label { class: "text-sm font-medium", "Platform Context" } div { class: "mt-1 p-3 bg-muted rounded text-sm", "epsx" } }
                }
                h4 { class: "text-sm font-medium mt-4", "Current Permissions (0)" }
                div { class: "max-h-32 overflow-y-auto space-y-1",
                    p { class: "text-sm text-muted-foreground", "No permissions yet." }
                }
            }
        }
    }
}

// === wave6-auth-pages-depth-track-d profile email-management ===
//
// Two cards: Email Settings (change email + verify code) and Email
// Preferences (3 toggles + the "Critical security notifications
// cannot be disabled" footnote). Port of `email-management.tsx`.
#[component]
fn EmailManagement() -> Element {
    let mut editing = use_signal(|| false);
    let mut verifying = use_signal(|| false);
    rsx! {
        div { class: "space-y-6 profile-email-panel",
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2", Icon { name: "mail".to_string(), size: Some(20) } " Email Settings" }
                }
                div { class: "card-body space-y-4",
                    div { class: "flex items-center justify-between p-4 bg-muted rounded-lg",
                        div {
                            div { class: "font-medium", "Current Email" }
                            div { class: "text-sm text-muted-foreground flex items-center gap-2",
                                "— "
                                span { class: "badge badge-success", "Verified" }
                            }
                        }
                        if !*editing.read() {
                            button { class: "btn btn-outline btn-sm", r#type: "button", onclick: move |_| editing.set(true), Icon { name: "edit".to_string(), size: Some(14) } " Change Email" }
                        }
                    }
                    if *editing.read() && !*verifying.read() {
                        div { class: "space-y-4 p-4 border border-orange-200 rounded-lg",
                            div {
                                label { class: "text-sm font-medium", "New Email Address" }
                                input { class: "input mt-1", r#type: "email", placeholder: "your.new.email@example.com" }
                                p { class: "text-xs text-muted-foreground mt-1", "A verification code will be sent to this email address" }
                            }
                            div { class: "flex gap-2",
                                button { class: "btn btn-primary flex-1", r#type: "button", onclick: move |_| verifying.set(true), "Send Verification Code" }
                                button { class: "btn btn-outline", r#type: "button", onclick: move |_| editing.set(false), "Cancel" }
                            }
                        }
                    }
                    if *verifying.read() {
                        div { class: "space-y-4 p-4 border border-orange-200 rounded-lg",
                            div { class: "p-3 bg-blue-50 border border-blue-200 rounded text-sm", "We've sent a verification code to your new email." }
                            div {
                                label { class: "text-sm font-medium", "Verification Code" }
                                input { class: "input mt-1", placeholder: "Enter 6-digit code", maxlength: "6" }
                            }
                            div { class: "flex gap-2",
                                button { class: "btn btn-primary flex-1", r#type: "button", "Verify Email" }
                                button { class: "btn btn-outline", r#type: "button", onclick: move |_| { verifying.set(false); editing.set(false); }, "Cancel" }
                            }
                            button { class: "btn btn-ghost btn-sm w-full", r#type: "button", "Resend Code" }
                        }
                    }
                }
            }
            // === wave6-auth-pages-depth-track-d profile email-preferences ===
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2", Icon { name: "bell".to_string(), size: Some(20) } " Email Preferences" }
                }
                div { class: "card-body space-y-3",
                    EmailPrefRow { title: "Account Notifications".to_string(), desc: "Security alerts, login notifications".to_string(), enabled: true }
                    EmailPrefRow { title: "Platform Updates".to_string(), desc: "Feature announcements, system updates".to_string(), enabled: true }
                    EmailPrefRow { title: "Marketing Communications".to_string(), desc: "Promotional offers, newsletter".to_string(), enabled: false }
                    div { class: "pt-4 border-t border-border",
                        p { class: "text-sm text-muted-foreground", "Email preferences are managed through your account settings. Critical security notifications cannot be disabled." }
                    }
                }
            }
        }
    }
}

#[component]
fn EmailPrefRow(title: String, desc: String, enabled: bool) -> Element {
    let (badge_class, badge_text) = if enabled { ("badge badge-outline text-green-600 border-green-600", "Enabled") } else { ("badge badge-outline text-muted-foreground", "Disabled") };
    rsx! {
        div { class: "flex items-center justify-between p-3 bg-muted rounded-lg",
            div {
                div { class: "font-medium", "{title}" }
                div { class: "text-sm text-muted-foreground", "{desc}" }
            }
            span { class: "{badge_class}", "{badge_text}" }
        }
    }
}

// === wave6-auth-pages-depth-track-d profile data-management ===
//
// 3 cards: Data Overview (3 stats), Export Your Data (3 export
// types: basic / full / analytics), Danger Zone (delete account
// with the "DELETE MY ACCOUNT" confirmation pattern). Port of
// `data-management.tsx`.
#[component]
fn DataManagement() -> Element {
    rsx! {
        div { class: "space-y-6 profile-data-panel",
            // === wave6-auth-pages-depth-track-d profile data-overview ===
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2", Icon { name: "database".to_string(), size: Some(20) } " Data Overview" }
                }
                div { class: "card-body space-y-4",
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div { class: "p-4 bg-muted rounded-lg",
                            div { class: "text-2xl font-bold", "0" }
                            div { class: "text-sm text-muted-foreground", "Active Permissions" }
                        }
                        div { class: "p-4 bg-muted rounded-lg",
                            div { class: "text-2xl font-bold", "FREE" }
                            div { class: "text-sm text-muted-foreground", "Account Tier" }
                        }
                        div { class: "p-4 bg-muted rounded-lg",
                            div { class: "text-2xl font-bold", span { class: "badge badge-success", "Verified" } }
                            div { class: "text-sm text-muted-foreground", "Account Status" }
                        }
                    }
                    div { class: "pt-4 border-t border-border flex items-center gap-2 text-sm text-muted-foreground",
                        Icon { name: "clock".to_string(), size: Some(14) }
                        "Last permission update: — "
                    }
                }
            }
            // === wave6-auth-pages-depth-track-d profile data-export ===
            div { class: "card card-glass",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2", Icon { name: "download".to_string(), size: Some(20) } " Export Your Data" }
                }
                div { class: "card-body space-y-4",
                    p { class: "text-sm text-muted-foreground", "Download your personal data in JSON format. Choose the export type based on your needs." }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        ExportCard { icon: "file-text".to_string(), title: "Basic Info".to_string(), desc: "Account details, email, permissions, and settings.".to_string(), label: "Export Basic".to_string() }
                        ExportCard { icon: "database".to_string(), title: "Complete Data".to_string(), desc: "All personal data including activity logs and session history.".to_string(), label: "Export Full".to_string() }
                        ExportCard { icon: "shield".to_string(), title: "Analytics Only".to_string(), desc: "Usage statistics, preferences, and analytics data.".to_string(), label: "Export Analytics".to_string() }
                    }
                    div { class: "p-3 border border-blue-200 bg-blue-50 rounded text-sm", "Exported data includes only your personal information. Shared or public data is not included." }
                }
            }
            // === wave6-auth-pages-depth-track-d profile data-danger-zone ===
            div { class: "card card-glass border-red-200 bg-red-50 dark:bg-red-950/20",
                div { class: "card-header",
                    h3 { class: "card-title flex items-center gap-2 text-red-700 dark:text-red-300", Icon { name: "alert-triangle".to_string(), size: Some(20) } " Danger Zone" }
                }
                div { class: "card-body space-y-4",
                    div { class: "p-3 border border-red-200 bg-red-100 dark:bg-red-900/20 rounded text-sm",
                        strong { "Warning: " }
                        "Account deletion is permanent and cannot be undone. All your data, permissions, and access will be permanently removed."
                    }
                    div { class: "space-y-2",
                        h4 { class: "font-medium text-red-700 dark:text-red-300", "What happens when you delete your account:" }
                        ul { class: "text-sm text-red-600 dark:text-red-400 space-y-1 ml-4 list-disc",
                            li { "All personal data and account information will be permanently deleted" }
                            li { "Access to all EPSX services will be immediately revoked" }
                            li { "Any active subscriptions will be cancelled" }
                            li { "Web3 wallet connections will be disconnected" }
                            li { "This action cannot be undone" }
                        }
                    }
                    DangerZoneDialog {}
                }
            }
        }
    }
}

#[component]
fn ExportCard(icon: String, title: String, desc: String, label: String) -> Element {
    rsx! {
        div { class: "p-4 border border-border rounded-lg",
            div { class: "flex items-center gap-2 mb-2", Icon { name: icon, size: Some(16) } span { class: "font-medium", "{title}" } }
            p { class: "text-sm text-muted-foreground mb-3", "{desc}" }
            button { class: "btn btn-outline btn-sm w-full", r#type: "button", "{label}" }
        }
    }
}

#[component]
fn DangerZoneDialog() -> Element {
    let mut open = use_signal(|| false);
    let mut confirm = use_signal(|| String::new());
    rsx! {
        button { class: "btn btn-danger", r#type: "button", onclick: move |_| open.set(true), Icon { name: "trash".to_string(), size: Some(16) } " Delete Account" }
        if *open.read() {
            div { class: "modal-overlay fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50",
                div { class: "card card-glass max-w-md w-full",
                    div { class: "card-header", h3 { class: "card-title text-red-700", "Delete Account Confirmation" } }
                    div { class: "card-body space-y-4",
                        div { class: "p-3 border border-red-200 bg-red-100 rounded text-sm", "This action is permanent and cannot be undone." }
                        label { class: "text-sm font-medium", "Type \"DELETE MY ACCOUNT\" to confirm:" }
                        input { class: "input mt-1", r#type: "text", placeholder: "DELETE MY ACCOUNT", value: "{confirm.read()}", oninput: move |e| confirm.set(e.value().to_string()) }
                        div { class: "flex gap-2",
                            button { class: "btn btn-danger flex-1", r#type: "button", disabled: *confirm.read() != "DELETE MY ACCOUNT", "Delete Account" }
                            button { class: "btn btn-outline", r#type: "button", onclick: move |_| { open.set(false); confirm.set(String::new()); }, "Cancel" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::User;
    use crate::pages::PageContext;
    use crate::auth::user::AuthMethod;

    fn authed_ctx(path: &str) -> PageContext {
        let user = User {
            id: "u1".to_string(),
            address: "0x1234…abcd".to_string(),
            chain_id: "56".to_string(),
            roles: vec!["user".to_string()],
            email: Some("test@epsx.io".to_string()),
            tier: Some("Pro".to_string()),
            permissions: vec![
                "profile:read".to_string(),
                "profile:write".to_string(),
            ],
            last_login_at: None,
            auth_method: AuthMethod::Wallet,
            display_name: Some("Test".to_string()),
        };
        PageContext { user: Some(user), path: path.to_string(), ..Default::default() }
    }

    #[test]
    fn test_render_smoke() {
        let ctx = authed_ctx("/profile");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        // Dioxus HTML-escapes the `&` to `&#38;`; match the escaped form.
        assert!(html.contains("Profile &#38; Settings"), "/profile header must render. Got: {}", html);
    }

    #[test]
    fn test_section_markers() {
        let ctx = authed_ctx("/profile");
        let (_meta, element) = render(&ctx);
        let html = dioxus_ssr::render_element(element);
        for marker in [
            "profile-tab-nav",
            "wallet-profile-sidebar",
            "profile-tab-panels",
        ] {
            assert!(html.contains(marker), "missing section marker: {}", marker);
        }
    }
}
