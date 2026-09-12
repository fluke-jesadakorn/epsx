//! Progressive enhancement for merchant management and account-free checkout.
use super::*;
#[path = "checkout_wallet.rs"]
mod checkout_wallet;
#[path = "merchant_catalog.rs"]
mod merchant_catalog;

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.session_storage().ok().flatten())
}
fn random() -> Result<String, JsValue> {
    let w = web_sys::window().ok_or_else(|| JsValue::from_str("Browser unavailable"))?;
    let crypto = Reflect::get(&w, &JsValue::from_str("crypto"))?;
    let function =
        Reflect::get(&crypto, &JsValue::from_str("getRandomValues"))?.dyn_into::<Function>()?;
    let array = Uint8Array::new_with_length(32);
    function.call1(&crypto, &array)?;
    Ok(array.to_vec().iter().map(|b| format!("{b:02x}")).collect())
}
fn stored(key: &str) -> Result<String, JsValue> {
    let s = storage()
        .ok_or_else(|| JsValue::from_str("Session storage is required for checkout recovery"))?;
    if let Some(v) = s.get_item(key)? {
        return Ok(v);
    }
    let v = random()?;
    s.set_item(key, &v)?;
    Ok(v)
}
fn environment() -> String {
    let v = native_pay_input("merchant-environment");
    if !v.is_empty() {
        if let Some(s) = storage() {
            let _ = s.set_item("epsx.merchant.environment", &v);
        }
        v
    } else {
        storage()
            .and_then(|s| s.get_item("epsx.merchant.environment").ok().flatten())
            .unwrap_or_else(|| "test".into())
    }
}
fn checkout_token(cs: &str) -> String {
    storage()
        .and_then(|s| s.get_item(&format!("epsx.checkout.{cs}")).ok().flatten())
        .unwrap_or_default()
}
thread_local! {
    static REFRESHING: Cell<bool> = const { Cell::new(false) };
    static REFRESH_EPOCH: Cell<u64> = const { Cell::new(0) };
}
async fn call(
    path: &str,
    method: &str,
    body: Option<Value>,
    key: Option<&str>,
    cs: Option<&str>,
) -> Result<Value, JsValue> {
    let env = environment();
    let cap = cs.map(checkout_token).unwrap_or_default();
    let guest = stored("epsx.pay.guest")?;
    let mut headers = vec![
        ("x-pay-api-version", "2026-09-08"),
        ("x-pay-environment", env.as_str()),
        ("x-pay-guest-id", guest.as_str()),
    ];
    if !cap.is_empty() {
        headers.push(("x-pay-checkout-token", &cap))
    }
    if let Some(key) = key {
        headers.push(("idempotency-key", key))
    }
    let epoch = REFRESH_EPOCH.with(Cell::get);
    for attempt in 0..2 {
        let window = web_sys::window().ok_or("window unavailable")?;
        let options = RequestInit::new();
        options.set_method(method);
        options.set_credentials(web_sys::RequestCredentials::SameOrigin);
        if let Some(ref body) = body {
            options.set_body(&JsValue::from_str(&body.to_string()));
        }
        let request = Request::new_with_str_and_init(&format!("/api/v1/pay/{path}"), &options)?;
        request.headers().set("content-type", "application/json")?;
        for (name, value) in &headers {
            request.headers().set(name, value)?;
        }
        let response = JsFuture::from(window.fetch_with_request(&request))
            .await?
            .dyn_into::<Response>()?;
        if response.status() == 401 && cs.is_none() && attempt == 0 {
            if REFRESHING.with(Cell::get) {
                for _ in 0..200 {
                    if !REFRESHING.with(Cell::get) {
                        break;
                    }
                    delay(100).await;
                }
            } else if REFRESH_EPOCH.with(Cell::get) == epoch {
                REFRESHING.with(|v| v.set(true));
                let refreshed = fetch_json("/api/v1/auth/refresh", "POST", Some(json!({}))).await;
                REFRESHING.with(|v| v.set(false));
                if refreshed.is_err() {
                    return Err(JsValue::from_str(
                        "Sign in to manage your merchant account.",
                    ));
                }
                REFRESH_EPOCH.with(|v| v.set(v.get() + 1));
            }
            continue;
        }
        let value: Value = serde_wasm_bindgen::from_value(JsFuture::from(response.json()?).await?)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        if !response.ok() {
            return Err(JsValue::from_str(
                value["error"].as_str().unwrap_or("Pay request unavailable"),
            ));
        }
        return Ok(value);
    }
    Err(JsValue::from_str(
        "Sign in to manage your merchant account.",
    ))
}
fn status(message: &str) {
    native_pay_text("merchant-status", message)
}
fn container(id: &str) -> Option<(Document, Element)> {
    let (_, d) = window_document()?;
    let e = d.get_element_by_id(id)?;
    e.set_text_content(None);
    Some((d, e))
}
fn button(
    d: &Document,
    row: &Element,
    label: &str,
    action: &str,
    target: &str,
) -> Result<(), JsValue> {
    let b = d.create_element("button")?;
    b.set_attribute("class", "btn text-sm")?;
    b.set_attribute("data-epsx-action", action)?;
    b.set_attribute("data-target", target)?;
    b.set_text_content(Some(label));
    row.append_child(&b)?;
    Ok(())
}
fn line(d: &Document, container: &Element, label: &str) -> Result<Element, JsValue> {
    let row = d.create_element("div")?;
    row.set_attribute("class", "flex flex-wrap items-center gap-3 py-3 border-b")?;
    let text = d.create_element("span")?;
    text.set_attribute("class", "break-all")?;
    text.set_text_content(Some(label));
    row.append_child(&text)?;
    container.append_child(&row)?;
    Ok(row)
}
fn string<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
async fn dashboard() -> Result<(), JsValue> {
    if merchant_catalog::load().await? {
        return Ok(());
    }
    if window_document().is_some_and(|(_, d)| {
        d.query_selector("[data-merchant-admin]")
            .ok()
            .flatten()
            .is_some()
    }) {
        let value = call("escrows", "GET", None, None, None).await?;
        if let Some((d, root)) = container("merchant-payments") {
            if let Some(items) = value["items"].as_array() {
                for v in items {
                    let row = line(
                        &d,
                        &root,
                        &format!(
                            "{} · {} · {}",
                            string(v, "id"),
                            string(v, "token"),
                            string(v, "status")
                        ),
                    )?;
                    let a = d.create_element("a")?;
                    a.set_attribute(
                        "href",
                        &format!("/pay/merchant-escrows/{}", string(v, "id")),
                    )?;
                    a.set_text_content(Some("Inspect / resolve"));
                    row.append_child(&a)?;
                }
            }
        }
        return Ok(());
    }
    let config = call("config", "GET", None, None, None).await?;
    let env = environment();
    call("merchants/me", "GET", None, None, None).await?;
    status(if environment() == "test" {
        "Test environment · simulated funds"
    } else {
        "Live environment"
    });
    for (name, target) in [
        ("links", "merchant-links"),
        ("intents", "merchant-payments"),
        ("api-keys", "merchant-keys"),
        ("webhook-endpoints", "merchant-endpoints"),
        ("deliveries", "merchant-deliveries"),
    ] {
        if window_document().is_none_or(|(_, d)| d.get_element_by_id(target).is_none()) {
            continue;
        }
        let data = call(name, "GET", None, None, None).await?;
        let Some((d, root)) = container(target) else {
            continue;
        };
        if let Some(items) = data["items"].as_array() {
            if items.is_empty() {
                root.set_text_content(Some("No records yet."));
            }
            for v in items {
                let id = string(v, "id");
                match name {
                    "links" => {
                        let row = line(
                            &d,
                            &root,
                            &format!(
                                "{} · {} {} · {}",
                                string(v, "description"),
                                native_pay_display(
                                    string(v, "amount"),
                                    config["environments"]
                                        .as_array()
                                        .and_then(|ns| ns.iter().find(|n| n["environment"] == env))
                                        .and_then(|n| n["tokens"][string(v, "token")]["decimals"]
                                            .as_u64())
                                        .unwrap_or(0) as usize
                                ),
                                string(v, "token"),
                                string(v, "mode")
                            ),
                        )?;
                        let a = d.create_element("a")?;
                        a.set_attribute("href", &format!("/r/{id}"))?;
                        a.set_attribute("class", "underline")?;
                        a.set_text_content(Some("Open link"));
                        row.append_child(&a)?;
                        if v["disabled"] != true {
                            button(&d, &row, "Disable", "merchant-disable-link", id)?
                        }
                    }
                    "intents" => {
                        if window_document().is_some_and(|(_, d)| {
                            d.query_selector("[data-merchant-dashboard]")
                                .ok()
                                .flatten()
                                .is_some()
                        }) {
                            merchant_catalog::payment_row(&d, &root, v)?;
                            continue;
                        }
                        let amount = native_pay_display(
                            string(v, "amount"),
                            v["token_decimals"].as_u64().unwrap_or(0) as usize,
                        );
                        let fee = native_pay_display(
                            string(v, "fee_amount"),
                            v["token_decimals"].as_u64().unwrap_or(0) as usize,
                        );
                        let row = line(
                            &d,
                            &root,
                            &format!(
                                "{amount} {} · {} · fee {fee} · {} · {} · collection: {}",
                                string(v, "token"),
                                string(v, "status"),
                                string(v, "order_reference"),
                                string(&v["checkout_snapshot"], "item_name"),
                                string(v, "settlement_status")
                            ),
                        )?;
                        let a = d.create_element("a")?;
                        a.set_attribute("href", &format!("/payments/{id}"))?;
                        a.set_attribute("class", "underline")?;
                        a.set_text_content(Some("Details / collect / refund"));
                        row.append_child(&a)?;
                    }
                    "api-keys" => {
                        let row = line(
                            &d,
                            &root,
                            &format!(
                                "{} · {} · {}",
                                string(v, "name"),
                                string(v, "prefix"),
                                if v["revoked"] == true {
                                    "revoked"
                                } else {
                                    "active"
                                }
                            ),
                        )?;
                        if v["revoked"] != true {
                            button(&d, &row, "Revoke", "merchant-revoke-key", id)?
                        }
                    }
                    "webhook-endpoints" => {
                        let row = line(
                            &d,
                            &root,
                            &format!(
                                "{} · {}",
                                string(v, "url"),
                                if v["enabled"] == true {
                                    "enabled"
                                } else {
                                    "disabled"
                                }
                            ),
                        )?;
                        if v["enabled"] == true {
                            button(&d, &row, "Rotate secret", "merchant-rotate-webhook", id)?;
                            button(&d, &row, "Disable", "merchant-disable-webhook", id)?;
                        } else {
                            button(&d, &row, "Enable", "merchant-catalog-enable-webhook", id)?;
                        }
                        button(
                            &d,
                            &row,
                            "Replace with URL above",
                            "merchant-catalog-replace-webhook",
                            id,
                        )?;
                    }
                    "deliveries" => {
                        let row = line(
                            &d,
                            &root,
                            &format!(
                                "{} · {} · attempts {} · HTTP {}",
                                string(v, "event_id"),
                                string(v, "status"),
                                v["attempts"],
                                v["last_status"]
                            ),
                        )?;
                        button(&d, &row, "Replay", "merchant-replay", id)?;
                        button(&d, &row, "Attempts", "merchant-delivery-details", id)?;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
async fn detail(node: &Element) -> Result<(), JsValue> {
    let cs = node.get_attribute("data-checkout-id").unwrap_or_default();
    let pi = node.get_attribute("data-payment-id").unwrap_or_default();
    let link = node.get_attribute("data-link-id").unwrap_or_default();
    let v = if !cs.is_empty() {
        call(
            &format!("checkout-sessions/{cs}"),
            "GET",
            None,
            None,
            Some(&cs),
        )
        .await?
    } else if !pi.is_empty() {
        call(&format!("intents/{pi}"), "GET", None, None, None).await?
    } else {
        call(&format!("links/{link}"), "GET", None, None, None).await?
    };
    if !cs.is_empty()
        && ["funded", "disputed", "succeeded", "refunded", "expired"]
            .contains(&string(&v, "status"))
    {
        if let Some(s) = storage() {
            if let Some(source) = s.get_item(&format!("epsx.checkout.source.{cs}"))? {
                s.remove_item(&source)?;
            }
        }
    }
    let env = string(&v, "environment");
    if !env.is_empty() {
        if let Some(s) = storage() {
            s.set_item("epsx.merchant.environment", env)?;
        }
    }
    let decimals = if let Some(n) = v["token_decimals"].as_u64() {
        n
    } else {
        let config = call("config", "GET", None, None, None).await?;
        config["environments"]
            .as_array()
            .and_then(|ns| ns.iter().find(|n| n["environment"] == v["environment"]))
            .and_then(|n| n["tokens"][string(&v, "token")]["decimals"].as_u64())
            .unwrap_or(0)
    };
    native_pay_text("merchant-payment-description", string(&v, "description"));
    native_pay_text(
        "merchant-payment-amount",
        &format!(
            "{} {}",
            native_pay_display(string(&v, "amount"), decimals as usize),
            string(&v, "token")
        ),
    );
    let pricing = &v["checkout_snapshot"]["pricing"];
    if let Some(panel) = window_document().and_then(|(_, d)| d.get_element_by_id("pay-sale")) {
        if pricing["promotion_active"] == true {
            panel.remove_attribute("hidden")?;
            native_pay_text(
                "pay-sale-original",
                &format!(
                    "{} {}",
                    string(pricing, "original_price"),
                    string(&v, "token")
                ),
            );
            native_pay_text(
                "pay-sale-savings",
                &format!(
                    "Save {} {}",
                    string(pricing, "savings"),
                    string(&v, "token")
                ),
            );
            native_pay_text(
                "pay-sale-label",
                &format!(
                    "Sale · {:.0}% off",
                    pricing["promotion_discount"].as_f64().unwrap_or_default()
                ),
            );
        } else {
            panel.set_attribute("hidden", "")?;
        }
    }
    native_pay_text("merchant-payment-recipient", string(&v, "payee"));
    native_pay_text(
        "merchant-payment-mode",
        &format!(
            "{} · collection: {}",
            string(&v, "mode"),
            string(&v, "settlement_status")
        ),
    );
    native_pay_text(
        "merchant-payment-status",
        if !link.is_empty() {
            "Ready for checkout"
        } else {
            string(&v, "status")
        },
    );
    native_pay_text(
        "merchant-payment-terms",
        if v["payment_method"] == "transfer" {
            "Funds arrive at this checkout's dedicated receiver. Settle to your merchant wallet when ready; a 0.5% fee is deducted on collection. Refunds are funded by your merchant wallet."
        } else if v["mode"] == "escrow" {
            "Funds are held until release or refund. A 1% fee is charged only on release. Network gas is separate."
        } else {
            "Funds go to the merchant in the payment transaction. The merchant pays a 0.5% fee. Network gas is separate."
        },
    );
    render_transfer(&v, decimals)?;
    checkout_wallet::render(&v)?;
    checkout_wallet::refresh(&v).await;
    let (_, d) = window_document().ok_or_else(|| JsValue::from_str("Browser unavailable"))?;
    let buttons = d.query_selector_all("[data-merchant-operation]")?;
    for i in 0..buttons.length() {
        if let Some(e) = buttons.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
            let kind = e
                .get_attribute("data-merchant-operation")
                .unwrap_or_default();
            let available = v["available_actions"]
                .as_array()
                .is_some_and(|a| a.iter().any(|a| a == &kind));
            if available {
                e.remove_attribute("hidden")?
            } else {
                e.set_attribute("hidden", "")?
            }
        }
    }
    Ok(())
}
fn render_transfer(v: &Value, decimals: u64) -> Result<(), JsValue> {
    let (_, d) = window_document().ok_or_else(|| JsValue::from_str("Browser unavailable"))?;
    if d.get_element_by_id("pay-transfer-panel").is_none() {
        return Ok(());
    }
    let epsx = v["checkout_snapshot"]["kind"] == "epsx_plan";
    let snapshot = &v["checkout_snapshot"];
    native_pay_text("pay-shop-name", string(snapshot, "merchant_name"));
    native_pay_text(
        "pay-headline",
        if epsx {
            "A little more insight."
        } else {
            "A simple way to pay."
        },
    );
    native_pay_text(
        "pay-subtitle",
        if epsx {
            "One payment. More possibilities with your EPSX account."
        } else {
            "One-time crypto payment. No automatic renewal."
        },
    );
    native_pay_text(
        "pay-service-terms",
        &if epsx {
            "Access begins after payment verification".into()
        } else {
            format!(
                "{}{}",
                string(snapshot, "description"),
                snapshot["duration_days"]
                    .as_u64()
                    .map(|n| format!(" · {n} days of service"))
                    .unwrap_or_default()
            )
        },
    );
    native_pay_text(
        "pay-purchase-note",
        if epsx {
            "Payment confirmation and plan access are tracked separately. Check both in your EPSX account."
        } else {
            "Keep this checkout link as your payment receipt. Your merchant provides the purchased service after payment confirmation."
        },
    );
    native_pay_text(
        "pay-final-step",
        if epsx {
            "3 · Your plan is added to your EPSX account"
        } else {
            "3 · Your merchant receives payment confirmation"
        },
    );
    if !epsx {
        let store = format!(
            "/m/{}?environment={}",
            string(v, "merchant_id"),
            string(v, "environment")
        );
        for id in ["pay-result-link", "pay-back-link"] {
            if let Some(e) = d.get_element_by_id(id) {
                e.set_attribute("href", &store)?;
                e.set_attribute("data-restart-url", &store)?;
                e.set_text_content(Some("Back to merchant"));
            }
        }
    }
    let is_qr = v["payment_method"] == "transfer";
    let state = string(v, "status");
    let terminal = ["succeeded", "refunded", "expired", "verification_required"].contains(&state);
    for (id, show) in [
        ("pay-transfer-panel", is_qr && !terminal),
        ("pay-result", terminal),
        ("pay-legacy", !is_qr && !terminal),
        ("pay-test-label", v["environment"] == "test"),
        ("pay-local-note", v["chain_id"] == 31337),
    ] {
        if let Some(e) = d.get_element_by_id(id) {
            if show {
                e.remove_attribute("hidden")?;
            } else {
                e.set_attribute("hidden", "")?;
            }
        }
    }
    native_pay_text(
        "merchant-payment-status",
        match state {
            "succeeded" if epsx => "Payment confirmed · plan access is being updated",
            "succeeded" => "Payment confirmed",
            "expired" => "Checkout expired",
            "refunded" => "Payment refunded",
            "verification_required" => "Payment needs review",
            _ => "Waiting for your payment",
        },
    );
    if terminal {
        let (icon,title,description)=match state {
            "succeeded"=>("✓","Payment received","Your payment is confirmed. Check your EPSX account for the status of your plan access."),
            "expired"=>("↻","This checkout has expired","Return to plans to create a new checkout. Do not send funds to this expired payment address."),
            "refunded"=>("↩","Payment refunded","The merchant has returned this payment. View your purchase history for updated plan access."),
            _=>("!","Let's check this payment","The payment needs review. An incorrect amount or a late transfer will not activate your plan automatically. Contact support with your payment reference."),
        };
        native_pay_text("pay-result-icon", icon);
        native_pay_text("pay-result-title", title);
        let description = if epsx {
            description
        } else {
            match state {"succeeded"=>"Payment confirmed on the network. This is your payment receipt. Contact the merchant about service delivery.","expired"=>"This checkout has expired. Return to the merchant for a new checkout. Do not send funds to this address.","refunded"=>"The merchant has refunded this payment.",_=>"This payment needs review. Contact the merchant with your payment reference."}
        };
        native_pay_text("pay-result-description", description);
        native_pay_text("merchant-transaction", string(v, "tx_hash"));
        if state == "expired" {
            if let Some(e) = d.get_element_by_id("pay-result-link") {
                if let Some(url) = e.get_attribute("data-restart-url") {
                    e.set_attribute("href", &url)?;
                    e.set_text_content(Some(if epsx {
                        "Choose a plan"
                    } else {
                        "Back to merchant"
                    }));
                }
            }
        }
    }
    if !is_qr {
        return Ok(());
    }
    let amount = native_pay_display(string(v, "amount"), decimals as usize);
    native_pay_text(
        "pay-exact-amount",
        &format!("{amount} {}", string(v, "token")),
    );
    native_pay_text("pay-address", string(v, "deposit_address"));
    native_pay_text("pay-token", string(v, "token"));
    let network = match v["chain_id"].as_u64() {
        Some(31337) => "Anvil Local · 31337",
        Some(97) => "BSC Testnet · 97",
        Some(56) => "BNB Smart Chain · 56",
        _ => "Check network",
    };
    native_pay_text("pay-network", network);
    native_pay_text(
        "pay-instruction",
        &format!(
            "Send exactly {amount} {} on {network}. Do not subtract network fees from this amount.",
            string(v, "token")
        ),
    );
    let left = ((js_sys::Date::parse(string(v, "expires_at")) - js_sys::Date::now()) / 1000.0)
        .max(0.0) as u64;
    native_pay_text(
        "pay-expiry",
        &format!("{:02}:{:02} remaining", left / 60, left % 60),
    );
    if let Some(e) = d.get_element_by_id("pay-qr") {
        e.set_attribute(
            "src",
            &format!(
                "data:image/svg+xml,{}",
                js_sys::encode_uri_component(string(v, "qr_svg"))
            ),
        )?;
    }
    for (id, value) in [
        ("pay-copy-address", string(v, "deposit_address")),
        ("pay-copy-amount", amount.as_str()),
    ] {
        if let Some(e) = d.get_element_by_id(id) {
            e.set_attribute("data-copy", value)?;
        }
    }
    Ok(())
}
pub async fn start(node: Element) {
    let cs = node.get_attribute("data-checkout-id").unwrap_or_default();
    if !cs.is_empty() {
        if let Some((w, _)) = window_document() {
            if let Ok(fragment) = w.location().hash() {
                if let Some(token) = fragment
                    .strip_prefix("#token=")
                    .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                {
                    if let Some(s) = storage() {
                        let _ = s.set_item(&format!("epsx.checkout.{cs}"), token);
                    }
                }
            }
        }
    }
    if let Some((_, d)) = window_document() {
        if let Some(e) = d.get_element_by_id("merchant-environment") {
            if let Some(env) =
                storage().and_then(|s| s.get_item("epsx.merchant.environment").ok().flatten())
            {
                let _ = Reflect::set(&e, &JsValue::from_str("value"), &JsValue::from_str(&env));
            }
        }
    }
    let detail_page = ["data-checkout-id", "data-payment-id", "data-link-id"]
        .iter()
        .any(|a| node.get_attribute(a).is_some_and(|s| !s.is_empty()));
    let mut load_failed = false;
    loop {
        let result = if detail_page {
            detail(&node).await
        } else {
            dashboard().await
        };
        if let Err(e) = result {
            load_failed = true;
            status(
                &e.as_string()
                    .unwrap_or_else(|| "Unable to refresh Pay. Retrying…".into()),
            )
        } else if load_failed {
            status("");
            load_failed = false;
        }
        delay(if detail_page { 4000 } else { 15000 }).await;
        if !node.is_connected() {
            break;
        }
    }
}
fn reveal(secret: &str) {
    native_pay_text("merchant-secret", secret);
    if let Some((_, d)) = window_document() {
        if let Some(e) = d.get_element_by_id("merchant-secret-panel") {
            let _ = e.remove_attribute("hidden");
        }
    }
}
fn input(id: &str) -> String {
    native_pay_input(id)
}
fn operation_key(button: &Element, context: &str) -> Result<String, JsValue> {
    let key = format!("epsx.merchant.request.{}", context);
    button.set_attribute("data-pay-request-storage", &key)?;
    stored(&key)
}
pub async fn epsx_checkout(button: &Element, plan: &str, token: &str) -> Result<(), JsValue> {
    for _ in 0..2 {
        let key = operation_key(button, &format!("epsx-plan-qr-v1:{plan}:{token}"))?;
        let v = fetch_json_with_headers(
            "/api/payments/merchant-checkout",
            "POST",
            Some(json!({"plan_id":plan,"token":token})),
            &[("idempotency-key", &key)],
        )
        .await?;
        if ["succeeded", "refunded", "expired"].contains(&string(&v, "status")) {
            native_pay_complete_request(button);
            continue;
        }
        let url = v["pay_url"]
            .as_str()
            .ok_or_else(|| JsValue::from_str("Checkout URL unavailable"))?;
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("Browser unavailable"))?
            .location()
            .set_href(url)?;
        return Ok(());
    }
    Err(JsValue::from_str(
        "Refresh this page to start a new purchase.",
    ))
}
pub async fn action(button: Element, action: String) {
    if action.starts_with("merchant-wallet-") || action.starts_with("merchant-method-") {
        if let Err(e) = checkout_wallet::handle(&action).await {
            status(&e.as_string().unwrap_or_default());
        }
        return;
    }
    if action.starts_with("merchant-catalog-") {
        set_wallet_busy(&button, true);
        if let Err(e) = merchant_catalog::action(&button, &action).await {
            status(&e.as_string().unwrap_or_default());
        }
        set_wallet_busy(&button, false);
        return;
    }
    set_wallet_busy(&button, true);
    let result=async{
        let target=button.get_attribute("data-target").unwrap_or_default();
        match action.as_str(){
            "merchant-connect"=>{let provider=injected_wallet_provider("metamask")?;let address=request_provider_account(&provider,false).await?;status(&format!("Connected: {address}"));return Ok(())},
            "merchant-hide-secret"=>{reveal("");if let Some((_,d))=window_document(){if let Some(e)=d.get_element_by_id("merchant-secret-panel"){e.set_attribute("hidden","")?;}}return Ok(())},
            "merchant-refresh"=>return dashboard().await,
            "merchant-register"=>{call("merchants","POST",Some(json!({"name":input("merchant-name")})),None,None).await?;status("Merchant account created. You can create links and API keys.");return dashboard().await},
            "merchant-create-key"=>{let v=call("api-keys","POST",Some(json!({"name":input("merchant-key-name")})),None,None).await?;reveal(string(&v,"key"));return dashboard().await},
            "merchant-create-webhook"=>{let v=call("webhook-endpoints","POST",Some(json!({"url":input("merchant-webhook-url")})),None,None).await?;reveal(string(&v,"signing_secret"));return dashboard().await},
            "merchant-revoke-key"|"merchant-disable-link"|"merchant-disable-webhook"|"merchant-rotate-webhook"|"merchant-replay"=>{
                let path=match action.as_str(){"merchant-revoke-key"=>format!("api-keys/{target}/revoke"),"merchant-disable-link"=>format!("links/{target}/disable"),"merchant-disable-webhook"=>format!("webhook-endpoints/{target}/disable"),"merchant-rotate-webhook"=>format!("webhook-endpoints/{target}/rotate"),_=>format!("deliveries/{target}/replay")};
                let v=call(&path,"POST",Some(json!({})),None,None).await?;if let Some(secret)=v["signing_secret"].as_str(){reveal(secret)}return dashboard().await
            }
            "merchant-delivery-details"=>{let v=call(&format!("deliveries/{target}"),"GET",None,None,None).await?;status(&format!("Delivery attempts: {}",v["attempts"]));return Ok(())},
            "merchant-create-link"=>{
                let token=input("merchant-token");let env=environment();let config=call("config","GET",None,None,None).await?;
                let decimals=config["environments"].as_array().and_then(|ns|ns.iter().find(|n|n["environment"]==env)).and_then(|n|n["tokens"][&token]["decimals"].as_u64()).ok_or_else(||JsValue::from_str("Token is not configured in this environment"))?as usize;
                let amount=input("merchant-amount");let mut parts=amount.trim().split('.');let whole=parts.next().unwrap_or("");let fraction=parts.next().unwrap_or("");
                if parts.next().is_some()||whole.is_empty()||!whole.bytes().all(|b|b.is_ascii_digit())||!fraction.bytes().all(|b|b.is_ascii_digit())||fraction.len()>decimals{return Err(JsValue::from_str("Enter a valid token amount"))}
                let units=format!("{whole}{fraction}{}","0".repeat(decimals-fraction.len())).trim_start_matches('0').to_string();let uses=input("merchant-max-uses");
                let max=if uses.is_empty(){Value::Null}else{json!(uses.parse::<u32>().map_err(|_|JsValue::from_str("Invalid maximum payments"))?)};
                let days=input("merchant-expiry-days").parse::<u32>().map_err(|_|JsValue::from_str("Invalid expiry"))?;
                let body=json!({"payment_method":if input("merchant-mode")=="direct"&&token!="BNB"{"transfer"}else{"contract"},"mode":input("merchant-mode"),"token":token,"amount":units,"description":input("merchant-description"),"max_uses":max,"expires_in":days*86400});
                let key=operation_key(&button,&format!("{env}:link:{body}"))?;let v=call("links","POST",Some(body),Some(&key),None).await?;
                let url=string(&v,"url");native_pay_text("merchant-created-link",url);if let Some((_,d))=window_document(){if let Some(a)=d.get_element_by_id("merchant-created-link"){a.set_attribute("href",url)?;}}native_pay_complete_request(&button);status("Payment link created.");return dashboard().await
            }
            "merchant-checkout"=>{
                let link=button.get_attribute("data-link-id").unwrap_or_default();let key=operation_key(&button,&format!("checkout:{link}"))?;
                let v=call(&format!("links/{link}/checkouts"),"POST",Some(json!({})),Some(&key),None).await?;
                let url=string(&v,"pay_url");let w=web_sys::window().unwrap();let parsed=web_sys::Url::new(url)?;if parsed.origin()!=w.location().origin()?{return Err(JsValue::from_str("Unexpected checkout origin"))}
                if let Some(s)=storage(){let cs=parsed.pathname().trim_start_matches("/checkout/").to_string();if let Some(source)=button.get_attribute("data-pay-request-storage"){s.set_item(&format!("epsx.checkout.source.{cs}"),&source)?;}}w.location().set_href(url)?;return Ok(())
            }
            _=>{}
        }
        let control=action=="merchant-pause";
        if action!="merchant-operation" && !control{return Err(JsValue::from_str("Unknown payment action"))}
        let cs=button.get_attribute("data-checkout-id").unwrap_or_default();let pi=button.get_attribute("data-payment-id").unwrap_or_default();let kind=button.get_attribute("data-merchant-operation").unwrap_or_default();let guest=(!cs.is_empty()).then_some(cs.as_str());
        let provider=injected_wallet_provider("metamask")?;let address=request_provider_account(&provider,false).await?;
        let path=if control{format!("contracts/{}/pause",button.get_attribute("data-mode").unwrap_or_default())}else if guest.is_some(){format!("checkout-sessions/{cs}/{kind}")}else{format!("intents/{pi}/{kind}")};
        let body=if control{json!({"paused":button.get_attribute("data-paused").as_deref()==Some("true")})}else{json!({"payer":address})};
        let key=operation_key(&button,&format!("{path}:{address}:{body}"))?;
        let op=call(&path,"POST",Some(body),Some(&key),guest).await?;let params=&op["transaction_parameters"];
        let expected=string(params,"from");let chain_hex=string(params,"chainId");let chain_id=u64::from_str_radix(chain_hex.trim_start_matches("0x"),16).map_err(|_|JsValue::from_str("Invalid chain"))?;
        ensure_provider_account(&provider,expected).await?;ensure_payment_chain(&provider,chain_id,chain_hex).await?;
        let opid=string(&op,"id");let saved_key=format!("epsx.merchant.tx.{opid}");let existing=storage().and_then(|s|s.get_item(&saved_key).ok().flatten());
        let hash=if let Some(hash)=existing{hash}else{
            if op["approval_transaction"].is_object(){
                status("Approve the token amount in your wallet.");let hash=native_pay_send(&provider,&op["approval_transaction"]).await?;
                for attempt in 0..300{let args=Array::new();args.push(&JsValue::from_str(&hash));let receipt=wallet_request(&provider,"eth_getTransactionReceipt",args).await?;
                    if !receipt.is_null()&&!receipt.is_undefined(){let v:Value=serde_wasm_bindgen::from_value(receipt).map_err(|e|JsValue::from_str(&e.to_string()))?;if v["status"]!="0x1"{return Err(JsValue::from_str("Token approval failed"))}break}
                    if attempt==299{return Err(JsValue::from_str("Approval is still pending. Return later to continue."))}delay(2000).await;
                }
            }
            ensure_provider_account(&provider,expected).await?;ensure_payment_chain(&provider,chain_id,chain_hex).await?;status("Confirm the payment transaction in your wallet.");
            let hash=native_pay_send(&provider,params).await?;if let Some(s)=storage(){s.set_item(&saved_key,&hash)?;}hash
        };
        native_pay_text("merchant-transaction",&hash);status("Transaction submitted. Waiting for chain confirmations…");
        let operations=if control{"contract-controls"}else{"operations"};
        call(&format!("{operations}/{opid}/confirm"),"POST",Some(json!({"tx_hash":hash})),None,guest).await?;
        for _ in 0..240{let v=call(&format!("{operations}/{opid}"),"GET",None,None,guest).await?;
            if v["status"]=="confirmed"{native_pay_complete_request(&button);status("Transaction confirmed on chain.");return Ok(())}
            if v["status"]=="failed"{native_pay_complete_request(&button);return Err(JsValue::from_str("Transaction failed or did not match this operation. No successful payment was recorded."))}delay(3000).await;
        }status("Still confirming. This checkout can be reopened safely.");Ok::<(),JsValue>(())
    }.await;
    if let Err(e) = result {
        status(
            &e.as_string()
                .unwrap_or_else(|| "Unable to complete this action".into()),
        )
    }
    set_wallet_busy(&button, false);
}
