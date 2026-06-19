//! `ChatInput` — message input with send button.
//!
//! Port of `apps-old/frontend/components/chat/chat-input.tsx`
//! (177 LoC). The TS source renders a textarea + send button +
//! emoji picker. The Dioxus port renders the same visual
//! structure as a simple form (no emoji picker in SSR).

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ChatInput(
    /// Placeholder text for the textarea.
    #[props(default = "Type a message…".to_string())] placeholder: String,
    /// Fired when the user clicks the send button. The new
    /// message text is forwarded to the parent.
    #[props(default = None)] on_send: Option<EventHandler<String>>,
) -> Element {
    let mut text = use_signal(String::new);
    rsx! {
        div { class: "chat-input p-4 border-t border-slate-200 dark:border-slate-700",
            div { class: "chat-input-form flex items-end gap-2",
                textarea {
                    class: "chat-input-textarea flex-1 px-3 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-orange-500",
                    placeholder: "{placeholder}",
                    rows: "2",
                    value: "{text}",
                    oninput: move |e| text.set(e.value()),
                }
                button {
                    class: "chat-input-send p-2 bg-orange-500 text-white rounded-lg hover:bg-orange-600 disabled:opacity-50",
                    disabled: text.read().is_empty(),
                    onclick: move |_| {
                        if let Some(cb) = on_send.as_ref() {
                            cb.call(text.read().clone());
                        }
                        text.set(String::new());
                    },
                    Icon { name: "send".to_string(), size: Some(20) }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_input_smoke() {
        
    }
}
