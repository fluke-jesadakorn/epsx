//! Verified Pay BFF providers. No endpoint accepts arbitrary upstream paths.
use crate::{auth, AppState};
use axum::{http::HeaderMap, Extension, Router};
use epsx_dioxus_ui::fullstack::{pay::*, LoadError, Surface};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::Arc;

fn identifier(value: &str) -> Result<&str, LoadError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Err(LoadError::InvalidQuery)
    } else {
        Ok(value)
    }
}
fn validate_credentials(c: &Credentials) -> Result<(), LoadError> {
    if let Some(id) = &c.checkout {
        identifier(id)?;
    }
    for value in [&c.capability, &c.guest].into_iter().flatten() {
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(LoadError::InvalidQuery);
        }
    }
    Ok(())
}
async fn request<T: DeserializeOwned>(
    state: &AppState,
    auth_context: (&HeaderMap, &Credentials),
    path: &str,
    body: Option<Value>,
    key: Option<&str>,
    public: bool,
    merchant: bool,
) -> Result<T, LoadError> {
    let (headers, c) = auth_context;
    validate_credentials(c)?;
    let mut request = if body.is_some() {
        state
            .pay
            .auth_client()
            .post(format!("{}/api/v1/pay/{path}", state.pay_url))
    } else {
        state
            .pay
            .auth_client()
            .get(format!("{}/api/v1/pay/{path}", state.pay_url))
    };
    if !public {
        let (token, _) = state
            .session()
            .verified_access_token(headers)
            .await
            .ok_or(LoadError::Unauthenticated)?;
        request = request.bearer_auth(token);
    }
    if merchant {
        request = request
            .header("x-pay-api-version", "2026-09-08")
            .header("x-pay-environment", c.environment.as_str());
    }
    if let Some(token) = &c.capability {
        request = request.header("x-pay-checkout-token", token);
    }
    if let Some(guest) = &c.guest {
        request = request.header("x-pay-guest-id", guest);
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|_| LoadError::Unavailable)?;
    match response.status().as_u16() {
        200..=299 => {}
        401 => return Err(LoadError::Unauthenticated),
        403 => return Err(LoadError::Forbidden),
        400 | 409 | 422 => return Err(LoadError::InvalidQuery),
        404 => return Err(LoadError::NotFound),
        _ => return Err(LoadError::Unavailable),
    };
    response.json().await.map_err(|_| LoadError::Malformed)
}
#[derive(serde::Deserialize)]
struct Items<T> {
    items: Vec<T>,
}
async fn items<T: DeserializeOwned>(
    state: &AppState,
    auth_context: (&HeaderMap, &Credentials),
    path: &str,
) -> Result<Vec<T>, LoadError> {
    Ok(
        request::<Items<T>>(state, auth_context, path, None, None, false, true)
            .await?
            .items,
    )
}
async fn read(
    state: AppState,
    page: Page,
    c: Credentials,
    headers: HeaderMap,
) -> Result<PageData, LoadError> {
    validate_credentials(&c)?;
    let frontend_origin =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    // The first HTML response must not wait for either Pay or identity/JWKS.
    // Capability and optional authentication are read after client hydration.
    if matches!(&page, Page::Checkout(_)) && c.capability.is_none() {
        return Ok(PageData {
            frontend_origin,
            ..Default::default()
        });
    }
    let signed_in = state
        .session()
        .verified_access_token(&headers)
        .await
        .is_some();
    let mut data = PageData {
        signed_in,
        recover_session: !signed_in
            && epsx_bff::cookies::read_refresh_token(
                &headers,
                state.cookie_environment,
                epsx_bff::cookies::CookieClient::Pay,
            )
            .is_some(),
        frontend_origin,
        ..Default::default()
    };
    let get = |path: String, public| {
        let state = state.clone();
        let headers = headers.clone();
        let c = c.clone();
        async move { request::<Value>(&state, (&headers, &c), &path, None, None, public, true).await }
    };
    let native = matches!(
        &page,
        Page::NativeIntent(_) | Page::NativeLink(_) | Page::NativeDashboard
    );
    data.config = request(&state, (&headers, &c), "config", None, None, true, !native).await?;
    match &page {
        Page::Store(id) => {
            #[derive(serde::Deserialize)]
            struct Catalog {
                merchant: Merchant,
                items: Vec<Product>,
            }
            let catalog: Catalog = request(
                &state,
                (&headers, &c),
                &format!("catalog/{}", identifier(id)?),
                None,
                None,
                true,
                true,
            )
            .await?;
            data.merchant = Some(catalog.merchant);
            data.products = catalog.items;
        }
        Page::Product(id) => {
            data.products.push(
                request(
                    &state,
                    (&headers, &c),
                    &format!("catalog/products/{}", identifier(id)?),
                    None,
                    None,
                    true,
                    true,
                )
                .await?,
            );
        }
        Page::Checkout(id) => {
            data.payment = Some(
                request(
                    &state,
                    (&headers, &c),
                    &format!("checkout-sessions/{}", identifier(id)?),
                    None,
                    None,
                    true,
                    true,
                )
                .await?,
            );
            if let Some(token) = &c.capability {
                let result = state
                    .identity
                    .auth_client()
                    .get(format!(
                        "{}/api/payments/checkout-completion/{}",
                        state.api_url,
                        identifier(id)?
                    ))
                    .header("x-pay-checkout-token", token)
                    .timeout(std::time::Duration::from_secs(3))
                    .send()
                    .await;
                if let Ok(response) = result {
                    if response.status().is_success() {
                        #[derive(serde::Deserialize)]
                        struct CompletionReply {
                            completion: Option<CheckoutCompletion>,
                        }
                        if let Ok(reply) = response.json::<CompletionReply>().await {
                            if reply
                                .completion
                                .as_ref()
                                .is_none_or(|v| v.valid_return(&data.frontend_origin))
                            {
                                data.completion = reply.completion;
                                data.completion_available = true;
                            }
                        }
                    }
                }
            }
        }
        Page::Link(id) => {
            data.link = Some(
                request(
                    &state,
                    (&headers, &c),
                    &format!("links/{}", identifier(id)?),
                    None,
                    None,
                    true,
                    true,
                )
                .await?,
            );
        }
        Page::Payment(id) => {
            data.payment = Some(
                request(
                    &state,
                    (&headers, &c),
                    &format!("intents/{}", identifier(id)?),
                    None,
                    None,
                    false,
                    true,
                )
                .await?,
            );
        }
        Page::NativeIntent(id) => {
            data.payment = Some(
                request(
                    &state,
                    (&headers, &c),
                    &format!("intents/{}", identifier(id)?),
                    None,
                    None,
                    false,
                    false,
                )
                .await?,
            );
        }
        Page::NativeLink(id) => {
            #[derive(serde::Deserialize)]
            struct LegacyLink {
                link: PaymentLink,
            }
            data.link = Some(
                request::<LegacyLink>(
                    &state,
                    (&headers, &c),
                    &format!("links/{}", identifier(id)?),
                    None,
                    None,
                    true,
                    false,
                )
                .await?
                .link,
            );
        }
        Page::NativeDashboard => {
            if signed_in {
                data.payments = request::<Items<Payment>>(
                    &state,
                    (&headers, &c),
                    "intents",
                    None,
                    None,
                    false,
                    false,
                )
                .await?
                .items;
            }
        }
        _ => {
            if !signed_in {
                return Ok(data);
            }
            match request(
                &state,
                (&headers, &c),
                "merchants/me",
                None,
                None,
                false,
                true,
            )
            .await
            {
                Ok(merchant) => data.merchant = Some(merchant),
                Err(LoadError::NotFound) => return Ok(data),
                Err(e) => return Err(e),
            }
            match &page {
                Page::Dashboard => {
                    data.overview = items(&state, (&headers, &c), "overview").await?;
                    data.payments = items(&state, (&headers, &c), "intents").await?;
                }
                Page::Payments => data.payments = items(&state, (&headers, &c), "intents").await?,
                Page::Packages | Page::EditPackage(_) => {
                    data.products = items(&state, (&headers, &c), "products").await?;
                    if let Page::EditPackage(id) = &page {
                        let p = serde_json::from_value(
                            get(format!("products/{}", identifier(id)?), false).await?,
                        )
                        .map_err(|_| LoadError::Malformed)?;
                        data.products.retain(|p: &Product| p.id != *id);
                        data.products.insert(0, p);
                    }
                }
                Page::Links => data.links = items(&state, (&headers, &c), "links").await?,
                Page::Settings => data.keys = items(&state, (&headers, &c), "api-keys").await?,
                Page::Webhooks => {
                    data.webhooks = items(&state, (&headers, &c), "webhook-endpoints").await?;
                    data.deliveries = items(&state, (&headers, &c), "deliveries").await?;
                }
                _ => {}
            }
        }
    }
    Ok(data)
}
fn link_units(amount: &str, decimals: u32) -> Result<String, LoadError> {
    if decimals > 36 || amount.len() > 80 {
        return Err(LoadError::InvalidQuery);
    }
    let mut parts = amount.trim().split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > decimals as usize
    {
        return Err(LoadError::InvalidQuery);
    }
    Ok(format!(
        "{whole}{fraction}{}",
        "0".repeat(decimals as usize - fraction.len())
    )
    .trim_start_matches('0')
    .to_string())
}
async fn action(
    state: AppState,
    action: Action,
    c: Credentials,
    key: String,
    headers: HeaderMap,
) -> Result<ActionResult, LoadError> {
    if !auth::same_origin(&state, &headers) {
        return Err(LoadError::Forbidden);
    }
    validate_credentials(&c)?;
    if key.len() != 64 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(LoadError::InvalidQuery);
    }
    let mut public = false;
    let mut read = false;
    let mut merchant = true;
    let (path, body) = match action {
        Action::Auth(command) => {
            return crate::fullstack_auth::command(state, command, headers).await
        }
        Action::NativeCreateLink(input) => {
            merchant = false;
            let config: Config =
                request(&state, (&headers, &c), "config", None, None, true, false).await?;
            let amount = link_units(
                &input.amount,
                config
                    .decimals(c.environment, &input.token)
                    .ok_or(LoadError::InvalidQuery)?,
            )?;
            (
                "links".into(),
                json!({"amount":amount,"token":input.token,"description":input.description,"max_uses":input.max_uses,"expires_in":input.expires_in}),
            )
        }
        Action::NativeRedeem { id } => {
            merchant = false;
            (format!("links/{}/redeem", identifier(&id)?), json!({}))
        }
        Action::NativePrepare { id, kind } => {
            merchant = false;
            if !matches!(
                kind,
                OperationKind::Deposit
                    | OperationKind::Release
                    | OperationKind::Refund
                    | OperationKind::Dispute
            ) {
                return Err(LoadError::InvalidQuery);
            };
            (
                format!(
                    "{}/{}/{}",
                    if kind == OperationKind::Deposit {
                        "intents"
                    } else {
                        "escrows"
                    },
                    identifier(&id)?,
                    kind.as_str()
                ),
                json!({}),
            )
        }
        Action::NativeConfirm { id, tx_hash } => {
            merchant = false;
            (
                format!("operations/{}/confirm", identifier(&id)?),
                json!({"tx_hash":tx_hash}),
            )
        }
        Action::NativeRead { id } => {
            merchant = false;
            read = true;
            (format!("operations/{}", identifier(&id)?), json!({}))
        }
        Action::Register { name } => ("merchants".into(), json!({"name":name})),
        Action::Profile { name } => ("merchants/me/update".into(), json!({"name":name})),
        Action::SaveProduct { id, input } => (
            match id {
                Some(id) => format!("products/{}/update", identifier(&id)?),
                None => "products".into(),
            },
            json!(input),
        ),
        Action::CreateLink(input) => {
            let config: Config =
                request(&state, (&headers, &c), "config", None, None, true, true).await?;
            let amount = link_units(
                &input.amount,
                config
                    .decimals(c.environment, &input.token)
                    .ok_or(LoadError::InvalidQuery)?,
            )?;
            (
                "links".into(),
                json!({"payment_method":if input.mode=="direct"&&input.token!="BNB"{"transfer"}else{"contract"},"mode":input.mode,"token":input.token,"amount":amount,"description":input.description,"max_uses":input.max_uses,"expires_in":input.expires_in}),
            )
        }
        Action::CreateKey { name } => ("api-keys".into(), json!({"name":name})),
        Action::RevokeKey { id } => (format!("api-keys/{}/revoke", identifier(&id)?), json!({})),
        Action::CreateWebhook { url } => ("webhook-endpoints".into(), json!({"url":url})),
        Action::DisableWebhook { id } => (
            format!("webhook-endpoints/{}/disable", identifier(&id)?),
            json!({}),
        ),
        Action::EnableWebhook { id } => (
            format!("webhook-endpoints/{}/enable", identifier(&id)?),
            json!({}),
        ),
        Action::RotateWebhook { id } => (
            format!("webhook-endpoints/{}/rotate", identifier(&id)?),
            json!({}),
        ),
        Action::ReplaceWebhook { id, url } => (
            format!("webhook-endpoints/{}/replace", identifier(&id)?),
            json!({"url":url}),
        ),
        Action::ReplayDelivery { id } => {
            (format!("deliveries/{}/replay", identifier(&id)?), json!({}))
        }
        Action::DeliveryDetails { id } => {
            read = true;
            (format!("deliveries/{}", identifier(&id)?), json!({}))
        }
        Action::DisableLink { id } => (format!("links/{}/disable", identifier(&id)?), json!({})),
        Action::BuyProduct { id, token } => {
            public = true;
            (
                format!("products/{}/checkouts", identifier(&id)?),
                json!({"token":token}),
            )
        }
        Action::RedeemLink { id } => {
            public = true;
            (format!("links/{}/checkouts", identifier(&id)?), json!({}))
        }
        Action::PrepareTransfer { id, payer } => {
            public = true;
            if c.checkout.as_deref() != Some(&id) {
                return Err(LoadError::InvalidQuery);
            };
            (
                format!("checkout-sessions/{}/prepare-transfer", identifier(&id)?),
                json!({"payer":payer}),
            )
        }
        Action::PrepareOperation {
            id,
            checkout,
            kind,
            payer,
        } => {
            public = checkout;
            if checkout && c.checkout.as_deref() != Some(&id) {
                return Err(LoadError::InvalidQuery);
            };
            (
                format!(
                    "{}/{}/{}",
                    if checkout {
                        "checkout-sessions"
                    } else {
                        "intents"
                    },
                    identifier(&id)?,
                    kind.as_str()
                ),
                json!({"payer":payer}),
            )
        }
        Action::ConfirmOperation { id, tx_hash } => {
            public = c.checkout.is_some();
            (
                format!("operations/{}/confirm", identifier(&id)?),
                json!({"tx_hash":tx_hash}),
            )
        }
        Action::ReadOperation { id } => {
            public = c.checkout.is_some();
            read = true;
            (format!("operations/{}", identifier(&id)?), json!({}))
        }
    };
    let mut result: ActionResult = request(
        &state,
        (&headers, &c),
        &path,
        (!read).then_some(body),
        Some(&key),
        public,
        merchant,
    )
    .await?;
    if let Some(op) = &result.operation {
        result.id = op.id.clone();
    }
    if let Some(intent) = &result.intent {
        identifier(&intent.id)?;
        result.pay_url = Some(format!("/checkout/{}", intent.id));
    }
    Ok(result)
}
pub fn application(state: AppState) -> Router {
    validate_assets().expect("Pay hydration assets are missing. Build epsx-pay with dx and package its public directory; set DIOXUS_PUBLIC_PATH to that directory.");
    application_with_config(state, dioxus_server::ServeConfig::new())
}
pub(crate) fn validate_assets() -> Result<(), String> {
    let public = std::env::var("DIOXUS_PUBLIC_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .expect("executable path")
                .parent()
                .expect("executable directory")
                .join("public")
        });
    let index = std::fs::read_to_string(public.join("index.html")).map_err(|e| e.to_string())?;
    if !index.contains("<script")
        || !index.contains("id=\"main\"")
        || index.contains("epsx-browser-runtime")
    {
        return Err("Expected Dioxus hydration index, without legacy runtime".into());
    }
    fn has_wasm(path: &std::path::Path) -> bool {
        std::fs::read_dir(path)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .any(|entry| {
                let p = entry.path();
                p.extension().is_some_and(|e| e == "wasm") || p.is_dir() && has_wasm(&p)
            })
    }
    if !has_wasm(&public) {
        return Err("Missing Pay WebAssembly asset".into());
    }
    Ok(())
}
fn application_with_config(state: AppState, config: dioxus_server::ServeConfig) -> Router {
    let read_state = state.clone();
    let action_state = state;
    let provider = PayProvider {
        read: Arc::new(move |page, c, h| Box::pin(read(read_state.clone(), page, c, h))),
        action: Arc::new(move |a, c, k, h| Box::pin(action(action_state.clone(), a, c, k, h))),
    };
    let fullstack = dioxus_server::FullstackState::new(config, PayApp);
    let functions = epsx_bff::fullstack::server_functions(Surface::Pay, fullstack.clone());
    Router::new()
        .fallback(dioxus_server::FullstackState::render_handler)
        .with_state(fullstack)
        .merge(functions)
        .layer(Extension(provider))
        .layer(axum::middleware::from_fn(page_boundary))
}

