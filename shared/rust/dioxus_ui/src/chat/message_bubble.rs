//! `<MessageBubble>` — extracted chat-message primitive used by both
//! the user chat surface (`/chat`, `/chat/[id]`) and the admin chat
//! surface (`admin_chat_conversation_view.tsx`, ported in Wave 6B).
//!
//! This is a Wave 6A Track C extraction of the inline markup that
//! the Wave 1 port inlined into `chat_conversation.rs`. The original
//! lives at `apps-old/frontend/components/chat/chat-message-item.tsx`
//! (131 LoC, plus the wrapping `chat-message-list.tsx` 92 LoC for
//! the message-list shell).
//!
//! Surface (faithful port of the source):
//!   * System messages render as a centered pill with `Info` icon
//!   * User messages align right with a gradient bubble, no avatar;
//!     show a `Check` / `CheckCheck` read-receipt
//!   * Other-user messages align left with a small avatar (gradient
//!     pill — `Headset` for support, `Bot` for AI) and a sender
//!     label ("Support" / "AI Assistant") above the bubble
//!   * Attachments (image preview or download row) render under the
//!     bubble body when present
//!
//! Reuse: both `pages/chat.rs` (ChatPanel) and the future admin
//! `admin_pages/admin_chat_conversation_view.rs` import
//! `crate::chat::MessageBubble` with a `crate::chat::Message` payload.
//!
//! CSS: the bubble's own alignment + avatar + tail classes live in
//! `shared/rust/templates/src/lib.rs` under the
//! `// === wave6-auth-pages-depth-track-c ===` marker region. The
//! component is server-renderable (no JS state).

use dioxus::prelude::*;

use crate::primitives::Icon;

/// Single message in a chat conversation. Mirrors a subset of the
/// source `ChatMessage` type from `apps-old/frontend/shared/api/chat.ts`
/// (id, sender, content, timestamp, read flag, optional attachments,
/// optional markdown body) — narrowed to the fields the primitive
/// actually renders. If Wave 6B's admin port needs more, add fields
/// here in a backwards-compatible way (all default to `Default::`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Message {
    /// Server-assigned message id (used for list keys + read receipt).
    pub id: String,
    /// Display name shown on other-user bubbles. Ignored when
    /// `is_own_message` is true (we don't label our own messages).
    pub sender_name: String,
    /// Sender role label. The source uses `sender_type` ("user" |
    /// "ai" | "system" | "agent"); we collapse that to a string the
    /// bubble can render directly ("Support" / "AI Assistant" / "").
    /// Empty string means no label (typical for system messages
    /// which render as a centered pill instead).
    pub sender_role: String,
    /// Message body. Markdown is NOT rendered here — the source
    /// uses a `ChatMarkdown` sub-component which Wave 6A inlines as
    /// a `class="markdown-body"` wrapper on the body `<p>` (the
    /// portal theme's markdown CSS is loaded globally).
    pub body: String,
    /// ISO-8601 timestamp. The bubble renders this verbatim as a
    /// short relative or absolute time string. Wave 6A accepts
    /// whatever the BFF gives us (the source uses
    /// `date-fns::formatDistanceToNow`).
    pub created_at: String,
    /// Whether the message has been read by the *other* party. Only
    /// affects the read-receipt icon shown on own-message bubbles.
    #[allow(dead_code)] // public field — Wave 6B admin reuses it
    pub is_read: bool,
    /// True for messages sent by the viewing user. Drives alignment
    /// (right vs left), avatar visibility, and read-receipt display.
    pub is_own: bool,
    /// True for system messages. System messages render as a
    /// centered pill (no avatar, no tail), regardless of `is_own`.
    #[allow(dead_code)] // public field — Wave 6B admin reuses it
    pub is_system: bool,
    /// Sender category — drives the avatar icon for other-user
    /// bubbles. `"support"` → Headset icon, `"ai"` → Bot icon, any
    /// other value → no avatar (rare; usually means "another user").
    pub sender_type: String,
    /// Optional attachment payload. The bubble renders the first
    /// attachment inline (image preview OR file download row).
    pub attachment: Option<Attachment>,
}

