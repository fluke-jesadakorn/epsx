//! `ChatBubble` — minimal text bubble used by the chat-widget
//! floating panel.
//!
//! Port of `apps-old/frontend/components/chat/chat-bubble.tsx`
//! (42 LoC). The TS source renders a rounded bubble with a
//! sender label + body. The Dioxus port renders the same
//! structure.

use dioxus::prelude::*;

#[component]
pub fn ChatBubble(
    sender: String,
    body: String,
    /// Whether the bubble is from the current user (drives
    /// alignment + color).
    #[props(default = false)] is_self: bool,
) -> Element {
    let align = if is_self { "items-end" } else { "items-start" };
    let color = if is_self { "bg-orange-500 text-white" } else { "bg-slate-100 dark:bg-slate-800 text-foreground" };
    rsx! {
        div { class: "chat-bubble flex flex-col {align} gap-1",
            span { class: "chat-bubble-sender text-xs text-slate-500", "{sender}" }
            div { class: "chat-bubble-body max-w-xs rounded-2xl px-4 py-2 {color}",
                "{body}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_bubble_smoke() {
        
    }
}
