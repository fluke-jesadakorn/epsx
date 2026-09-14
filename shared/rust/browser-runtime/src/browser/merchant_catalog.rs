use super::*;
fn show(id: &str, visible: bool) -> Result<(), JsValue> {
    if let Some((_, d)) = window_document() {
        if let Some(e) = d.get_element_by_id(id) {
            if visible {
                e.remove_attribute("hidden")?
            } else {
                e.set_attribute("hidden", "")?
            }
        }
    }
    Ok(())
}
fn fill(id: &str, value: &str) -> Result<(), JsValue> {
    if let Some((_, d)) = window_document() {
        if let Some(e) = d.get_element_by_id(id) {
            if e.get_attribute("data-filled").is_none() {
                Reflect::set(&e, &"value".into(), &value.into())?;
                e.set_attribute("data-filled", "true")?;
            }
        }
    }
    Ok(())
}
fn path() -> String {
    window_document()
        .and_then(|(_, d)| d.query_selector("[data-merchant-dashboard]").ok().flatten())
        .and_then(|e| e.get_attribute("data-merchant-dashboard"))
        .unwrap_or_default()
}
fn decimals(config: &Value, token: &str) -> usize {
    config["environments"]
        .as_array()
        .and_then(|a| a.iter().find(|v| v["environment"] == environment()))
        .and_then(|v| v["tokens"][token]["decimals"].as_u64())
        .unwrap_or(18) as usize
}
fn link(d: &Document, row: &Element, label: &str, url: &str) -> Result<(), JsValue> {
    let a = d.create_element("a")?;
    a.set_attribute("href", url)?;
    a.set_attribute("class", "md-secondary")?;
    a.set_text_content(Some(label));
    row.append_child(&a)?;
    Ok(())
}
fn cards(items: &Value, config: &Value, public: bool) -> Result<(), JsValue> {
    let Some((d, root)) = container(if public {
        "merchant-store-products"
    } else {
        "merchant-products"
    }) else {
        return Ok(());
    };
    let Some(items) = items.as_array() else {
        return Ok(());
    };
    if items.is_empty() {
        root.set_text_content(Some("No packages available yet."));
    }
    for item in items {
        let card = d.create_element("article")?;
        card.set_attribute("class", "md-product")?;
        for (tag, text) in [
            ("h3", string(item, "name").to_string()),
            ("p", string(item, "description").to_string()),
            (
                "p",
                item["duration_days"]
                    .as_u64()
                    .map(|n| format!("{n} days of service · one-time payment"))
                    .unwrap_or_else(|| "One-time payment · no automatic renewal".into()),
            ),
        ] {
            let e = d.create_element(tag)?;
            e.set_text_content(Some(&text));
            card.append_child(&e)?;
        }
        if let Some(prices) = item["prices"].as_object() {
            for (token, amount) in prices {
                let price =
                    native_pay_display(amount.as_str().unwrap_or(""), decimals(config, token));
                let e = d.create_element("strong")?;
                e.set_text_content(Some(&format!("{price} {token}")));
                card.append_child(&e)?;
                if public {
                    let e = d.create_element("button")?;
                    e.set_attribute("class", "md-primary")?;
                    e.set_attribute("data-epsx-action", "merchant-catalog-buy")?;
                    e.set_attribute("data-target", string(item, "id"))?;
                    e.set_attribute("data-token", token)?;
                    e.set_text_content(Some(&format!("Pay {price} {token}")));
                    card.append_child(&e)?;
                }
            }
        }
        if !public {
            let e = d.create_element("p")?;
            e.set_text_content(Some(if item["enabled"] == true {
                "Open for sale"
            } else {
                "Hidden from sale"
            }));
            card.append_child(&e)?;
            link(
                &d,
                &card,
                "Edit",
                &format!("/packages/{}/edit", string(item, "id")),
            )?;
            link(
                &d,
                &card,
                "Payment link",
                &format!(
                    "/packages/{}?environment={}",
                    string(item, "id"),
                    environment()
                ),
            )?;
        }
        root.append_child(&card)?;
    }
    Ok(())
}
pub async fn load() -> Result<bool, JsValue> {
    let path = path();
    if path.is_empty() {
        return Ok(false);
    }
    let public =
        path.starts_with("/m/") || path.starts_with("/packages/pkg_") && !path.ends_with("/edit");
    if public {
        if let Some((w, _)) = window_document() {
            let u = web_sys::Url::new(&w.location().href()?)?;
            if let Some(env) = u
                .search_params()
                .get("environment")
                .filter(|v| v == "test" || v == "live")
            {
                if let Some(s) = storage() {
                    s.set_item("epsx.merchant.environment", &env)?;
                }
            }
        }
        let config = call("config", "GET", None, None, None).await?;
        if let Some(mid) = path.strip_prefix("/m/") {
            let v = call(&format!("catalog/{mid}"), "GET", None, None, None).await?;
            native_pay_text("merchant-shop-name", string(&v["merchant"], "name"));
            cards(&v["items"], &config, true)?;
        } else {
            let pid = path.trim_start_matches("/packages/");
            let v = call(&format!("catalog/products/{pid}"), "GET", None, None, None).await?;
            native_pay_text("merchant-shop-name", string(&v, "merchant_name"));
            cards(&json!([v]), &config, true)?;
        }
        return Ok(true);
    }
    let me = match call("merchants/me", "GET", None, None, None).await {
        Ok(v) => v,
        Err(e) => {
            show("merchant-onboarding", true)?;
            show("merchant-dashboard-content", false)?;
            if e.as_string().as_deref() == Some("not_found") {
                status("Choose a shop name to finish setting up your merchant account.");
                return Ok(true);
            }
            return Err(e);
        }
    };
    show("merchant-onboarding", false)?;
    show("merchant-dashboard-content", true)?;
    native_pay_text("merchant-shop-name", string(&me, "name"));
    native_pay_text("merchant-signin", "Switch wallet");
    native_pay_text("merchant-owner", string(&me, "wallet"));
    fill("merchant-settings-name", string(&me, "name"))?;
    if let Some((_, d)) = window_document() {
        if let Some(a) = d.get_element_by_id("merchant-store-link") {
            a.set_attribute(
                "href",
                &format!(
                    "/m/{}?environment={}",
                    string(&me, "merchant_id"),
                    environment()
                ),
            )?;
            a.remove_attribute("hidden")?;
        }
    }
    let config = call("config", "GET", None, None, None).await?;
    if path.starts_with("/packages") {
        let v = call("products", "GET", None, None, None).await?;
        cards(&v["items"], &config, false)?;
        if path.ends_with("/edit") {
            let pid = path
                .trim_start_matches("/packages/")
                .trim_end_matches("/edit");
            let p = call(&format!("products/{pid}"), "GET", None, None, None).await?;
            for (id, key) in [
                ("product-name", "name"),
                ("product-description", "description"),
            ] {
                fill(id, string(&p, key))?;
            }
            fill(
                "product-duration",
                &p["duration_days"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            )?;
            fill(
                "product-enabled",
                if p["enabled"] == true {
                    "true"
                } else {
                    "false"
                },
            )?;
            for (token, id) in [("USDT", "product-usdt"), ("USDC", "product-usdc")] {
                fill(
                    id,
                    &p["prices"][token]
                        .as_str()
                        .map(|v| native_pay_display(v, decimals(&config, token)))
                        .unwrap_or_default(),
                )?;
            }
        }
    }
    if path == "/" || path == "/dashboard" {
        let v = call("overview", "GET", None, None, None).await?;
        if let Some((d, root)) = container("merchant-overview") {
            if let Some(items) = v["items"].as_array() {
                if items.is_empty() {
                    root.set_text_content(Some(
                        "Your first payment starts here. Create a package to get going.",
                    ));
                }
                for v in items {
                    for (key, label) in [
                        ("paid", "Customer payments confirmed"),
                        ("ready", "Ready to collect · after fee"),
                        ("fees", "Processing fees · estimated until collection"),
                    ] {
                        let card = d.create_element("div")?;
                        card.set_attribute("class", "md-stat")?;
                        let small = d.create_element("small")?;
                        small.set_text_content(Some(label));
                        card.append_child(&small)?;
                        let big = d.create_element("strong")?;
                        big.set_text_content(Some(&format!(
                            "{} {}",
                            native_pay_display(
                                string(v, key),
                                v["decimals"].as_u64().unwrap_or(18) as usize
                            ),
                            string(v, "token")
                        )));
                        card.append_child(&big)?;
                        root.append_child(&card)?;
                    }
                }
            }
        }
    }
    Ok(false)
}
pub async fn action(button: &Element, action: &str) -> Result<(), JsValue> {
    let target = button.get_attribute("data-target").unwrap_or_default();
    match action {
        "merchant-catalog-save" => {
            let mut prices = json!({});
            for (token, id) in [("USDT", "product-usdt"), ("USDC", "product-usdc")] {
                let value = input(id);
                if !value.trim().is_empty() {
                    prices[token] = json!(value.trim());
                }
            }
            let duration = input("product-duration");
            let duration = if duration.is_empty() {
                Value::Null
            } else {
                json!(duration
                    .parse::<u32>()
                    .map_err(|_| JsValue::from_str("Enter a whole number of days"))?)
            };
            let body = json!({"name":input("product-name"),"description":input("product-description"),"prices":prices,"duration_days":duration,"enabled":input("product-enabled")=="true"});
            let path = path();
            let route = if path.ends_with("/edit") {
                format!(
                    "products/{}/update",
                    path.trim_start_matches("/packages/")
                        .trim_end_matches("/edit")
                )
            } else {
                "products".into()
            };
            let key = operation_key(button, &format!("{}:{route}:{body}", environment()))?;
            call(&route, "POST", Some(body), Some(&key), None).await?;
            native_pay_complete_request(button);
            status("Package saved. Existing checkouts keep their original terms.");
            load().await?;
        }
        "merchant-catalog-profile" => {
            call(
                "merchants/me/update",
                "POST",
                Some(json!({"name":input("merchant-settings-name")})),
                None,
                None,
            )
            .await?;
            status("Shop name saved.");
            load().await?;
        }
        "merchant-catalog-buy" => {
            let token = button.get_attribute("data-token").unwrap_or_default();
            let key = operation_key(
                button,
                &format!("product:{target}:{token}:{}", environment()),
            )?;
            let v = call(
                &format!("products/{target}/checkouts"),
                "POST",
                Some(json!({"token":token})),
                Some(&key),
                None,
            )
            .await?;
            let w = web_sys::window().ok_or("Browser unavailable")?;
            let url = string(&v, "pay_url");
            if web_sys::Url::new(url)?.origin() != w.location().origin()? {
                return Err("Unexpected checkout origin".into());
            }
            w.location().set_href(url)?;
        }
        "merchant-catalog-replace-webhook" | "merchant-catalog-enable-webhook" => {
            let replace = action == "merchant-catalog-replace-webhook";
            let route = format!(
                "webhook-endpoints/{target}/{}",
                if replace { "replace" } else { "enable" }
            );
            let v = call(
                &route,
                "POST",
                Some(if replace {
                    json!({"url":input("merchant-webhook-url")})
                } else {
                    json!({})
                }),
                None,
                None,
            )
            .await?;
            if let Some(s) = v["signing_secret"].as_str() {
                reveal(s)
            }
            dashboard().await?;
        }
        _ => return Err("Unknown merchant action".into()),
    }
    Ok(())
}

pub fn payment_row(d: &Document, root: &Element, v: &Value) -> Result<(), JsValue> {
    let row = d.create_element("div")?;
    row.set_attribute("class", "md-payment-row")?;
    let item = d.create_element("div")?;
    let title = d.create_element("strong")?;
    title.set_text_content(Some(string(&v["checkout_snapshot"], "item_name")));
    item.append_child(&title)?;
    let states = d.create_element("small")?;
    let paid = match string(v, "status") {
        "succeeded" => "Paid",
        "awaiting_payment" => "Awaiting payment",
        "verification_required" => "Needs review",
        "refunded" => "Refunded",
        "expired" => "Expired",
        s => s,
    };
    let settled = match string(v, "settlement_status") {
        "ready" => "Ready to collect",
        "settled" => "Funds in wallet",
        _ => "Not collected",
    };
    states.set_text_content(Some(&format!("{paid} · {settled}")));
    item.append_child(&states)?;
    row.append_child(&item)?;
    let amount = d.create_element("strong")?;
    amount.set_text_content(Some(&format!(
        "{} {}",
        native_pay_display(
            string(v, "amount"),
            v["token_decimals"].as_u64().unwrap_or(18) as usize
        ),
        string(v, "token")
    )));
    row.append_child(&amount)?;
    link(
        d,
        &row,
        "View payment →",
        &format!("/payments/{}", string(v, "id")),
    )?;
    root.append_child(&row)?;
    Ok(())
}