/// Simplified attachment payload. Mirrors `ChatAttachment` from the
/// source (`apps-old/frontend/shared/api/chat.ts`) — narrowed to
/// what the bubble renders. Wave 6A only renders the first
/// attachment per message; the source renders a list.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Attachment {
    pub filename: String,
    pub url: String,
    /// Image MIME (`image/png`, `image/jpeg`, …) drives the
    /// preview-vs-download rendering. Anything else renders as a
    /// file download row.
    pub file_type: String,
    pub size: u64,
}

/// `<MessageBubble>` — render a single chat message.
///
/// Mirrors `chat-message-item.tsx` (131 LoC) from the source:
///   * Centered pill for system messages (`is_system == true`)
///   * Right-aligned gradient bubble with read-receipt for own
///     messages (`is_own_message == true`)
///   * Left-aligned bubble with avatar + sender name for other
///     messages
///
/// Reused by:
///   * `pages/chat.rs` — the main user chat surface
///   * `pages/chat_conversation.rs` — the dynamic `[id]` route
///   * `admin_pages/admin_chat_conversation_view.rs` — Wave 6B
///
/// The component is fully server-renderable — no signals, no
/// resources, no client-only state.
#[component]
pub fn MessageBubble(
    /// The message to render. The `is_own` field on `Message` is
    /// ignored — pass `is_own_message` explicitly so the call site
    /// decides ownership (the BFF knows the wallet address; the
    /// primitive does not).
    message: Message,
    /// Whether the message was sent by the *viewing* user. Drives
    /// alignment, avatar visibility, and the read-receipt icon.
    is_own_message: bool,
) -> Element {
    let is_system = message.is_system;
    let sender_type = message.sender_type.clone();
    let sender_name = message.sender_name.clone();
    let sender_role = message.sender_role.clone();
    let body = message.body.clone();
    let created_at = message.created_at.clone();
    let attachment = message.attachment.clone();

    if is_system {
        return rsx! {
            div { class: "chat-message chat-message-system",
                div { class: "chat-message-system-pill",
                    Icon { name: "info".to_string(), size: Some(12) }
                    span { class: "chat-message-system-text", "{body}" }
                }
            }
        };
    }

    let row_class = if is_own_message {
        "chat-message chat-message-self flex-row-reverse"
    } else {
        "chat-message chat-message-other"
    };
    let bubble_class = if is_own_message {
        "chat-bubble chat-bubble-self"
    } else {
        "chat-bubble chat-bubble-other"
    };

    rsx! {
        div { class: "{row_class}",
            // Avatar (only on other-user messages).
            if !is_own_message {
                div { class: "chat-message-avatar",
                    if sender_type == "ai" {
                        Icon { name: "bot".to_string(), size: Some(16) }
                    } else {
                        Icon { name: "headset".to_string(), size: Some(16) }
                    }
                }
            }
            // Bubble column.
            div { class: "chat-message-col",
                // Sender label (other-user only).
                if !is_own_message && !sender_name.is_empty() {
                    span { class: "chat-message-sender",
                        if !sender_role.is_empty() {
                            "{sender_role}"
                        } else {
                            "{sender_name}"
                        }
                    }
                }
                // Bubble body.
                div { class: "{bubble_class}",
                    if !body.is_empty() {
                        p { class: "chat-bubble-body markdown-body", "{body}" }
                    }
                    if let Some(att) = attachment {
                        AttachmentView { att: att }
                    }
                }
                // Meta row: timestamp + (own only) read receipt.
                div { class: "chat-message-meta",
                    span { class: "chat-message-timestamp", "{created_at}" }
                    if is_own_message {
                        if message.is_read {
                            Icon { name: "check-check".to_string(), size: Some(12) }
                        } else {
                            Icon { name: "check".to_string(), size: Some(12) }
                        }
                    }
                }
            }
        }
    }
}

