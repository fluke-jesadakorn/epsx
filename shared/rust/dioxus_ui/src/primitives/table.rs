use dioxus::prelude::*;

#[component]
pub fn Table(headers: Vec<String>, striped: Option<bool>, hover: Option<bool>, children: Element) -> Element {
    let mut cls = "table".to_string();
    if striped.unwrap_or(false) { cls.push_str(" table-striped"); }
    if hover.unwrap_or(true) { cls.push_str(" table-hover"); }
    rsx! {
        div { class: "table-wrap",
            table { class: "{cls}",
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
