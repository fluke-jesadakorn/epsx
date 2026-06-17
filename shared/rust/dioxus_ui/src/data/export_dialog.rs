//! `<ExportDialog>` — modal for selecting a data export format and firing
//! the download. Generic across analytics + developer + any future surface.
//!
//! Ported from `apps-old/frontend/components/analytics/analytics-export-dialog.tsx`
//! (180 LoC, `apps-old/frontend/lib/export-utils.ts`) as a Wave 6A Track B
//! new primitive. Reused by analytics page and (per design doc) Track D
//! analytics-export pattern.
//!
//! The component is fully controlled: the parent decides when the dialog
//! is visible via `open` and reacts to `on_close` / `on_export` callbacks.
//! Internal state (selected format, filename, scope) lives in local
//! `use_signal`s so the parent doesn't have to re-wire controlled state for
//! every consumer.
//!
//! ## Section markers
//!
//! - `export-dialog` — outer container
//! - `export-dialog-fmt-{format}` — individual format option rows
//! - `export-dialog-trigger` — the export action button (purple/gradient)
//! - `export-dialog-filename` — filename input
//! - `export-dialog-options` — wrapper for the "include metadata" /
//!   "include quarterly data" checkbox cluster
//!
//! These markers are part of the section-level contract used by the page
//! unit tests.

use dioxus::prelude::*;

/// The export formats the dialog offers. Mirrors the `ExportFormat`
/// type from the Next.js `lib/export-utils.ts` source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
    Parquet,
}

impl ExportFormat {
    /// Lowercase string identifier — used as a CSS class suffix and as
    /// the `value` attribute on the format rows.
    pub fn as_str(self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Parquet => "parquet",
        }
    }

    /// Human-readable label rendered on the format row.
    pub fn label(self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV",
            ExportFormat::Json => "JSON",
            ExportFormat::Parquet => "Parquet",
        }
    }
}

/// Scope selector — matches the `exportType` state in the source
/// `AnalyticsExportDialog`. `Current` exports the visible page,
/// `Filtered` exports everything matching the active filter set,
/// `Full` exports the entire dataset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportScope {
    Current,
    Filtered,
    Full,
}

impl ExportScope {
    pub fn as_str(self) -> &'static str {
        match self {
            ExportScope::Current => "current",
            ExportScope::Filtered => "filtered",
            ExportScope::Full => "full",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            ExportScope::Current => "Current Page",
            ExportScope::Filtered => "Filtered Data",
            ExportScope::Full => "Full Dataset",
        }
    }
}

