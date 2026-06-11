//! /admin/settings — system settings.

use crate::primitives::*;

use dioxus::prelude::*;
use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::admin("Settings");
    (meta, rsx! {
        AuthGate { user: ctx.user.clone(), feature: Some("platform settings".to_string()),
            div { class: "container page-content",
                h1 { class: "text-2xl font-bold mb-6", "Settings" }
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4",
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "System" } }
                        div { class: "card-body",
                            dl { class: "space-y-2 text-sm",
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "Environment" } dd { "production" } }
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "Database" } dd { "PostgreSQL" } }
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "Cache" } dd { "Redis" } }
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "Version" } dd { "1.0.0" } }
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "Region" } dd { "local" } }
                            }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Indexers" } }
                        div { class: "card-body",
                            dl { class: "space-y-2 text-sm",
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "BSC Mainnet" } dd { span { class: "badge badge-success", "synced" } } }
                                div { class: "flex justify-between", dt { class: "text-muted-foreground", "BSC Testnet" } dd { span { class: "badge badge-success", "synced" } } }
                            }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Authentication" } }
                        div { class: "card-body",
                            Form { method: "POST".to_string(), action: "/api/v1/admin/settings/auth".to_string(),
                                CheckboxField { name: "demo_login".to_string(), label: "Enable demo login".to_string(), checked: true }
                                CheckboxField { name: "siwe_required".to_string(), label: "Require SIWE for all routes".to_string(), checked: false }
                                CheckboxField { name: "rate_limit".to_string(), label: "Enable rate limiting (100 req/min)".to_string(), checked: true }
                                FormActions {
                                    button { class: "btn btn-primary", r#type: "submit", "Save" }
                                }
                            }
                        }
                    }
                    div { class: "card card-glass",
                        div { class: "card-header", h3 { class: "card-title", "Maintenance" } }
                        div { class: "card-body",
                            Form { method: "POST".to_string(), action: "/api/v1/admin/settings/maintenance".to_string(),
                                CheckboxField { name: "maintenance".to_string(), label: "Maintenance mode (show banner to users)".to_string(), checked: false }
                                div { class: "field",
                                    label { class: "field-label", "Maintenance message" }
                                    textarea { class: "input", name: "message", rows: "3", placeholder: "We'll be back shortly..." }
                                }
                                FormActions {
                                    button { class: "btn btn-primary", r#type: "submit", "Save" }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
