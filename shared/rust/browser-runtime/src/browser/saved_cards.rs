//! Frontend-only pointer/keyboard card sorting. Persistence remains in the BFF.
use super::*;

const BOARD: &str = "body.epsx-frontend [data-fe-saved-board]";
const ITEM: &str = "[data-watchlist-item]";
const GROUP: &str = "[data-watchlist-group]";
const HANDLE: &str = "[data-watchlist-item-handle],[data-watchlist-group-handle]";

thread_local! {
    static DRAG: RefCell<Option<Drag>> = const { RefCell::new(None) };
}

struct Drag {
    source: Element,
    handle: Element,
    placeholder: Element,
    ghost: Element,
    snapshot: Value,
    group: bool,
    pointer: Option<i32>,
    offset: (f64, f64),
    point: (f64, f64),
    valid: bool,
}

fn board() -> Option<Element> {
    window_document()?.1.query_selector(BOARD).ok().flatten()
}

pub(super) fn owns(element: &Element) -> bool {
    element
        .closest("body.epsx-frontend [data-fe-saved-board]")
        .ok()
        .flatten()
        .is_some()
}

pub(super) fn busy() -> bool {
    board().is_some_and(|b| b.get_attribute("aria-busy").as_deref() == Some("true"))
}

pub(super) fn set_busy(value: bool) {
    if let Some(b) = board() {
        let _ = b.set_attribute("aria-busy", if value { "true" } else { "false" });
    }
}

fn reduced_motion() -> bool {
    web_sys::window()
        .and_then(|w| {
            w.match_media("(prefers-reduced-motion: reduce)")
                .ok()
                .flatten()
        })
        .is_some_and(|m| m.matches())
}

pub(super) fn positions() -> Vec<(Element, f64, f64)> {
    let Some(b) = board() else {
        return vec![];
    };
    elements(&b, "[data-watchlist-item],[data-watchlist-group]")
        .into_iter()
        .filter(|e| !e.has_attribute("hidden") && !e.class_list().contains("fe-saved-drag-source"))
        .map(|e| {
            let r = e.get_bounding_client_rect();
            (e, r.x(), r.y())
        })
        .collect()
}

pub(super) fn animate_reflow(before: Vec<(Element, f64, f64)>) {
    if reduced_motion() {
        return;
    }
    for (element, x, y) in before {
        let rect = element.get_bounding_client_rect();
        let (dx, dy) = (x - rect.x(), y - rect.y());
        if dx.abs() < 1.0 && dy.abs() < 1.0 {
            continue;
        }
        let frames = Array::new();
        for transform in [format!("translate({dx}px,{dy}px)"), "translate(0,0)".into()] {
            let frame = Object::new();
            let _ = Reflect::set(&frame, &"transform".into(), &transform.into());
            frames.push(&frame);
        }
        // web-sys gates these bindings behind its unstable API flag. Call the
        // standard Web Animations method without changing workspace flags.
        let options = Object::new();
        let _ = Reflect::set(&options, &"duration".into(), &170.into());
        let _ = Reflect::set(
            &options,
            &"easing".into(),
            &"cubic-bezier(.2,.8,.2,1)".into(),
        );
        if let Ok(animate) = Reflect::get(&element, &"animate".into()) {
            if let Some(animate) = animate.dyn_ref::<Function>() {
                let _ = animate.call2(&element, &frames, &options);
            }
        }
    }
}

fn remove(element: &Element) {
    if let Some(parent) = element.parent_node() {
        let _ = parent.remove_child(element);
    }
}

pub(super) fn focus(element: &Element) {
    // The original handle stays offscreen while its visual clone is dragged.
    // Keep keyboard ownership without scrolling the page to that hidden source.
    if let Ok(method) = Reflect::get(element, &"focus".into()) {
        if let Some(method) = method.dyn_ref::<Function>() {
            let options = Object::new();
            let _ = Reflect::set(&options, &"preventScroll".into(), &true.into());
            let _ = method.call1(element, &options);
        }
    }
}

fn announce(message: &str) {
    if let Some(b) = board() {
        organizer_feedback(&b, message, false);
    }
}

