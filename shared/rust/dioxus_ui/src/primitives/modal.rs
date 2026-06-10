use dioxus::prelude::*;

#[component]
pub fn Modal(
    open: bool,
    on_close: EventHandler<MouseEvent>,
    title: Option<String>,
    description: Option<String>,
    size: Option<String>,
    children: Element,
) -> Element {
    if !open { return rsx! { Fragment {} }; }
    let size_cls = match size.as_deref() {
        Some("sm") => "modal modal-sm",
        Some("lg") => "modal modal-lg",
        Some("xl") => "modal modal-xl",
        Some("full") => "modal modal-full",
        _ => "modal",
    };
    rsx! {
        div { class: "modal-overlay", onclick: move |e| on_close.call(e),
            div { class: "{size_cls}", role: "dialog", "aria-modal": "true",
                onclick: |e| e.stop_propagation(),
                if let Some(t) = &title {
                    div { class: "modal-header",
                        h2 { class: "modal-title", "{t}" }
                        button { class: "modal-close", onclick: move |e| on_close.call(e), "✕" }
                    }
                }
                if let Some(d) = &description {
                    p { class: "modal-description text-sm text-muted-foreground mb-4", "{d}" }
                }
                div { class: "modal-body", {children} }
            }
        }
    }
}
