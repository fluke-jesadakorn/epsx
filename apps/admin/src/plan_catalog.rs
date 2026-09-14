//! The Admin BFF renders and forwards catalog data; Rust backend owns all rules.
use crate::AppState;
#[cfg(test)]
use axum::{extract::OriginalUri, response::Html};
use axum::{
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use dioxus::prelude::*;
#[cfg(test)]
use epsx_dioxus_ui::payment::orders::{text, OrdersPage, OrdersPageProps};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(crate) async fn read(
    state: &AppState,
    headers: &HeaderMap,
    path: &str,
) -> Result<Value, Response> {
    let Some((token, _)) = state.session().verified_access_token(headers).await else {
        return Err(Redirect::to("/auth").into_response());
    };
    let r = state
        .content
        .auth_client()
        .get(format!("{}{path}", state.api_url))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY.into_response())?;
    if !r.status().is_success() {
        return Err(r.status().into_response());
    }
    r.json()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY.into_response())
}
#[cfg(test)]
fn page_html(title: &str, dom: &mut VirtualDom) -> Response {
    dom.rebuild_in_place();
    (
        [("cache-control", "no-store")],
        Html(epsx_templates::page_shell_with_body_class(
            title,
            "EPSX Admin",
            "",
            &dioxus_ssr::render(dom),
            false,
            "page-bg",
        )),
    )
        .into_response()
}
#[cfg(test)]
#[component]
fn CatalogPage(data: Value, saved: bool) -> Element {
    let detail = data.get("id").is_some();
    let rows = data
        .pointer("/data/plans")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let permissions = data["permissions"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let metadata = &data["metadata"];
    rsx! {main{class:"container-x max-w-5xl mx-auto py-10 space-y-6",
        nav{class:"flex gap-5",epsx_dioxus_ui::navigation::AppLink {href:"/",class:"underline","Admin home"}epsx_dioxus_ui::navigation::AppLink {href:"/plans",class:"underline","EPSX Plans"}epsx_dioxus_ui::navigation::AppLink {href:"/payments/epsx",class:"underline","Plan purchases"}epsx_dioxus_ui::navigation::AppLink {href:"/pay/merchant-escrows",class:"underline","Escrow disputes"}}
        h1{class:"text-3xl font-bold","EPSX Plan catalog"}
        p{class:"text-muted-foreground","These plans appear on the main EPSX website. Set regular token prices, then enable a catalog promotion for checkout. Existing orders keep their reserved price."}
        if saved{p{role:"status",class:"text-emerald-600","Plan saved."}}
        if detail{
            form{method:"post",action:format!("/plans/{}",text(&data,"id")),class:"grid gap-5 rounded-2xl border bg-card p-6",
                label{class:"grid gap-2","Name",input{name:"name",value:text(&data,"name"),required:true,class:"input input-bordered"}}
                label{class:"grid gap-2","Description",textarea{name:"description",class:"textarea textarea-bordered",{data["description"].as_str().unwrap_or("")}}}
                for symbol in ["USDT","USDC"]{label{class:"grid gap-2","{symbol} regular price",input{name:symbol,value:metadata["pay_prices"][symbol].as_str().unwrap_or(""),required:true,inputmode:"decimal",class:"input input-bordered"}}}
                fieldset{class:"grid gap-4 rounded-xl border p-5",
                    legend{class:"px-2 font-semibold","Sale promotion"}
                    label{class:"grid gap-2","Apply catalog promotion to token checkout",select{name:"pay_use_catalog_promotion",class:"select select-bordered",option{value:"true",selected:metadata["pay_use_catalog_promotion"]==true,"Enabled"}option{value:"false",selected:metadata["pay_use_catalog_promotion"]!=true,"Disabled"}}}
                    label{class:"grid gap-2","Promotion status",select{name:"promotion_enabled",class:"select select-bordered",option{value:"true",selected:metadata["promotion"]["enabled"]==true,"Enabled (uses schedule)"}option{value:"false",selected:metadata["promotion"]["enabled"]!=true,"Disabled"}}}
                    label{class:"grid gap-2","Discount type",select{name:"promotion_type",class:"select select-bordered",option{value:"percentage",selected:metadata["promotion"]["type"]=="percentage","Percentage"}option{value:"fixed",selected:metadata["promotion"]["type"]!="percentage","Fixed amount off"}}}
                    label{class:"grid gap-2","Discount value",input{name:"promotion_value",inputmode:"decimal",value:metadata["promotion"]["value"].as_str().map(str::to_owned).unwrap_or_else(||metadata["promotion"].get("value").map(Value::to_string).unwrap_or_else(||"0".into())),class:"input input-bordered"}}
                    label{class:"grid gap-2","Optional sale price (overrides discount; blank to use discount)",input{name:"promotion_price",inputmode:"decimal",value:metadata["promotion"]["price"].as_str().map(str::to_owned).unwrap_or_else(||metadata["promotion"]["price"].as_f64().filter(|p|*p>0.0).map(|p|p.to_string()).unwrap_or_default()),class:"input input-bordered"}}
                    for (key,label_text) in [("start_date","Starts at (UTC, e.g. 2026-09-10T00:00:00Z)"),("end_date","Ends at (UTC; blank for no end date)")] {label{class:"grid gap-2","{label_text}",input{name:format!("promotion_{key}"),value:metadata["promotion"][key].as_str().unwrap_or(""),class:"input input-bordered"}}}
                    p{class:"text-sm text-muted-foreground","The backend applies the sale once. Scheduled and expired promotions charge the regular price."}
                }
                label{class:"grid gap-2","Duration in days (blank for lifetime)",input{name:"duration_days",r#type:"number",min:1,max:3650,value:metadata["duration_days"].as_i64().map(|n|n.to_string()).unwrap_or_default(),class:"input input-bordered"}}
                label{class:"grid gap-2","Available for new purchases",select{name:"is_active",class:"select select-bordered",option{value:"true",selected:data["is_active"]==true,"Enabled"}option{value:"false",selected:data["is_active"]==false,"Disabled"}}}
                label{class:"grid gap-2","Permissions (one per line)",textarea{name:"permissions",rows:8,class:"textarea textarea-bordered font-mono","{permissions}"}}
                button{r#type:"submit",class:"btn btn-primary","Save plan"}
            }
        }else{
            for row in rows{article{class:"rounded-2xl border bg-card p-6 flex flex-wrap justify-between gap-5",
                div{h2{class:"text-xl font-semibold",{text(&row,"name")}}p{{row["metadata"]["duration_days"].as_i64().map(|days|format!("{days} days")).unwrap_or_else(||text(&row,"billing_model"))}}p{if row["is_active"]==true{"Enabled"}else{"Disabled"}}}
                epsx_dioxus_ui::navigation::AppLink {href:format!("/plans/{}",text(&row,"id")),class:"btn btn-outline","Edit plan"}
            }}
        }
    }}
}
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Retired route retained only as a migration fixture"
)]
pub async fn catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let suffix = uri.path().strip_prefix("/plans").unwrap_or("");
    if !suffix.is_empty() && uuid::Uuid::parse_str(suffix.trim_start_matches('/')).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let data = match read(&state, &headers, &format!("/api/admin/plans{suffix}")).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    page_html(
        "EPSX Plans",
        &mut VirtualDom::new_with_props(
            CatalogPage,
            CatalogPageProps {
                data,
                saved: uri.query() == Some("saved=true"),
            },
        ),
    )
}
pub async fn save(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Form(fields): Form<HashMap<String, String>>,
) -> Response {
    if !crate::same_origin_admin_notification_form(&headers) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let days = match fields
        .get("duration_days")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(s) => match s.parse::<i64>() {
            Ok(n) => Some(n),
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        },
        None => None,
    };
    let active = fields.get("is_active").and_then(|s| s.parse::<bool>().ok());
    let promotion = json!({
        "enabled":fields.get("promotion_enabled").is_some_and(|s|s=="true"),
        "type":fields.get("promotion_type").map(String::as_str).unwrap_or("fixed"),
        "value":fields.get("promotion_value").filter(|s| !s.trim().is_empty()).map(String::as_str).unwrap_or("0"),
        "price":fields.get("promotion_price").filter(|s| !s.trim().is_empty()).map(String::as_str).unwrap_or("0"),
        "start_date":fields.get("promotion_start_date").map(String::as_str).unwrap_or(""),
        "end_date":fields.get("promotion_end_date").map(String::as_str).unwrap_or("")
    });
    let body = json!({"name":fields.get("name"),"description":fields.get("description"),"is_active":active,"billing_model":if days.is_none(){Some("lifetime")}else{None},"permissions":fields.get("permissions").map(|s|s.lines().map(str::trim).filter(|s|!s.is_empty()).collect::<Vec<_>>()),"metadata":{"promotion":promotion,"pay_use_catalog_promotion":fields.get("pay_use_catalog_promotion").is_some_and(|s|s=="true"),"pay_prices":{"USDT":fields.get("USDT"),"USDC":fields.get("USDC")},"duration_days":days}});
    let Ok(r) = state
        .content
        .auth_client()
        .put(format!("{}/api/admin/plans/{id}", state.api_url))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
    else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    if !r.status().is_success() {
        return (
            r.status(),
            "Plan was not saved. Check prices, duration and permissions.",
        )
            .into_response();
    }
    Redirect::to(&format!("/plans/{id}?saved=true")).into_response()
}
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Retired route retained only as a migration fixture"
)]
pub async fn orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Response {
    let suffix = uri.path().strip_prefix("/payments/epsx").unwrap_or("");
    if !suffix.is_empty() && uuid::Uuid::parse_str(suffix.trim_start_matches('/')).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let query = uri.query().map(|q| format!("?{q}")).unwrap_or_default();
    let data = match read(
        &state,
        &headers,
        &format!("/api/admin/pay-orders{suffix}{query}"),
    )
    .await
    {
        Ok(v) => v,
        Err(r) => return r,
    };
    page_html(
        "Plan purchases",
        &mut VirtualDom::new_with_props(OrdersPage, OrdersPageProps { data, admin: true }),
    )
}
