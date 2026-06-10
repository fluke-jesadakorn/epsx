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
) -> Element {
    let mut sort_key = use_signal(|| initial_sort.as_ref().map(|x| x.0.clone()));
    let mut sort_dir = use_signal(|| initial_sort.as_ref().map(|x| x.1).unwrap_or(SortDir::Asc));
    let mut page = use_signal(|| 0usize);
    let mut filter = use_signal(String::new);

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

    rsx! {
        div { class: "data-table",
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
                    thead {
                        tr { for col in &columns {
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
                        } }
                    }
                    tbody {
                        if loading {
                            tr { td { colspan: columns.len().to_string(), class: "text-center py-8",
                                div { class: "spinner" }
                            } }
                        } else if page_rows.is_empty() {
                            tr { td { colspan: columns.len().to_string(), class: "text-center py-8 text-muted-foreground",
                                "{empty_message.as_deref().unwrap_or(\"No data\")}"
                            } }
                        } else {
                            for r in page_rows {
                                {
                                    let row_id = r.id.clone();
                                    let row_cells = r.cells.clone();
                                    let col_aligns: Vec<Align> = columns.iter().map(|c| c.align).collect();
                                    let col_classes: Vec<Option<String>> = columns.iter().map(|c| c.class_name.clone()).collect();
                                    rsx! {
                                        tr {
                                            class: "data-table-row",
                                            onclick: move |_| if let Some(h) = &on_row_click { h.call(row_id.clone()); },
                                            for (i, _c) in col_aligns.iter().enumerate() {
                                                td {
                                                    class: "{align_cls(col_aligns[i])}",
                                                    class: if let Some(extra) = &col_classes[i] { "{extra}" },
                                                    "{row_cells.get(i).map(|s| s.as_str()).unwrap_or(\"\")}"
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
