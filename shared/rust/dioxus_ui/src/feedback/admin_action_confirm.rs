//! `<AdminActionConfirm>` — destructive-action confirmation modal.
//!
//! Generic confirm/cancel modal for admin actions that need a second
//! tap before doing something destructive (revoke API key, delete
//! news article, cancel a scheduled notification, purge broadcast,
//! disable wallet, etc.). All three Wave 6B Track B pages
//! (`audit_log`, `news`, `notifications`) need this primitive — the
//! source Next.js code embeds a small modal in each component; we
//! extract the pattern once and reuse it.
//!
//! The variant is `Destructive | Warning | Info` — the source uses
//! mostly `red` (destructive), `amber` (warning), and `cyan` (info)
//! to color the confirm button. Dioxus signal-free: open/close is
//! controlled by the parent via the `open` prop and the
//! `on_confirm` / `on_cancel` event handlers.
//!
//! Source-of-truth pre-flight: see
//! `apps-old/admin-frontend/components/news/news-management.tsx`
//! (DeleteModal), `components/notifications/notification-management.tsx`
//! (DeleteModal) — same shape, different copy. The audit-log page
//! in the source doesn't have a confirm modal (audit entries are
//! read-only), but the same primitive covers "Acknowledge log entry"
//! for the management side.

use dioxus::prelude::*;

/// Visual variant for the confirm button. The text label and
/// button class follow the source Next.js design system:
/// - `Destructive` — red (delete, revoke, purge)
/// - `Warning` — amber (cancel scheduled, unpublish)
/// - `Info` — cyan/primary (generic confirm)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmVariant {
    /// Red confirm button — used for permanent destructive ops
    /// (delete article, purge notification, revoke API key).
    Destructive,
    /// Amber confirm button — used for ops that revert state but
    /// don't destroy data (cancel scheduled notification,
    /// unpublish article, disable wallet).
    Warning,
    /// Brand / cyan confirm button — generic non-destructive
    /// confirm (acknowledge log, send for review).
    Info,
}

impl Default for ConfirmVariant {
    fn default() -> Self {
        ConfirmVariant::Destructive
    }
}

/// Render a button class string for the given variant. Centralized
/// so a future theme change is a one-line edit.
fn confirm_btn_class(variant: ConfirmVariant) -> &'static str {
    match variant {
        ConfirmVariant::Destructive => "btn btn-danger",
        ConfirmVariant::Warning => "btn btn-warning",
        ConfirmVariant::Info => "btn btn-primary",
    }
}

