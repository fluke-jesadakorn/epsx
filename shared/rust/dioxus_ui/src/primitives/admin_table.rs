//! `AdminTable<T>` — `DataTable` + filter chips + pagination wrapper used by
//! all 4 Track C admin list views (payment links, credit transactions,
//! plan list, wallet access list). Includes the action column with
//! `<AdminActionConfirm>` modal trigger for delete/revoke actions.
//!
//! Wave 6B Track C new primitive — extracted from the section-level port of
//! the Next.js `apps-old/admin-frontend/components/{payments,credits,plans,wallet}/*`
//! sub-components. The source pattern is "table with an action column that
//! triggers a confirmation modal for destructive actions" — this primitive
//! captures that contract so the 4 pages don't each re-implement it.
//!
//! Per Wave 6A's "Reuse `src/tests/mod.rs::test_user_with()`" lesson, the
//! pages call this primitive through a thin shell, NOT a per-page copy.
//!
//! ## Component signature
//!
//! ```ignore
//! AdminTable {
//!     rows: vec![
//!         PaymentLinkRow { id: "1".into(), name: "Pro plan".into(), .. },
//!     ],
//!     columns: vec![
//!         AdminTableColumn { header: "Name".into(), render: |r| rsx! { "{r.name}" } },
//!     ],
//!     page_size: 25,
//!     on_action: move |action| match action { ... },
//! }
//! ```
//!
//! The action callback receives an `AdminTableAction<T>` enum:
//! - `Edit(T)` — row's Edit button clicked
//! - `Delete(T)` — row's Delete button clicked (caller should show
//!   `<AdminActionConfirm>` modal)
//! - `Revoke(T)` — row's Revoke button clicked (caller should show
//!   `<AdminActionConfirm>` modal)
//!
//! `Delete` and `Revoke` semantically mean "destructive confirmation
//! required"; the primitive doesn't itself render the modal (Track B's
//! `AdminActionConfirm` is a separate primitive, not yet on this branch)
//! — instead it just emits the action and the caller wires up the modal.
//!
//! ## Filter chips
//!
//! Pass a non-empty `filter_chips` slice to render a row of toggle chips
//! above the table. State is internal to the primitive; the filtered row
//! count appears next to the chips.
//!
//! ## Pagination
//!
//! Uses the same Prev/Next + page indicator pattern as the existing
//! `DataTable` primitive. `page_size` is required (no default) because
//! the 4 pages each have a deliberate page size (20 for credits, 25 for
//! payment links, etc.).

use dioxus::prelude::*;

/// Action callback payload — the row type is generic so the caller
/// knows what was clicked. The `Delete` and `Revoke` variants are
/// semantically "needs destructive confirmation" (caller should show
/// `<AdminActionConfirm>`); `Edit` is non-destructive.
#[derive(Clone, Debug, PartialEq)]
pub enum AdminTableAction<T: Clone + PartialEq + std::fmt::Debug + 'static> {
    Edit(T),
    Delete(T),
    Revoke(T),
}

/// Column descriptor — each column has a header string + a render
/// closure that turns a row into an `Element`. We use a plain `fn`
/// pointer rather than a closure (dioxus 0.7 components don't take
/// generic closures as props well).
#[derive(Clone)]
pub struct AdminTableColumn<T: Clone + PartialEq + std::fmt::Debug + 'static> {
    pub header: String,
    pub width: Option<String>,
    pub align: AdminTableAlign,
    pub render: fn(&T) -> Element,
}

