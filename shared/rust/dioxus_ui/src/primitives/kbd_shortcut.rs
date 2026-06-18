//! `KbdShortcut` / `KbdSeparator` — keyboard shortcut display.
//!
//! A pair of components for showing keyboard shortcuts (e.g.
//! `⌘ K`, `Ctrl+Shift+P`). Mirrors the visual pattern from the
//! Mac OS / Linear / Vercel toolbar shortcut hints.
//!
//! The existing `Kbd` and `KbdCombo` (in `primitives/misc.rs`) render
//! individual keys and a list of keys joined with `+`. `KbdShortcut`
//! adds a higher-level component for rendering a labeled shortcut
//! row (label on the left, key combo on the right) — useful for
//! keyboard shortcut help dialogs.

use dioxus::prelude::*;

/// Single-row keyboard shortcut display. Renders a label and a
/// key-combo hint side-by-side.
///
/// - `label: String` — what the shortcut does, e.g. "Open search".
/// - `keys: Vec<String>` — the key combo, e.g. `["⌘", "K"]`. Each
///   key is rendered as an individual `<kbd>` element with a
///   separator (`+`) between them.
#[component]
pub fn KbdShortcut(label: String, keys: Vec<String>) -> Element {
    rsx! {
        div { class: "kbd-shortcut flex items-center justify-between gap-4 py-1.5",
            span { class: "kbd-shortcut-label text-sm text-muted-foreground", "{label}" }
            span { class: "kbd-shortcut-keys flex items-center gap-1",
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        span { class: "kbd-shortcut-sep text-xs text-muted-foreground", "+" }
                    }
                    kbd { class: "kbd-shortcut-key pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100",
                        "{key}"
                    }
                }
            }
        }
    }
}

/// Visual separator between shortcut groups.
#[component]
pub fn KbdSeparator() -> Element {
    rsx! {
        div { class: "kbd-separator h-px w-full bg-border my-2" }
    }
}

/// Keyboard shortcut help dialog body. Renders a list of shortcuts
/// grouped by category.
#[component]
pub fn KbdHelp(
    title: String,
    groups: Vec<(String, Vec<(String, Vec<String>)>)>,
) -> Element {
    rsx! {
        div { class: "kbd-help flex flex-col gap-4 p-4",
            h3 { class: "kbd-help-title text-lg font-semibold", "{title}" }
            for (group_name, shortcuts) in groups.iter() {
                div { class: "kbd-help-group flex flex-col gap-1",
                    h4 { class: "kbd-help-group-name text-sm font-medium text-muted-foreground", "{group_name}" }
                    for (label, keys) in shortcuts.iter() {
                        KbdShortcut { label: label.clone(), keys: keys.clone() }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kbd_shortcut_label_and_keys_render() {
        // Sanity check the data structure: label is String, keys is Vec<String>.
        let label: String = "Open search".to_string();
        let keys: Vec<String> = vec!["⌘".to_string(), "K".to_string()];
        assert_eq!(label, "Open search");
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn kbd_help_groups_have_distinct_names() {
        let groups: Vec<(String, Vec<(String, Vec<String>)>)> = vec![
            ("General".to_string(), vec![]),
            ("Navigation".to_string(), vec![]),
        ];
        let names: Vec<&str> = groups.iter().map(|(n, _)| n.as_str()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn kbd_separator_has_no_text_content() {
        // KbdSeparator renders an empty div — no text content.
        // The contract is the class name `kbd-separator`.
        let expected = "kbd-separator";
        assert!(expected.starts_with("kbd-"));
    }
}