pub(super) fn refresh() {
    let Some(b) = board() else {
        return;
    };
    for section in elements(&b, GROUP) {
        let Some(id) = section.get_attribute("data-group-id") else {
            continue;
        };
        let Some(list) = group_list(&id) else {
            continue;
        };
        let count = list_symbols(&list).len();
        if let Ok(Some(label)) = section.query_selector("[data-fe-group-count]") {
            label.set_text_content(Some(&count.to_string()));
        }
        if let Ok(Some(empty)) = list.query_selector("[data-fe-empty-group]") {
            if count == 0 {
                let _ = empty.remove_attribute("hidden");
            } else {
                let _ = empty.set_attribute("hidden", "");
            }
        }
        if id == "ungrouped" {
            for label in elements(&b, "[data-fe-ungrouped-count]") {
                label.set_text_content(Some(&count.to_string()));
            }
        }
    }
    for card in elements(&b, ITEM) {
        let Some(symbol) = card.get_attribute("data-symbol") else {
            continue;
        };
        let group = card.get_attribute("data-group-id").unwrap_or_default();
        let count = card_occurrences(&symbol)
            .iter()
            .filter(|c| c.get_attribute("data-group-id").as_deref() != Some("ungrouped"))
            .count();
        let _ = card.set_attribute("data-membership-count", &count.to_string());
        if let Ok(Some(label)) = card.query_selector("[data-fe-membership-label]") {
            label.set_text_content(Some(&match count {
                0 => "Ready to organize".into(),
                1 => "1 group".into(),
                _ => format!("{count} groups"),
            }));
        }
        if let Ok(Some(select)) = card.query_selector("[data-watchlist-move-to-group]") {
            if let Some(select) = select.dyn_ref::<HtmlSelectElement>() {
                select.set_value(&group);
            }
        }
        // The menu follows the new group after an in-place move.
        if let Ok(Some(button)) = card.query_selector("[data-watchlist-remove-membership]") {
            if group == "ungrouped" {
                let _ = button.set_attribute("hidden", "");
            } else {
                let _ = button.remove_attribute("hidden");
            }
        }
        for choice in elements(&card, "[data-watchlist-membership-choice]") {
            if let Some(input) = choice.dyn_ref::<HtmlInputElement>() {
                let member = card_occurrences(&symbol).iter().any(|c| {
                    c.get_attribute("data-group-id").as_deref() == Some(input.value().as_str())
                });
                input.set_checked(member);
                input.set_disabled(member);
                if let Some(label) = input.parent_element() {
                    if let Ok(Some(added)) = label.query_selector("[data-fe-membership-added]") {
                        if member {
                            let _ = added.remove_attribute("hidden");
                        } else {
                            let _ = added.set_attribute("hidden", "");
                        }
                    }
                }
            }
        }
    }
}

fn clean(drag: &Drag) {
    let _ = drag.source.class_list().remove_1("fe-saved-drag-source");
    let _ = drag.handle.remove_attribute("aria-pressed");
    if let Some(id) = drag.pointer {
        let _ = drag.handle.release_pointer_capture(id);
    }
    remove(&drag.ghost);
    remove(&drag.placeholder);
    if let Some((_, doc)) = window_document() {
        for el in document_elements(&doc, ".fe-saved-dropzone") {
            let _ = el.class_list().remove_1("fe-saved-dropzone");
        }
        if let Some(body) = doc.body() {
            let _ = body.class_list().remove_1("fe-saved-drag-active");
        }
    }
}

fn cancel() {
    let drag = DRAG.with(|d| d.borrow_mut().take());
    if let Some(drag) = drag {
        let before = positions();
        clean(&drag);
        restore_watchlist_layout(&drag.snapshot);
        refresh();
        animate_reflow(before);
        focus(&drag.handle);
        announce("Move cancelled. Your order is unchanged.");
    }
}

