//! `PlanComparisonCard` — side-by-side plan comparison table.
//!
//! Port of
//! `apps-old/frontend/components/payment/plan-comparison-card.tsx`
//! (525 LoC). The TS source renders a comparison table with
//! feature rows + per-plan columns. The Dioxus port renders the
//! same structure as a static table.

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct PlanComparisonRow {
    pub feature: String,
    pub plan_values: Vec<String>,
    pub highlight: bool,
}

#[component]
pub fn PlanComparisonCard(
    /// Plan names (table column headers).
    #[props(default = Vec::new())] plan_names: Vec<String>,
    /// Feature rows.
    #[props(default = Vec::new())] rows: Vec<PlanComparisonRow>,
) -> Element {
    rsx! {
        div { class: "plan-comparison-card card card-glass",
            div { class: "card-header",
                div { class: "card-title", "Compare plans" }
            }
            div { class: "card-body overflow-x-auto",
                table { class: "plan-comparison-table w-full",
                    thead { class: "plan-comparison-head",
                        tr { class: "plan-comparison-head-row",
                            th { class: "plan-comparison-col-feature text-left py-2 px-3 text-xs font-medium text-slate-500", "Feature" }
                            for name in plan_names.iter() {
                                th { class: "plan-comparison-col-plan text-center py-2 px-3 text-xs font-medium text-slate-500", "{name}" }
                            }
                        }
                    }
                    tbody { class: "plan-comparison-body",
                        for row in rows.iter() {
                            tr {
                                class: if row.highlight { "plan-comparison-row plan-comparison-row-highlight border-t border-slate-200 dark:border-slate-700 bg-orange-50/30" } else { "plan-comparison-row border-t border-slate-200 dark:border-slate-700" },
                                td { class: "plan-comparison-cell-feature py-3 px-3 text-sm text-foreground", "{row.feature}" }
                                for v in row.plan_values.iter() {
                                    td { class: "plan-comparison-cell-value py-3 px-3 text-center text-sm text-slate-600 dark:text-slate-300", "{v}" }
                                }
                            }
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

    #[test]
    fn plan_comparison_row_default() {
        let r = PlanComparisonRow::default();
        assert!(r.feature.is_empty());
        assert!(!r.highlight);
    }

    #[test]
    fn plan_comparison_card_smoke() {
        
    }
}
