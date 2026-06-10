//! Popover, HoverCard, Accordion, Collapsible, CommandPalette.

use dioxus::prelude::*;

// === Popover ===
#[component]
pub fn Popover(trigger: Element, children: Element) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "popover",
            div { class: "popover-trigger", onclick: move |_| open.toggle(),
                {trigger}
            }
            if *open.read() {
                div { class: "popover-content",
                    {children}
                }
            }
        }
    }
}

// === HoverCard (CSS-only) ===
#[component]
pub fn HoverCard(trigger: Element, children: Element) -> Element {
    rsx! {
        div { class: "hover-card",
            {trigger}
            div { class: "hover-card-content", {children} }
        }
    }
}

// === Accordion ===
#[derive(Clone, Debug, PartialEq)]
pub struct AccordionItem {
    pub key: String,
    pub title: String,
    pub content: Element,
}

#[component]
pub fn Accordion(items: Vec<AccordionItem>, #[props(default = None)] initial_open: Option<String>, #[props(default = true)] allow_multiple: bool) -> Element {
    let mut open_keys = use_signal(|| {
        initial_open.clone().map(|k| vec![k]).unwrap_or_default()
    });
    rsx! {
        div { class: "accordion",
            for item in items.iter() {
                {
                    let key = item.key.clone();
                    let title = item.title.clone();
                    let is_open = open_keys.read().contains(&key);
                    let toggle = move |_| {
                        if allow_multiple {
                            if is_open { open_keys.write().retain(|k| k != &key); }
                            else { open_keys.write().push(key.clone()); }
                        } else {
                            if is_open { open_keys.write().clear(); }
                            else { open_keys.write().clear(); open_keys.write().push(key.clone()); }
                        }
                    };
                    rsx! {
                        div { class: if is_open { "accordion-item open" } else { "accordion-item" },
                            button { class: "accordion-trigger", r#type: "button",
                                onclick: toggle,
                                "{title}"
                                span { class: "accordion-icon", if is_open { "−" } else { "+" } }
                            }
                            if is_open {
                                div { class: "accordion-content",
                                    {
                                        let content_el = item.content.clone();
                                        rsx! { Fragment { {content_el} } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// === Collapsible ===
#[component]
pub fn Collapsible(trigger: Element, children: Element, #[props(default = false)] initial_open: bool) -> Element {
    let mut open = use_signal(|| initial_open);
    rsx! {
        div { class: "collapsible",
            div { class: "collapsible-trigger", onclick: move |_| open.toggle(), {trigger} }
            if *open.read() {
                div { class: "collapsible-content", {children} }
            }
        }
    }
}

// === CommandPalette ===
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub hint: Option<String>,
    pub icon: Option<String>,
    pub action: String,
}

#[component]
pub fn CommandPalette(
    commands: Vec<Command>,
    #[props(default = "Search...".to_string())] placeholder: String,
    #[props(default = false)] open: bool,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    let mut query = use_signal(String::new);
    let mut focus_idx = use_signal(|| 0usize);
    let q = query.read().to_lowercase();
    let filtered: Vec<Command> = if q.is_empty() {
        commands.clone()
    } else {
        commands.iter().filter(|c| c.label.to_lowercase().contains(&q)).cloned().collect()
    };
    rsx! {
        div { class: "command-palette-overlay",
            div { class: "command-palette",
                input {
                    class: "command-input",
                    r#type: "text",
                    placeholder: "{placeholder}",
                    value: "{query.read()}",
                    oninput: move |e| { query.set(e.value().to_string()); focus_idx.set(0); },
                }
                div { class: "command-list",
                    if filtered.is_empty() {
                        div { class: "command-empty text-muted-foreground p-4", "No matches" }
                    } else {
                        for (i, c) in filtered.iter().enumerate() {
                            a {
                                class: if i == *focus_idx.read() { "command-item active" } else { "command-item" },
                                href: "{c.action}",
                                "{c.label}"
                                if let Some(h) = &c.hint {
                                    span { class: "command-hint text-muted-foreground ml-auto", "{h}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
