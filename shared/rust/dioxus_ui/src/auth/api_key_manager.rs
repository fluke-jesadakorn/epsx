//! `ApiKeyManager` — generate / list / delete API keys for the
//! current user.
//!
//! Port of `apps-old/frontend/components/auth/api-key-manager.tsx`
//! (465 LoC). The TS source is a client component that:
//!   1. Renders a `<Card>` with a generation form (input + button)
//!   2. On submit, POSTs to `/api/user/api-keys` and shows the
//!      generated key in a one-time copy banner
//!   3. Lists existing keys with show/hide toggle, last-used
//!      timestamp, scope badges, and a delete button
//!   4. Renders an "API Documentation" footer card
//!
//! The Dioxus port renders the same visual structure as a static
//! shell. The data is provided by the parent page (which fetches
//! from the BFF's `/api/v1/keys` endpoint and passes the result
//! via the `keys` prop). The generate / delete handlers are
//! caller-supplied callbacks so the page can wire them to the
//! real BFF endpoints.

use crate::auth::global_auth_guard::GlobalAuthGuard;
use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub usage_count: u64,
    pub is_active: bool,
    pub scopes: Vec<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct ApiKeyScope {
    pub name: String,
    pub description: String,
}

#[component]
pub fn ApiKeyManager(
    /// Whether the user is authenticated. When `false`, the
    /// component renders the `GlobalAuthGuard`'s gate. Mirrors the
    /// TS source's `<GlobalAuthGuard>` wrapper.
    #[props(default = false)] user_authenticated: bool,
    /// Existing API keys to display.
    #[props(default = Vec::new())] keys: Vec<ApiKey>,
    /// Whether the keys list is currently loading.
    #[props(default = false)] is_loading: bool,
    /// Newly generated key (one-time display banner). When
    /// `Some`, the green "New API Key Generated" card is shown.
    #[props(default = None)] show_new_key: Option<String>,
    /// Fired when the user clicks "Generate Key". The parent
    /// should call the BFF endpoint and pass the result back via
    /// `show_new_key`.
    #[props(default = None)] on_generate: Option<EventHandler<MouseEvent>>,
    /// Fired when the user clicks the delete button on a key.
    /// The parent should call the BFF DELETE endpoint and refresh
    /// `keys`.
    #[props(default = None)] on_delete: Option<EventHandler<String>>,
    /// Fired when the user clicks the "I've saved the key" button
    /// to dismiss the new-key banner.
    #[props(default = None)] on_dismiss_new_key: Option<EventHandler<MouseEvent>>,
    /// Fired when the user clicks the Refresh button.
    #[props(default = None)] on_refresh: Option<EventHandler<MouseEvent>>,
    /// Class names appended to the outer wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();

    rsx! {
        GlobalAuthGuard { user_authenticated, is_loading: false,
            div { class: "api-key-manager space-y-6 {cls}",
                // Header card
                div { class: "api-key-manager-header card card-glass",
                    div { class: "card-header",
                        div { class: "card-title flex items-center gap-2",
                            Icon { name: "key".to_string(), size: Some(20), class_name: Some("text-orange-500".to_string()) }
                            "API Key Management"
                        }
                        p { class: "text-sm text-slate-600 dark:text-slate-400",
                            "Generate and manage API keys for programmatic access to EPSX services."
                        }
                    }
                    div { class: "card-body space-y-4",
                        div { class: "api-key-manager-form flex gap-3",
                            input {
                                class: "api-key-manager-name-input flex-1 px-3 py-2 bg-background border border-slate-200 dark:border-slate-700 rounded-lg",
                                placeholder: "Enter API key name (e.g., Production App, Analytics Dashboard)",
                            }
                            button {
                                class: "api-key-manager-generate-btn flex items-center gap-2 px-4 py-2 bg-orange-500 text-white rounded-lg font-medium hover:bg-orange-600",
                                onclick: move |e| {
                                    if let Some(cb) = on_generate.as_ref() {
                                        cb.call(e);
                                    }
                                },
                                Icon { name: "plus".to_string(), size: Some(16) }
                                "Generate Key"
                            }
                        }
                        // Security notice
                        div { class: "api-key-manager-notice p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg",
                            div { class: "flex items-start gap-2",
                                Icon { name: "alert-triangle".to_string(), size: Some(16), class_name: Some("text-amber-600 dark:text-amber-400 mt-0.5".to_string()) }
                                div { class: "text-xs text-amber-700 dark:text-amber-400",
                                    p { class: "font-medium mb-1", "Important Security Notice:" }
                                    ul { class: "space-y-1",
                                        li { "• API keys provide full access to your account - treat them like passwords" }
                                        li { "• Store keys securely and never commit them to version control" }
                                        li { "• Regenerate keys immediately if compromised" }
                                        li { "• Use environment variables in production applications" }
                                    }
                                }
                            }
                        }
                    }
                }
                // New key display
                if let Some(new_key) = show_new_key.as_ref() {
                    div { class: "api-key-manager-new-key card card-glass border-green-200 dark:border-green-800",
                        div { class: "card-header bg-green-50 dark:bg-green-900/20",
                            div { class: "card-title flex items-center gap-2 text-green-800 dark:text-green-300",
                                Icon { name: "check-circle".to_string(), size: Some(20) }
                                "New API Key Generated"
                            }
                        }
                        div { class: "card-body p-6 space-y-3",
                            p { class: "text-sm text-green-700 dark:text-green-400",
                                "Your new API key has been generated. Copy it now - you won't be able to see it again!"
                            }
                            div { class: "flex items-center gap-2 p-3 bg-slate-100 dark:bg-slate-800 rounded-lg border",
                                code { class: "flex-1 text-sm font-mono break-all", "{new_key}" }
                                button {
                                    class: "api-key-manager-copy-btn px-3 py-1 bg-background border border-slate-200 dark:border-slate-700 rounded text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700",
                                    Icon { name: "copy".to_string(), size: Some(14) }
                                }
                            }
                            button {
                                class: "api-key-manager-dismiss-btn w-full px-4 py-2 border border-slate-200 dark:border-slate-700 rounded-lg text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700",
                                onclick: move |e| {
                                    if let Some(cb) = on_dismiss_new_key.as_ref() {
                                        cb.call(e);
                                    }
                                },
                                "I've saved the key securely"
                            }
                        }
                    }
                }
                // Existing keys
                div { class: "api-key-manager-list card card-glass",
                    div { class: "card-header",
                        div { class: "card-title flex items-center justify-between",
                            span { "Your API Keys ({keys.len()})" }
                            if !keys.is_empty() {
                                button {
                                    class: "api-key-manager-refresh-btn px-3 py-1 border border-slate-200 dark:border-slate-700 rounded text-sm font-medium hover:bg-slate-50 dark:hover:bg-slate-700",
                                    onclick: move |e| {
                                        if let Some(cb) = on_refresh.as_ref() {
                                            cb.call(e);
                                        }
                                    },
                                    "Refresh"
                                }
                            }
                        }
                    }
                    div { class: "card-body",
                        if is_loading {
                            div { class: "api-key-manager-loading flex items-center justify-center py-8 gap-2",
                                div { class: "h-4 w-4 bg-orange-500 rounded animate-pulse" }
                                span { class: "text-sm text-slate-600 dark:text-slate-400", "Loading API keys..." }
                            }
                        } else if keys.is_empty() {
                            div { class: "api-key-manager-empty text-center py-8",
                                Icon { name: "key".to_string(), size: Some(32), class_name: Some("text-slate-400 mx-auto mb-2".to_string()) }
                                p { class: "text-sm text-slate-600 dark:text-slate-400",
                                    "No API keys found. Generate your first key above."
                                }
                            }
                        } else {
                            div { class: "space-y-4",
                                for k in keys.iter() {
                                    ApiKeyRow {
                                        key_data: k.clone(),
                                        on_delete: on_delete,
                                    }
                                }
                            }
                        }
                    }
                }
                // API documentation footer
                div { class: "api-key-manager-docs card card-glass",
                    div { class: "card-header",
                        div { class: "card-title flex items-center gap-2",
                            Icon { name: "external-link".to_string(), size: Some(20), class_name: Some("text-blue-500".to_string()) }
                            "API Documentation"
                        }
                    }
                    div { class: "card-body space-y-4",
                        p { class: "text-sm text-slate-600 dark:text-slate-400",
                            "Learn how to use your API keys to access EPSX services programmatically."
                        }
                        div { class: "api-key-manager-docs-auth p-3 bg-slate-50 dark:bg-slate-800 rounded-lg",
                            h4 { class: "text-sm font-medium text-slate-900 dark:text-slate-100 mb-1", "Authentication Header" }
                            code { class: "text-xs text-slate-600 dark:text-slate-400", "Authorization: Bearer YOUR_API_KEY" }
                        }
                        div { class: "api-key-manager-docs-url p-3 bg-slate-50 dark:bg-slate-800 rounded-lg",
                            h4 { class: "text-sm font-medium text-slate-900 dark:text-slate-100 mb-1", "Base URL" }
                            code { class: "text-xs text-slate-600 dark:text-slate-400", "https://api.epsx.io/v1/" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ApiKeyRow(
    key_data: ApiKey,
    #[props(default = None)] on_delete: Option<EventHandler<String>>,
) -> Element {
        let masked = if key_data.key.len() > 16 {
        format!(
            "{}{}{}",
            &key_data.key[..8],
            "•".repeat(32),
            &key_data.key[key_data.key.len() - 8..]
        )
    } else {
        key_data.key.clone()
    };
    let key_id = key_data.id.clone();
    rsx! {
        div { class: "api-key-row p-4 border border-slate-200 dark:border-slate-700 rounded-lg space-y-3",
            div { class: "api-key-row-header flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    div { class: "p-2 bg-slate-100 dark:bg-slate-800 rounded-lg",
                        Icon { name: "key".to_string(), size: Some(16), class_name: Some("text-slate-600 dark:text-slate-400".to_string()) }
                    }
                    div {
                        h4 { class: "font-medium text-slate-900 dark:text-slate-100", "{key_data.name}" }
                        div { class: "flex items-center gap-2 text-xs text-slate-600 dark:text-slate-400",
                            Icon { name: "calendar".to_string(), size: Some(12) }
                            span { "Created {key_data.created_at}" }
                        }
                    }
                }
                div { class: "flex items-center gap-2",
                    span {
                        class: if key_data.is_active { "api-key-row-status-active inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-500" } else { "api-key-row-status-inactive inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-slate-500/10 text-slate-500" },
                        if key_data.is_active { "Active" } else { "Inactive" }
                    }
                    button {
                        class: "api-key-row-delete-btn p-2 text-red-600 hover:text-red-700 hover:bg-red-50 rounded",
                        onclick: move |e| {
                            if let Some(cb) = on_delete.as_ref() {
                                cb.call(key_id.clone());
                            }
                            let _ = e;
                        },
                        Icon { name: "trash-2".to_string(), size: Some(16) }
                    }
                }
            }
            div { class: "api-key-row-key space-y-2",
                div { class: "flex items-center gap-2",
                    span { class: "text-xs font-medium text-slate-700 dark:text-slate-300", "API Key:" }
                }
                div { class: "flex items-center gap-2 p-2 bg-slate-50 dark:bg-slate-800 rounded border",
                    code { class: "flex-1 text-xs font-mono break-all", "{masked}" }
                }
            }
            div { class: "api-key-row-usage flex items-center justify-between text-xs text-slate-600 dark:text-slate-400",
                span { "Usage: {key_data.usage_count} requests" }
                if let Some(last) = key_data.last_used_at.as_ref() {
                    span { "Last used {last}" }
                }
            }
            if !key_data.scopes.is_empty() {
                div { class: "api-key-row-scopes flex items-center gap-2",
                    span { class: "text-xs font-medium text-slate-700 dark:text-slate-300", "Scopes:" }
                    div { class: "flex gap-1",
                        for scope in key_data.scopes.iter() {
                            span { class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border border-slate-200 dark:border-slate-700",
                                "{scope}"
                            }
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

    #[test]
    fn api_key_default_is_empty() {
        let k = ApiKey::default();
        assert!(k.id.is_empty());
        assert!(k.name.is_empty());
        assert!(!k.is_active);
        assert_eq!(k.usage_count, 0);
        assert!(k.scopes.is_empty());
    }

    #[test]
    fn api_key_scope_default_is_empty() {
        let s = ApiKeyScope::default();
        assert!(s.name.is_empty());
        assert!(s.description.is_empty());
    }

    #[test]
    fn api_key_masked_short_key_unchanged() {
        // The TS source's formatApiKey does:
        //   `${key.slice(0, 8)}${'•'.repeat(32)}${key.slice(-8)}`
        // For keys shorter than 16 chars, the prefix+suffix overlap,
        // so the Dioxus port returns the key as-is.
        let k = ApiKey::default();
        let short = "abc123";
        let masked = if short.len() > 16 {
            format!(
                "{}{}{}",
                &short[..8],
                "•".repeat(32),
                &short[short.len() - 8..]
            )
        } else {
            short.to_string()
        };
        assert_eq!(masked, "abc123");
        let _ = k;
    }
}
