//! `DataTable<T>` — sortable, filterable, paginatable data table.
//!
//! 1:1 port of `apps/frontend/components/ui/table.tsx` and
//! `apps/admin-frontend/components/wallet/wallet-permission-table.tsx`.
//!
//! Caller passes a row list + column descriptors; the table owns sort state,
//! page state, and (optional) text filter state. The actual data is not
//! fetched — the caller controls that (typical: SSR'd rows + client
//! re-fetches).
//!
//! ```
//! DataTable {
//!     columns: vec![
//!         Column { key: "addr".into(), label: "Address".into(), sortable: true, ..Default::default() },
//!         Column { key: "bal".into(), label: "Balance".into(), align: Align::Right, ..Default::default() },
//!     ],
//!     rows: vec![
//!         Row { id: "1".into(), cells: vec!["0xabc".into(), "1.0".into()] },
//!     ],
//! }
//! ```
//!
//! ## Custom cell rendering
//!
//! Pass `cell_renders: Some(vec![(column_key, callback), ...])` to override
//! the default string rendering for specific columns. The callback receives
//! the row and returns an `Element`. Unspecified columns fall back to the
//! raw string cell.
//!
//! ## Row selection
//!
//! Set `selectable: true` to add a leading checkbox column. Use
//! `selected: Vec<String>` to control the selection state externally
//! (controlled mode) and `on_select_change` to be notified when the user
//! toggles a row. When `bulk_actions` is provided, it is rendered above
//! the table whenever at least one row is selected.

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align { Left, Center, Right }

impl Default for Align { fn default() -> Self { Align::Left } }

#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub align: Align,
    pub width: Option<String>,
    pub class_name: Option<String>,
}