fn begin(handle: Element, pointer: Option<i32>, x: f64, y: f64) {
    if busy() {
        return;
    }
    cancel();
    let group = handle.has_attribute("data-watchlist-group-handle");
    let Ok(Some(source)) = handle.closest(if group { GROUP } else { ITEM }) else {
        return;
    };
    let Some(snapshot) = collect_watchlist_layout() else {
        return;
    };
    let Some((_, doc)) = window_document() else {
        return;
    };
    let rect = source.get_bounding_client_rect();
    let Ok(ghost) = source
        .clone_node_with_deep(true)
        .and_then(|n| n.dyn_into::<Element>().map_err(JsValue::from))
    else {
        return;
    };
    // Visual clones must not participate in forms, layout serialization or focus.
    for node in std::iter::once(ghost.clone()).chain(elements(&ghost, "*")) {
        let names = node.get_attribute_names();
        for name in names.iter().filter_map(|v| v.as_string()) {
            if name.starts_with("data-") || name == "id" || name == "name" {
                let _ = node.remove_attribute(&name);
            }
        }
    }
    let _ = ghost.set_attribute("inert", "");
    let _ = ghost.set_attribute("aria-hidden", "true");
    let _ = ghost.class_list().add_1("fe-saved-ghost");
    let _ = ghost.set_attribute(
        "style",
        &format!(
            "width:{}px;left:0;top:0;max-height:{}px;overflow:hidden",
            rect.width(),
            rect.height().min(280.0)
        ),
    );
    let Ok(placeholder) = doc.create_element("div") else {
        return;
    };
    placeholder.set_class_name("fe-saved-placeholder");
    let _ = placeholder.set_attribute("aria-hidden", "true");
    let _ = placeholder.set_attribute("style", &format!("height:{}px", rect.height()));
    placeholder.set_text_content(Some("Drop here"));
    let Some(parent) = source.parent_node() else {
        return;
    };
    let _ = parent.insert_before(&placeholder, Some(&source));
    let _ = source.class_list().add_1("fe-saved-drag-source");
    let _ = handle.set_attribute("aria-pressed", "true");
    if let Some(body) = doc.body() {
        let _ = body.append_child(&ghost);
        let _ = body.class_list().add_1("fe-saved-drag-active");
    }
    let offset = if pointer.is_some() {
        (x - rect.x(), y - rect.y())
    } else {
        (rect.width() / 2.0, 24.0)
    };
    let point = if pointer.is_some() {
        (x, y)
    } else {
        (rect.x() + offset.0, rect.y() + offset.1)
    };
    if let Some(id) = pointer {
        let _ = handle.set_pointer_capture(id);
    }
    focus(&handle);
    let name = source
        .get_attribute("data-symbol")
        .unwrap_or_else(|| "group".into());
    DRAG.with(|state| {
        *state.borrow_mut() = Some(Drag {
            source,
            handle,
            placeholder,
            ghost,
            snapshot,
            group,
            pointer,
            offset,
            point,
            valid: true,
        })
    });
    draw_ghost();
    announce(&format!(
        "Picked up {name}. Choose a position. Escape cancels."
    ));
    if pointer.is_some() {
        scroll_frame();
    }
}

fn draw_ghost() {
    DRAG.with(|state| {
        if let Some(d) = state.borrow().as_ref() {
            if let Some(html) = d.ghost.dyn_ref::<HtmlElement>() {
                let (x, y) = if d.pointer.is_some() {
                    (d.point.0 - d.offset.0, d.point.1 - d.offset.1)
                } else {
                    let r = d.placeholder.get_bounding_client_rect();
                    (r.x(), r.y())
                };
                let rotation = if reduced_motion() {
                    ""
                } else {
                    " rotate(1.5deg) scale(1.02)"
                };
                let _ = html.style().set_property(
                    "transform",
                    &format!("translate3d({x}px,{y}px,0){rotation}"),
                );
            }
        }
    });
}

fn insert_placeholder(drag: &Drag, parent: &Element, before: Option<&Element>) {
    if drag
        .placeholder
        .parent_element()
        .as_ref()
        .is_some_and(|p| p.is_same_node(Some(parent)))
        && match (drag.placeholder.next_element_sibling(), before) {
            (Some(next), Some(before)) => next.is_same_node(Some(before)),
            (None, None) => true,
            _ => false,
        }
    {
        return;
    }
    let positions = positions();
    let _ = parent.insert_before(&drag.placeholder, before.map(|e| &**e));
    if let Ok(Some(empty)) = parent.query_selector("[data-fe-empty-group]") {
        let _ = empty.set_attribute("hidden", "");
    }
    animate_reflow(positions);
}

fn place_at(x: f64, y: f64) {
    let Some((_, doc)) = window_document() else {
        return;
    };
    let target = doc.element_from_point(x as f32, y as f32);
    DRAG.with(|state| {
        let mut state = state.borrow_mut();
        let Some(d) = state.as_mut() else {
            return;
        };
        d.point = (x, y);
        d.valid = false;
        for el in document_elements(&doc, ".fe-saved-dropzone") {
            let _ = el.class_list().remove_1("fe-saved-dropzone");
        }
        let Some(target) = target else {
            return;
        };
        let Some(section) = target.closest(GROUP).ok().flatten().filter(owns) else {
            return;
        };
        if d.group {
            let Some(parent) = section.parent_element() else {
                return;
            };
            let rect = section.get_bounding_client_rect();
            let before = if section.get_attribute("data-group-id").as_deref() == Some("ungrouped")
                || y < rect.y() + rect.height() / 2.0
            {
                Some(section.clone())
            } else {
                section.next_element_sibling()
            };
            if before
                .as_ref()
                .is_some_and(|e| e.is_same_node(Some(&d.placeholder)))
            {
                d.valid = true;
                return;
            }
            insert_placeholder(d, &parent, before.as_ref());
        } else {
            let Some(list) = section
                .query_selector("[data-watchlist-items]")
                .ok()
                .flatten()
            else {
                return;
            };
            let cards = elements(&list, ITEM);
            let before = cards
                .iter()
                .filter(|e| !e.is_same_node(Some(&d.source)) && !e.has_attribute("hidden"))
                .find(|e| {
                    let r = e.get_bounding_client_rect();
                    y < r.top() || (y <= r.bottom() && x < r.x() + r.width() / 2.0)
                });
            insert_placeholder(d, &list, before);
            let _ = list.class_list().add_1("fe-saved-dropzone");
        }
        d.valid = true;
    });
    draw_ghost();
}

