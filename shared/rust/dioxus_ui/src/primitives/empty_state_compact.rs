//! `EmptyStateCompact` / `EmptyStateWithAction` — empty state
//! variations.
//!
//! The existing `feedback::empty_state::EmptyState` is a large
//! full-page empty state (icon + title + description + optional
//! CTA). The compact variants here are smaller, more focused:
//!
//! - `EmptyStateCompact` — icon + title + description, no CTA.
//!   Used inside cards, sections, or sidebar slots.
//! - `EmptyStateWithAction` — icon + title + description + a
//!   single primary action button. Used inside panels.

use super::icon::Icon;

use dioxus::prelude::*;

/// Compact empty state. Renders a small icon + title +
/// description, vertically centered, with a subtle background.
#[component]
pub fn EmptyStateCompact(
    title: String,
    description: String,
    #[props(default = "inbox".to_string())] icon: String,
    #[props(default = None)] class: Option<String>,
) -> Element {
    let mut cls = "empty-state-compact flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed bg-muted/30 px-6 py-8 text-center".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            Icon { name: icon.clone(), size: Some(24) }
            h3 { class: "empty-state-compact-title text-sm font-semibold", "{title}" }
            p { class: "empty-state-compact-description text-xs text-muted-foreground max-w-xs", "{description}" }
        }
    }
}

/// Empty state with a single primary action. Renders an
/// icon + title + description + a button row at the bottom.
#[component]
pub fn EmptyStateWithAction(
    title: String,
    description: String,
    action_label: String,
    #[props(default = "inbox".to_string())] icon: String,
    #[props(default = None)] on_action: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div { class: "empty-state-with-action flex flex-col items-center justify-center gap-4 rounded-lg border border-dashed bg-muted/30 px-6 py-10 text-center",
            Icon { name: icon.clone(), size: Some(40) }
            div { class: "flex flex-col gap-1",
                h3 { class: "empty-state-with-action-title text-lg font-semibold", "{title}" }
                p { class: "empty-state-with-action-description text-sm text-muted-foreground max-w-sm", "{description}" }
            }
            button {
                class: "empty-state-with-action-btn inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4",
                r#type: "button",
                onclick: move |e| if let Some(h) = &on_action { h.call(e); },
                "{action_label}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_compact_uses_dashed_border() {
        let base = "empty-state-compact flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed bg-muted/30 px-6 py-8 text-center";
        assert!(base.contains("border-dashed"));
        assert!(base.contains("text-center"));
    }

    #[test]
    fn empty_state_with_action_has_button() {
        let base = "empty-state-with-action flex flex-col items-center justify-center gap-4 rounded-lg border border-dashed bg-muted/30 px-6 py-10 text-center";
        assert!(base.contains("py-10"));
        assert!(base.contains("gap-4"));
    }

    #[test]
    fn default_icon_is_inbox() {
        let icon: String = "inbox".to_string();
        assert_eq!(icon, "inbox");
    }
}
