//! Combobox — async-searchable, keyboard-navigable select with free-text.

use dioxus::prelude::*;

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
        div { class: "field combobox",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            div { class: "combobox-wrap",
                input {
                    class: "input",
                    name: "{name}",
                    r#type: "text",
                    role: "combobox",
                    "aria-expanded": "false",
                    "aria-autocomplete": "list",
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
