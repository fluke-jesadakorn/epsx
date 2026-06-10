//! Combobox — async-searchable, keyboard-navigable select with free-text.
//!
//! Three variants:
//! - `Combobox`     — fixed option list, single-select.
//! - `ComboboxAsync` — externally-loaded option list (the parent fetches
//!   options and passes them in along with a `loading` flag).
//! - `ComboboxMulti` — multi-select with chips.

use dioxus::prelude::*;

/// Single-select combobox with a fixed option list.
///
/// Renders a `<input role="combobox">` with an absolutely-positioned listbox
/// underneath. Keyboard navigation: ArrowUp/Down to move selection, Escape
/// to close, Enter to commit (handled by the browser default for inputs).
#[component]
pub fn Combobox(
    name: String,
    label: Option<String>,
    options: Vec<(String, String)>,
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    #[props(default = false)] allow_free: bool,
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(|| value.clone().unwrap_or_default());
    let mut focus_idx = use_signal(|| 0usize);

    let q = query.read().to_lowercase();
    let filtered: Vec<(String, String)> = if q.is_empty() {
        options.clone()
    } else {
        options.iter().filter(|(_, l)| l.to_lowercase().contains(&q)).cloned().collect()
    };
    let filtered_len = filtered.len();

    let mut wrap_cls = "combobox-wrap".to_string();
    if let Some(c) = &class_name { wrap_cls.push(' '); wrap_cls.push_str(c); }

    rsx! {
        div { class: "field combobox",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            div { class: "{wrap_cls}",
                input {
                    class: "input",
                    name: "{name}",
                    r#type: "text",
                    role: "combobox",
                    "aria-expanded": open.read().to_string(),
                    "aria-autocomplete": "list",
                    "aria-invalid": if error.is_some() { "true" } else { "false" },
                    placeholder: placeholder.as_deref().unwrap_or("Type to search..."),
                    required: required && !allow_free,
                    value: "{query.read()}",
                    onfocus: move |_| open.set(true),
                    onblur: move |_| { let _ = open.set(false); },
                    oninput: move |e| { query.set(e.value().to_string()); open.set(true); focus_idx.set(0); },
                    onkeydown: move |e| {
                        let k = e.key();
                        if k == Key::ArrowDown {
                            let f = (*focus_idx.read() + 1).min(filtered_len.saturating_sub(1));
                            focus_idx.set(f);
                        } else if k == Key::ArrowUp {
                            let f = focus_idx.read().saturating_sub(1);
                            focus_idx.set(f);
                        } else if k == Key::Escape {
                            open.set(false);
                        }
                    },
                }
                if *open.read() && !filtered.is_empty() {
                    ul { class: "combobox-menu", role: "listbox",
                        for (i, (val, lbl)) in filtered.iter().enumerate() {
                            {
                                let val_clone = val.clone();
                                let lbl_clone = lbl.clone();
                                let val_for_href = val_clone.clone();
                                rsx! {
                                    li {
                                        role: "option",
                                        "aria-selected": (i == *focus_idx.read()).to_string(),
                                        class: if i == *focus_idx.read() { "combobox-item active" } else { "combobox-item" },
                                        onclick: move |_| {
                                            query.set(lbl_clone.clone());
                                            let _ = val_for_href.clone();
                                            open.set(false);
                                        },
                                        "{lbl}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}

/// Combobox with an externally-loaded option list.
///
/// The parent component owns the fetch logic (e.g. a debounced search call
/// to `/api/...`) and passes the resulting `options: Vec<(value, label)>`
/// plus a `loading: bool` flag. The combobox renders a spinner when
/// `loading` is true and a friendly empty-state when the list is empty.
///
/// `on_select: Option<EventHandler<String>>` is called when the user picks
/// an option from the dropdown. The parent typically uses this to flip
/// `value` and trigger downstream effects.
#[component]
pub fn ComboboxAsync(
    name: String,
    /// Externally-loaded option list. The parent may pass an empty Vec
    /// while the initial fetch is in flight.
    options: Vec<(String, String)>,
    /// When `true`, render a loading indicator inside the menu.
    #[props(default = false)] loading: bool,
    /// Currently-selected value (controlled).
    #[props(default = None)] value: Option<String>,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    /// Text shown when `options` is empty and not loading.
    #[props(default = None)] empty_text: Option<String>,
    #[props(default = false)] required: bool,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(|| value.clone().unwrap_or_default());
    let mut focus_idx = use_signal(|| 0usize);

    let q = query.read().to_lowercase();
    let filtered: Vec<(String, String)> = if q.is_empty() {
        options.clone()
    } else {
        options.iter().filter(|(_, l)| l.to_lowercase().contains(&q)).cloned().collect()
    };
    let filtered_len = filtered.len();

    rsx! {
        div { class: "field combobox combobox-async",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            div { class: "combobox-wrap",
                input {
                    class: "input",
                    name: "{name}",
                    r#type: "text",
                    role: "combobox",
                    "aria-expanded": open.read().to_string(),
                    "aria-busy": loading.to_string(),
                    "aria-autocomplete": "list",
                    "aria-invalid": if error.is_some() { "true" } else { "false" },
                    placeholder: placeholder.as_deref().unwrap_or("Type to search..."),
                    required: required,
                    value: "{query.read()}",
                    onfocus: move |_| open.set(true),
                    onblur: move |_| { let _ = open.set(false); },
                    oninput: move |e| { query.set(e.value().to_string()); open.set(true); focus_idx.set(0); },
                    onkeydown: move |e| {
                        let k = e.key();
                        if k == Key::ArrowDown {
                            let f = (*focus_idx.read() + 1).min(filtered_len.saturating_sub(1));
                            focus_idx.set(f);
                        } else if k == Key::ArrowUp {
                            let f = focus_idx.read().saturating_sub(1);
                            focus_idx.set(f);
                        } else if k == Key::Escape {
                            open.set(false);
                        }
                    },
                }
                if *open.read() {
                    ul { class: "combobox-menu", role: "listbox",
                        if loading {
                            li { class: "combobox-loading text-muted-foreground", role: "presentation", "Loading…" }
                        } else if filtered.is_empty() {
                            li { class: "combobox-empty text-muted-foreground", role: "presentation",
                                "{empty_text.as_deref().unwrap_or(\"No matches\")}"
                            }
                        } else {
                            for (i, (val, lbl)) in filtered.iter().enumerate() {
                                {
                                    let val_clone = val.clone();
                                    let lbl_clone = lbl.clone();
                                    let val_for_pick = val_clone.clone();
                                    let val_for_href = val_clone.clone();
                                    rsx! {
                                        li {
                                            role: "option",
                                            "aria-selected": (i == *focus_idx.read()).to_string(),
                                            class: if i == *focus_idx.read() { "combobox-item active" } else { "combobox-item" },
                                            onclick: move |_| {
                                                query.set(lbl_clone.clone());
                                                let _ = val_for_href.clone();
                                                open.set(false);
                                                if let Some(h) = &on_select { h.call(val_for_pick.clone()); }
                                            },
                                            "{lbl}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}

/// Multi-select combobox with chips.
///
/// Renders a row of removable chips for the currently-selected values plus
/// a combobox input to search and add more. Selection state is fully
/// controlled via `value: Vec<String>` + `on_change: EventHandler<Vec<String>>`.
#[component]
pub fn ComboboxMulti(
    name: String,
    options: Vec<(String, String)>,
    value: Vec<String>,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
    /// Maximum number of selections allowed. None = unlimited.
    #[props(default = None)] max: Option<usize>,
    on_change: Option<EventHandler<Vec<String>>>,
) -> Element {
    // Use a Signal so the closures can clone on every call without moving.
    let value_signal = use_signal(|| value.clone());
    let mut open = use_signal(|| false);
    let mut query = use_signal(|| String::new());
    let value_for_render = value.clone();
    let placeholder_str = placeholder.unwrap_or_else(|| "Type to search…".to_string());

    let q = query.read().to_lowercase();
    let filtered: Vec<(String, String)> = if q.is_empty() {
        options.clone()
    } else {
        options.iter().filter(|(_, l)| l.to_lowercase().contains(&q)).cloned().collect()
    };

    rsx! {
        div { class: "field combobox combobox-multi",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" if required { span { class: "text-red-500 ml-1", "*" } } }
            }
            div {
                class: "combobox-multi-control flex flex-wrap items-center gap-2 input",
                role: "listbox",
                "aria-multiselectable": "true",
                for v in value_for_render.iter() {
                    {
                        let v_for_chip = v.clone();
                        let v_for_remove = v.clone();
                        let label_for_chip = options.iter()
                            .find(|(val, _)| val == &v_for_chip)
                            .map(|(_, l)| l.clone())
                            .unwrap_or_else(|| v_for_chip.clone());
                        let value_signal_for_remove = value_signal.clone();
                        rsx! {
                            span { class: "combobox-multi-chip badge badge-primary", role: "presentation",
                                "{label_for_chip}"
                                button {
                                    r#type: "button",
                                    class: "combobox-multi-chip-remove",
                                    "aria-label": format!("Remove {}", label_for_chip),
                                    onclick: move |_| {
                                        let next: Vec<String> = value_signal_for_remove.read().iter()
                                            .filter(|x| x.as_str() != v_for_remove.as_str())
                                            .cloned()
                                            .collect();
                                        if let Some(h) = &on_change { h.call(next); }
                                    },
                                    "×"
                                }
                            }
                        }
                    }
                }
                input {
                    r#type: "text",
                    class: "combobox-multi-input flex-1 min-w-[8rem]",
                    role: "combobox",
                    "aria-expanded": open.read().to_string(),
                    "aria-autocomplete": "list",
                    placeholder: placeholder_str.clone(),
                    value: "{query.read()}",
                    onfocus: move |_| open.set(true),
                    onblur: move |_| { let _ = open.set(false); },
                    oninput: move |e| { query.set(e.value().to_string()); open.set(true); },
                    onkeydown: move |e| {
                        if e.key() == Key::Escape { open.set(false); }
                    },
                }
            }
            if *open.read() {
                ul { class: "combobox-menu", role: "listbox",
                    for (val, lbl) in filtered.iter() {
                        {
                            let val_for_toggle = val.clone();
                            let at_max = max.map_or(false, |m| value_signal.read().len() >= m);
                            let is_selected = value_signal.read().contains(&val_for_toggle);
                            let disabled_for_option = at_max && !is_selected;
                            let value_signal_for_toggle = value_signal.clone();
                            let val_for_toggle_in_closure = val_for_toggle.clone();
                            rsx! {
                                li {
                                    role: "option",
                                    "aria-selected": is_selected.to_string(),
                                    class: if is_selected { "combobox-item active" } else { "combobox-item" },
                                    onclick: move |_| {
                                        if disabled_for_option { return; }
                                        let mut next: Vec<String> = value_signal_for_toggle.read().iter().cloned().collect();
                                        if next.contains(&val_for_toggle_in_closure) {
                                            next.retain(|x| x != &val_for_toggle_in_closure);
                                        } else {
                                            next.push(val_for_toggle_in_closure.clone());
                                        }
                                        if let Some(h) = &on_change { h.call(next); }
                                        query.set(String::new());
                                    },
                                    "{lbl}"
                                }
                            }
                        }
                    }
                }
            }
            // Native mirror so the form submits the chips as repeated form fields.
            for v in value_for_render.iter() {
                input { r#type: "hidden", name: "{name}", value: "{v}" }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}
