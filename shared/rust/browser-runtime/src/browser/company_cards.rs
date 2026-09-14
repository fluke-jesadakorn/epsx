//! Frontend-only native dialogs and confirmed, in-place saved-company updates.
use super::*;
use web_sys::HtmlDialogElement;

pub(super) fn init(document: &Document) -> Result<(), JsValue> {
    for node in document_elements(document, "body.epsx-frontend [data-fe-company-dialog]") {
        let Ok(dialog) = node.dyn_into::<HtmlDialogElement>() else {
            continue;
        };
        let closed_dialog = dialog.clone();
        let close = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
            let Some((window, document)) = window_document() else {
                return;
            };
            if document
                .query_selector("[data-fe-company-dialog][open]")
                .ok()
                .flatten()
                .is_some()
            {
                return;
            }
            if let Some(body) = document.body() {
                let _ = body.class_list().remove_1("fe-company-open");
            }
            if let Some(trigger) = closed_dialog
                .get_attribute("data-return-focus")
                .and_then(|id| document.get_element_by_id(&id))
                .and_then(|node| node.dyn_into::<HtmlElement>().ok())
            {
                let _ = trigger.focus();
            }
            if let Some(y) = closed_dialog
                .get_attribute("data-return-scroll")
                .and_then(|value| value.parse::<f64>().ok())
            {
                window.scroll_to_with_x_and_y(0.0, y);
            }
        });
        dialog.add_event_listener_with_callback("close", close.as_ref().unchecked_ref())?;
        close.forget();
        // Require both ends of the pointer gesture on the backdrop, so selecting
        // text inside the panel and releasing outside never dismisses it.
        for event_name in ["pointerdown", "pointerup"] {
            let pointer_dialog = dialog.clone();
            let pointer = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
                let bounds = pointer_dialog.get_bounding_client_rect();
                let x = f64::from(event.client_x());
                let y = f64::from(event.client_y());
                let outside = x < bounds.left()
                    || x >= bounds.right()
                    || y < bounds.top()
                    || y >= bounds.bottom();
                if event_name == "pointerdown" {
                    let _ = pointer_dialog.set_attribute(
                        "data-backdrop-start",
                        if outside { "true" } else { "false" },
                    );
                } else {
                    let started = pointer_dialog
                        .get_attribute("data-backdrop-start")
                        .as_deref()
                        == Some("true");
                    let _ = pointer_dialog.remove_attribute("data-backdrop-start");
                    if outside && started && event.button() == 0 {
                        pointer_dialog.close();
                    }
                }
            });
            dialog
                .add_event_listener_with_callback(event_name, pointer.as_ref().unchecked_ref())?;
            pointer.forget();
        }
    }
    Ok(())
}

pub(super) fn open(trigger: &Element) {
    let Some((window, document)) = window_document() else {
        return;
    };
    let Some(dialog) = trigger
        .get_attribute("aria-controls")
        .and_then(|id| document.get_element_by_id(&id))
        .and_then(|node| node.dyn_into::<HtmlDialogElement>().ok())
    else {
        return;
    };
    if !dialog.has_attribute("data-fe-company-dialog") || dialog.open() {
        return;
    }
    frontend_navigation_close(false);
    for menu in document_elements(&document, ".fe-account-menu[open], .fe-glossary[open]") {
        let _ = menu.remove_attribute("open");
    }
    let _ = dialog.set_attribute("data-return-focus", &trigger.id());
    let _ = dialog.set_attribute(
        "data-return-scroll",
        &window.scroll_y().unwrap_or_default().to_string(),
    );
    if dialog.show_modal().is_ok() {
        if let Some(body) = document.body() {
            let _ = body.class_list().add_1("fe-company-open");
        }
    }
}

pub(super) fn close(trigger: &Element) {
    if let Some(dialog) = trigger
        .closest("[data-fe-company-dialog]")
        .ok()
        .flatten()
        .and_then(|node| node.dyn_into::<HtmlDialogElement>().ok())
    {
        dialog.close();
    }
}

pub(super) fn key(document: &Document, event: &KeyboardEvent) -> bool {
    let Some(dialog) = document
        .query_selector("body.epsx-frontend [data-fe-company-dialog][open]")
        .ok()
        .flatten()
    else {
        return false;
    };
    if event.key() == "Tab" {
        let controls = elements(&dialog, "a[href], button:not([disabled]), [tabindex='0']");
        if let (Some(first), Some(last), Some(active)) =
            (controls.first(), controls.last(), document.active_element())
        {
            let destination = if event.shift_key() && active == *first {
                Some(last)
            } else if (!event.shift_key() && active == *last) || !dialog.contains(Some(&active)) {
                Some(first)
            } else {
                None
            };
            if let Some(destination) = destination.and_then(|node| node.dyn_ref::<HtmlElement>()) {
                event.prevent_default();
                let _ = destination.focus();
            }
        }
    }
    // Let the native cancel event handle Escape; the workspace drawer must not
    // compete for focus while a company dialog owns the top layer.
    true
}

pub(super) fn confirmed(element: &Element, saved: bool) {
    let Some((_, document)) = window_document() else {
        return;
    };
    let symbol = element.get_attribute("data-symbol").unwrap_or_default();
    for control in document_elements(&document, "[data-watchlist-in-place=true]") {
        if control.get_attribute("data-symbol").as_deref() != Some(&symbol) {
            continue;
        }
        let _ = control.set_attribute("data-watchlisted", if saved { "true" } else { "false" });
        let _ = control.set_attribute("aria-pressed", if saved { "true" } else { "false" });
        let _ = control.set_attribute(
            "aria-label",
            &if saved {
                format!("Saved · Remove {symbol} from saved companies")
            } else {
                format!("Save {symbol}")
            },
        );
    }
    set_watchlist_busy(element, false);
}

pub(super) fn feedback(element: &Element, message: &str, error: bool) {
    if element.get_attribute("data-watchlist-in-place").as_deref() != Some("true") {
        return;
    }
    let Some((_, document)) = window_document() else {
        return;
    };
    let message = message.replace("Saved to your saved companies.", "Company saved.");
    for status in document_elements(&document, "[data-fe-company-feedback]") {
        if status.get_attribute("data-symbol") != element.get_attribute("data-symbol") {
            continue;
        }
        let card = status.get_attribute("data-fe-company-feedback").as_deref() == Some("card");
        status.set_text_content(Some(if card && !error { "" } else { &message }));
        let _ = status.set_attribute("role", if error { "alert" } else { "status" });
    }
}
