use crate::primitives::icon::Icon;

use dioxus::prelude::*;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastKind { Info, Success, Warning, Error }

#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
    pub id: u64,
    pub kind: ToastKind,
    pub title: String,
    pub description: Option<String>,
    pub action_label: Option<String>,
    pub action_href: Option<String>,
    pub timeout_ms: Option<u64>,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static TOASTS: Mutex<Vec<ToastItem>> = Mutex::new(Vec::new());

pub fn push_toast(mut item: ToastItem) {
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    item.id = id;
    if let Ok(mut g) = TOASTS.lock() {
        g.push(item);
    }
}

pub fn push_info(title: impl Into<String>, desc: Option<String>) {
    push_toast(ToastItem { id: 0, kind: ToastKind::Info, title: title.into(), description: desc, action_label: None, action_href: None, timeout_ms: Some(4000) });
}
pub fn push_success(title: impl Into<String>, desc: Option<String>) {
    push_toast(ToastItem { id: 0, kind: ToastKind::Success, title: title.into(), description: desc, action_label: None, action_href: None, timeout_ms: Some(3500) });
}
pub fn push_warning(title: impl Into<String>, desc: Option<String>) {
    push_toast(ToastItem { id: 0, kind: ToastKind::Warning, title: title.into(), description: desc, action_label: None, action_href: None, timeout_ms: Some(5000) });
}
pub fn push_error(title: impl Into<String>, desc: Option<String>) {
    push_toast(ToastItem { id: 0, kind: ToastKind::Error, title: title.into(), description: desc, action_label: None, action_href: None, timeout_ms: Some(8000) });
}
pub fn dismiss_toast(id: u64) {
    if let Ok(mut g) = TOASTS.lock() {
        g.retain(|t| t.id != id);
    }
}

pub fn snapshot_toasts() -> Vec<ToastItem> {
    TOASTS.lock().map(|g| g.clone()).unwrap_or_default()
}

#[component]
pub fn ToastProvider() -> Element {
    let toasts = snapshot_toasts();
    rsx! {
        div { class: "toast-container",
            for t in toasts {
                ToastItemView { item: t }
            }
        }
    }
}

#[component]
pub fn ToastItemView(item: ToastItem) -> Element {
    let cls = match item.kind {
        ToastKind::Info => "toast toast-info",
        ToastKind::Success => "toast toast-success",
        ToastKind::Warning => "toast toast-warning",
        ToastKind::Error => "toast toast-error",
    };
    let id = item.id;
    let icon_name = match item.kind {
        ToastKind::Info => "info",
        ToastKind::Success => "check-circle",
        ToastKind::Warning => "alert-triangle",
        ToastKind::Error => "x-circle",
    };
    rsx! {
        div { class: "{cls}", role: "alert",
            span { class: "toast-icon", Icon { name: icon_name.to_string(), size: Some(18) } }
            div { class: "flex-1",
                div { class: "toast-title", "{item.title}" }
                if let Some(d) = &item.description {
                    div { class: "toast-description text-sm", "{d}" }
                }
            }
            if let (Some(lbl), Some(href)) = (&item.action_label, &item.action_href) {
                a { class: "toast-action", href: "{href}", "{lbl}" }
            }
            button { class: "toast-close", "aria-label": "Dismiss",
                onclick: move |_| dismiss_toast(id),
                "✕"
            }
        }
    }
}
