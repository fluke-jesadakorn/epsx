//! `FinancialDataTable` — sortable table of stock/financial rows.
//!
//! Port of
//! `apps-old/frontend/components/home/financial-data-table.tsx`
//! (237 LoC). The TS source renders a 6-column table (Symbol /
//! Company / Price / Change / Volume / Market Cap) with sortable
//! columns and a per-row click handler. The Dioxus port renders
//! the same structure as a static table.

use crate::primitives::icon::Icon;

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct FinancialDataRow {
    pub symbol: String,
    pub company: String,
    pub price: String,
    pub change: String,
    pub change_positive: bool,
    pub volume: String,
    pub market_cap: String,
}

#[component]
pub fn FinancialDataTable(
    #[props(default = Vec::new())] rows: Vec<FinancialDataRow>,
    /// Optional click handler for a row. The TS source uses this
    /// to navigate to the asset detail page.
    #[props(default = None)] on_row_click: Option<EventHandler<String>>,
    /// Optional class names appended to the wrapper.
    #[props(default = None)] class_name: Option<String>,
) -> Element {
    let cls = class_name.clone().unwrap_or_default();
    rsx! {
        div { class: "financial-data-table {cls}",
            table { class: "financial-data-table-element w-full",
                thead { class: "financial-data-table-head",
                    tr { class: "financial-data-table-head-row",
                        th { class: "financial-data-table-col-symbol text-left py-2 px-3 text-xs font-medium text-slate-500", "Symbol" }
                        th { class: "financial-data-table-col-company text-left py-2 px-3 text-xs font-medium text-slate-500", "Company" }
                        th { class: "financial-data-table-col-price text-right py-2 px-3 text-xs font-medium text-slate-500", "Price" }
                        th { class: "financial-data-table-col-change text-right py-2 px-3 text-xs font-medium text-slate-500", "Change" }
                        th { class: "financial-data-table-col-volume text-right py-2 px-3 text-xs font-medium text-slate-500", "Volume" }
                        th { class: "financial-data-table-col-market-cap text-right py-2 px-3 text-xs font-medium text-slate-500", "Market Cap" }
                    }
                }
                tbody { class: "financial-data-table-body",
                    for row in rows.iter() {
                        tr {
                            class: "financial-data-table-row border-t border-slate-200 dark:border-slate-700 hover:bg-slate-50 dark:hover:bg-slate-800",
                            onclick: {
                                let sym = row.symbol.clone();
                                move |e| {
                                    let _ = e;
                                    if let Some(cb) = on_row_click.as_ref() {
                                        cb.call(sym.clone());
                                    }
                                }
                            },
                            td { class: "financial-data-table-cell-symbol py-3 px-3 font-bold text-foreground", "{row.symbol}" }
                            td { class: "financial-data-table-cell-company py-3 px-3 text-sm text-slate-600 dark:text-slate-300", "{row.company}" }
                            td { class: "financial-data-table-cell-price py-3 px-3 text-right font-mono text-foreground", "{row.price}" }
                            td {
                                class: if row.change_positive {
                                    "financial-data-table-cell-change financial-data-table-cell-change-positive py-3 px-3 text-right font-mono text-green-500"
                                } else {
                                    "financial-data-table-cell-change financial-data-table-cell-change-negative py-3 px-3 text-right font-mono text-red-500"
                                },
                                if row.change_positive { "▲ " } else { "▼ " }
                                "{row.change}"
                            }
                            td { class: "financial-data-table-cell-volume py-3 px-3 text-right text-sm text-slate-600 dark:text-slate-300", "{row.volume}" }
                            td { class: "financial-data-table-cell-market-cap py-3 px-3 text-right text-sm text-slate-600 dark:text-slate-300", "{row.market_cap}" }
                        }
                    }
                }
            }
            if rows.is_empty() {
                div { class: "financial-data-table-empty text-center py-8 text-slate-500",
                    Icon { name: "inbox".to_string(), size: Some(24), class_name: Some("mx-auto mb-2 text-slate-400".to_string()) }
                    "No data available"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn financial_data_row_default_is_empty() {
        let r = FinancialDataRow::default();
        assert!(r.symbol.is_empty());
        assert!(!r.change_positive);
        assert!(r.volume.is_empty());
    }

    #[test]
    fn financial_data_table_smoke() {
        
    }
}
