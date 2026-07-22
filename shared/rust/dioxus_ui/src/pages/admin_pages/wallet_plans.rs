//! Truthful admin wallet-plan shells for the list and detail routes.
//!
//! The admin BFF owns the authenticated application shell, while typed,
//! backend-authoritative plan reads and mutations are not connected here yet.
//! These leaf renderers therefore keep only the session boundary and an
//! explicit unavailable state. They do not infer plan access from frontend
//! roles, permissions, or entitlement fields, and they expose no compatibility
//! catalog, metrics, editor state, or mutation controls.

use dioxus::prelude::*;

use super::super::{PageContext, PageMeta};
use crate::auth::AuthGate;
use crate::primitives::Icon;

const PLANS_PATH: &str = "/wallet-management/access/plans";
const MAX_PLAN_REFERENCE_SCALARS: usize = 64;

#[derive(Clone, Copy, PartialEq)]
enum PlansSurface {
    List,
    Detail,
}

impl PlansSurface {
    fn meta_title(self) -> &'static str {
        match self {
            Self::List => "Wallet plans unavailable",
            Self::Detail => "Wallet plan unavailable",
        }
    }

    fn marker(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Detail => "detail",
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::List => "Plan workspace",
            Self::Detail => "Plan detail workspace",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::List => "Wallet plans are unavailable",
            Self::Detail => "This wallet plan cannot be verified",
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::List => {
                "No plan records or operational plan data are shown because a backend-authoritative plan read contract is not connected."
            }
            Self::Detail => {
                "No plan record or operational plan data are shown because the backend has not verified the requested plan."
            }
        }
    }

    fn retry_label(self) -> &'static str {
        match self {
            Self::List => "Retry plan availability",
            Self::Detail => "Retry plan detail",
        }
    }
}

/// `/wallet-management/access/plans` — the dispatcher-reachable list entry.
pub fn render(ctx: &PageContext) -> (PageMeta, Element) {
    render_surface(ctx, PlansSurface::List, None)
}

/// Compatibility entry for callers that predate the dispatcher using
/// [`render`]. It deliberately shares the same fail-closed implementation.
pub fn render_plans(ctx: &PageContext) -> (PageMeta, Element) {
    render(ctx)
}

/// `/wallet-management/access/plans/{planId}` — a fail-closed detail shell.
///
/// The route value is only a bounded, control-free diagnostic reference. It is
/// HTML-escaped by Dioxus, explicitly labelled unverified, and percent-encoded
/// as one path segment for its native retry link. It never proves existence,
/// ownership, readability, or manageability of a plan.
pub fn render_editor(ctx: &PageContext) -> (PageMeta, Element) {
    let route_reference = bounded_plan_reference(
        ctx.params
            .get("planId")
            .map(String::as_str)
            .unwrap_or_default(),
    );
    render_surface(ctx, PlansSurface::Detail, route_reference)
}

fn render_surface(
    ctx: &PageContext,
    surface: PlansSurface,
    route_reference: Option<String>,
) -> (PageMeta, Element) {
    let meta = PageMeta::admin(surface.meta_title());
    let retry_href = route_reference
        .as_deref()
        .map(plan_detail_href)
        .unwrap_or_else(|| PLANS_PATH.to_string());

    // Query parameters and legacy hydration values are intentionally ignored.
    // The frontend applies only a session boundary; the Rust backend remains
    // responsible for plan authorization, reads, and mutations.
    (
        meta,
        rsx! {
            AuthGate {
                user: ctx.user.clone(),
                feature: Some("the wallet plan workspace".to_string()),
                // A signed-out detail response must not reflect its route value.
                return_url: Some(PLANS_PATH.to_string()),
                WalletPlansUnavailable { surface, route_reference, retry_href }
            }
        },
    )
}

/// Remove control characters and cap the visible reference by Unicode scalar
/// count. Returning `None` keeps an empty or control-only value out of output.
fn bounded_plan_reference(raw: &str) -> Option<String> {
    let cleaned = raw
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    let cleaned = cleaned.trim();

    if cleaned.is_empty() {
        return None;
    }

    if cleaned.chars().count() <= MAX_PLAN_REFERENCE_SCALARS {
        return Some(cleaned.to_string());
    }

    let mut bounded = cleaned
        .chars()
        .take(MAX_PLAN_REFERENCE_SCALARS.saturating_sub(1))
        .collect::<String>();
    bounded.push('…');
    Some(bounded)
}

/// Encode the already-bounded reference as exactly one URL path segment.
fn plan_detail_href(reference: &str) -> String {
    let mut encoded = String::with_capacity(reference.len());
    for byte in reference.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(*byte));
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[(byte >> 4) as usize]));
                encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
            }
        }
    }
    format!("{PLANS_PATH}/{encoded}")
}