/// Confirmation modal for destructive admin actions.
///
/// Required:
/// - `open: bool` — whether the dialog is shown. The parent owns
///   the open state; this component is a pure renderer that returns
///   an empty fragment when closed.
/// - `title: String` — modal title, e.g. "Delete article?".
/// - `message: String` — body copy, e.g. "This will be permanently
///   deleted.".
/// - `confirm_label: String` — short verb that fits inside the
///   confirm button, e.g. "Delete", "Revoke", "Purge".
/// - `confirm_variant: ConfirmVariant` — Destructive | Warning | Info.
/// - `on_confirm: EventHandler<MouseEvent>` — fired when the user
///   clicks the confirm button. Parent is responsible for closing
///   the modal in response.
/// - `on_cancel: EventHandler<MouseEvent>` — fired when the user
///   clicks the cancel button or the overlay backdrop. Parent is
///   responsible for closing the modal in response.
///
/// Optional:
/// - `cancel_label: Option<String>` — defaults to "Cancel".
#[component]
pub fn AdminActionConfirm(
    open: bool,
    title: String,
    message: String,
    confirm_label: String,
    confirm_variant: ConfirmVariant,
    on_confirm: EventHandler<MouseEvent>,
    on_cancel: EventHandler<MouseEvent>,
    #[props(default = None)] cancel_label: Option<String>,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }
    let cancel_text = cancel_label.unwrap_or_else(|| "Cancel".to_string());
    let confirm_cls = confirm_btn_class(confirm_variant);

    // SSR-stable id for the panel. The title is sanitized into a
    // lowercase id suffix so multiple modals on the same page get
    // distinct ids. Not a security boundary.
    let panel_id = format!("admin-action-confirm-{}", open_label(&title));

    rsx! {
        div {
            class: "admin-action-confirm-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": format!("{}-title", panel_id),
            "aria-describedby": format!("{}-message", panel_id),
            tabindex: "-1",
            onclick: move |e| on_cancel.call(e),
            div {
                class: "admin-action-confirm-panel",
                id: "{panel_id}",
                onclick: |e| e.stop_propagation(),
                h3 {
                    class: "admin-action-confirm-title",
                    id: format!("{}-title", panel_id),
                    "{title}"
                }
                p {
                    class: "admin-action-confirm-message",
                    id: format!("{}-message", panel_id),
                    "{message}"
                }
                div { class: "admin-action-confirm-actions",
                    button {
                        class: "btn btn-outline",
                        r#type: "button",
                        onclick: move |e| on_cancel.call(e),
                        "{cancel_text}"
                    }
                    button {
                        class: "{confirm_cls}",
                        r#type: "button",
                        onclick: move |e| on_confirm.call(e),
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}

/// Sanitize `title` into a stable lowercase id suffix. Pure ASCII,
/// no spaces — good enough for an SSR panel id, not a security
/// boundary.
fn open_label(title: &str) -> String {
    title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fresh `VirtualDom` wrapping the supplied `Element`-yielding
    /// closure, rebuild the tree, and return the rendered HTML string.
    ///
    /// Components that take `EventHandler<MouseEvent>` need a real Dioxus
    /// scope to construct — `dioxus_ssr::render_element` on a raw
    /// `rsx!` block fails with "Must be called from inside a Dioxus
    /// runtime". The `VirtualDom::new(harness)` + `rebuild_in_place()`
    /// + `dioxus_ssr::render(&vdom)` pattern (mirrored from
    /// `data/export_dialog.rs::tests`) is the working one.
    fn render_html(harness: fn() -> Element) -> String {
        let mut vdom = dioxus::prelude::VirtualDom::new(harness);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    /// Harness returning the closed-modal `AdminActionConfirm`.
    fn harness_closed() -> Element {
        rsx! {
            AdminActionConfirm {
                open: false,
                title: "Delete article?".to_string(),
                message: "Are you sure?".to_string(),
                confirm_label: "Delete".to_string(),
                confirm_variant: ConfirmVariant::Destructive,
                on_confirm: move |_| {},
                on_cancel: move |_| {},
            }
        }
    }

    /// Harness returning the open destructive variant.
    fn harness_open_destructive() -> Element {
        rsx! {
            AdminActionConfirm {
                open: true,
                title: "Delete article?".to_string(),
                message: "Are you sure you want to delete?".to_string(),
                confirm_label: "Delete".to_string(),
                confirm_variant: ConfirmVariant::Destructive,
                on_confirm: move |_| {},
                on_cancel: move |_| {},
            }
        }
    }

    /// Harness returning the open warning variant.
    fn harness_open_warning() -> Element {
        rsx! {
            AdminActionConfirm {
                open: true,
                title: "Revoke key".to_string(),
                message: "This API key will be revoked.".to_string(),
                confirm_label: "Revoke".to_string(),
                confirm_variant: ConfirmVariant::Warning,
                on_confirm: move |_| {},
                on_cancel: move |_| {},
            }
        }
    }

    /// Harness returning the open warning variant with a custom
    /// cancel label.
    fn harness_custom_cancel() -> Element {
        rsx! {
            AdminActionConfirm {
                open: true,
                title: "Cancel schedule".to_string(),
                message: "Cancel the scheduled notification?".to_string(),
                confirm_label: "Cancel schedule".to_string(),
                confirm_variant: ConfirmVariant::Warning,
                cancel_label: Some("Keep scheduled".to_string()),
                on_confirm: move |_| {},
                on_cancel: move |_| {},
            }
        }
    }

    /// Harness returning the open info variant.
    fn harness_info() -> Element {
        rsx! {
            AdminActionConfirm {
                open: true,
                title: "Acknowledge log".to_string(),
                message: "Acknowledge this log entry?".to_string(),
                confirm_label: "Acknowledge".to_string(),
                confirm_variant: ConfirmVariant::Info,
                on_confirm: move |_| {},
                on_cancel: move |_| {},
            }
        }
    }

    /// When `open=false`, the component renders nothing — empty
    /// Fragment. This is the "hidden" contract the parent relies
    /// on to keep the modal out of the DOM.
    #[test]
    fn admin_action_confirm_renders_nothing_when_closed() {
        let html = render_html(harness_closed);
        assert!(
            !html.contains("Delete article?"),
            "Closed modal must not render its title in the DOM. Got: {}",
            html
        );
    }

    /// When `open=true`, the component renders the title and
    /// confirm label in the DOM — the design-doc spec calls this
    /// out as the section-marker test for this primitive.
    #[test]
    fn admin_action_confirm_renders_when_open() {
        let html = render_html(harness_open_destructive);
        assert!(
            html.contains("Delete article?"),
            "AdminActionConfirm must render the title. Got: {}",
            html
        );
        assert!(
            html.contains("Delete"),
            "AdminActionConfirm must render the confirm label. Got: {}",
            html
        );
        // Cancel default label
        assert!(
            html.contains("Cancel"),
            "AdminActionConfirm must render the cancel button. Got: {}",
            html
        );
    }

    /// The dialog has proper ARIA + section-marker hooks. This is
    /// the contract the test_section_markers tests across all 3
    /// Track B pages can rely on.
    #[test]
    fn admin_action_confirm_has_dialog_role() {
        let html = render_html(harness_open_warning);
        assert!(
            html.contains("admin-action-confirm-panel"),
            "AdminActionConfirm must have its section-marker class. Got: {}",
            html
        );
        assert!(
            html.contains("Revoke"),
            "AdminActionConfirm must render the confirm label. Got: {}",
            html
        );
        assert!(
            html.contains("btn-warning"),
            "Warning variant must use btn-warning class. Got: {}",
            html
        );
    }

    /// The 3 variants map to the 3 button classes — locked at
    /// compile-time by the `confirm_btn_class` helper, but a
    /// smoke test guards against a future refactor that swaps
    /// the wrong class.
    #[test]
    fn confirm_btn_class_matches_variants() {
        assert_eq!(confirm_btn_class(ConfirmVariant::Destructive), "btn btn-danger");
        assert_eq!(confirm_btn_class(ConfirmVariant::Warning), "btn btn-warning");
        assert_eq!(confirm_btn_class(ConfirmVariant::Info), "btn btn-primary");
    }

    /// `open_label` sanitizes the title into a stable id suffix.
    #[test]
    fn open_label_sanitizes_title() {
        assert_eq!(open_label("Delete article?"), "delete_article");
        assert_eq!(open_label("Revoke API key"), "revoke_api_key");
        assert_eq!(open_label("  Trim me!  "), "trim_me");
    }

    /// Default variant is Destructive — this is the most common
    /// case (most admin actions are destructive), and we don't
    /// want callers to have to spell it out.
    #[test]
    fn default_variant_is_destructive() {
        let v: ConfirmVariant = Default::default();
        assert_eq!(v, ConfirmVariant::Destructive);
    }

    /// Custom cancel label overrides the default "Cancel" — the
    /// source uses "Abort" in the notifications management
    /// component, "Unpublish" on the news action menu, etc.
    #[test]
    fn custom_cancel_label_is_used() {
        let html = render_html(harness_custom_cancel);
        assert!(
            html.contains("Keep scheduled"),
            "Custom cancel label must be used. Got: {}",
            html
        );
    }

    /// Info variant uses the brand primary button. Smoke test
    /// the variant wiring.
    #[test]
    fn info_variant_uses_primary_class() {
        let html = render_html(harness_info);
        assert!(
            html.contains("btn-primary"),
            "Info variant must use btn-primary class. Got: {}",
            html
        );
    }
}
