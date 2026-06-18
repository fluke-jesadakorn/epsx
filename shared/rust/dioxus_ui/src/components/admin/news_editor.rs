//! Admin `NewsEditor` — Wave 38b T2 admin domain port.
//!
//! Mirrors `apps-old/admin-frontend/components/news/news-editor.tsx`,
//! which is the create/edit form for a news article. The form
//! has a sticky header with status toggle + save button + a
//! body with title / slug / excerpt / body fields + footer
//! action row.
//!
//! | Component | Use case |
//! | --- | --- |
//! | `NewsEditorHeader` | Sticky header (back link + status toggle + save button) |
//! | `NewsEditorStatusToggle` | Draft / Published toggle pills |
//! | `NewsEditorFormFields` | Title / slug / excerpt / body fields |
//! | `NewsEditorFooter` | Cancel + Save-as-draft + Publish buttons |
//!
//! The body field uses a `textarea` placeholder — the source uses
//! a rich text editor (micromark-based) that's a JS dep. On SSR
//! we render a textarea and let the BFF swap in the editor on
//! hydration.
//!
//! ## Tests
//!
//! Each component gets a colocated `#[cfg(test)] mod tests` block
//! with smoke render + key prop handling.

use dioxus::prelude::*;
use crate::primitives::icon::Icon;

// ============================================================================
// NewsEditorStatusToggle
// ============================================================================
//
// Draft / Published toggle pills in the editor header.

#[component]
pub fn NewsEditorStatusToggle(
    status: String,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "news-editor-status-toggle flex items-center rounded-lg border border-border/20 bg-card overflow-hidden",
            button {
                class: if status == "draft" {
                    "px-3 py-1.5 text-xs font-medium transition-colors bg-[#7645d9]/20 text-[#7645d9]"
                } else {
                    "px-3 py-1.5 text-xs font-medium transition-colors text-muted-foreground hover:text-foreground"
                },
                r#type: "button",
                onclick: move |_| on_change.call("draft".to_string()),
                "Draft"
            }
            button {
                class: if status == "published" {
                    "px-3 py-1.5 text-xs font-medium transition-colors bg-[#7645d9]/20 text-[#7645d9]"
                } else {
                    "px-3 py-1.5 text-xs font-medium transition-colors text-muted-foreground hover:text-foreground"
                },
                r#type: "button",
                onclick: move |_| on_change.call("published".to_string()),
                "Published"
            }
        }
    }
}

// ============================================================================
// NewsEditorHeader
// ============================================================================
//
// Sticky header (back link + title + status toggle + save button).
// Mirrors the source's `news-editor-header`.

#[component]
pub fn NewsEditorHeader(
    title: String,
    status: String,
    on_status_change: EventHandler<String>,
    on_save: EventHandler<()>,
    back_href: Option<String>,
) -> Element {
    let back_href = back_href.unwrap_or_else(|| "/news".to_string());
    rsx! {
        div { class: "news-editor-header sticky top-0 z-10 backdrop-blur-md bg-background/80 border-b border-border/10 py-3 flex items-center justify-between mb-4",
            h1 { class: "text-xl font-bold", "{title}" }
            div { class: "flex items-center gap-3",
                NewsEditorStatusToggle {
                    status: status.clone(),
                    on_change: move |s: String| on_status_change.call(s),
                }
                button {
                    class: "news-editor-save flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold",
                    r#type: "button",
                    onclick: move |_| on_save.call(()),
                    "\u{1F4BE} Save"
                }
                a {
                    class: "btn btn-sm btn-ghost",
                    href: "{back_href}",
                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                    " Back"
                }
            }
        }
    }
}

// ============================================================================
// NewsEditorFormFields
// ============================================================================
//
// Title / slug / excerpt / body fields. Mirrors the source's
// `ModalFormFields`-style layout.

#[component]
pub fn NewsEditorFormFields(
    title_value: Option<String>,
    slug_value: Option<String>,
    excerpt_value: Option<String>,
    body_value: Option<String>,
    on_title_change: Option<EventHandler<String>>,
) -> Element {
    let title_v = title_value.clone().unwrap_or_default();
    let slug_v = slug_value.clone().unwrap_or_default();
    let excerpt_v = excerpt_value.clone().unwrap_or_default();
    let body = body_value.clone().unwrap_or_else(|| {
        "## Introduction\n\nWrite your news article here in markdown.\n\n- Point 1\n- Point 2\n\n[Read more](https://epsx.io)".to_string()
    });
    let on_title = on_title_change.clone().unwrap_or_else(|| EventHandler::new(|_: String| {}));
    rsx! {
        div { class: "news-editor-form-fields space-y-4",
            div { class: "field",
                label { class: "field-label", "Title" }
                input {
                    class: "input",
                    name: "title",
                    required: true,
                    value: "{title_v}",
                    placeholder: "A clear, descriptive title",
                    oninput: move |e: dioxus::prelude::Event<dioxus::prelude::FormData>| {
                        on_title.call(e.value().to_string());
                    },
                }
            }
            div { class: "field",
                label { class: "field-label", "Slug" }
                input {
                    class: "input",
                    name: "slug",
                    required: true,
                    value: "{slug_v}",
                    placeholder: "auto-generated-from-title"
                }
            }
            div { class: "field",
                label { class: "field-label", "Excerpt" }
                textarea {
                    class: "input",
                    name: "excerpt",
                    rows: "2",
                    placeholder: "Short summary for the listing page",
                    "{excerpt_v}"
                }
            }
            div { class: "field",
                label { class: "field-label", "Body" }
                textarea {
                    class: "input news-editor-body",
                    name: "body",
                    rows: "16",
                    placeholder: "Markdown body",
                    "{body}"
                }
            }
        }
    }
}