/// `<ExportDialog>` primitive.
///
/// ## Props
///
/// - `open` — whether the dialog is visible. When `false`, nothing is
///   rendered.
/// - `on_close` — fired on overlay click, close button, or after a
///   successful export.
/// - `on_export` — fired with the chosen `ExportFormat` when the user
///   clicks the "Export" action button. Caller is responsible for the
///   actual download (this primitive is view-only; it doesn't touch
///   the network).
/// - `default_format` — which format is preselected. Defaults to
///   `ExportFormat::Json` (matches the source).
/// - `default_scope` — which export scope is preselected. Defaults to
///   `ExportScope::Current`.
/// - `title` — dialog title. Defaults to "Export Data".
/// - `description` — optional helper text under the title.
/// - `available_formats` — formats the user can pick. Defaults to
///   `[Csv, Json, Parquet]` (the source only offers CSV + JSON; we
///   include Parquet per the Wave 6A design doc's `ExportDialog`
///   contract).
/// - `available_scopes` — scopes the user can pick. Defaults to
///   `[Current, Filtered, Full]`.
/// - `filename_placeholder` — placeholder text for the optional
///   filename input.
/// - `show_metadata_option` — whether to render the
///   "Include metadata" checkbox.
/// - `show_quarterly_option` — whether to render the
///   "Include quarterly data" checkbox.
#[component]
pub fn ExportDialog(
    open: bool,
    on_close: EventHandler,
    on_export: EventHandler<ExportFormat>,
    #[props(default = ExportFormat::Json)] default_format: ExportFormat,
    #[props(default = ExportScope::Current)] default_scope: ExportScope,
    #[props(default = None)] title: Option<String>,
    #[props(default = None)] description: Option<String>,
    #[props(default = None)] available_formats: Option<Vec<ExportFormat>>,
    #[props(default = None)] available_scopes: Option<Vec<ExportScope>>,
    #[props(default = None)] filename_placeholder: Option<String>,
    #[props(default = true)] show_metadata_option: bool,
    #[props(default = true)] show_quarterly_option: bool,
) -> Element {
    if !open {
        return rsx! { Fragment {} };
    }

    // Local UI state — the dialog is self-contained, the parent only
    // sees the `on_export` callback.
    let mut selected_format = use_signal(|| default_format);
    let mut selected_scope = use_signal(|| default_scope);
    let mut filename = use_signal(String::new);
    let mut include_metadata = use_signal(|| true);
    let mut include_quarterly = use_signal(|| true);

    let formats = available_formats.unwrap_or_else(|| {
        vec![ExportFormat::Csv, ExportFormat::Json, ExportFormat::Parquet]
    });
    let scopes = available_scopes.unwrap_or_else(|| {
        vec![ExportScope::Current, ExportScope::Filtered, ExportScope::Full]
    });

    let title_str = title.unwrap_or_else(|| "Export Data".to_string());
    let filename_placeholder_str = filename_placeholder
        .unwrap_or_else(|| "Leave empty for auto-generated name".to_string());

    // Stable id for the dialog panel so test assertions can target it.
    let panel_id = "export-dialog-panel".to_string();

    rsx! {
        // === wave6-auth-pages-depth-track-b ===
        // <ExportDialog> — modal primitive; section-marker class names
        // are surfaced to the integration gate via the page unit tests.
        div {
            class: "modal-overlay export-dialog-overlay",
            "data-section": "export-dialog",
            onclick: move |_| on_close.call(()),
            div {
                class: "modal export-dialog {panel_id}",
                id: "{panel_id}",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "export-dialog-title",
                onclick: |e| e.stop_propagation(),
                // Header
                div { class: "modal-header",
                    h2 {
                        class: "modal-title",
                        id: "export-dialog-title",
                        "{title_str}"
                    }
                    button {
                        class: "modal-close",
                        r#type: "button",
                        "aria-label": "Close export dialog",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                if let Some(d) = &description {
                    p { class: "modal-description text-sm text-muted-foreground mb-4", "{d}" }
                }
                div { class: "modal-body export-dialog-body space-y-4",
                    // Scope selector
                    div { class: "export-dialog-scope",
                        label { class: "form-label", "Export Type" }
                        div { class: "export-dialog-scopes flex flex-wrap gap-2",
                            for scope in scopes.iter() {
                                {
                                    let scope_val = *scope;
                                    let scope_str = scope_val.as_str();
                                    let scope_label = scope_val.label();
                                    let is_active = *selected_scope.read() == scope_val;
                                    let active_suffix = if is_active { " active" } else { "" };
                                    let scope_class = format!("export-dialog-scope-btn {scope_str}{active_suffix}");
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            class: "{scope_class}",
                                            "data-scope": "{scope_str}",
                                            onclick: move |_| selected_scope.set(scope_val),
                                            "{scope_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Format selector
                    div { class: "export-dialog-format",
                        label { class: "form-label", "Format" }
                        div { class: "export-dialog-formats flex flex-wrap gap-2",
                            for fmt in formats.iter() {
                                {
                                    let fmt_val = *fmt;
                                    let fmt_str = fmt_val.as_str();
                                    let fmt_label = fmt_val.label();
                                    let is_active = *selected_format.read() == fmt_val;
                                    let class_key = format!("export-dialog-fmt-{fmt_str}");
                                    let active_suffix = if is_active { " active" } else { "" };
                                    let button_class = format!("{class_key}{active_suffix}");
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            class: "{button_class}",
                                            "data-format": "{fmt_str}",
                                            onclick: move |_| selected_format.set(fmt_val),
                                            "{fmt_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Filename (optional)
                    div { class: "export-dialog-filename",
                        label { class: "form-label", r#for: "export-dialog-filename-input", "Filename (optional)" }
                        input {
                            class: "input",
                            id: "export-dialog-filename-input",
                            r#type: "text",
                            placeholder: "{filename_placeholder_str}",
                            value: "{filename.read()}",
                            oninput: move |e| filename.set(e.value().to_string()),
                        }
                    }
                    // Optional checkboxes
                    if show_metadata_option || show_quarterly_option {
                        div { class: "export-dialog-options space-y-2",
                            if show_metadata_option {
                                label { class: "export-dialog-option flex items-center gap-2",
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox",
                                        checked: *include_metadata.read(),
                                        onchange: move |e| {
                                            include_metadata.set(e.checked());
                                        },
                                    }
                                    span { "Include metadata" }
                                }
                            }
                            if show_quarterly_option {
                                label { class: "export-dialog-option flex items-center gap-2",
                                    input {
                                        r#type: "checkbox",
                                        class: "checkbox",
                                        checked: *include_quarterly.read(),
                                        onchange: move |e| {
                                            include_quarterly.set(e.checked());
                                        },
                                    }
                                    span { "Include quarterly data" }
                                }
                            }
                        }
                    }
                    // Action row
                    div { class: "export-dialog-actions flex justify-end gap-2 mt-2",
                        button {
                            r#type: "button",
                            class: "btn btn-outline",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-primary export-dialog-trigger",
                            "data-section": "export-dialog-trigger",
                            onclick: move |_| {
                                on_export.call(*selected_format.read());
                                on_close.call(());
                            },
                            "Export"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test harness — a `#[component]` so Dioxus sets up a runtime
    /// for `use_signal` calls inside the rendered subtree. Renders the
    /// `<ExportDialog>` with `open=true` so the dialog body is in
    /// scope.
    #[component]
    fn TestHarness() -> Element {
        rsx! {
            ExportDialog {
                open: true,
                on_close: move |_| {},
                on_export: move |_fmt: ExportFormat| {},
                title: Some("Export Data".to_string()),
            }
        }
    }

    /// Render the dialog with `open=true` and assert a static "Export"
    /// string appears in the rendered HTML. The dialog body always
    /// contains the word "Export" (button label + title), so a
    /// substring assertion is enough to confirm the dialog rendered.
    #[test]
    fn export_dialog_renders_when_open() {
        let mut vdom = dioxus::prelude::VirtualDom::new(TestHarness);
        // Build the tree before rendering. `dioxus_ssr::render` reads
        // the rebuilt tree; without `rebuild()` it errors with "The
        // tree has not been built yet".
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Export"),
            "ExportDialog should render the word `Export` in the dialog body. Got: {html}"
        );
        assert!(
            html.contains("export-dialog"),
            "ExportDialog should render the `export-dialog` section marker. Got: {html}"
        );
    }
}