fn scroll_frame() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once_into_js(move || {
        let point = DRAG.with(|d| {
            d.borrow()
                .as_ref()
                .filter(|d| d.pointer.is_some())
                .map(|d| d.point)
        });
        let Some((x, y)) = point else {
            return;
        };
        if let Some(window) = web_sys::window() {
            let h = window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or_default();
            let dy = if y < 92.0 {
                -12.0
            } else if y > h - 56.0 {
                12.0
            } else {
                0.0
            };
            if dy != 0.0 {
                window.scroll_by_with_x_and_y(0.0, dy);
                place_at(x, y);
            }
        }
        scroll_frame();
    });
    let _ = window.request_animation_frame(callback.unchecked_ref());
}

fn finish() {
    let Some(d) = DRAG.with(|state| state.borrow_mut().take()) else {
        return;
    };
    if !d.valid {
        DRAG.with(|state| *state.borrow_mut() = Some(d));
        cancel();
        return;
    }
    let before = positions();
    let destination = d.placeholder.parent_element();
    if let Some(parent) = destination.as_ref() {
        if d.group {
            let _ = parent.insert_before(&d.source, Some(&d.placeholder));
        } else if let Some(id) = parent.get_attribute("data-group-id") {
            move_card_to_group_dom(&d.source, &id, Some(&d.placeholder));
        }
    }
    clean(&d);
    refresh();
    animate_reflow(before);
    let active_card = if d.source.has_attribute("hidden") && !d.group {
        d.source.get_attribute("data-symbol").and_then(|symbol| {
            card_occurrences(&symbol).into_iter().find(|card| {
                destination
                    .as_ref()
                    .is_some_and(|parent| parent.contains(Some(card)))
            })
        })
    } else {
        None
    };
    let control = active_card
        .as_ref()
        .and_then(|card| card.query_selector(HANDLE).ok().flatten())
        .unwrap_or_else(|| d.handle.clone());
    focus(&control);
    if collect_watchlist_layout().as_ref() == Some(&d.snapshot) {
        announce("Your order is unchanged.");
        return;
    }
    let _ = d.source.class_list().add_1("fe-saved-arrived");
    save_current_watchlist_layout(active_card.unwrap_or(d.source), d.snapshot);
}

fn keyboard_position(key: &str, shift: bool) {
    DRAG.with(|state| {
        let state = state.borrow();
        let Some(d) = state.as_ref() else {
            return;
        };
        let Some(parent) = d.placeholder.parent_element() else {
            return;
        };
        if key == "Tab" && !d.group {
            let Some(b) = board() else {
                return;
            };
            let lists = elements(&b, "[data-watchlist-items]");
            let index = lists
                .iter()
                .position(|p| p.is_same_node(Some(&parent)))
                .unwrap_or(0);
            let next = if shift {
                (index + lists.len() - 1) % lists.len()
            } else {
                (index + 1) % lists.len()
            };
            let cards = elements(&lists[next], ITEM);
            let before = cards
                .iter()
                .find(|c| !c.is_same_node(Some(&d.source)) && !c.has_attribute("hidden"));
            insert_placeholder(d, &lists[next], before);
        } else if matches!(key, "ArrowLeft" | "ArrowUp" | "ArrowRight" | "ArrowDown") {
            let mut nodes = Vec::new();
            let mut node = parent.first_element_child();
            while let Some(el) = node {
                node = el.next_element_sibling();
                if !el.is_same_node(Some(&d.source))
                    && !el.has_attribute("hidden")
                    && (el.has_attribute(if d.group {
                        "data-watchlist-group"
                    } else {
                        "data-watchlist-item"
                    }) || el.is_same_node(Some(&d.placeholder)))
                {
                    nodes.push(el);
                }
            }
            let Some(index) = nodes
                .iter()
                .position(|p| p.is_same_node(Some(&d.placeholder)))
            else {
                return;
            };
            if matches!(key, "ArrowLeft" | "ArrowUp") && index > 0 {
                insert_placeholder(d, &parent, Some(&nodes[index - 1]));
            } else if matches!(key, "ArrowRight" | "ArrowDown")
                && index + 1 < nodes.len()
                && (!d.group
                    || nodes[index + 1].get_attribute("data-group-id").as_deref()
                        != Some("ungrouped"))
            {
                insert_placeholder(d, &parent, nodes[index + 1].next_element_sibling().as_ref());
            }
        }
        if let Some(list) = d.placeholder.parent_element() {
            if let Ok(method) = Reflect::get(&d.placeholder, &"scrollIntoView".into()) {
                if let Some(method) = method.dyn_ref::<Function>() {
                    let options = Object::new();
                    let _ = Reflect::set(&options, &"block".into(), &"nearest".into());
                    let _ = method.call1(&d.placeholder, &options);
                }
            }
            let group = list.closest(GROUP).ok().flatten();
            let name = group
                .and_then(|g| {
                    g.query_selector("[data-watchlist-group-name]")
                        .ok()
                        .flatten()
                })
                .and_then(|n| n.dyn_into::<HtmlInputElement>().ok())
                .map(|n| n.value())
                .unwrap_or_else(|| "Ungrouped".into());
            announce(&format!(
                "Position selected in {name}. Press Space to drop."
            ));
        }
    });
    draw_ghost();
}

