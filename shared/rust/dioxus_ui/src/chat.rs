//! Chat primitive module — shared widgets for the chat surface.
//!
//! Wave 6A Track C registers the new `<MessageBubble>` primitive
//! (extracted from `chat-message-item.tsx`, 131 LoC) here. The
//! primitive is reused by both the user chat surface
//! (`pages/chat.rs`, `pages/chat_conversation.rs`) and the admin
//! chat surface (`admin_pages/admin_chat_conversation_view.rs`,
//! Wave 6B territory).
//!
//! Module path: `crate::chat::MessageBubble` (re-exported below).
//!
//! CSS for the bubble's alignment, avatar, and tail classes lives
//! in `shared/rust/templates/src/lib.rs` under the
//! `// === wave6-auth-pages-depth-track-c ===` marker region.

pub mod message_bubble;

pub use message_bubble::{Attachment, Message, MessageBubble};
