//! `ChatWidget` — floating chat bubble trigger that opens the
//! chat panel.
//!
//! Port of `apps-old/frontend/components/chat/chat-widget.tsx`
//! (67 LoC). The TS source renders a fixed-position floating
//! bubble in the bottom-right corner that, when clicked, opens
//! the chat panel. The Dioxus port renders the same visual
//! structure.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ChatWidget(
    /// Whether the panel is currently open.
    #[props(default = false)] is_open: bool,
    /// Fired when the user clicks the bubble trigger.
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        button {
            class: "chat-widget fixed bottom-4 right-4 w-14 h-14 rounded-full bg-orange-500 text-white shadow-2xl flex items-center justify-center hover:bg-orange-600 transition-all z-40",
            onclick: move |e| {
                if let Some(cb) = on_click.as_ref() {
                    cb.call(e);
                }
            },
            Icon {
                name: if is_open { "x".to_string() } else { "message-circle".to_string() },
                size: Some(24),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_widget_smoke() {
        
    }
}