/// Render an inline attachment. Mirrors the source's
/// `AttachmentView` — image preview for `image/*` MIME types, a
/// download row otherwise. The source's `<ImageLightbox>` lightbox
/// is dropped (Wave 6A keeps the portal theme minimal).
#[component]
fn AttachmentView(att: Attachment) -> Element {
    let is_image = att.file_type.starts_with("image/");
    let size_kb = (att.size as f64 / 1024.0).max(0.1);
    if is_image {
        rsx! {
            a {
                class: "chat-attachment chat-attachment-image",
                href: "{att.url}",
                target: "_blank",
                rel: "noopener noreferrer",
                img {
                    class: "chat-attachment-thumb",
                    src: "{att.url}",
                    alt: "{att.filename}",
                }
            }
        }
    } else {
        rsx! {
            a {
                class: "chat-attachment chat-attachment-file",
                href: "{att.url}",
                target: "_blank",
                rel: "noopener noreferrer",
                download: "{att.filename}",
                Icon { name: "file-text".to_string(), size: Some(16) }
                div { class: "chat-attachment-info",
                    p { class: "chat-attachment-name", "{att.filename}" }
                    p { class: "chat-attachment-size", "{size_kb:.1} KB" }
                }
                Icon { name: "download".to_string(), size: Some(14) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `MessageBubble` must surface the sender name in the rendered
    /// HTML when rendering an other-user message. This is the
    /// minimum contract the Wave 6A section-marker gate relies on.
    #[test]
    fn message_bubble_renders_with_sender() {
        let msg = Message {
            id: "m1".to_string(),
            sender_name: "Alex".to_string(),
            sender_role: "Support".to_string(),
            body: "Hello there".to_string(),
            created_at: "10:32".to_string(),
            is_read: false,
            is_own: false,
            is_system: false,
            sender_type: "agent".to_string(),
            attachment: None,
        };
        let el = rsx! { MessageBubble { message: msg, is_own_message: false } };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("Support"),
            "MessageBubble must render the sender role label. Got: {}",
            html
        );
        assert!(
            html.contains("Hello there"),
            "MessageBubble must render the body text. Got: {}",
            html
        );
    }

    /// Own-message bubbles align right (no sender label, no
    /// avatar) and surface the read-receipt. The body must still
    /// render verbatim.
    #[test]
    fn message_bubble_self_omits_sender_label() {
        let msg = Message {
            id: "m2".to_string(),
            sender_name: "Me".to_string(),
            sender_role: String::new(),
            body: "My reply".to_string(),
            created_at: "10:33".to_string(),
            is_read: true,
            is_own: true,
            is_system: false,
            sender_type: "user".to_string(),
            attachment: None,
        };
        let el = rsx! { MessageBubble { message: msg, is_own_message: true } };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("chat-message-self"),
            "Self bubble must carry the chat-message-self class. Got: {}",
            html
        );
        assert!(
            !html.contains("chat-message-sender"),
            "Self bubble must NOT show a sender label. Got: {}",
            html
        );
        assert!(
            html.contains("My reply"),
            "Self bubble must still render the body. Got: {}",
            html
        );
    }

    /// System messages render as a centered pill — no avatar, no
    /// sender label, body wrapped in the system-pill class.
    #[test]
    fn message_bubble_system_renders_as_pill() {
        let msg = Message {
            id: "m3".to_string(),
            sender_name: String::new(),
            sender_role: String::new(),
            body: "Conversation opened".to_string(),
            created_at: "10:30".to_string(),
            is_read: false,
            is_own: false,
            is_system: true,
            sender_type: "system".to_string(),
            attachment: None,
        };
        let el = rsx! { MessageBubble { message: msg, is_own_message: false } };
        let html = dioxus_ssr::render_element(el);
        assert!(
            html.contains("chat-message-system"),
            "System bubble must carry the system class. Got: {}",
            html
        );
        assert!(
            html.contains("Conversation opened"),
            "System bubble must show the body text. Got: {}",
            html
        );
    }
}
