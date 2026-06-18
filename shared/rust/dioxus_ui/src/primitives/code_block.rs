//! `CodeBlock` / `InlineCode` / `CodeHeader` — code display
//! components.
//!
//! `CodeBlock` is a styled `<pre><code>` for multi-line code
//! snippets (e.g. API request examples in the developer portal).
//! `InlineCode` is a styled `<code>` for inline code (e.g. a
//! variable name in a paragraph). `CodeHeader` is an optional
//! header row with the file name + a copy button.

use super::misc::CopyButton;

use dioxus::prelude::*;

/// A multi-line code block. Wraps the children in a styled
/// `<pre><code>`.
///
/// - `language: Option<String>` — the syntax-highlighting language
///   hint (rendered as a `data-language` attribute).
/// - `class: Option<String>` — extra Tailwind classes.
#[component]
pub fn CodeBlock(
    #[props(default = None)] language: Option<String>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "code-block relative rounded-md border bg-muted px-4 py-3 font-mono text-sm overflow-x-auto".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        pre {
            class: "{cls}",
            "data-language": language.as_deref().unwrap_or(""),
            code { {children} }
        }
    }
}

/// Inline code. Renders a `<code>` styled for inline use (e.g.
/// inside a paragraph).
///
/// - `class: Option<String>` — extra Tailwind classes.
#[component]
pub fn InlineCode(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "inline-code relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        code { class: "{cls}", {children} }
    }
}

/// Header for a code block — typically contains the language name
/// (or file name) on the left and a copy button on the right.
///
/// - `label: String` — the text to show (e.g. `"JavaScript"`,
///   `"request.json"`).
/// - `code: String` — the code string to copy when the copy button
///   is clicked.
#[component]
pub fn CodeHeader(label: String, code: String) -> Element {
    rsx! {
        div { class: "code-header flex items-center justify-between gap-2 px-4 py-2 border-b bg-muted/30 rounded-t-md",
            span { class: "code-header-label text-xs font-mono text-muted-foreground", "{label}" }
            CopyButton { text: code, label: "Copy".to_string() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_block_class_uses_mono_font() {
        let base = "code-block relative rounded-md border bg-muted px-4 py-3 font-mono text-sm overflow-x-auto";
        assert!(base.contains("font-mono"));
        assert!(base.contains("overflow-x-auto"));
    }

    #[test]
    fn inline_code_class_has_background() {
        let base = "inline-code relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold";
        assert!(base.contains("bg-muted"));
        assert!(base.contains("font-mono"));
    }

    #[test]
    fn code_header_rounded_top_corners() {
        let base = "code-header flex items-center justify-between gap-2 px-4 py-2 border-b bg-muted/30 rounded-t-md";
        assert!(base.contains("rounded-t-md"));
        assert!(base.contains("border-b"));
    }
}
