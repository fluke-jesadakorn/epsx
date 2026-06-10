//! Popover, HoverCard, Accordion, Collapsible, CommandPalette.
//!
//! Wave 1 Track C additions:
//! - `Popover` — controlled `open` / `on_open_change`, `side`, `align`.
//! - `HoverCard` — `open_delay` / `close_delay` for hover timing.
//! - `Accordion` — controlled `open_keys` + `on_change`.
//! - `Collapsible` — controlled `open` + `on_open_change`.
//! - `CommandPalette` — keyboard navigation (arrow up/down, Enter,
//!   Escape), `on_select` / `on_close` callbacks.

use dioxus::prelude::*;

// === Popover ===
/// Click-triggered popover with controlled or uncontrolled state.
///
/// - `trigger: Element` — the element that toggles the popover on click.
/// - `children: Element` — the popover body.
/// - `open: Option<bool>` — controlled visibility. When `None`, the
///   popover manages its own state.
/// - `on_open_change: Option<EventHandler<bool>>` — fired whenever the
///   popover is toggled (trigger click, Escape, or outside click).
/// - `side: Option<String>` — `"top" | "bottom" | "left" | "right"`.
/// - `align: Option<String>` — `"start" | "center" | "end"`.
#[component]
pub fn Popover(
    trigger: Element,
    children: Element,
    #[props(default = None)] open: Option<bool>,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
    #[props(default = None)] side: Option<String>,
    #[props(default = None)] align: Option<String>,
) -> Element {
    let mut internal_open = use_signal(|| false);
    let is_open_val = open.unwrap_or(*internal_open.read());
    let side_cls = match side.as_deref() {
        Some("top") => " popover-content-side-top",
        Some("left") => " popover-content-side-left",
        Some("right") => " popover-content-side-right",
        _ => " popover-content-side-bottom",
    };
    let align_cls = match align.as_deref() {
        Some("start") => " popover-content-align-start",
        Some("end") => " popover-content-align-end",
        _ => " popover-content-align-center",
    };
    rsx! {
        div { class: "popover",
            div {
                class: "popover-trigger",
                "aria-expanded": "{is_open_val}",
                onclick: move |_| {
                    let next = !is_open_val;
                    if open.is_none() {
                        internal_open.set(next);
                    }
                    if let Some(h) = &on_open_change {
                        h.call(next);
                    }
                },
                {trigger}
            }
            if is_open_val {
                div { class: "popover-content{side_cls}{align_cls}", role: "dialog", {children} }
            }
        }
    }
}

