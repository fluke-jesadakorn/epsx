//! `ChatStatusBadge` — small badge showing chat status
//! (online / offline / typing).
//!
//! Port of `apps-old/frontend/components/chat/chat-status-badge.tsx`
//! (20 LoC). The TS source renders a tiny pill with the status
//! label. The Dioxus port renders the same structure.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ChatStatusKind {
    #[default]
    Online,
    Offline,
    Typing,
    Away,
}

#[component]
pub fn ChatStatusBadge(kind: ChatStatusKind) -> Element {
    let (label, color) = match kind {
        ChatStatusKind::Online => ("Online", "green"),
        ChatStatusKind::Offline => ("Offline", "slate"),
        ChatStatusKind::Typing => ("Typing…", "orange"),
        ChatStatusKind::Away => ("Away", "yellow"),
    };
    rsx! {
        span { class: "chat-status-badge inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-{color}-500/10 text-{color}-500",
            span { class: "h-1.5 w-1.5 rounded-full bg-{color}-500" }
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_status_kind_default_is_online() {
        let k = ChatStatusKind::default();
        assert_eq!(k, ChatStatusKind::Online);
    }

    #[test]
    fn chat_status_badge_smoke() {
        
    }
}
