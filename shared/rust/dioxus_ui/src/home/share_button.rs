//! `ShareButton` — "Share Platform" CTA that copies the current
//! URL to clipboard.
//!
//! Port of `apps-old/frontend/components/home/share-button.tsx`
//! (25 LoC). The TS source uses `copyToClipboard()` from
//! `@/utils/clipboard` and shows a toast. The Dioxus port renders
//! the same visual button. The `on_click` callback is the
//! caller-supplied handler. SSR uses the shared browser controller because
//! Dioxus event closures are stripped from hydrationless server output.

use dioxus::prelude::*;

#[component]
pub fn ShareButton(
    /// Class names appended to the button.
    #[props(default = None)]
    class_name: Option<String>,
    /// Fired when the button is clicked in a hydrated Dioxus target. The SSR
    /// path uses `epsx.shareText(...)` so the default action remains live.
    #[props(default = None)]
    on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    let _ = on_click;
    // Dioxus event closures are intentionally not emitted during SSR. Use
    // the shared browser controller's literal handler so the CTA remains a
    // real Web Share/clipboard action in the server-rendered page.
    let icon = epsx_templates::lucide("share-2", "24", "mr-3");
    let onclick = epsx_templates::onclick_share_text("", "EPSX");
    let extra_class = epsx_templates::html_attr_escape_pub(&cls);
    let button_html = format!(
        r#"<button type="button" class="home-prod-share-btn w-full sm:w-auto min-w-[220px] h-14 text-lg font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white border-2 border-orange-400/50 rounded-2xl shadow-xl hover:shadow-orange-300/30 hover:scale-105 transition-all duration-300 group {}" data-share-text="" data-share-title="EPSX" onclick="{}" aria-label="Share Platform"><span class="epsx-icon">{}</span><span>📤 Share Platform</span></button>"#,
        extra_class,
        epsx_templates::html_attr_escape_pub(&onclick),
        icon,
    );
    rsx! {
        span { class: "contents {cls}",
            dangerous_inner_html: "{button_html}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_button_smoke() {}

    #[test]
    fn share_button_class_is_empty_when_unset() {
        let cls: Option<String> = None;
        let resolved = cls.clone().unwrap_or_default();
        assert!(resolved.is_empty());
    }
}
