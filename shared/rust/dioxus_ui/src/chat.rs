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
// === wave40-t2 domain subdirs port ===
// 10 NEW chat components ported from
// `apps-old/frontend/components/chat/`:
//   - chat_bubble              (42 LoC)
//   - chat_conversation_list   (83 LoC)
//   - chat_header              (63 LoC)
//   - chat_inbox               (414 LoC, simplified stub)
//   - chat_input               (177 LoC)
//   - chat_message_list        (92 LoC)
//   - chat_panel               (253 LoC)
//   - chat_status_badge        (20 LoC)
//   - chat_topic_selector      (233 LoC)
//   - chat_widget              (67 LoC)
// (chat_message_item is already ported as message_bubble)
pub mod chat_bubble;
pub mod chat_conversation_list;
pub mod chat_header;
pub mod chat_inbox;
pub mod chat_input;
pub mod chat_message_list;
pub mod chat_panel;
pub mod chat_status_badge;
pub mod chat_topic_selector;
pub mod chat_widget;

pub use message_bubble::{Attachment, Message, MessageBubble};
pub use chat_bubble::ChatBubble;
pub use chat_conversation_list::{ChatConversationList, ConversationSummary};
pub use chat_header::{ChatHeader, ChatHeaderData};
pub use chat_inbox::ChatInbox;
pub use chat_input::ChatInput;
pub use chat_message_list::ChatMessageList;
pub use chat_panel::ChatPanel;
pub use chat_status_badge::{ChatStatusBadge, ChatStatusKind};
pub use chat_topic_selector::{ChatTopic, ChatTopicSelector};
pub use chat_widget::ChatWidget;