impl Default for Column {
    fn default() -> Self {
        Self { key: String::new(), label: String::new(), sortable: false, align: Align::Left, width: None, class_name: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: String,
    pub cells: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortDir { Asc, Desc }

#[component]
pub fn DataTable(
    columns: Vec<Column>,
    rows: Vec<Row>,
    #[props(default = false)] striped: bool,
    #[props(default = true)] hover: bool,
    #[props(default = 10)] page_size: usize,
    #[props(default = None)] initial_sort: Option<(String, SortDir)>,
    #[props(default = None)] filter_placeholder: Option<String>,
    #[props(default = None)] on_row_click: Option<EventHandler<String>>,
    #[props(default = false)] loading: bool,
    #[props(default = None)] empty_message: Option<String>,
    /// Optional custom cell renders. Each entry is `(column_key, render_fn)`.
    /// When a column's key has a render, the cell shows `render_fn(row)` instead
    /// of the raw string.
    #[props(default = None)] cell_renders: Option<Vec<(String, Callback<Row, Element>)>>,
    /// When true, render a leading checkbox column and a "select all" header.
    #[props(default = false)] selectable: bool,
    /// Controlled selection: list of selected row ids. When `None`, the
    /// table manages selection internally.
    #[props(default = None)] selected: Option<Vec<String>>,
    /// Fired whenever the selection changes. The handler receives the new
    /// complete list of selected row ids.
    #[props(default = None)] on_select_change: Option<EventHandler<Vec<String>>>,
    /// Slot for bulk-action UI (e.g. "Delete selected"). Rendered above the
    /// table whenever the selection is non-empty.
    #[props(default = None)] bulk_actions: Option<Element>,
    /// Optional table caption — emitted as `<caption>` for accessibility.
    #[props(default = None)] caption: Option<String>,
) -> Element {
    let mut sort_key = use_signal(|| initial_sort.as_ref().map(|x| x.0.clone()));
    let mut sort_dir = use_signal(|| initial_sort.as_ref().map(|x| x.1).unwrap_or(SortDir::Asc));
    let mut page = use_signal(|| 0usize);
    let mut filter = use_signal(String::new);
    // Internal mirror of the controlled selection — lets header checkbox
    // toggle per-page state without round-tripping to the parent.
    let mut local_selected = use_signal(Vec::<String>::new);
    {
        let sel = selected.clone();
        let mut ls = local_selected.clone();
        use_effect(move || {
            if let Some(s) = sel.clone() {
                ls.set(s);
            }
        });
    }

    let filter_str = filter.read().clone();
    let mut visible: Vec<&Row> = rows.iter().filter(|r| {
        if filter_str.is_empty() { return true; }
        let q = filter_str.to_lowercase();
        r.cells.iter().any(|c| c.to_lowercase().contains(&q))
    }).collect();

    let sort_key_val = sort_key.read().clone();
    let sort_dir_val = *sort_dir.read();
    if let Some(k) = sort_key_val {
        let d = sort_dir_val;
        let col_idx = columns.iter().position(|c| c.key == k);
        if let Some(i) = col_idx {
            visible.sort_by(|a, b| {
                let av = a.cells.get(i).map(|s| s.as_str()).unwrap_or("");
                let bv = b.cells.get(i).map(|s| s.as_str()).unwrap_or("");
                let ord = av.cmp(bv);
                if d == SortDir::Desc { ord.reverse() } else { ord }
            });
        }
    }

    let total = visible.len();
    let page_size = page_size.max(1);
    let total_pages = (total + page_size - 1) / page_size;
    let cur_page = (*page.read()).min(total_pages.saturating_sub(1));
    let start = cur_page * page_size;
    let end = (start + page_size).min(total);
    let page_rows: Vec<&Row> = if total == 0 { Vec::new() } else { visible[start..end].to_vec() };

    let align_cls = |a: Align| match a {
        Align::Left => "text-left",
        Align::Center => "text-center",
        Align::Right => "text-right",
    };

    // Build a fast lookup of cell renders by column key.
    let render_map: std::collections::HashMap<String, Callback<Row, Element>> = cell_renders
        .as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    // "Select all" reflects the current page's rows.
    let page_ids: Vec<String> = page_rows.iter().map(|r| r.id.clone()).collect();
    let mut selected_signal = local_selected.clone();
    let on_select_change_handler = on_select_change.clone();
    let selected_for_all = selected.clone();
    let mut select_all = move |all_page_selected: bool, page_ids: Vec<String>| {
        let current: Vec<String> = match &selected_for_all {
            Some(s) => s.clone(),
            None => selected_signal.read().clone(),
        };
        let mut next = current;
        if all_page_selected {
            next.retain(|id| !page_ids.contains(id));
        } else {
            for id in &page_ids {
                if !next.contains(id) { next.push(id.clone()); }
            }
        }
        if let Some(h) = &on_select_change_handler {
            h.call(next.clone());
        }
        selected_signal.set(next);
    };

    let total_selected_count = match &selected {
        Some(s) => s.len(),
        None => local_selected.read().len(),
    };

    rsx! {
        div { class: "data-table",
            if selectable && total_selected_count > 0 {
                div { class: "data-table-bulk-actions",
                    {bulk_actions.clone().unwrap_or_else(|| rsx! {
                        span { class: "text-sm text-muted-foreground", "{total_selected_count} selected" }
                    })}
                }
            }
            if filter_placeholder.is_some() {
                div { class: "data-table-toolbar",
                    input {
                        class: "input",
                        r#type: "search",
                        placeholder: filter_placeholder.as_deref().unwrap_or("Filter..."),
                        value: "{filter_str}",
                        oninput: move |e| {
                            filter.set(e.value().to_string());
                            page.set(0);
                        },
                    }
                    div { class: "data-table-count text-sm text-muted-foreground", "{total} row(s)" }
                }
            }
            div { class: "table-wrap",
                table { class: if striped { "table table-striped" } else { "table" },
                    if let Some(cap) = &caption {
                        caption { class: "text-sm text-muted-foreground text-left p-2", "{cap}" }
                    }
                    thead {
                        tr {
                            if selectable {
                                {
                                    let current_sel: Vec<String> = match &selected {
                                        Some(s) => s.clone(),
                                        None => local_selected.read().clone(),
                                    };
                                    let page_ids_for_header = page_ids.clone();
                                    let all_sel = !page_ids_for_header.is_empty()
                                        && page_ids_for_header.iter().all(|id| current_sel.contains(id));
                                    let some_sel = page_ids_for_header.iter().any(|id| current_sel.contains(id));
                                    let mut sel_all = select_all.clone();
                                    rsx! {
                                        th { class: "w-10",
                                            input {
                                                r#type: "checkbox",
                                                checked: all_sel,
                                                "aria-checked": if all_sel { "true" } else if some_sel { "mixed" } else { "false" },
                                                role: "checkbox",
                                                onclick: move |e| {
                                                    sel_all(all_sel, page_ids_for_header.clone());
                                                    e.stop_propagation();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            for col in &columns {
                                {
                                    let col_key = col.key.clone();
                                    let col_label = col.label.clone();
                                    let col_sortable = col.sortable;
                                    let col_align = col.align;
                                    let col_width = col.width.clone();
                                    rsx! {
                                        th {
                                            class: if col_sortable { format!("{} cursor-pointer select-none", align_cls(col_align)) } else { align_cls(col_align).to_string() },
                                            style: col_width.as_ref().map(|w| format!("width:{}", w)).unwrap_or_default(),
                                            onclick: move |_| if col_sortable {
                                                let k = col_key.clone();
                                                if sort_key.read().as_deref() == Some(k.as_str()) {
                                                    sort_dir.set(if *sort_dir.read() == SortDir::Asc { SortDir::Desc } else { SortDir::Asc });
                                                } else {
                                                    sort_key.set(Some(k));
                                                    sort_dir.set(SortDir::Asc);
                                                }
                                            },
                                            "{col_label}"
                                            if col_sortable {
                                                if sort_key.read().as_deref() == Some(col_key.as_str()) {
                                                    span { class: "ml-1", if *sort_dir.read() == SortDir::Asc { "▲" } else { "▼" } }
                                                } else { span { class: "ml-1 opacity-30", "↕" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tbody {
                        if loading {
                            tr { td { colspan: (columns.len() + if selectable { 1 } else { 0 }).to_string(), class: "text-center py-8",
                                div { class: "spinner" }
                            } }
                        } else if page_rows.is_empty() {
                            tr { td { colspan: (columns.len() + if selectable { 1 } else { 0 }).to_string(), class: "text-center py-8 text-muted-foreground",
                                "{empty_message.as_deref().unwrap_or(\"No data\")}"
                            } }
                        } else {
                            for r in page_rows {
                                {
                                    let row_id = r.id.clone();
                                    let row_for_render = r.clone();
                                    let row_cells = r.cells.clone();
                                    let col_aligns: Vec<Align> = columns.iter().map(|c| c.align).collect();
                                    let col_classes: Vec<Option<String>> = columns.iter().map(|c| c.class_name.clone()).collect();
                                    let col_keys: Vec<String> = columns.iter().map(|c| c.key.clone()).collect();
                                    let onclick_handler = on_row_click.clone();
                                    let on_select_change_for_row = on_select_change.clone();
                                    let selected_for_row = selected.clone();
                                    let mut selected_signal_for_row = local_selected.clone();
                                    let row_id_for_onclick = row_id.clone();
                                    rsx! {
                                        tr {
                                            class: "data-table-row",
                                            "data-state": {
                                                let current: Vec<String> = match &selected_for_row {
                                                    Some(s) => s.clone(),
                                                    None => selected_signal_for_row.read().clone(),
                                                };
                                                if current.contains(&row_id) { "selected" } else { "" }
                                            },
                                            onclick: move |_| if let Some(h) = &onclick_handler { h.call(row_id_for_onclick.clone()); },
                                            if selectable {
                                                {
                                                    let row_id_for_check = row_id.clone();
                                                    let row_id_for_check_inner = row_id_for_check.clone();
                                                    let mut selected_signal_inner = selected_signal_for_row.clone();
                                                    let on_select_change_inner = on_select_change_for_row.clone();
                                                    let selected_inner = selected_for_row.clone();
                                                    rsx! {
                                                        td { class: "w-10",
                                                            input {
                                                                r#type: "checkbox",
                                                                checked: {
                                                                    let current: Vec<String> = match &selected_inner {
                                                                        Some(s) => s.clone(),
                                                                        None => selected_signal_inner.read().clone(),
                                                                    };
                                                                    current.contains(&row_id_for_check)
                                                                },
                                                                "aria-label": "Select row",
                                                                onclick: move |e| {
                                                                    let current: Vec<String> = match &selected_inner {
                                                                        Some(s) => s.clone(),
                                                                        None => selected_signal_inner.read().clone(),
                                                                    };
                                                                    let mut next = current;
                                                                    let id = row_id_for_check_inner.clone();
                                                                    if next.contains(&id) {
                                                                        next.retain(|x| x != &id);
                                                                    } else {
                                                                        next.push(id);
                                                                    }
                                                                    if let Some(h) = &on_select_change_inner {
                                                                        h.call(next.clone());
                                                                    }
                                                                    selected_signal_inner.set(next);
                                                                    e.stop_propagation();
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            for (i, _c) in col_aligns.iter().enumerate() {
                                                {
                                                    let key = col_keys[i].clone();
                                                    let cell_text = row_cells.get(i).cloned().unwrap_or_default();
                                                    if let Some(cb) = render_map.get(&key) {
                                                        let cb = cb.clone();
                                                        let row_clone = row_for_render.clone();
                                                        rsx! {
                                                            td {
                                                                class: "{align_cls(col_aligns[i])}",
                                                                class: if let Some(extra) = &col_classes[i] { "{extra}" },
                                                                {cb.call(row_clone)}
                                                            }
                                                        }
                                                    } else {
                                                        rsx! {
                                                            td {
                                                                class: "{align_cls(col_aligns[i])}",
                                                                class: if let Some(extra) = &col_classes[i] { "{extra}" },
                                                                "{cell_text}"
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
                }
            }
            if total_pages > 1 {
                div { class: "pagination",
                    button { class: "btn btn-sm btn-outline", disabled: cur_page == 0,
                        onclick: move |_| if cur_page > 0 { page.set(cur_page - 1); },
                        "Prev"
                    }
                    span { class: "pagination-info", "Page {cur_page + 1} of {total_pages}" }
                    button { class: "btn btn-sm btn-outline", disabled: cur_page + 1 >= total_pages,
                        onclick: move |_| if cur_page + 1 < total_pages { page.set(cur_page + 1); },
                        "Next"
                    }
                }
            }
        }
    }
}
