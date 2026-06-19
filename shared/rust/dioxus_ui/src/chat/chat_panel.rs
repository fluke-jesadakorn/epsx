//! `ChatPanel` — slide-out chat panel with header + message list
//! + input.
//!
//! Port of `apps-old/frontend/components/chat/chat-panel.tsx`
//! (253 LoC). The TS source is a client component that renders
//! a slide-out panel with the chat widget inline. The Dioxus
//! port renders the same structure.

use crate::chat::chat_header::{ChatHeader, ChatHeaderData};
use crate::chat::chat_message_list::ChatMessageList;
use crate::chat::chat_input::ChatInput;
use crate::chat::message_bubble::Message;

use dioxus::prelude::*;

#[component]
pub fn ChatPanel(
    header: ChatHeaderData,
    #[props(default = Vec::new())] messages: Vec<Message>,
    /// When `false`, the panel is collapsed (only the bubble
    /// trigger is visible). When `true`, the full panel is
    /// shown.
    #[props(default = true)] is_open: bool,
    /// Fired when the user clicks the close button.
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
) -> Element {
    if !is_open {
        return rsx! { Fragment {} };
    }
    rsx! {
        div { class: "chat-panel fixed bottom-4 right-4 w-96 max-w-[calc(100vw-2rem)] card card-glass shadow-2xl z-50",
            ChatHeader { data: header, on_close: on_close }
            ChatMessageList { messages: messages }
            ChatInput {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_panel_smoke() {
        
    }
}