impl<T: Clone + PartialEq + std::fmt::Debug + 'static> PartialEq for AdminTableColumn<T> {
    fn eq(&self, _other: &Self) -> bool {
        // Function pointers are comparable; the only time this matters
        // is the per-row click test. Headers / widths are the
        // discriminating fields.
        self.header == _other.header && self.width == _other.width && self.align == _other.align
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AdminTableAlign { Left, #[default] Center, Right }

impl AdminTableAlign {
    fn cls(self) -> &'static str {
        match self {
            AdminTableAlign::Left => "text-left",
            AdminTableAlign::Center => "text-center",
            AdminTableAlign::Right => "text-right",
        }
    }
}

/// Filter chip — a toggleable pill above the table. The `key` is the
/// stable id; `label` is the displayed text; `value` is the chip's
/// payload (e.g. "active", "inactive", "all"). Active state is
/// tracked internally by the primitive via the `selected` prop
/// (controlled) or local signal (uncontrolled).
#[derive(Clone, Debug, PartialEq)]
pub struct AdminTableFilterChip {
    pub key: String,
    pub label: String,
    pub active: bool,
}

/// The `AdminTable<T>` component — the shared list wrapper.
///
/// Props:
/// - `rows: Vec<T>` — the data rows. Caller decides the row type.
/// - `columns: Vec<AdminTableColumn<T>>` — column descriptors with
///   render closures. The last column is the action column.
/// - `page_size: usize` — items per page.
/// - `on_action: EventHandler<AdminTableAction<T>>` — fired when
///   the row's Edit / Delete / Revoke button is clicked.
/// - `filter_chips: Option<Vec<AdminTableFilterChip>>` — optional
///   chip row above the table (status / type filters).
/// - `filter_placeholder: Option<String>` — text filter input
///   placeholder. When set, an `<input type="search">` is rendered.
/// - `caption: Option<String>` — table caption for accessibility.
/// - `actions` slot — fully override the action column by passing
///   an `Element` (for pages that need an action column other than
///   the default Edit / Delete / Revoke pattern). When set, the
///   default action column is hidden and `on_action` is not fired.
#[component]
pub fn AdminTable<T: Clone + PartialEq + std::fmt::Debug + 'static>(
    rows: Vec<T>,
    columns: Vec<AdminTableColumn<T>>,
    page_size: usize,
    on_action: EventHandler<AdminTableAction<T>>,
    #[props(default = None)] filter_chips: Option<Vec<AdminTableFilterChip>>,
    #[props(default = None)] filter_placeholder: Option<String>,
    #[props(default = None)] caption: Option<String>,
    /// Override the action column. When set, the default
    /// Edit / Delete / Revoke button row is hidden.
    #[props(default = None)] actions: Option<Element>,
) -> Element {
    let mut filter = use_signal(String::new);
    let mut page = use_signal(|| 0usize);
    let mut chip_states = use_signal(|| {
        filter_chips
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|c| (c.key, c.active))
            .collect::<Vec<(String, bool)>>()
    });

    // Apply text filter.
    let filter_str = filter.read().clone();
    let filtered: Vec<T> = if filter_str.is_empty() {
        rows.clone()
    } else {
        let q = filter_str.to_lowercase();
        rows.iter()
            .filter(|r| {
                let s = format!("{:?}", r).to_lowercase();
                s.contains(&q)
            })
            .cloned()
            .collect()
    };

    // Paginate.
    let total = filtered.len();
    let page_size = page_size.max(1);
    let total_pages = (total + page_size - 1) / page_size;
    let cur_page = (*page.read()).min(total_pages.saturating_sub(1));
    let start = cur_page * page_size;
    let end = (start + page_size).min(total);
    let page_rows: Vec<T> = if total == 0 { Vec::new() } else { filtered[start..end].to_vec() };

    let align_cls = |a: AdminTableAlign| -> &'static str { a.cls() };

    rsx! {
        // === wave6b-admin-pages-depth-track-c admin-table ===
        div { class: "admin-table data-table",
            if filter_chips.is_some() || filter_placeholder.is_some() {
                div { class: "admin-table-toolbar data-table-toolbar",
                    if let Some(chips) = &filter_chips {
                        div { class: "admin-table-chips",
                            for chip in chips {
                                {
                                    let key = chip.key.clone();
                                    let label = chip.label.clone();
                                    let mut states = chip_states.clone();
                                    let active = *states.read().iter()
                                        .find(|(k, _)| k == &key)
                                        .map(|(_, a)| a)
                                        .unwrap_or(&false);
                                    let active_cls = if active { "chip chip-active" } else { "chip" };
                                    rsx! {
                                        button {
                                            class: "{active_cls}",
                                            r#type: "button",
                                            onclick: move |_| {
                                                let k = key.clone();
                                                let mut s = states.write();
                                                if let Some(entry) = s.iter_mut().find(|(kk, _)| kk == &k) {
                                                    entry.1 = !entry.1;
                                                }
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(ph) = &filter_placeholder {
                        input {
                            class: "input admin-table-filter",
                            r#type: "search",
                            placeholder: "{ph}",
                            value: "{filter_str}",
                            oninput: move |e| {
                                filter.set(e.value().to_string());
                                page.set(0);
                            },
                        }
                    }
                    div { class: "admin-table-count text-sm text-muted-foreground",
                        "{total} row(s)"
                    }
                }
            }
            div { class: "table-wrap",
                table { class: "table",
                    if let Some(cap) = &caption {
                        caption { class: "text-sm text-muted-foreground text-left p-2 admin-table-caption",
                            "{cap}"
                        }
                    }
                    thead {
                        tr {
                            for col in &columns {
                                {
                                    let align = col.align;
                                    let width = col.width.clone();
                                    let header = col.header.clone();
                                    rsx! {
                                        th {
                                            class: "{align_cls(align)}",
                                            style: width.as_ref().map(|w| format!("width:{}", w)).unwrap_or_default(),
                                            "{header}"
                                        }
                                    }
                                }
                            }
                            th { class: "text-right admin-table-actions-col", "Actions" }
                        }
                    }
                    tbody {
                        if page_rows.is_empty() {
                            tr {
                                td {
                                    colspan: (columns.len() + 1).to_string(),
                                    class: "text-center py-8 text-muted-foreground admin-table-empty",
                                    "No data"
                                }
                            }
                        } else {
                            for r in &page_rows {
                                {
                                    let r_clone = r.clone();
                                    let r_for_edit = r.clone();
                                    let r_for_delete = r.clone();
                                    let r_for_revoke = r.clone();
                                    let on_action_edit = on_action.clone();
                                    let on_action_delete = on_action.clone();
                                    let on_action_revoke = on_action.clone();
                                    rsx! {
                                        tr { class: "admin-table-row data-table-row",
                                            for col in &columns {
                                                {
                                                    let align = col.align;
                                                    let render_fn = col.render;
                                                    let row_ref = r;
                                                    rsx! {
                                                        td {
                                                            class: "{align_cls(align)}",
                                                            {render_fn(row_ref)}
                                                        }
                                                    }
                                                }
                                            }
                                            td { class: "text-right admin-table-actions",
                                                if let Some(actions_slot) = &actions {
                                                    {actions_slot}
                                                } else {
                                                    button {
                                                        class: "btn btn-sm btn-ghost admin-table-action-edit",
                                                        r#type: "button",
                                                        title: "Edit",
                                                        onclick: move |_| on_action_edit.call(AdminTableAction::Edit(r_for_edit.clone())),
                                                        "Edit"
                                                    }
                                                    button {
                                                        class: "btn btn-sm btn-ghost text-destructive admin-table-action-delete",
                                                        r#type: "button",
                                                        title: "Delete (requires confirmation)",
                                                        onclick: move |_| on_action_delete.call(AdminTableAction::Delete(r_for_delete.clone())),
                                                        "Delete"
                                                    }
                                                    button {
                                                        class: "btn btn-sm btn-ghost text-warning admin-table-action-revoke",
                                                        r#type: "button",
                                                        title: "Revoke (requires confirmation)",
                                                        onclick: move |_| on_action_revoke.call(AdminTableAction::Revoke(r_for_revoke.clone())),
                                                        "Revoke"
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
            if total_pages > 1 {
                div { class: "pagination admin-table-pagination",
                    button {
                        class: "btn btn-sm btn-outline",
                        disabled: cur_page == 0,
                        r#type: "button",
                        onclick: move |_| if cur_page > 0 { page.set(cur_page - 1); },
                        "Prev"
                    }
                    span { class: "pagination-info admin-table-page-info",
                        "Page {cur_page + 1} of {total_pages}"
                    }
                    button {
                        class: "btn btn-sm btn-outline",
                        disabled: cur_page + 1 >= total_pages,
                        r#type: "button",
                        onclick: move |_| if cur_page + 1 < total_pages { page.set(cur_page + 1); },
                        "Next"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    struct TestRow {
        id: String,
        name: String,
        amount: i64,
    }

    fn render_row_name(r: &TestRow) -> Element {
        rsx! { span { class: "row-name", "{r.name}" } }
    }

    fn render_row_amount(r: &TestRow) -> Element {
        rsx! { span { class: "row-amount", "{r.amount}" } }
    }

    /// Test wrapper component — wraps `AdminTable` in a `#[component]`
    /// so its `use_signal` calls execute inside a Dioxus runtime
    /// scope. The same pattern is used by every page-level test
    /// (`fn render() -> (PageMeta, Element) { rsx! { SomeComponent {...} } }`).
    #[component]
    fn TestHarness(rows: Vec<TestRow>, caption: Option<String>) -> Element {
        let columns = vec![
            AdminTableColumn {
                header: "Name".into(),
                width: Some("40%".into()),
                align: AdminTableAlign::Left,
                render: render_row_name,
            },
            AdminTableColumn {
                header: "Amount".into(),
                width: Some("30%".into()),
                align: AdminTableAlign::Right,
                render: render_row_amount,
            },
        ];
        rsx! {
            AdminTable::<TestRow> {
                rows,
                columns,
                page_size: 10,
                on_action: move |_a| {},
                caption,
            }
        }
    }

    fn render_admin_table(rows: Vec<TestRow>, caption: Option<String>) -> String {
        dioxus_ssr::render_element(rsx! {
            TestHarness { rows, caption }
        })
    }

    /// `admin_table_renders_column_headers` — the canonical
    /// section-marker test. The rendered HTML must contain the
    /// column header strings the caller passed in. This is the
    /// contract: if a page adds the table with `header: "Wallet"`
    /// the page's own `test_section_markers` will see "Wallet" in
    /// the HTML and pass.
    #[test]
    fn admin_table_renders_column_headers() {
        let rows = vec![
            TestRow { id: "1".into(), name: "Alpha".into(), amount: 100 },
            TestRow { id: "2".into(), name: "Beta".into(), amount: 200 },
        ];
        let html = render_admin_table(rows, Some("Test table".to_string()));
        assert!(html.contains("Name"), "AdminTable must render column header 'Name'. Got: {html}");
        assert!(html.contains("Amount"), "AdminTable must render column header 'Amount'. Got: {html}");
        assert!(html.contains("Test table"), "AdminTable must render the caption. Got: {html}");
    }

    /// Empty rows produce the empty-state message and don't crash.
    #[test]
    fn admin_table_renders_empty_state() {
        let html = render_admin_table(vec![], None);
        assert!(html.contains("No data"), "AdminTable empty state must render 'No data'. Got: {html}");
    }

    /// Action column is present (Edit / Delete / Revoke buttons) when
    /// no `actions` override slot is given.
    #[test]
    fn admin_table_renders_action_column() {
        let rows = vec![
            TestRow { id: "1".into(), name: "Alpha".into(), amount: 100 },
        ];
        let html = render_admin_table(rows, None);
        assert!(html.contains("admin-table-action-edit"), "AdminTable must render Edit action button. Got: {html}");
        assert!(html.contains("admin-table-action-delete"), "AdminTable must render Delete action button. Got: {html}");
        assert!(html.contains("admin-table-action-revoke"), "AdminTable must render Revoke action button. Got: {html}");
    }
}