async fn page_boundary(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let path = request.uri().path();
    if (path.starts_with("/api/") || path.starts_with("/_server/"))
        && !Surface::Pay.owns_server_path(path)
    {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    }
    let known = known_page_path(path) || Surface::Pay.owns_server_path(path);
    let mut response = next.run(request).await;
    if !known {
        *response.status_mut() = axum::http::StatusCode::NOT_FOUND;
    }
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{http::StatusCode, response::IntoResponse};
    fn fixture_state(url: String) -> AppState {
        let client = Arc::new(epsx_client::ServiceClient::new(epsx_client::ClientConfig {
            base_url: url.clone(),
            timeout: std::time::Duration::from_secs(2),
        }));
        let verifier = epsx_bff::session::JwksVerifier::with_http(
            epsx_bff::session::JwksVerifierConfig::new(
                format!("{url}/jwks"),
                "https://issuer.test",
                epsx_bff::session::PAY_CLIENT_ID,
                std::time::Duration::from_secs(60),
            )
            .unwrap(),
        )
        .unwrap();
        AppState {
            pay: client.clone(),
            identity: client,
            api_url: url.clone(),
            pay_url: url,
            public_origin: "http://pay.test".into(),
            cookie_environment: epsx_bff::cookies::CookieEnvironment::Local,
            verifier: Arc::new(verifier),
        }
    }
    #[tokio::test]
    async fn mutation_rejects_cross_origin_and_forged_sessions_before_upstream() {
        let state = fixture_state("http://127.0.0.1:9".into());
        let a = Action::Register {
            name: "Example".into(),
        };
        let key = "a".repeat(64);
        assert_eq!(
            action(
                state.clone(),
                a.clone(),
                Credentials::default(),
                key.clone(),
                HeaderMap::new()
            )
            .await,
            Err(LoadError::Forbidden)
        );
        let mut headers = HeaderMap::new();
        headers.insert("origin", "http://pay.test".parse().unwrap());
        headers.insert("cookie", "epsx-pay-access-token=forged".parse().unwrap());
        assert_eq!(
            action(state, a, Credentials::default(), key, headers).await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn typed_checkout_reads_capability_and_preserves_service_status() {
        let app=Router::new().route("/api/v1/pay/config",axum::routing::get(||async{axum::Json(json!({"environments":[{"environment":"live","tokens":{"USDT":{"decimals":6}}}]}))}))
            .route("/api/v1/pay/checkout-sessions/cs_fixture",axum::routing::get(|headers:HeaderMap|async move{
                assert_eq!(headers.get("x-pay-api-version").unwrap(),"2026-09-08");assert_eq!(headers.get("x-pay-checkout-token").unwrap(),&"b".repeat(64));
                axum::Json(json!({"id":"pi_fixture","checkout_id":"cs_fixture","environment":"live","amount":"5125000","token":"USDT","description":null,"token_decimals":6,"chain_id":56,"status":"awaiting_payment","available_actions":[],"tx_hash":null,"payment_method":"transfer"}))
            }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let result = read(
            fixture_state(format!("http://{addr}")),
            Page::Checkout("cs_fixture".into()),
            Credentials {
                checkout: Some("cs_fixture".into()),
                capability: Some("b".repeat(64)),
                ..Default::default()
            },
            HeaderMap::new(),
        )
        .await
        .unwrap();
        let payment = result.payment.unwrap();
        assert_eq!(payment.status, "awaiting_payment");
        assert_eq!(payment.environment, Environment::Live);
        assert_eq!(
            display_amount(&payment.amount, payment.token_decimals.unwrap()),
            "5.125"
        );
        task.abort();
    }
    #[tokio::test]
    async fn completion_is_scoped_and_failures_preserve_payment() {
        let origin =
            std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let order = uuid::Uuid::nil();
        let reply = Arc::new(std::sync::Mutex::new(
            json!({"completion":{"order_id":order,"payment_status":"succeeded","fulfillment_status":"pending","return_url":format!("{}/account/payments/{order}",origin.trim_end_matches('/'))}}),
        ));
        let upstream = reply.clone();
        let app = Router::new()
            .route(
                "/api/v1/pay/config",
                axum::routing::get(|| async { axum::Json(json!({})) }),
            )
            .route(
                "/api/v1/pay/checkout-sessions/cs_completion",
                axum::routing::get(|| async {
                    axum::Json(json!({"checkout_id":"cs_completion","status":"succeeded"}))
                }),
            )
            .route(
                "/api/payments/checkout-completion/cs_completion",
                axum::routing::get(move |headers: HeaderMap| {
                    let reply = upstream.clone();
                    async move {
                        assert_eq!(
                            headers.get("x-pay-checkout-token").unwrap(),
                            &"b".repeat(64)
                        );
                        assert!(headers.get("cookie").is_none());
                        let value = reply.lock().unwrap().clone();
                        if value.is_null() {
                            return (StatusCode::SERVICE_UNAVAILABLE, axum::Json(value))
                                .into_response();
                        }
                        axum::Json(value).into_response()
                    }
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let state = fixture_state(format!("http://{}", listener.local_addr().unwrap()));
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let credentials = Credentials {
            checkout: Some("cs_completion".into()),
            capability: Some("b".repeat(64)),
            ..Default::default()
        };
        let read_fixture = || {
            read(
                state.clone(),
                Page::Checkout("cs_completion".into()),
                credentials.clone(),
                HeaderMap::new(),
            )
        };
        let data = read_fixture().await.unwrap();
        assert!(data.completion_available);
        assert!(!data.completion.as_ref().unwrap().ready());
        reply.lock().unwrap()["completion"]["fulfillment_status"] = json!("granted");
        assert!(read_fixture().await.unwrap().completion.unwrap().ready());
        reply.lock().unwrap()["completion"]["return_url"] = json!(
            "https://untrusted.invalid/account/payments/00000000-0000-0000-0000-000000000000"
        );
        let rejected = read_fixture().await.unwrap();
        assert!(!rejected.completion_available && rejected.completion.is_none());
        *reply.lock().unwrap() = json!({"completion":null});
        let merchant = read_fixture().await.unwrap();
        assert!(merchant.completion_available && merchant.completion.is_none());
        *reply.lock().unwrap() = Value::Null;
        let outage = read_fixture().await.unwrap();
        assert!(!outage.completion_available);
        assert_eq!(outage.payment.unwrap().status, "succeeded");
        task.abort();
    }
    #[test]
    fn registration_is_exactly_scoped_to_pay() {
        let names: Vec<_> = dioxus_server::ServerFunction::collect()
            .into_iter()
            .filter(|f| Surface::Pay.owns_server_path(f.path()))
            .map(|f| f.path().to_string())
            .collect();
        assert!(names.contains(&"/_server/pay/read".into()));
        assert!(names.contains(&"/_server/pay/action".into()));
        assert_eq!(names.len(), 2);
    }
    #[tokio::test]
    async fn fullstack_ssr_renders_public_home_and_docs_without_payment_upstream() {
        let app = application_with_config(
            fixture_state("http://127.0.0.1:9".into()),
            dioxus_server::ServeConfig::new(),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::new();
        for (query, environment) in [("", "test"), ("?environment=live", "live")] {
            let response = client
                .get(format!("http://{addr}/{query}"))
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), 200);
            assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
            let html = response.text().await.unwrap();
            assert!(html.contains("Crypto payments,"));
            assert!(html.contains("Start accepting payments"));
            assert!(html.contains("CHECKOUT PREVIEW"));
            assert!(html.contains(&format!("/dashboard?environment={environment}")));
            assert!(html.contains("name=\"description\""));
            assert!(!html.contains("YOUR MERCHANT WORKSPACE"));
            assert!(!html.contains("Pay is unavailable"));
        }
        let checkout = client
            .get(format!("http://{addr}/checkout/cs_fixture"))
            .send()
            .await
            .unwrap();
        assert_eq!(checkout.status(), 200);
        let html = checkout.text().await.unwrap();
        assert!(html.contains("Preparing your checkout"));
        assert!(!html.contains("The requested page was not found"));
        assert!(!html.contains("Workspace"));
        // The dashboard still reads Pay; even its unavailable shell must show
        // the requested environment correctly before client hydration.
        let response = client
            .get(format!("http://{addr}/dashboard?environment=live"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 502);
        let html = response.text().await.unwrap();
        assert!(html.contains("YOUR MERCHANT WORKSPACE"));
        let live_option = html
            .split("<option")
            .filter_map(|part| part.split('>').next())
            .find(|tag| tag.contains("value=\"live\""))
            .expect("Live environment option");
        assert!(live_option.contains("selected"));
        let response = client
            .get(format!("http://{addr}/docs"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers().get("cache-control").unwrap(), "no-store");
        let html = response.text().await.unwrap();
        assert!(html.contains("Back to Pay"));
        assert!(!html.contains("epsx-browser-runtime"));
        assert_eq!(
            client
                .get(format!("http://{addr}/not-a-pay-page"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        assert_eq!(
            client
                .get(format!("http://{addr}/_server/admin/users"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        task.abort();
    }

    #[tokio::test]
    async fn typed_auth_challenge_uses_existing_pay_identity_contract() {
        let app=Router::new().route("/api/auth/web3/challenge",axum::routing::post(|axum::Json(body):axum::Json<serde_json::Value>|async move{assert_eq!(body["client_id"],"epsx-pay");axum::Json(json!({"success":true,"wallet_address":body["wallet_address"],"message":"Fixture challenge","nonce":"fixture","expires_at":2000000000}))}));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let mut headers = HeaderMap::new();
        headers.insert("origin", "http://pay.test".parse().unwrap());
        let result = action(
            fixture_state(format!("http://{addr}")),
            Action::Auth(PayAuthCommand::Challenge {
                address: format!("0x{}", "1".repeat(40)),
            }),
            Credentials::default(),
            "a".repeat(64),
            headers,
        )
        .await
        .unwrap();
        assert_eq!(result.challenge.unwrap().nonce, "fixture");
        task.abort();
    }

    #[test]
    fn typed_ids_cannot_change_upstream_paths() {
        for s in ["../merchants", "a/b", "a?x=y", "a%2fb", "", "a#b"] {
            assert!(identifier(s).is_err());
        }
        assert!(identifier("cs_abc123").is_ok());
    }
    #[test]
    fn amount_conversion_preserves_precision() {
        assert_eq!(link_units("5.125", 6).unwrap(), "5125000");
        assert!(link_units("5.0000001", 6).is_err());
        assert!(link_units("-1", 18).is_err());
    }
    #[test]
    fn credentials_reject_invalid_capabilities() {
        assert!(validate_credentials(&Credentials {
            capability: Some("abc".into()),
            ..Default::default()
        })
        .is_err());
    }
}
