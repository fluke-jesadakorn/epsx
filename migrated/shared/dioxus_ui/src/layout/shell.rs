use dioxus::prelude::*;
use super::sidebar::{Sidebar, SidebarItem};

#[component]
pub fn DashboardShell(current_path: String, page_title: String, user: Option<crate::auth::User>, children: Element) -> Element {
    let items = vec![
        SidebarItem { label: "Dashboard".into(), href: "/".into(), icon: "layout-dashboard".into(), badge: None, active_paths: vec!["/".into()] },
        SidebarItem { label: "Analytics".into(), href: "/analytics".into(), icon: "chart-line".into(), badge: None, active_paths: vec!["/analytics".into()] },
        SidebarItem { label: "Audit Log".into(), href: "/audit-log".into(), icon: "history".into(), badge: None, active_paths: vec!["/audit-log".into()] },
        SidebarItem { label: "Wallets".into(), href: "/wallet-management/wallets".into(), icon: "wallet".into(), badge: None, active_paths: vec!["/wallet-management".into()] },
        SidebarItem { label: "Credits".into(), href: "/wallet-management/credits".into(), icon: "credit-card".into(), badge: None, active_paths: vec!["/wallet-management/credits".into()] },
        SidebarItem { label: "Access".into(), href: "/wallet-management/access".into(), icon: "key".into(), badge: None, active_paths: vec!["/wallet-management/access".into()] },
        SidebarItem { label: "Payments".into(), href: "/payments".into(), icon: "credit-card".into(), badge: None, active_paths: vec!["/payments".into()] },
        SidebarItem { label: "Chat".into(), href: "/chat".into(), icon: "message-circle".into(), badge: None, active_paths: vec!["/chat".into()] },
        SidebarItem { label: "Notifications".into(), href: "/notifications/manage".into(), icon: "bell".into(), badge: None, active_paths: vec!["/notifications".into()] },
        SidebarItem { label: "News".into(), href: "/news".into(), icon: "newspaper".into(), badge: None, active_paths: vec!["/news".into()] },
        SidebarItem { label: "Media".into(), href: "/media".into(), icon: "file-text".into(), badge: None, active_paths: vec!["/media".into()] },
        SidebarItem { label: "Developer".into(), href: "/developer-portal".into(), icon: "code".into(), badge: None, active_paths: vec!["/developer-portal".into()] },
        SidebarItem { label: "Settings".into(), href: "/settings".into(), icon: "settings".into(), badge: None, active_paths: vec!["/settings".into()] },
    ];
    rsx! {
        div { class: "admin-shell",
            Sidebar { items, current_path: current_path.clone(), header: Some("EPSX Admin".to_string()) }
            div { class: "admin-main",
                header { class: "admin-header",
                    div { class: "admin-header-left", h1 { class: "admin-page-title", "{page_title}" } }
                    div { class: "admin-header-right",
                        if let Some(u) = &user {
                            span { class: "admin-user-badge", "{u.short_address()}" }
                        }
                    }
                }
                main { class: "admin-content", {children} }
            }
        }
    }
}

#[component]
pub fn DeveloperShell(current_path: String, children: Element) -> Element {
    let items = vec![
        SidebarItem { label: "Overview".into(), href: "/developer".into(), icon: "home".into(), badge: None, active_paths: vec!["/developer".into()] },
        SidebarItem { label: "API Keys".into(), href: "/developer".into(), icon: "key".into(), badge: None, active_paths: vec!["/developer".into()] },
        SidebarItem { label: "Usage".into(), href: "/developer/usage".into(), icon: "chart-line".into(), badge: None, active_paths: vec!["/developer/usage".into()] },
        SidebarItem { label: "Docs".into(), href: "/developer/docs".into(), icon: "book".into(), badge: None, active_paths: vec!["/developer/docs".into()] },
    ];
    rsx! {
        div { class: "developer-shell",
            Sidebar { items, current_path, header: Some("Developer".to_string()) }
            main { class: "developer-main", {children} }
        }
    }
}
