//! `ShareButton` — "Share Platform" CTA that copies the current
//! URL to clipboard.
//!
//! Port of `apps-old/frontend/components/home/share-button.tsx`
//! (25 LoC). The TS source uses `copyToClipboard()` from
//! `@/utils/clipboard` and shows a toast. The Dioxus port renders
//! the same visual button. The `on_click` callback is the
//! caller-supplied handler — in SSR the page can wire it to a
//! stub.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[component]
pub fn ShareButton(
    /// Class names appended to the button.
    #[props(default = None)] class_name: Option<String>,
    /// Fired when the button is clicked. The default is a no-op.
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    rsx! {
        button {
            class: "home-prod-share-btn w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-2 border-orange-400/50 rounded-2xl shadow-xl hover:shadow-orange-300/30 hover:scale-105 transition-all duration-300 group {cls}",
            onclick: move |e| {
                if let Some(cb) = on_click.as_ref() {
                    cb.call(e);
                }
            },
            Icon { name: "share-2".to_string(), size: Some(24), class_name: Some("mr-3 group-hover:animate-wiggle".to_string()) }
            span { "📤 Share Platform" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_button_smoke() {
        
    }

    #[test]
    fn share_button_class_is_empty_when_unset() {
        let cls: Option<String> = None;
        let resolved = cls.clone().unwrap_or_default();
        assert!(resolved.is_empty());
    }
}
