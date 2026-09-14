//! Display-only views of backend purchase and fulfillment projections.
use dioxus::prelude::*;
use serde_json::Value;

pub fn text(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or("—").to_string()
}
pub fn amount(v: &Value) -> String {
    let raw = v["amount"].as_str().unwrap_or("0");
    let decimals = v["token_decimals"].as_u64().unwrap_or(0).min(36) as usize;
    if !raw.bytes().all(|b| b.is_ascii_digit()) {
        return "—".into();
    }
    if decimals == 0 {
        return format!("{raw} {}", text(v, "token"));
    }
    let padded = format!("{raw:0>width$}", width = decimals + 1);
    let (whole, fraction) = padded.split_at(padded.len() - decimals);
    let fraction = fraction.trim_end_matches('0');
    format!(
        "{whole}.{} {}",
        if fraction.is_empty() { "00" } else { fraction },
        text(v, "token")
    )
}
#[component]
pub fn OrdersPage(data: Value, admin: bool) -> Element {
    if !admin {
        return rsx! { PurchaseHistory { data } };
    }
    use super::purchases::{follow, PurchasesNavigation};
    let navigation = try_use_context::<PurchasesNavigation>();
    let base = if admin {
        "/payments/epsx"
    } else {
        "/account/payments"
    };
    let detail = data.get("order_id").is_some();
    let rows = data["orders"].as_array().cloned().unwrap_or_default();
    rsx! {main {class:"container-x max-w-6xl mx-auto py-10 space-y-6",
        nav {class:"flex flex-wrap gap-5 text-sm",
            crate::navigation::AppLink {href:if admin {"/"}else{"/account"},class:"underline",if admin {"Admin home"}else{"My account"}}
            crate::navigation::AppLink {href:"/plans",class:"underline","Plans"}
            if admin {crate::navigation::AppLink {href:"/payments",class:"underline","Other payments"} crate::navigation::AppLink {href:"/pay/merchant-escrows",class:"underline","Escrow disputes"}}
        }
        h1 {class:"text-3xl font-bold","EPSX Plan purchases"}
        p {class:"text-muted-foreground","Payment confirmation and plan access are tracked separately. Refresh to check the latest status."}
        if let Some(nav) = navigation { button { r#type:"button",class:"btn btn-outline",disabled:(nav.pending)(),onclick:move |_|nav.refresh.call(()),"Refresh status" } } else { crate::navigation::AppLink {href:if detail {format!("{base}/{}",text(&data,"order_id"))}else{base.into()},class:"btn btn-outline","Refresh status"} }
        if detail {
            section {class:"border rounded-2xl p-6 space-y-4 bg-card",
                h2 {class:"text-xl font-semibold",{text(&data,"plan_name")}}
                p {class:"text-2xl font-semibold",{amount(&data)}}
                dl {class:"grid sm:grid-cols-2 gap-3 break-all",
                    for (label,key) in [("Order","order_id"),("Buyer wallet","wallet_address"),("Payment","payment_id"),("Payment status","payment_status"),("Fulfillment","fulfillment_status"),("Transaction","tx_hash"),("Contract","contract_address"),("Created","created_at")] {
                        div {dt {class:"text-sm text-muted-foreground","{label}"} dd {{text(&data,key)}}}
                    }
                }
                if data["payment_status"]=="succeeded" && data["fulfillment_status"]=="pending" {
                    p {role:"status",class:"rounded-lg p-4 bg-amber-50 text-amber-900","Payment confirmed. Waiting for plan activation — please refresh shortly."}
                }
                if data["payment_available"]==false {p {role:"status","Payment verification is temporarily unavailable. Your recorded order is preserved."}}
                if data["fulfillment_status"]=="granted" {p {role:"status",class:"text-emerald-600 font-semibold","Plan activation recorded. Your current access is available in My account."}}
            }
            crate::navigation::AppLink {href:base,onclick:move |event|follow(event,navigation,base.into()),class:"underline","All purchases"}
        } else {
            if rows.is_empty() {p {role:"status","No Pay purchases yet."}}
            div {class:"overflow-x-auto border rounded-2xl bg-card",
                table {class:"w-full text-left",thead {tr {for label in ["Plan","Amount","Order status","Plan access","Created"] {th {class:"p-4","{label}"}}}}
                    tbody {for row in rows {tr {class:"border-t",
                        td {class:"p-4", {let url = format!("{base}/{}",text(&row,"order_id")); rsx!{crate::navigation::AppLink {class:"underline font-semibold",href:url.clone(),onclick:move |event|follow(event,navigation,url.clone()),{text(&row,"plan_name")}}}}}
                        td {class:"p-4 whitespace-nowrap",{amount(&row)}}
                        td {class:"p-4",{text(&row,"status")}}
                        td {class:"p-4",{text(&row,"fulfillment_status")}}
                        td {class:"p-4 text-sm",{text(&row,"created_at")}}
                    }}}
                }
            }
            if let Some(offset)=data["next_offset"].as_i64() {crate::navigation::AppLink {href:format!("{base}?offset={offset}"),onclick:move |event|follow(event,navigation,format!("{base}?offset={offset}")),class:"btn btn-outline","Older purchases"}}
        }
    }}
}

fn purchase_date(value: &Value) -> String {
    chrono::DateTime::parse_from_rfc3339(&text(value, "created_at"))
        .map(|date| {
            date.with_timezone(&chrono::Utc)
                .format("%d %b %Y · %H:%M UTC")
                .to_string()
        })
        .unwrap_or_else(|_| "Date unavailable".into())
}

#[component]
fn PurchaseBadge(status: String) -> Element {
    let (label, tone) = match status.as_str() {
        "succeeded" => ("Paid", "positive"),
        "granted" => ("Granted", "positive"),
        "pending" => ("Pending", "warning"),
        "processing" => ("Processing", "warning"),
        "expired" => ("Expired", "neutral"),
        "failed" => ("Failed", "danger"),
        "cancelled" | "canceled" => ("Cancelled", "neutral"),
        _ => (status.as_str(), "neutral"),
    };
    rsx! { span { class: "fe-purchase-badge", "data-tone": tone, "{label}" } }
}

#[component]
fn PurchaseHistory(data: Value) -> Element {
    use super::purchases::{follow, PurchasesNavigation};
    use crate::enterprise::FrontendIcon as Icon;
    let navigation = try_use_context::<PurchasesNavigation>();
    let detail = data.get("order_id").is_some();
    let rows = data["orders"].as_array().cloned().unwrap_or_default();
    let refresh = if detail {
        format!("/account/payments/{}", text(&data, "order_id"))
    } else {
        "/account/payments".into()
    };
    rsx! { section { class: "fe-page fe-purchases",
        header { class: "fe-page-header",
            div { p { class: "fe-purchase-eyebrow", "YOUR ACCOUNT" }
                h1 { if detail { "Purchase details" } else { "Plan purchases" } }
                p { "Your payment history and plan activation, in one place." }
            }
            div { class: "fe-purchase-actions",
                if let Some(nav) = navigation {
                    button { r#type: "button", class: "fe-button", disabled: (nav.pending)(), onclick: move |_| nav.refresh.call(()), Icon { name: "refresh-cw", size: 16 } "Refresh status" }
                } else { crate::navigation::AppLink { href: refresh, class: "fe-button", Icon { name: "refresh-cw", size: 16 } "Refresh status" } }
                crate::navigation::AppLink { href: "/plans", class: "fe-button fe-primary", "Explore plans" Icon { name: "arrow-right", size: 16 } }
            }
        }
        if detail {
            crate::navigation::AppLink { href: "/account/payments", onclick: move |event| follow(event, navigation, "/account/payments".into()), class: "fe-text-link", "← All purchases" }
            section { class: "fe-purchase-panel",
                header { class: "fe-purchase-panel-heading", h2 { {text(&data,"plan_name")} } strong { {amount(&data)} } }
                dl { class: "fe-purchase-details",
                    div { dt { "Payment" } dd { PurchaseBadge { status: text(&data,"payment_status") } } }
                    div { dt { "Plan access" } dd { PurchaseBadge { status: text(&data,"fulfillment_status") } } }
                    div { dt { "Purchased on" } dd { {purchase_date(&data)} } }
                    for (label,key) in [("Order ID","order_id"),("Buyer wallet","wallet_address"),("Payment ID","payment_id"),("Transaction","tx_hash"),("Contract","contract_address")] {
                        div { dt { "{label}" } dd { class: "fe-purchase-reference", {text(&data,key)} } }
                    }
                }
            }
            if data["payment_status"]=="succeeded" && data["fulfillment_status"]=="pending" {
                p { role: "status", class: "fe-purchase-note", "Payment received. Your plan is being activated. Refresh shortly to check progress." }
            }
            if data["payment_available"]==false {
                p { role: "status", class: "fe-purchase-note", "Payment verification is temporarily unavailable. Your recorded order is preserved." }
            }
        } else if rows.is_empty() {
            section { class: "fe-state", role: "status",
                div { class: "fe-state-art", Icon { name: "receipt", size: 28 } }
                h2 { "No purchases yet" }
                p { "Once you purchase a plan, your payment and activation details will appear here." }
                crate::navigation::AppLink { href: "/plans", class: "fe-button fe-primary", "Find a plan" }
            }
        } else {
            section { class: "fe-purchase-panel", "aria-label": "Purchase history",
                header { class: "fe-purchase-panel-heading", h2 { "Purchase history" } span { "{rows.len()} purchases on this page" } }
                table { class: "fe-purchase-table",
                    thead { tr { for label in ["Plan", "Amount", "Payment", "Plan access", "Purchased on", ""] { th { scope: "col", "{label}" } } } }
                    tbody { for row in rows {
                        { let detail_url = format!("/account/payments/{}",text(&row,"order_id")); let plan_url = detail_url.clone(); rsx! { tr {
                        td { "data-label": "Plan", crate::navigation::AppLink { class: "fe-purchase-plan", onclick: move |event| follow(event, navigation, plan_url.clone()), href: format!("/account/payments/{}",text(&row,"order_id")), {text(&row,"plan_name")} } }
                        td { "data-label": "Amount", class: "fe-purchase-amount", {amount(&row)} }
                        td { "data-label": "Payment", PurchaseBadge { status: text(&row,"status") } }
                        td { "data-label": "Plan access", PurchaseBadge { status: text(&row,"fulfillment_status") } }
                        td { "data-label": "Purchased on", time { datetime: text(&row,"created_at"), {purchase_date(&row)} } }
                        td { crate::navigation::AppLink { class: "fe-text-link", onclick: move |event| follow(event, navigation, detail_url.clone()), href: format!("/account/payments/{}",text(&row,"order_id")), "aria-label": format!("View {} purchase from {}",text(&row,"plan_name"),purchase_date(&row)), "Details" Icon { name: "arrow-right", size: 14 } } }
                    } } } } }
                }
            }
            if let Some(offset) = data["next_offset"].as_i64() { crate::navigation::AppLink { href: format!("/account/payments?offset={offset}"), onclick: move |event| follow(event, navigation, format!("/account/payments?offset={offset}")), class: "fe-button", "Older purchases" } }
        }
        aside { class: "fe-purchase-note", Icon { name: "info", size: 18 }
            p { "Payment and plan activation are tracked separately. A granted purchase records activation; check " crate::navigation::AppLink { href: "/account", "My account" } " for your current access." }
        }
    } }
}

#[cfg(test)]
mod purchase_tests {
    use super::*;

    #[test]
    fn purchase_dates_are_readable_and_normalized_to_utc() {
        assert_eq!(
            purchase_date(&serde_json::json!({"created_at":"2026-09-09T10:37:17.184758+07:00"})),
            "09 Sep 2026 · 03:37 UTC"
        );
        assert_eq!(
            purchase_date(&serde_json::json!({"created_at":"invalid"})),
            "Date unavailable"
        );
    }

    #[test]
    fn purchase_history_preserves_independent_statuses_and_detail_links() {
        let mut dom = VirtualDom::new_with_props(
            OrdersPage,
            OrdersPageProps {
                admin: false,
                data: serde_json::json!({"orders":[{"order_id":"example","plan_name":"1 Day Package","amount":"5000000","token_decimals":6,"token":"USDT","status":"succeeded","fulfillment_status":"expired","created_at":"2026-09-09T03:37:17Z"}]}),
            },
        );
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Paid"));
        assert!(html.contains("Expired"));
        assert!(html.contains("5.00 USDT"));
        assert!(html.contains("/account/payments/example"));
        assert!(!html.contains("<main"));
    }
}
