//! `ChatInbox` — full chat inbox view (list + active conversation).
//!
//! Port of `apps-old/frontend/components/chat/chat-inbox.tsx`
//! (414 LoC). The TS source is a client component that fetches
//! conversations and the active conversation's messages, then
//! renders a 2-pane layout (sidebar + main). The Dioxus port
//! renders the same shell with a static layout (the data is
//! provided by the caller via page context).

use crate::chat::chat_conversation_list::{ChatConversationList, ConversationSummary};
use crate::chat::chat_header::{ChatHeader, ChatHeaderData};
use crate::chat::chat_message_list::ChatMessageList;
use crate::chat::chat_input::ChatInput;

use dioxus::prelude::*;

#[component]
pub fn ChatInbox(
    #[props(default = Vec::new())] conversations: Vec<ConversationSummary>,
    /// Header data for the active conversation.
    active_header: ChatHeaderData,
    /// Messages in the active conversation.
    #[props(default = Vec::new())] active_messages: Vec<crate::chat::message_bubble::Message>,
) -> Element {
    rsx! {
        div { class: "chat-inbox grid grid-cols-1 md:grid-cols-3 gap-4",
            div { class: "chat-inbox-sidebar md:col-span-1",
                ChatConversationList { items: conversations }
            }
            div { class: "chat-inbox-main md:col-span-2 card card-glass",
                ChatHeader { data: active_header }
                ChatMessageList { messages: active_messages }
                ChatInput {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_inbox_smoke() {
        
    }
}
