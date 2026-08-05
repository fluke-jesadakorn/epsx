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
    r##"<div id="chat-widget" style="position:fixed;bottom:1.5rem;right:1.5rem;z-index:50;">
  <a class="chat-bubble-btn" href="/chat" aria-label="Open support chat" style="width:3.5rem;height:3.5rem;border-radius:9999px;background:linear-gradient(135deg,#3b82f6 0%,#2563eb 55%,#4f46e5 100%);color:white;border:none;cursor:pointer;box-shadow:0 10px 15px -3px rgba(0,0,0,.2),0 4px 6px -4px rgba(0,0,0,.2);display:flex;align-items:center;justify-content:center;position:relative;transition:all 0.3s;text-decoration:none;">
    <i data-lucide="message-circle" style="width:1.5rem;height:1.5rem;"></i>
  </a>
</div>
<style>
.chat-bubble-btn:hover { transform:scale(1.05); box-shadow:0 20px 25px -5px rgba(59,130,246,.25),0 8px 10px -6px rgba(59,130,246,.25); }
</style>
"##
    .to_string()
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
