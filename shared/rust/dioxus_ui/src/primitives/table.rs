use dioxus::prelude::*;

#[component]
pub fn Table(
    headers: Vec<String>,
    striped: Option<bool>,
    hover: Option<bool>,
    /// Optional table caption. When present, rendered inside the `<table>`
    /// as a `<caption>` element (per HTML semantics) — accessible to screen
    /// readers even if visually hidden via the `sr-only` utility.
    caption: Option<String>,
    children: Element,
) -> Element {
    let mut cls = "table".to_string();
    if striped.unwrap_or(false) { cls.push_str(" table-striped"); }
    if hover.unwrap_or(true) { cls.push_str(" table-hover"); }
    rsx! {
        div { class: "table-wrap",
            table { class: "{cls}",
                if let Some(c) = &caption {
                    caption { class: "text-sm text-muted-foreground text-left p-2", "{c}" }
                }
                thead { tr { for h in headers { th { "{h}" } } } }
                tbody { {children} }
            }
        }
    }
}

#[component]
pub fn TableRow(children: Element) -> Element { rsx! { tr { {children} } } }
#[component]
pub fn TableCell(children: Element) -> Element { rsx! { td { {children} } } }

/// Standard `<tfoot>` wrapper. Use for totals rows beneath `Table`.
/// `variant` controls the background: "muted" (default) or "primary"
/// (gradient).
#[component]
pub fn TableFooter(
    /// "muted" matches the admin style; "primary" applies the gradient
    /// background used on the frontend.
    variant: Option<String>,
    class_name: Option<String>,
    children: Element,
) -> Element {
    let v = variant.unwrap_or_else(|| "muted".to_string());
    let mut cls = "font-medium [&>tr]:last:border-b-0".to_string();
    match v.as_str() {
        "muted" => cls.push_str(" border-t border-border bg-white/5 backdrop-blur-sm"),
        "primary" => cls.push_str(" bg-gradient-to-r from-purple-500 to-orange-500 text-white"),
        _ => {}
    }
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! { tfoot { class: "{cls}", {children} } }
}

/// Empty-state row. Spans the given number of columns and centers the
/// message. Use inside a `Table`'s `tbody` when there is no data.
#[component]
pub fn TableEmpty(colspan: usize, class_name: Option<String>, children: Element) -> Element {
    let mut cls = "text-center py-8 text-muted-foreground".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! {
        tr { td { colspan: "{colspan}", class: "{cls}", {children} } }
    }
}

/// Loading-state row. Spans the given number of columns and renders a
/// centered spinner. Use inside a `Table`'s `tbody` while data loads.
#[component]
pub fn TableLoading(colspan: usize, class_name: Option<String>) -> Element {
    let mut cls = "text-center py-8".to_string();
    if let Some(c) = class_name { cls.push(' '); cls.push_str(&c); }
    rsx! {
        tr { td { colspan: "{colspan}", class: "{cls}",
            div { class: "spinner mx-auto" }
        } }
    }
}
