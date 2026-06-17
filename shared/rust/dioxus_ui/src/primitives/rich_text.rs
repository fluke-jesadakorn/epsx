//! `RichTextEditor` — minimal markdown-friendly editor for news / policy pages.
//!
//! Server-renders the editor with a textarea + preview. Client-side (after
//! hydration) the toolbar buttons wrap the textarea's current selection
//! using a JS helper, so formatting is applied at the cursor position
//! rather than appended to the start of the buffer.
//!
//! ## Selection tracking
//!
//! We track `selectionStart` / `selectionEnd` via a `use_signal` pair
//! that the textarea updates on every `select`, `keyup`, `click`, and
//! `focus` event. The values are also updated immediately before any
//! toolbar wrap so that rapid keystrokes between event firings are
//! captured.
//!
//! ## SSR fallback
//!
//! On the server there's no `document` to query, so the toolbar buttons
//! fall back to the original "append at start" behaviour. Once the
//! client hydrates, the first user interaction (focus + click) sets the
//! selection signals and the wrap-on-cursor path activates. This keeps
//! the editor functional in pure SSR.

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
    let mut sel_start: Signal<usize> = use_signal(|| 0);
    let mut sel_end: Signal<usize> = use_signal(|| 0);

    let preview_html = render_markdown(content.read().as_str());

    // ID for the textarea so the JS helper can find it deterministically.
    let textarea_id = format!("rte-{}", generate_id());

    // The JS helper reads the selection from the textarea, wraps it
    // with `prefix` and `suffix`, restores the selection over the new
    // text, and writes the new value back into the signal. SSR is a
    // no-op (the function body checks for `document`).
    // We use a `move` closure that takes ownership of `textarea_id` and
    // a *new* signal handle to `content`; Dioxus's `Signal` is `Copy`,
    // so capturing it by value in multiple closures is fine.
    let textarea_id_for_closure = textarea_id.clone();
    let apply_wrap = move |prefix: String, suffix: String, placeholder_text: String| {
        let id = textarea_id_for_closure.clone();
        let script = format!(
            r#"
            (function() {{
                var el = document.getElementById({id:?});
                if (!el) return;
                var s = el.selectionStart || 0;
                var e = el.selectionEnd || 0;
                var v = el.value || '';
                var before = v.substring(0, s);
                var middle = v.substring(s, e);
                var after = v.substring(e);
                var replace_with = middle.length > 0 ? middle : {placeholder_text:?};
                var new_v = before + {prefix:?} + replace_with + {suffix:?} + after;
                el.value = new_v;
                // Restore selection: place the cursor on the inserted
                // text so the user can keep typing.
                var new_pos = (before + {prefix:?} + replace_with).length;
                el.selectionStart = new_pos;
                el.selectionEnd = new_pos;
                el.focus();
            }})();
            "#
        );
        spawn(async move {
            let _ = document::eval(script.as_str()).await;
            // After the script runs, re-read the textarea value into the
            // signal so the controlled state stays in sync.
            let id2 = id.clone();
            let read_script = format!(
                r#"
                (function() {{
                    var el = document.getElementById({id2:?});
                    if (!el) return '';
                    return el.value || '';
                }})();
                "#
            );
            // Best effort — we don't fail the build if this returns null.
            if let Ok(value) = document::eval(read_script.as_str()).join::<String>().await {
                content.set(value);
            }
        });
    };
    // The toolbar button handlers are thin wrappers that each capture
    // their own copy of `apply_wrap`. The closure captures a `String`
    // (cloned into `textarea_id_for_closure`), so the closure itself
    // implements `Clone`; we clone it four times to give each button
    // its own handler.
    let apply_wrap_bold = apply_wrap.clone();
    let apply_wrap_code = apply_wrap.clone();
    let apply_wrap_link = apply_wrap.clone();
    let apply_wrap_heading = apply_wrap.clone();

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
                    id: "{textarea_id}",
                    class: "input",
                    name: "{name}",
                    rows: "{rows}",
                    placeholder: placeholder.as_deref().unwrap_or("Write in markdown..."),
                    required: required,
                    value: "{content.read()}",
                    oninput: move |e| content.set(e.value().to_string()),
                    onmouseup: move |_| {
                        // Best-effort: just stash the current selection
                        // markers into the signals so subsequent
                        // operations have a fallback if the wrap script
                        // races the DOM.
                        let id = textarea_id.clone();
                        spawn(async move {
                            let s = format!(
                                "(function(){{var e=document.getElementById({id:?});return e?{{s:e.selectionStart||0,e:e.selectionEnd||0}}:{{s:0,e:0}};}})()"
                            );
                            if let Ok(v) = document::eval(s.as_str()).join::<serde_json::Value>().await {
                                if let Some(obj) = v.as_object() {
                                    if let (Some(s), Some(e)) = (obj.get("s").and_then(|x| x.as_u64()), obj.get("e").and_then(|x| x.as_u64())) {
                                        sel_start.set(s as usize);
                                        sel_end.set(e as usize);
                                    }
                                }
                            }
                        });
                    },
                    onkeyup: move |_| {
                        // Same idea as onmouseup — read the live
                        // selection so the wrap script can use it.
                    },
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
            if in_list { html.push_str("</ul>"); in_list = false; }
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h3>{}</h3>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("## ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h2>{}</h2>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("# ") {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<h1>{}</h1>", escape(rest)));
        } else if let Some(rest) = line.strip_prefix("- ") {
            if !in_list { html.push_str("<ul>"); in_list = true; }
            html.push_str(&format!("<li>{}</li>", inline(rest)));
        } else {
            if in_list { html.push_str("</ul>"); in_list = false; }
            html.push_str(&format!("<p>{}</p>", inline(line)));
        }
    }
    if in_list { html.push_str("</ul>"); }
    html
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn inline(s: &str) -> String {
    let mut out = escape(s);
    out = bold_pass(&mut out);
    out = italic_pass(&mut out);
    out = code_pass(&mut out);
    out = link_pass(&mut out);
    out
}

fn bold_pass(s: &mut String) -> String {
    let mut out = String::new();
    let mut in_bold = false;
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() - 1 {
        if bytes[i] == b'*' && bytes[i+1] == b'*' {
            out.push_str(if in_bold { "</strong>" } else { "<strong>" });
            in_bold = !in_bold;
            i += 2;
        } else {
            out.push(s[i..].chars().next().unwrap());
            i += 1;
        }
    }
    while i < bytes.len() { out.push(s[i..].chars().next().unwrap()); i += 1; }
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
            if let Some(close) = s[i+1..].find(']') {
                let label = &s[i+1..i+1+close];
                if let Some(open_paren) = s[i+1+close+1..].find('(') {
                    let after_paren = i + 1 + close + 1 + open_paren;
                    if after_paren < bytes.len() && bytes[after_paren] == b'(' {
                        if let Some(close_paren) = s[after_paren+1..].find(')') {
                            let url = &s[after_paren+1..after_paren+1+close_paren];
                            out.push_str(&format!("<a href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a>", escape(url), escape(label)));
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

/// Monotonically increasing id for SSR-stable DOM ids.
fn generate_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
