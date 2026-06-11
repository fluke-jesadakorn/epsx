//! /profile — user profile with email mgmt, data mgmt.

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
    let mut tab = use_signal(|| "profile".to_string());
    rsx! {
        MainLayout { ctx: ctx.clone(),
            AuthGate { user: ctx.user.clone(), feature: Some("your profile".to_string()),
                div { class: "container page-content max-w-3xl",
                    PageHeader { title: "Profile".to_string(), description: Some("Manage your account settings".to_string()), icon: Some("user".to_string()) }
                    div { class: "tabs mb-4",
                        button { class: if *tab.read() == "profile" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("profile".to_string()), "Profile" }
                        button { class: if *tab.read() == "email" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("email".to_string()), "Email" }
                        button { class: if *tab.read() == "data" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("data".to_string()), "Data" }
                        button { class: if *tab.read() == "security" { "btn btn-primary" } else { "btn btn-outline" }, onclick: move |_| tab.set("security".to_string()), "Security" }
                    }
                    if *tab.read() == "profile" {
                        div { class: "card card-glass", div { class: "card-body",
                            if let Some(u) = &ctx.user {
                                div { class: "space-y-3",
                                    div { span { class: "text-muted-foreground", "Wallet: " } span { class: "font-mono text-sm", "{u.address}" } }
                                    div { span { class: "text-muted-foreground", "Chain: " } span { "{u.chain_id}" } }
                                    div { span { class: "text-muted-foreground", "Roles: " } span { "{u.roles.join(\", \")}" } }
                                }
                            }
                        } }
                    } else if *tab.read() == "email" { EmailForm {} } else if *tab.read() == "data" { DataForm {} } else { SecurityForm {} }
                }
            }
        }
    }
}

#[component]
fn EmailForm() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                Form { method: "POST".to_string(), action: "/api/v1/profile/email".to_string(),
                    div { class: "field",
                        label { class: "field-label", "Email" }
                        input { class: "input", name: "email", r#type: "email", placeholder: "you@example.com" }
                    }
                    div { class: "field",
                        CheckboxField { name: "notify".to_string(), label: "Send me product updates".to_string(), checked: true }
                    }
                    FormActions {
                        button { class: "btn btn-primary", r#type: "submit", "Save" }
                    }
                }
            }
        }
    }
}

#[component]
fn DataForm() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                h3 { class: "text-lg font-bold", "Data management" }
                p { class: "text-muted-foreground mt-2", "Export or delete your account data" }
                div { class: "flex gap-2 mt-4",
                    button { class: "btn btn-outline", r#type: "button", Icon { name: "download".to_string(), size: Some(16) } " Export data" }
                    button { class: "btn btn-danger", r#type: "button", Icon { name: "trash".to_string(), size: Some(16) } " Delete account" }
                }
            }
        }
    }
}

#[component]
fn SecurityForm() -> Element {
    rsx! {
        div { class: "card card-glass",
            div { class: "card-body",
                h3 { class: "text-lg font-bold", "Security" }
                div { class: "space-y-3 mt-4",
                    div { class: "flex items-center justify-between p-3 bg-muted rounded",
                        div { p { class: "font-semibold", "Two-factor authentication" } p { class: "text-sm text-muted-foreground", "Add an extra layer of security" } }
                        button { class: "btn btn-sm btn-primary", r#type: "button", "Enable" }
                    }
                    div { class: "flex items-center justify-between p-3 bg-muted rounded",
                        div { p { class: "font-semibold", "Active sessions" } p { class: "text-sm text-muted-foreground", "2 devices" } }
                        button { class: "btn btn-sm btn-outline", r#type: "button", "Manage" }
                    }
                }
            }
        }
    }
}