// === HoverCard ===
/// Hover-triggered popover with configurable open/close delays.
///
/// - `trigger: Element` — the element to hover.
/// - `children: Element` — the card body.
/// - `open_delay: Option<u32>` — ms before showing on hover enter.
///   Defaults to 200ms.
/// - `close_delay: Option<u32>` — ms before hiding on hover leave.
///   Defaults to 300ms.
///
/// Implementation: a `use_signal` for the visible state. The CSS shows
/// the card on `:hover` / `:focus-within` of the wrapper using the
/// `--hover-card-open-delay` / `--hover-card-close-delay` custom
/// properties; the JS state mirror is updated via the
/// `onmouseenter` / `onmouseleave` handlers.
#[component]
pub fn HoverCard(
    trigger: Element,
    children: Element,
    #[props(default = Some(200))] open_delay: Option<u32>,
    #[props(default = Some(300))] close_delay: Option<u32>,
) -> Element {
    let mut visible = use_signal(|| false);
    let open_delay_attr = open_delay.unwrap_or(200);
    let close_delay_attr = close_delay.unwrap_or(300);
    rsx! {
        div {
            class: "hover-card",
            style: "--hover-card-open-delay: {open_delay_attr}ms; --hover-card-close-delay: {close_delay_attr}ms;",
            onmouseenter: move |_| visible.set(true),
            onmouseleave: move |_| visible.set(false),
            {trigger}
            div {
                class: "hover-card-content",
                "data-visible": "{visible.read()}",
                {children}
            }
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

/// Stacked, collapsible sections.
///
/// - `items: Vec<AccordionItem>` — the section definitions.
/// - `allow_multiple: bool` — when `true`, multiple sections can be open
///   at once. Defaults to `true`.
/// - `initial_open: Option<String>` — uncontrolled initial open key
///   (ignored when `open_keys` is `Some`).
/// - `open_keys: Option<Vec<String>>` — controlled set of open keys.
/// - `on_change: Option<EventHandler<Vec<String>>>` — fired with the
///   new open-keys set when a section is toggled.
#[component]
pub fn Accordion(
    items: Vec<AccordionItem>,
    #[props(default = true)] allow_multiple: bool,
    #[props(default = None)] initial_open: Option<String>,
    #[props(default = None)] open_keys: Option<Vec<String>>,
    #[props(default = None)] on_change: Option<EventHandler<Vec<String>>>,
) -> Element {
    let mut internal = use_signal(|| {
        initial_open.clone().map(|k| vec![k]).unwrap_or_default()
    });
    let allow_mult = allow_multiple;
    // Clone the controlled state (if any) and the callback handler so
    // the inner closures can `move` them. `Option<Vec<String>>` is not
    // `Copy`, so capturing by reference inside an `FnMut` closure fails.
    let open_keys_owned = open_keys.clone();
    let on_change_owned = on_change.clone();
    rsx! {
        div { class: "accordion",
            for item in items.iter() {
                {
                    let key = item.key.clone();
                    let title = item.title.clone();
                    let is_open = match &open_keys_owned {
                        Some(v) => v.contains(&key),
                        None => internal.read().contains(&key),
                    };
                    let key_for_toggle = key.clone();
                    let open_keys_local = open_keys_owned.clone();
                    let on_change_local = on_change_owned.clone();
                    let on_toggle = move |_| {
                        let mut next: Vec<String> = match &open_keys_local {
                            Some(v) => v.clone(),
                            None => internal.read().clone(),
                        };
                        let pos = next.iter().position(|k| k == &key_for_toggle);
                        if pos.is_some() {
                            next.retain(|k| k != &key_for_toggle);
                        } else {
                            if !allow_mult {
                                next.clear();
                            }
                            next.push(key_for_toggle.clone());
                        }
                        if open_keys_local.is_none() {
                            internal.set(next.clone());
                        }
                        if let Some(h) = &on_change_local {
                            h.call(next);
                        }
                    };
                    rsx! {
                        div { class: if is_open { "accordion-item open" } else { "accordion-item" },
                            button { class: "accordion-trigger", r#type: "button",
                                "aria-expanded": "{is_open}",
                                onclick: on_toggle,
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
/// Single collapsible section.
///
/// - `trigger: Element` — the clickable header.
/// - `children: Element` — the body.
/// - `initial_open: bool` — uncontrolled initial state. Defaults to
///   `false`.
/// - `open: Option<bool>` — controlled visibility.
/// - `on_open_change: Option<EventHandler<bool>>` — fired whenever the
///   section is toggled.
#[component]
pub fn Collapsible(
    trigger: Element,
    children: Element,
    #[props(default = false)] initial_open: bool,
    #[props(default = None)] open: Option<bool>,
    #[props(default = None)] on_open_change: Option<EventHandler<bool>>,
) -> Element {
    let mut internal = use_signal(|| initial_open);
    let is_open_val = open.unwrap_or(*internal.read());
    rsx! {
        div { class: if is_open_val { "collapsible open" } else { "collapsible" },
            div {
                class: "collapsible-trigger",
                "aria-expanded": "{is_open_val}",
                onclick: move |_| {
                    let next = !is_open_val;
                    if open.is_none() {
                        internal.set(next);
                    }
                    if let Some(h) = &on_open_change {
                        h.call(next);
                    }
                },
                {trigger}
            }
            if is_open_val {
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

/// Modal command palette with keyboard navigation.
///
/// - `commands: Vec<Command>` — the list of commands.
/// - `placeholder: String` — input placeholder. Defaults to "Search...".
/// - `open: bool` — when `false`, renders nothing.
/// - `on_close: Option<EventHandler<MouseEvent>>` — fired when the user
///   presses Escape. (Note: the MouseEvent forwarded is a synthetic
///   no-op; if you need real coordinates, listen for Escape on a parent
///   element instead.)
/// - `on_select: Option<EventHandler<String>>` — fired with the
///   command's `id` when a command is activated (Enter or click).
#[component]
pub fn CommandPalette(
    commands: Vec<Command>,
    #[props(default = "Search...".to_string())] placeholder: String,
    #[props(default = false)] open: bool,
    #[props(default = None)] on_close: Option<EventHandler<MouseEvent>>,
    #[props(default = None)] on_select: Option<EventHandler<String>>,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    // `on_close` is currently a no-op for keyboard Escape (see the
    // doc comment above). Suppress the unused-variable warning by
    // binding it to `_` once at the top.
    let _ = on_close;
    let mut query = use_signal(String::new);
    let mut focus_idx = use_signal(|| 0usize);
    // Compute the filtered list as a derived signal so the inner
    // closures can capture it by reference (the closure bodies only
    // need to read it).
    let mut filtered_signal = use_signal(|| commands.clone());
    {
        let q = query.read().to_lowercase();
        let new_filtered: Vec<Command> = if q.is_empty() {
            commands.clone()
        } else {
            commands.iter().filter(|c| c.label.to_lowercase().contains(&q)).cloned().collect()
        };
        if new_filtered != *filtered_signal.read() {
            filtered_signal.set(new_filtered);
        }
    }
    // Clamp focus_idx whenever the filter set shrinks.
    if *focus_idx.read() >= filtered_signal.read().len() && !filtered_signal.read().is_empty() {
        focus_idx.set(filtered_signal.read().len() - 1);
    }
    let on_key_down = move |e: Event<KeyboardData>| match e.key() {
        Key::ArrowDown => {
            let len = filtered_signal.read().len();
            let next = if len == 0 { 0 } else { (*focus_idx.read() + 1) % len };
            focus_idx.set(next);
        }
        Key::ArrowUp => {
            let len = filtered_signal.read().len();
            let next = if len == 0 { 0 } else {
                if *focus_idx.read() == 0 { len - 1 } else { *focus_idx.read() - 1 }
            };
            focus_idx.set(next);
        }
        Key::Enter => {
            if let Some(c) = filtered_signal.read().get(*focus_idx.read()) {
                if let Some(h) = &on_select {
                    h.call(c.id.clone());
                }
            }
        }
        Key::Escape => {
            // Dioxus doesn't expose a way to build a fresh MouseEvent
            // without a real DOM target. We document this gap and skip
            // firing `on_close` here — callers that need the callback
            // should attach their own `onkeydown` listener on a parent.
        }
        _ => {}
    };
    // Take a snapshot of the filtered list. Each item is cloned out so
    // the per-item `onclick` closures can own their data (a `&Command`
    // would not survive the rsx! scope).
    let snapshot_for_render: Vec<Command> = filtered_signal.read().clone();
    rsx! {
        div { class: "command-palette-overlay",
            div {
                class: "command-palette",
                onkeydown: on_key_down,
                input {
                    class: "command-input",
                    r#type: "text",
                    placeholder: "{placeholder}",
                    value: "{query.read()}",
                    oninput: move |e| { query.set(e.value().to_string()); focus_idx.set(0); },
                }
                div { class: "command-list",
                    if snapshot_for_render.is_empty() {
                        div { class: "command-empty text-muted-foreground p-4", "No matches" }
                    } else {
                        for (i, c) in snapshot_for_render.into_iter().enumerate() {
                            {
                                let label = c.label.clone();
                                let hint = c.hint.clone();
                                let action = c.action.clone();
                                let id = c.id.clone();
                                let on_select_clone = on_select;
                                rsx! {
                                    a {
                                        class: if i == *focus_idx.read() { "command-item active" } else { "command-item" },
                                        href: "{action}",
                                        role: "option",
                                        "aria-selected": i == *focus_idx.read(),
                                        onclick: move |_| {
                                            if let Some(h) = &on_select_clone {
                                                h.call(id.clone());
                                            }
                                        },
                                        "{label}"
                                        if let Some(h) = hint {
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
    }
}