#[component]
fn WalletPlansUnavailable(
    surface: PlansSurface,
    route_reference: Option<String>,
    retry_href: String,
) -> Element {
    let title_id = format!("admin-wallet-plans-{}-unavailable-title", surface.marker());

    rsx! {
        div {
            class: "container page-content max-w-6xl py-10",
            "data-admin-wallet-plans-state": "unavailable",
            "data-admin-wallet-plans-surface": surface.marker(),
            section {
                class: "relative overflow-hidden rounded-3xl border border-border/40 bg-card shadow-2xl",
                role: "status",
                aria_labelledby: title_id.clone(),
                div { class: "absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#ed4b9e]" }
                div { class: "grid gap-8 p-8 md:grid-cols-[auto_1fr] md:p-12",
                    div {
                        class: "flex h-16 w-16 items-center justify-center rounded-2xl border border-cyan-500/20 bg-cyan-500/10 text-[#1fc7d4]",
                        aria_hidden: "true",
                        Icon { name: "layers".to_string(), size: Some(30) }
                    }
                    div {
                        p { class: "text-xs font-black uppercase tracking-[0.22em] text-[#1fc7d4]",
                            {surface.eyebrow()}
                        }
                        h1 { id: title_id, class: "mt-3 text-3xl font-black tracking-tight text-foreground",
                            {surface.title()}
                        }
                        div { class: "mt-5 rounded-2xl border border-amber-500/20 bg-amber-500/10 p-5",
                            p { class: "text-sm font-semibold leading-6 text-foreground",
                                {surface.detail()}
                            }
                        }
                        if let Some(reference) = route_reference {
                            p { class: "mt-4 rounded-xl border border-border/30 bg-background/50 px-4 py-3 text-sm text-muted-foreground",
                                "Unverified plan reference: "
                                code { "data-admin-wallet-plans-reference": "bounded-unverified", "{reference}" }
                            }
                        }
                        p { class: "mt-5 max-w-3xl text-sm leading-6 text-muted-foreground",
                            "The verified session keeps this workspace private. Only the Rust backend may authorize plan reads or management and return typed plan data."
                        }
                        div { class: "mt-8 grid gap-4 sm:grid-cols-3",
                            BoundaryItem {
                                icon: "database",
                                title: "Plan data",
                                detail: "Records remain hidden until a typed backend response is available."
                            }
                            BoundaryItem {
                                icon: "shield",
                                title: "Authorization",
                                detail: "Frontend roles and entitlement fields never grant plan authority."
                            }
                            BoundaryItem {
                                icon: "lock",
                                title: "Operations",
                                detail: "Plan operations remain disabled without verified backend mutations."
                            }
                        }
                        nav { class: "mt-8 flex flex-wrap gap-3", aria_label: "Wallet plan recovery",
                            a { class: "btn btn-primary", href: retry_href,
                                Icon { name: "refresh-cw".to_string(), size: Some(16) }
                                " {surface.retry_label()}"
                            }
                            if surface == PlansSurface::Detail {
                                a { class: "btn btn-outline", href: PLANS_PATH,
                                    Icon { name: "arrow-left".to_string(), size: Some(16) }
                                    " Plan list"
                                }
                            }
                            a { class: "btn btn-ghost", href: "/",
                                Icon { name: "home".to_string(), size: Some(16) }
                                " Admin home"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoundaryItem(icon: &'static str, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/20 bg-background/40 p-5",
            div { class: "flex items-center gap-2 font-semibold text-foreground",
                Icon { name: icon.to_string(), size: Some(18) }
                "{title}"
            }
            p { class: "mt-2 text-sm leading-6 text-muted-foreground", "{detail}" }
            span { class: "mt-3 inline-flex rounded-full border border-amber-500/30 bg-amber-500/10 px-2 py-1 text-xs font-medium text-amber-400",
                "Unavailable"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::auth::User;

    fn session() -> User {
        User {
            id: "plan-session".to_string(),
            address: "0x1234".to_string(),
            chain_id: "56".to_string(),
            roles: vec![],
            permissions: vec![],
            ..Default::default()
        }
    }

    fn context(path: &str, signed_in: bool) -> PageContext {
        PageContext {
            user: signed_in.then(session),
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn detail_context(plan_id: &str, signed_in: bool) -> PageContext {
        PageContext {
            user: signed_in.then(session),
            path: format!("{PLANS_PATH}/{plan_id}"),
            params: HashMap::from([("planId".to_string(), plan_id.to_string())]),
            ..Default::default()
        }
    }

    fn html(element: Element) -> String {
        dioxus_ssr::render_element(element)
    }

    #[test]
    fn signed_out_entries_keep_plan_state_and_detail_reference_private() {
        let live_list = html(render(&context(PLANS_PATH, false)).1);
        let compatibility_list = html(render_plans(&context(PLANS_PATH, false)).1);
        let detail = html(render_editor(&detail_context("private-plan", false)).1);

        for rendered in [live_list, compatibility_list, detail] {
            assert!(rendered.contains("Sign in required"));
            assert!(!rendered.contains("data-admin-wallet-plans-state"));
            assert!(!rendered.contains("Wallet plans are unavailable"));
            assert!(!rendered.contains("private-plan"));
            assert!(rendered.contains("return_url=%2Fwallet-management%2Faccess%2Fplans"));
        }
    }

    #[test]
    fn empty_role_session_reaches_every_public_unavailable_entry() {
        let live_list = html(render(&context(PLANS_PATH, true)).1);
        let compatibility_list = html(render_plans(&context(PLANS_PATH, true)).1);
        let detail = html(render_editor(&detail_context("plan-42", true)).1);

        for rendered in [&live_list, &compatibility_list] {
            assert!(rendered.contains("data-admin-wallet-plans-state=\"unavailable\""));
            assert!(rendered.contains("data-admin-wallet-plans-surface=\"list\""));
            assert!(!rendered.contains("Permission required"));
        }
        assert!(detail.contains("data-admin-wallet-plans-state=\"unavailable\""));
        assert!(detail.contains("data-admin-wallet-plans-surface=\"detail\""));
        assert!(detail.contains("This wallet plan cannot be verified"));
        assert!(!detail.contains("Permission required"));
    }

    #[test]
    fn unavailable_entries_emit_no_samples_catalog_metrics_or_controls() {
        let list = html(render(&context(PLANS_PATH, true)).1);
        let compatibility = html(render_plans(&context(PLANS_PATH, true)).1);
        let detail = html(render_editor(&detail_context("plan-42", true)).1);
        let combined = format!("{list}{compatibility}{detail}");

        for forbidden in [
            "$29",
            "$299",
            "$999",
            "Subscribers",
            "Active Plans",
            "Total wallets",
            "plan-list-sidebar",
            "plan-item-card",
            "plan-editor-drawer",
            "Permissions granted",
            "API limitations",
            "Promotion &amp; discounts",
            "Create plan",
            "New plan",
            "Duplicate plan",
            "Edit plan",
            "Delete plan",
            "Save plan",
            "Discard",
            "Search plans",
            "<form",
            "<input",
            "<select",
            "<textarea",
            "<button",
        ] {
            assert!(!combined.contains(forbidden), "leaked plan UI: {forbidden}");
        }
    }

    #[test]
    fn hostile_plan_id_is_bounded_control_free_escaped_and_one_segment() {
        let hostile = format!(
            "\u{0}\n\t\"><script>alert(1)</script>/../?mode=edit#{}",
            "🦀".repeat(100)
        );
        let bounded = bounded_plan_reference(&hostile).expect("visible reference");
        assert!(bounded.chars().count() <= MAX_PLAN_REFERENCE_SCALARS);
        assert!(!bounded.chars().any(char::is_control));

        let mut ctx = detail_context(&hostile, true);
        ctx.query = "price=999&subscriber=sample&action=delete".to_string();
        ctx.params
            .insert("name".to_string(), "Injected plan".to_string());
        let rendered = html(render_editor(&ctx).1);

        assert!(rendered.contains("Unverified plan reference"));
        assert!(rendered.contains("data-admin-wallet-plans-reference=\"bounded-unverified\""));
        assert!(!rendered.contains("<script>"));
        assert!(rendered.contains("&#60;script&#62;"));
        assert!(rendered.contains("%22%3E%3Cscript%3E"));
        assert!(rendered.contains("%2F..%2F%3Fmode%3Dedit%23"));
        for ignored in [
            "price=999",
            "subscriber=sample",
            "action=delete",
            "Injected plan",
        ] {
            assert!(
                !rendered.contains(ignored),
                "legacy value leaked: {ignored}"
            );
        }
    }

    #[test]
    fn recovery_is_native_exact_and_has_no_mutation_handler() {
        let list = html(render(&context(PLANS_PATH, true)).1);
        let detail = html(render_editor(&detail_context("plan 42/blue", true)).1);

        assert!(list.contains(&format!("href=\"{PLANS_PATH}\"")));
        assert!(list.contains("href=\"/\""));
        assert!(detail.contains(&format!("href=\"{PLANS_PATH}/plan%2042%2Fblue\"")));
        assert!(detail.contains(&format!("href=\"{PLANS_PATH}\"")));
        assert!(detail.contains("href=\"/\""));
        for rendered in [list, detail] {
            assert!(!rendered.contains("onclick="));
            assert!(!rendered.contains("javascript:"));
            assert!(!rendered.contains("method=\"POST\""));
        }
    }

    #[test]
    fn bff_shell_is_not_duplicated_across_entries() {
        let live_list = html(render(&context(PLANS_PATH, true)).1);
        let compatibility_list = html(render_plans(&context(PLANS_PATH, true)).1);
        let detail = html(render_editor(&detail_context("plan-42", true)).1);

        for rendered in [live_list, compatibility_list, detail] {
            assert!(!rendered.contains("admin-shell"));
            assert!(!rendered.contains("<main"));
        }
    }
}
