//! `RichTextEditor` — minimal markdown-friendly editor for news / policy pages.
//!
//! Server-renders the editor with a textarea + preview. Hydrated toolbar
//! actions update Dioxus signals directly; no browser eval bridge is used.

use dioxus::prelude::*;

#[component]
pub fn RichTextEditor(
    name: String,
    #[props(default = None)] label: Option<String>,
    #[props(default = None)] value: Option<String>,
    #[props(default = 8)] rows: usize,
    #[props(default = None)] placeholder: Option<String>,
    #[props(default = None)] help: Option<String>,
    #[props(default = None)] error: Option<String>,
    #[props(default = false)] required: bool,
) -> Element {
    let mut mode = use_signal(|| "edit".to_string());
    let mut content = use_signal(|| value.clone().unwrap_or_default());

    let preview_html = render_markdown(content.read().as_str());

    // Formatting is deterministic and signal-owned. Without browser
    // selection state, insert at the end and keep the field fully usable.
    let apply_wrap = move |prefix: String, suffix: String, placeholder_text: String| {
        content.with_mut(|value| {
            if !value.is_empty() && !value.ends_with('\n') {
                value.push('\n');
            }
            value.push_str(&prefix);
            value.push_str(&placeholder_text);
            value.push_str(&suffix);
        });
    };
    // The toolbar button handlers are thin wrappers that each capture
    // their own copy of `apply_wrap`. The closure captures a `String`
    // (cloned into `textarea_id_for_closure`), so the closure itself
    // implements `Clone`; we clone it four times to give each button
    // its own handler.
    let mut apply_wrap_bold = apply_wrap;
    let mut apply_wrap_code = apply_wrap;
    let mut apply_wrap_link = apply_wrap;
    let mut apply_wrap_heading = apply_wrap;

    rsx! {
        div { class: "field rich-text-editor",
            if let Some(l) = &label {
                label { class: "field-label", "{l}" }
            }
            div { class: "rte-toolbar flex gap-2 mb-2 border-b border-border pb-2",
                button {
                    class: "btn btn-sm btn-ghost", r#type: "button",
                    onclick: move |_| apply_wrap_bold("**".to_string(), "**".to_string(), "bold".to_string()),
                    "B"
                }
                button {
                    class: "btn btn-sm btn-ghost", r#type: "button",
                    onclick: move |_| apply_wrap_code("`".to_string(), "`".to_string(), "code".to_string()),
                    "C"
                }
                button {
                    class: "btn btn-sm btn-ghost", r#type: "button",
                    onclick: move |_| apply_wrap_link("[".to_string(), "](https://)".to_string(), "link".to_string()),
                    "🔗"
                }
                button {
                    class: "btn btn-sm btn-ghost", r#type: "button",
                    onclick: move |_| apply_wrap_heading("\n## ".to_string(), "\n".to_string(), "Heading".to_string()),
                    "H"
                }
                div { class: "ml-auto flex gap-1",
                    button { class: if *mode.read() == "edit" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                        r#type: "button",
                        onclick: move |_| mode.set("edit".to_string()),
                        "Edit"
                    }
                    button { class: if *mode.read() == "preview" { "btn btn-sm btn-primary" } else { "btn btn-sm btn-outline" },
                        r#type: "button",
                        onclick: move |_| mode.set("preview".to_string()),
                        "Preview"
                    }
                }
            }
            if *mode.read() == "edit" {
                textarea {
                    class: "input",
                    name: "{name}",
                    rows: "{rows}",
                    placeholder: placeholder.as_deref().unwrap_or("Write in markdown..."),
                    required: required,
                    value: "{content.read()}",
                    oninput: move |e| content.set(e.value().to_string()),
                }
            } else {
                div { class: "rte-preview p-4 border border-border rounded min-h-[12rem]",
                    dangerous_inner_html: "{preview_html}"
                }
            }
            if let Some(h) = &help {
                p { class: "field-help text-sm text-muted-foreground mt-1", "{h}" }
            }
            if let Some(e) = &error {
                p { class: "field-error text-sm text-danger", "{e}" }
            }
        }
    }
}

/// Minimal markdown -> HTML. Supports headings, bold, italic, code, links,
/// lists, and paragraphs. Doesn't aim to be CommonMark-complete — the news
/// editor is the only consumer and it uses a small subset.
fn render_markdown(src: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;
    for line in src.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("## ") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<h2>{}</h2>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("# ") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<h1>{}</h1>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("- ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>", inline(rest)));
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<p>{}</p>", inline(line)));
        }
    }
    if in_list {
        html.push_str("</ul>");
    }
    html
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn inline(s: &str) -> String {
    let mut out = escape(s);
    out = bold_pass(&mut out);
    out = italic_pass(&out);
    out = code_pass(&out);
    out = link_pass(&out);
    out
}

fn bold_pass(s: &mut String) -> String {
    let mut out = String::new();
    let mut in_bold = false;
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() - 1 {
        if bytes[i] == b'*' && bytes[i + 1] == b'*' {
            out.push_str(if in_bold { "</strong>" } else { "<strong>" });
            in_bold = !in_bold;
            i += 2;
        } else {
            out.push(s[i..].chars().next().unwrap());
            i += 1;
        }
    }
    while i < bytes.len() {
        out.push(s[i..].chars().next().unwrap());
        i += 1;
    }
    out
}

fn italic_pass(s: &str) -> String {
    let mut out = String::new();
    let mut in_i = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'*' {
            out.push_str(if in_i { "</em>" } else { "<em>" });
            in_i = !in_i;
        } else {
            out.push(s[i..].chars().next().unwrap());
        }
        i += 1;
    }
    out
}

fn code_pass(s: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            out.push_str(if in_code { "</code>" } else { "<code>" });
            in_code = !in_code;
        } else {
            out.push(s[i..].chars().next().unwrap());
        }
        i += 1;
    }
    out
}

fn link_pass(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // find ] and (url)
            if let Some(close) = s[i + 1..].find(']') {
                let label = &s[i + 1..i + 1 + close];
                if let Some(open_paren) = s[i + 1 + close + 1..].find('(') {
                    let after_paren = i + 1 + close + 1 + open_paren;
                    if after_paren < bytes.len() && bytes[after_paren] == b'(' {
                        if let Some(close_paren) = s[after_paren + 1..].find(')') {
                            let url = &s[after_paren + 1..after_paren + 1 + close_paren];
                            out.push_str(&format!(
                                "<a href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a>",
                                escape(url),
                                escape(label)
                            ));
                            i = after_paren + 1 + close_paren + 1;
                            continue;
                        }
                    }
                }
            }
        }
        out.push(s[i..].chars().next().unwrap());
        i += 1;
    }
    out
}
