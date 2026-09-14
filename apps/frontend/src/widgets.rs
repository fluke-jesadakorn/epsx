//! Global widgets injected into frontend pages.

/// Floating support link shown on authenticated pages outside `/chat`.
///
/// The backend does not expose a verified unread-chat counter or an embeddable
/// conversation contract. Keep the familiar floating affordance, but make it a
/// plain navigation link instead of presenting a synthetic online status,
/// response-time promise, message, or notification-derived unread badge.
pub fn chat_widget(is_authed: bool, _user_id: &str) -> String {
    if !is_authed {
        return String::new();
    }
    let icon = epsx_templates::lucide("message-circle", "24", "chat-bubble-icon");
    format!(
        r##"<div id="chat-widget" class="fixed bottom-6 right-6 z-50">
  <a class="chat-bubble-btn relative flex size-14 items-center justify-center rounded-full border-0 bg-[linear-gradient(135deg,#3b82f6_0%,#2563eb_55%,#4f46e5_100%)] text-white no-underline shadow-lg cursor-pointer transition-all duration-300 hover:scale-105" href="/chat" aria-label="Open support chat">
    {icon}
  </a>
</div>
"##
    )
}

#[cfg(test)]
mod tests {
    use super::chat_widget;

    #[test]
    fn support_affordance_is_authenticated_navigation_without_fake_runtime() {
        assert!(chat_widget(false, "owner").is_empty());

        let rendered = chat_widget(true, "owner");
        assert!(rendered.contains("href=\"/chat\""));
        assert!(rendered.contains("aria-label=\"Open support chat\""));
        assert!(rendered.contains("<svg"));
        assert!(rendered.contains("lucide-message-circle"));
        assert!(!rendered.contains("data-lucide"));
        for forbidden in [
            "<script",
            "<button",
            "unread-count",
            "chat-bubble-badge",
            "setInterval",
            "Online",
            "replies within minutes",
            "Hi! How can we help?",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "unsupported chat widget claim or control leaked: {forbidden}"
            );
        }
    }
}
