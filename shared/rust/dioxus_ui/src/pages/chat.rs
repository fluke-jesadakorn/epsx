//! `/chat` — private support-chat inbox.
//!
//! The pinned development implementation loads owner-scoped conversations and
//! topics before rendering its inbox. The Rust frontend does not yet have a
//! frozen owner-scoped loader or mutation contract, so this page deliberately
//! renders no conversation, topic, unread, presence, response-time, or message
//! claims. Authentication protects the private route; authorization remains a
//! backend concern once a real chat contract exists.

use dioxus::prelude::*;

use crate::auth::AuthGate;
use crate::layout::main_layout::MainLayout;
use crate::layout::PageHeader;
use crate::primitives::*;

use super::{PageContext, PageMeta};

pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    let meta = PageMeta::app("Support Center");
    (meta, rsx! { RenderChatInbox { ctx: ctx.clone() } })
}

/// Preserve the recognizable inbox/panel shell without implying that an empty
/// owner response was received. No legacy query or hydration payload is read.
#[component]
fn RenderChatInbox(ctx: PageContext) -> Element {
    rsx! {
        MainLayout { ctx: ctx.clone(),
            if ctx.user.is_some() || ctx.wallet.address.is_some() {
                AuthGate {
                    user: ctx.user.clone(),
                    feature: Some("private support chat".to_string()),
                    return_url: Some("/chat".to_string()),
                    div { class: "container page-content chat-page",
                    PageHeader {
                        title: "Support chat".to_string(),
                        description: Some("Private conversations with support".to_string()),
                        icon: Some("message-circle".to_string()),
                        a { class: "btn btn-outline btn-sm", href: "/chat/history",
                            Icon { name: "history".to_string(), size: Some(14) }
                            " History"
                        }
                    }
                    div { class: "chat-inbox-row",
                        aside { class: "chat-inbox",
                            div { class: "chat-inbox-header",
                                div { class: "chat-inbox-brand",
                                    div { class: "chat-inbox-avatar",
                                        Icon { name: "headset".to_string(), size: Some(20) }
                                    }
                                    div { class: "chat-inbox-titles",
                                        h2 { class: "chat-inbox-title", "Support Center" }
                                        p { class: "chat-inbox-subtitle", "Private support workspace" }
                                    }
                                }
                            }
                            div { class: "chat-inbox-list",
                                div { class: "chat-inbox-empty",
                                    div { class: "chat-inbox-empty-icon",
                                        Icon { name: "inbox".to_string(), size: Some(20) }
                                    }
                                    p { class: "chat-inbox-empty-title", "Inbox unavailable" }
                                    p { class: "chat-inbox-empty-hint",
                                        "No conversation count or empty-inbox claim is being inferred."
                                    }
                                }
                            }
                        }
                        section {
                            class: "chat-panel chat-panel-empty",
                            role: "status",
                            "data-section": "chat-unavailable",
                            "data-chat-state": "unavailable",
                            aria_labelledby: "chat-unavailable-title",
                            div { class: "chat-panel-empty-icon",
                                Icon { name: "message-circle".to_string(), size: Some(32) }
                            }
                            h2 { id: "chat-unavailable-title", class: "chat-panel-empty-title",
                                "Support conversations unavailable"
                            }
                            p { class: "chat-panel-empty-hint",
                                "Conversation data and chat actions are temporarily unavailable. No topics, messages, participants, timestamps, or statuses are shown."
                            }
                            div { class: "auth-gate-actions",
                                a { class: "btn btn-outline", href: "/", "Back to home" }
                            }
                        }
                    }
                    }
                }
            } else {
                RenderPublicChat {}
            }
        }
    }
}