pub(super) fn init(document: &Document) -> Result<(), JsValue> {
    if board().is_none() {
        return Ok(());
    }
    let down = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
        if event.button() != 0 || !event.is_primary() {
            return;
        }
        let Some(target) = event.target().and_then(|v| v.dyn_into::<Element>().ok()) else {
            return;
        };
        let Some(handle) = target.closest(HANDLE).ok().flatten().filter(owns) else {
            return;
        };
        event.prevent_default();
        begin(
            handle,
            Some(event.pointer_id()),
            f64::from(event.client_x()),
            f64::from(event.client_y()),
        );
    });
    document.add_event_listener_with_callback("pointerdown", down.as_ref().unchecked_ref())?;
    down.forget();
    let movement = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
        if DRAG.with(|d| {
            d.borrow()
                .as_ref()
                .is_some_and(|d| d.pointer == Some(event.pointer_id()))
        }) {
            event.prevent_default();
            place_at(f64::from(event.client_x()), f64::from(event.client_y()));
        }
    });
    document.add_event_listener_with_callback("pointermove", movement.as_ref().unchecked_ref())?;
    movement.forget();
    let up = Closure::<dyn FnMut(PointerEvent)>::new(move |event: PointerEvent| {
        if DRAG.with(|d| {
            d.borrow()
                .as_ref()
                .is_some_and(|d| d.pointer == Some(event.pointer_id()))
        }) {
            event.prevent_default();
            place_at(f64::from(event.client_x()), f64::from(event.client_y()));
            finish();
        }
    });
    document.add_event_listener_with_callback("pointerup", up.as_ref().unchecked_ref())?;
    up.forget();
    let cancelled = Closure::<dyn FnMut(Event)>::new(move |_| cancel());
    document
        .add_event_listener_with_callback("pointercancel", cancelled.as_ref().unchecked_ref())?;
    cancelled.forget();
    let key = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        let active = DRAG.with(|d| d.borrow().is_some());
        if active {
            match event.key().as_str() {
                "Escape" => {
                    event.prevent_default();
                    cancel();
                }
                " " | "Enter" => {
                    event.prevent_default();
                    finish();
                }
                "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Tab" => {
                    event.prevent_default();
                    keyboard_position(&event.key(), event.shift_key());
                }
                _ => {}
            }
        } else if matches!(event.key().as_str(), " " | "Enter") {
            let Some(target) = event.target().and_then(|v| v.dyn_into::<Element>().ok()) else {
                return;
            };
            if let Some(handle) = target.closest(HANDLE).ok().flatten().filter(owns) {
                event.prevent_default();
                begin(handle, None, 0.0, 0.0);
            }
        }
    });
    document.add_event_listener_with_callback("keydown", key.as_ref().unchecked_ref())?;
    key.forget();
    if let Some(window) = web_sys::window() {
        let blur = Closure::<dyn FnMut(Event)>::new(move |_| cancel());
        window.add_event_listener_with_callback("blur", blur.as_ref().unchecked_ref())?;
        blur.forget();
    }
    Ok(())
}