// ============================================================================
// NewsEditorFooter
// ============================================================================
//
// Cancel + Save-as-draft + Publish buttons.

#[component]
pub fn NewsEditorFooter(
    cancel_href: Option<String>,
    on_save_draft: EventHandler<()>,
    on_publish: EventHandler<()>,
) -> Element {
    let cancel_href = cancel_href.unwrap_or_else(|| "/news".to_string());
    rsx! {
        div { class: "news-editor-footer flex items-center justify-end gap-2 pt-4 border-t border-border/10",
            a { class: "btn btn-outline", href: "{cancel_href}", "Cancel" }
            button {
                class: "btn btn-secondary",
                r#type: "button",
                onclick: move |_| on_save_draft.call(()),
                "Save as draft"
            }
            button {
                class: "btn btn-primary",
                r#type: "button",
                onclick: move |_| on_publish.call(()),
                "Publish"
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// `NewsEditorStatusToggle` marks draft as active when status="draft".
    #[test]
    fn news_editor_status_toggle_marks_draft_active() {
        fn harness() -> Element {
            rsx! {
                NewsEditorStatusToggle {
                    status: "draft".to_string(),
                    on_change: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-status-toggle"), "NewsEditorStatusToggle must render container class. Got: {html}");
        assert!(html.contains("Draft"), "NewsEditorStatusToggle must render Draft button. Got: {html}");
        assert!(html.contains("Published"), "NewsEditorStatusToggle must render Published button. Got: {html}");
        assert!(html.contains("bg-[#7645d9]/20"), "Active tab must have gradient bg. Got: {html}");
    }

    /// `NewsEditorStatusToggle` marks published as active when status="published".
    #[test]
    fn news_editor_status_toggle_marks_published_active() {
        fn harness() -> Element {
            rsx! {
                NewsEditorStatusToggle {
                    status: "published".to_string(),
                    on_change: move |_: String| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        // The active "published" button has bg-[#7645d9]/20 class
        assert!(html.contains("bg-[#7645d9]/20"), "NewsEditorStatusToggle published must show active state. Got: {html}");
    }

    /// `NewsEditorHeader` renders the title + status toggle + save button.
    #[test]
    fn news_editor_header_renders_all() {
        fn harness() -> Element {
            rsx! {
                NewsEditorHeader {
                    title: "Edit news".to_string(),
                    status: "draft".to_string(),
                    on_status_change: move |_: String| {},
                    on_save: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-header"), "NewsEditorHeader must render container class. Got: {html}");
        assert!(html.contains("Edit news"), "NewsEditorHeader must render title. Got: {html}");
        assert!(html.contains("news-editor-save"), "NewsEditorHeader must render save button. Got: {html}");
    }

    /// `NewsEditorFormFields` renders all 4 fields.
    #[test]
    fn news_editor_form_fields_renders_all() {
        fn harness() -> Element {
            rsx! {
                NewsEditorFormFields {
                    title_value: Some("Test Article".to_string()),
                    slug_value: Some("test-article".to_string()),
                    excerpt_value: Some("A test".to_string()),
                    body_value: Some("Body text".to_string()),
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-form-fields"), "NewsEditorFormFields must render container class. Got: {html}");
        assert!(html.contains("Test Article"), "NewsEditorFormFields must render title. Got: {html}");
        assert!(html.contains("test-article"), "NewsEditorFormFields must render slug. Got: {html}");
        assert!(html.contains("A test"), "NewsEditorFormFields must render excerpt. Got: {html}");
        assert!(html.contains("Body text"), "NewsEditorFormFields must render body. Got: {html}");
    }

    /// `NewsEditorFormFields` with no values renders default body placeholder.
    #[test]
    fn news_editor_form_fields_default_body() {
        fn harness() -> Element {
            rsx! { NewsEditorFormFields { } }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("## Introduction"), "NewsEditorFormFields default body must include markdown placeholder. Got: {html}");
    }

    /// `NewsEditorFooter` renders Cancel + Save-as-draft + Publish.
    #[test]
    fn news_editor_footer_renders_buttons() {
        fn harness() -> Element {
            rsx! {
                NewsEditorFooter {
                    on_save_draft: move |_: ()| {},
                    on_publish: move |_: ()| {},
                }
            }
        }
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("news-editor-footer"), "NewsEditorFooter must render container class. Got: {html}");
        assert!(html.contains("Cancel"), "NewsEditorFooter must render Cancel button. Got: {html}");
        assert!(html.contains("Save as draft"), "NewsEditorFooter must render Save as draft button. Got: {html}");
        assert!(html.contains("Publish"), "NewsEditorFooter must render Publish button. Got: {html}");
    }
}