/// Signed-out `/chat` remains a browsable marketing surface in production.
/// It explains the value of support chat and provides an explicit wallet sign-in
/// destination without rendering private conversations or fabricated counts.
#[component]
fn RenderPublicChat() -> Element {
    rsx! {
        div { class: "container mx-auto max-w-xl px-4 py-12 chat-public-page",
            style: "max-width: 36rem; width: 100%; margin-left: auto; margin-right: auto; padding: 3rem 1rem; box-sizing: border-box;",
            div { class: "mb-8 flex items-center gap-4",
                div { class: "flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-[#7645d9] to-[#1fc7d4] shadow-lg shadow-[#7645d9]/25",
                    Icon { name: "headset".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                }
                div {
                    h1 { class: "text-xl font-bold tracking-tight", "Support Center" }
                    p { class: "mt-0.5 text-xs text-muted-foreground", "Get help from our team · Usually replies in minutes" }
                }
            }
            div { class: "relative mb-6 overflow-hidden rounded-2xl border border-purple-500/30 bg-gradient-to-r from-purple-900/40 via-purple-800/30 to-pink-900/40 backdrop-blur-sm",
                div { class: "pointer-events-none absolute -right-10 -top-10 h-40 w-40 rounded-full bg-purple-500/20 blur-3xl", aria_hidden: "true" }
                div { class: "pointer-events-none absolute -bottom-6 -left-6 h-24 w-24 rounded-full bg-pink-500/20 blur-2xl", aria_hidden: "true" }
                div { class: "relative flex flex-col gap-4 p-5 sm:flex-row sm:items-center sm:justify-between",
                    div { class: "flex items-start gap-4",
                        div { class: "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-pink-500 shadow-lg shadow-purple-500/30",
                            Icon { name: "lock".to_string(), size: Some(24), class_name: Some("text-white".to_string()) }
                        }
                        div {
                            p { class: "text-base font-bold text-white", "Sign in to access Support Chat" }
                            p { class: "mt-0.5 text-sm text-purple-300/80", "Connect your wallet to start a conversation with our team" }
                            div { class: "mt-2 flex flex-wrap gap-3",
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "📈 Top 100 stock rankings" }
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "📊 Real-time EPS data" }
                                span { class: "flex items-center gap-1 text-xs text-purple-300/70", "⚡ AI-powered insights" }
                            }
                        }
                    }
                    a { class: "group inline-flex shrink-0 items-center gap-2 rounded-xl bg-gradient-to-r from-purple-500 to-pink-500 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/30 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-xl hover:shadow-purple-500/40 focus:outline-none",
                        href: "/auth?return_url=%2Fchat",
                        Icon { name: "log-in".to_string(), size: Some(16) }
                        "Sign In to Chat"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::user::{AuthMethod, User};

    fn signed_in_ctx() -> PageContext {
        PageContext {
            user: Some(User {
                id: "u1".to_string(),
                address: "0x1234…abcd".to_string(),
                chain_id: "56".to_string(),
                roles: vec!["user".to_string()],
                email: None,
                tier: None,
                permissions: vec![],
                last_login_at: None,
                auth_method: AuthMethod::Wallet,
                display_name: None,
            }),
            path: "/chat".to_string(),
            ..Default::default()
        }
    }

    fn html(ctx: &PageContext) -> String {
        let (_, element) = render(ctx);
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_route_keeps_private_state_hidden() {
        let rendered = html(&PageContext {
            path: "/chat".to_string(),
            ..Default::default()
        });
        assert!(rendered.contains("Sign in to access Support Chat"));
        assert!(rendered.contains("href=\"/auth?return_url=%2Fchat\""));
        assert!(!rendered.contains("data-chat-state"));
    }

    #[test]
    fn authenticated_user_without_frontend_permission_sees_unavailable_state() {
        let rendered = html(&signed_in_ctx());
        assert!(rendered.contains("data-chat-state=\"unavailable\""));
        assert!(rendered.contains("aria-labelledby=\"chat-unavailable-title\""));
        assert!(!rendered.contains("Permission required"));
    }

    #[test]
    fn sample_and_presence_claims_are_suppressed() {
        let rendered = html(&signed_in_ctx());
        for forbidden in [
            "Plan upgrade question",
            "Payment issue",
            "API key question",
            "Usually replies in minutes",
            "Billing &amp; Payments",
            "EPSX Support",
            "Today",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "sample claim leaked: {forbidden}"
            );
        }
    }

    #[test]
    fn unavailable_inbox_has_no_fake_mutations_filters_or_self_recovery() {
        let rendered = html(&signed_in_ctx());
        for forbidden in [
            "<button",
            "<textarea",
            "<input",
            "<select",
            "New Conversation",
            "Start Conversation",
            "Resolve",
            "Type your reply",
            "Search conversations",
            ">Check again</a>",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "fake chat control leaked: {forbidden}"
            );
        }
        assert!(!rendered.contains("href=\"/chat\""));
        assert!(rendered.contains("href=\"/chat/history\""));
        assert!(rendered.contains("href=\"/\""));
    }
}
