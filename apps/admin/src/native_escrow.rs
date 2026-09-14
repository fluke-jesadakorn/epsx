use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[cfg(test)]
use axum::{
    http::{HeaderMap, Uri},
    response::Html,
};
#[cfg(test)]
use dioxus::prelude::*;
#[cfg(test)]
use epsx_templates::page_shell_with_body_class;
#[cfg(test)]
#[component]
fn EscrowPage(id: String) -> Element {
    rsx! {main {class:"container-x py-10 max-w-4xl mx-auto space-y-6","data-native-pay-page":"true","data-pay-admin":"true","data-pay-intent":id.clone(),
        epsx_dioxus_ui::navigation::AppLink {href:"/",class:"underline","Admin home"}
        epsx_dioxus_ui::navigation::AppLink {href:"/pay/merchant-escrows",class:"underline","Merchant escrow disputes"}
        h1 {class:"text-2xl font-semibold","Pay escrow disputes"}
        p {id:"native-pay-status",role:"status","Only disputed escrows can be resolved. Sign the selected transaction with the configured Admin wallet."}
        div {class:"flex gap-3",
            button {class:"btn btn-outline","data-epsx-action":"native-pay-pause","data-pay-paused":"true","Pause new deposits"}
            button {class:"btn btn-outline","data-epsx-action":"native-pay-pause","data-pay-paused":"false","Resume deposits"}
        }
        if !id.is_empty(){section {class:"border rounded-2xl p-6 space-y-4",
            p {id:"native-pay-amount"}p {id:"native-pay-recipient",class:"break-all"}p {id:"native-pay-deal-status"}
            for (kind,label) in [("resolve-release","Release to recipient"),("resolve-refund","Refund payer")]{
                button {class:"btn btn-primary",hidden:true,"data-epsx-action":"native-pay-operation","data-pay-kind":kind,"data-pay-intent":id.clone(),"{label}"}
            }
            p {id:"native-pay-tx",class:"break-all"}
        }}
        section {class:"space-y-3",h2 {class:"text-xl font-semibold","Escrows"}div {id:"native-pay-history","Loading…"}}
    }}
}

#[cfg(test)]
#[component]
fn MerchantEscrowPage(id: String) -> Element {
    rsx! {main{class:"container-x max-w-5xl mx-auto py-10 space-y-6","data-merchant-pay-page":"true","data-merchant-admin":"true","data-payment-id":id.clone(),
        epsx_dioxus_ui::navigation::AppLink {href:"/pay/escrows",class:"underline","Legacy escrows"}
        h1{class:"text-2xl font-semibold","Merchant escrow disputes"}
        p{id:"merchant-status",role:"status","Resolve disputed payments using the configured Admin wallet."}
        select{id:"merchant-environment",class:"input",option{value:"test","Test"}option{value:"live","Live"}}
        button{class:"btn","data-epsx-action":"merchant-refresh","Refresh"}
        if !id.is_empty(){
            p{id:"merchant-payment-description"}p{id:"merchant-payment-amount"}p{id:"merchant-payment-recipient"}p{id:"merchant-payment-mode"}p{id:"merchant-payment-status"}p{id:"merchant-payment-terms"}
            for(kind,label)in[("resolve-release","Release to merchant"),("resolve-refund","Refund payer")]{button{class:"btn btn-primary",hidden:true,"data-epsx-action":"merchant-operation","data-merchant-operation":kind,"data-payment-id":id.clone(),"{label}"}}
            p{id:"merchant-transaction"}
        }
        section{class:"space-y-3",h2{class:"text-xl font-semibold","New payment controls"}
            p{"Pausing blocks new payments and deposits. Existing escrows can still be released or refunded."}
            for mode in ["direct","escrow"] { div{class:"flex gap-3 items-center",strong{"{mode}"}
                for (paused,label) in [("true","Pause"),("false","Resume")] {button{class:"btn btn-outline","data-epsx-action":"merchant-pause","data-mode":mode,"data-paused":paused,"{label}"}}
            }}
        }
        div{id:"merchant-payments","Loading merchant escrows…"}
    }}
}
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Retired route retained only as a migration fixture"
)]
pub async fn merchant_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if state.session().current_user(&headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let id = uri
        .path()
        .strip_prefix("/pay/merchant-escrows/")
        .unwrap_or_default()
        .to_string();
    let mut dom = VirtualDom::new_with_props(MerchantEscrowPage, MerchantEscrowPageProps { id });
    dom.rebuild_in_place();
    Html(page_shell_with_body_class(
        "Merchant escrow disputes",
        "EPSX Admin",
        "",
        &dioxus_ssr::render(&dom),
        false,
        "page-bg",
    ))
    .into_response()
}
pub async fn merchant_proxy(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Response {
    let (parts, body) = request.into_parts();
    if parts.method == axum::http::Method::POST
        && !crate::same_origin_admin_notification_form(&parts.headers)
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some((token, _)) = state.session().verified_access_token(&parts.headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(body) = axum::body::to_bytes(body, 64 * 1024).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let mut r = state
        .payment
        .auth_client()
        .request(
            parts.method,
            format!(
                "{}{}",
                state.api_url.trim_end_matches('/'),
                parts
                    .uri
                    .path_and_query()
                    .map(|p| p.as_str())
                    .unwrap_or(parts.uri.path())
            ),
        )
        .bearer_auth(token)
        .header("content-type", "application/json")
        .header("x-pay-api-version", "2026-09-08")
        .body(body);
    for h in ["x-pay-environment", "idempotency-key"] {
        if let Some(v) = parts.headers.get(h) {
            r = r.header(h, v)
        }
    }
    let Ok(response) = r.send().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    let status = response.status();
    let Ok(body) = response.bytes().await else {
        return StatusCode::BAD_GATEWAY.into_response();
    };
    (
        status,
        [
            ("content-type", "application/json"),
            ("cache-control", "no-store"),
        ],
        body,
    )
        .into_response()
}
#[cfg(test)]
#[expect(
    dead_code,
    reason = "Retired route retained only as a migration fixture"
)]
pub async fn page(State(state): State<AppState>, headers: HeaderMap, uri: Uri) -> Response {
    let Some(_) = state.session().current_user(&headers).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<a href=\"/auth\">Sign in to Admin</a>"),
        )
            .into_response();
    };
    let id = uri
        .path()
        .strip_prefix("/pay/escrows/")
        .unwrap_or_default()
        .to_string();
    let mut dom = VirtualDom::new_with_props(EscrowPage, EscrowPageProps { id });
    dom.rebuild_in_place();
    Html(page_shell_with_body_class(
        "Pay escrow disputes",
        "EPSX Admin",
        "",
        &dioxus_ssr::render(&dom),
        false,
        "page-bg",
    ))
    .into_response()
}
