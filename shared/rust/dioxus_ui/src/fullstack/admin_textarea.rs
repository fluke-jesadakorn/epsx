//! Narrow workaround for Dioxus 0.7 SSR inserting hydration comments into
//! textarea raw text. This accepts plain text only, never HTML or a fragment.
use dioxus::prelude::*;
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
#[component]
pub fn TextArea(
    name: String,
    value: String,
    #[props(default)] class: String,
    #[props(default = 2)] rows: u32,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] maxlength: Option<u32>,
    #[props(default)] required: bool,
    #[props(default)] oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    // Dioxus owns this node. Escaping prevents closing the textarea or creating
    // markup; there are no manual DOM writes, listeners or fetched fragments.
    rsx! {textarea{id:name.clone(),name,class,rows,placeholder,maxlength,required,value:value.clone(),oninput:move |event|{if let Some(handler)=oninput{handler.call(event);}},dangerous_inner_html:escape_text(&value)}}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_text_cannot_escape_textarea() {
        let source = "</textarea><script>alert(1)</script>&hello";
        let mut dom = VirtualDom::new_with_props(
            TextArea,
            TextAreaProps {
                name: "test".into(),
                value: source.into(),
                class: String::new(),
                rows: 2,
                placeholder: None,
                maxlength: None,
                required: false,
                oninput: None,
            },
        );
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("&lt;/textarea&gt;&lt;script&gt;alert(1)&lt;/script&gt;&amp;hello"));
        assert_eq!(html.matches("</textarea>").count(), 1);
        assert!(!html.contains("<script>"));
    }
}
