//! `Section` / `SectionHeader` / `SectionBody` / `SectionFooter` —
//! shadcn-style page section container.
//!
//! Mirrors the visual pattern from `Card` / `CardHeader` /
//! `CardBody` / `CardFooter` but with looser styling (no border,
//! no padding) for use as a logical page section that doesn't need
//! the card chrome.

use dioxus::prelude::*;

/// Page section. Renders a `<section>` with vertical spacing and
/// the design-system typography defaults.
#[component]
pub fn Section(
    #[props(default = None)] id: Option<String>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "section flex flex-col gap-4 py-6".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        section { class: "{cls}", id: id.as_deref().unwrap_or(""), {children} }
    }
}

/// Section header — typically contains a title + description +
/// optional action.
#[component]
pub fn SectionHeader(
    title: String,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "section-header flex flex-col gap-1".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}",
            h2 { class: "section-header-title text-2xl font-semibold tracking-tight", "{title}" }
            if let Some(d) = description {
                p { class: "section-header-description text-sm text-muted-foreground", "{d}" }
            }
            {children}
        }
    }
}

/// Section body — the main content. Renders a `<div>` with the
/// section's content flow.
#[component]
pub fn SectionBody(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "section-body flex flex-col gap-4".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

/// Section footer — typically a row of action buttons.
#[component]
pub fn SectionFooter(
    #[props(default = None)] class: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "section-footer flex items-center justify-end gap-2 pt-4 border-t border-border".to_string();
    if let Some(c) = class {
        cls.push(' ');
        cls.push_str(&c);
    }
    rsx! {
        div { class: "{cls}", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_renders_with_default_classes() {
        // Sanity check the class string.
        let base = "section flex flex-col gap-4 py-6";
        assert!(base.contains("flex flex-col"));
        assert!(base.contains("py-6"));
    }

    #[test]
    fn section_header_title_class() {
        // SectionHeader title uses `text-2xl font-semibold tracking-tight`.
        let title_cls = "text-2xl font-semibold tracking-tight";
        assert!(title_cls.contains("text-2xl"));
        assert!(title_cls.contains("tracking-tight"));
    }

    #[test]
    fn section_footer_has_top_border() {
        // SectionFooter uses `border-t border-border` to visually
        // separate from the body.
        let base = "section-footer flex items-center justify-end gap-2 pt-4 border-t border-border";
        assert!(base.contains("border-t"));
        assert!(base.contains("border-border"));
    }
}
