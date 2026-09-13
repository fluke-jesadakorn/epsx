//! Optional wallet transport for the same ERC-20 invoice shown by the QR view.
use super::*;
thread_local! {
    static PROVIDER: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    static ADDRESS: RefCell<String> = const { RefCell::new(String::new()) };
    static CHAIN: Cell<u64> = const { Cell::new(0) };
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static CONNECTING: Cell<bool> = const { Cell::new(false) };
    static EPOCH: Cell<u64> = const { Cell::new(0) };
    static USE_WALLET: Cell<bool> = const { Cell::new(false) };
    static USING_WC: Cell<bool> = const { Cell::new(false) };
    static LAST: RefCell<Value> = const { RefCell::new(Value::Null) };
}
fn visible(id: &str, show: bool) {
    if let Some((_, d)) = window_document() {
        if let Some(e) = d.get_element_by_id(id) {
            let _ = if show {
                e.remove_attribute("hidden")
            } else {
                e.set_attribute("hidden", "")
            };
        }
    }
}
fn feedback(s: &str) {
    native_pay_text("pay-wallet-feedback", s);
}
fn tx_key(cs: &str) -> String {
    format!("epsx.checkout.transfer.{cs}")
}
fn pending(cs: &str) -> Option<String> {
    storage().and_then(|s| s.get_item(&tx_key(cs)).ok().flatten())
}
fn bridge() -> Result<JsValue, JsValue> {
    let (w, _) = window_document().ok_or("Browser unavailable")?;
    let b = Reflect::get(&w, &JsValue::from_str("EPSXWalletConnect"))?;
    if b.is_undefined() {
        Err(JsValue::from_str("WalletConnect is unavailable"))
    } else {
        Ok(b)
    }
}
async fn invoke(name: &str, arg: &JsValue) -> Result<JsValue, JsValue> {
    let b = bridge()?;
    let f = Reflect::get(&b, &JsValue::from_str(name))?.dyn_into::<Function>()?;
    JsFuture::from(js_sys::Promise::resolve(&f.call1(&b, arg)?)).await
}
async fn load_bridge() -> Result<(), JsValue> {
    if bridge().is_ok() {
        return Ok(());
    }
    let (_, d) = window_document().ok_or("Browser unavailable")?;
    let config = fetch_json("/api/wallet-config", "GET", None).await?;
    if config["projectId"].as_str().is_none_or(|s| s.is_empty()) {
        return Err(JsValue::from_str(
            "WalletConnect is not configured. Use MetaMask or QR transfer.",
        ));
    }
    if d.get_element_by_id("epsx-walletconnect-sdk").is_none() {
        let script = d.create_element("script")?;
        script.set_id("epsx-walletconnect-sdk");
        script.set_attribute("src", string(&config, "sdkUrl"))?;
        script.set_attribute("integrity", string(&config, "sdkIntegrity"))?;
        script.set_attribute("crossorigin", "anonymous")?;
        d.body().ok_or("Page unavailable")?.append_child(&script)?;
    }
    for _ in 0..150 {
        if bridge().is_ok() {
            return Ok(());
        }
        delay(100).await;
    }
    if let Some(e) = d.get_element_by_id("epsx-walletconnect-sdk") {
        e.remove();
    }
    Err(JsValue::from_str(
        "WalletConnect could not load. Try again or use QR transfer.",
    ))
}
pub fn render(v: &Value) -> Result<(), JsValue> {
    LAST.with(|s| *s.borrow_mut() = v.clone());
    if v["payment_method"] != "transfer" {
        return Ok(());
    }
    let wallet = USE_WALLET.with(Cell::get);
    visible("pay-qr-method", !wallet);
    visible("pay-wallet-method", wallet);
    let address = ADDRESS.with(|a| a.borrow().clone());
    let busy = BUSY.with(Cell::get);
    let connecting = CONNECTING.with(Cell::get);
    visible("pay-wallet-options", address.is_empty() && !connecting);
    visible("pay-wallet-account", !address.is_empty());
    visible("pay-wallet-pairing", connecting && USING_WC.with(Cell::get));
    native_pay_text("pay-wallet-address", &address);
    let chain = CHAIN.with(Cell::get);
    native_pay_text(
        "pay-wallet-chain",
        &if chain == v["chain_id"].as_u64().unwrap_or(0) {
            format!("Connected to chain {chain}")
        } else {
            format!("Chain {chain} · your wallet will request the checkout network before payment")
        },
    );
    if let Some((_, d)) = window_document() {
        for (id, active) in [("pay-method-qr", !wallet), ("pay-method-wallet", wallet)] {
            if let Some(e) = d.get_element_by_id(id) {
                e.set_attribute("aria-pressed", if active { "true" } else { "false" })?;
            }
        }
        let buttons = d.query_selector_all("[data-epsx-action^='merchant-wallet-']")?;
        for i in 0..buttons.length() {
            if let Some(e) = buttons
                .item(i)
                .and_then(|n| n.dyn_into::<HtmlButtonElement>().ok())
            {
                e.set_disabled(
                    busy && !(connecting
                        && e.get_attribute("data-epsx-action").as_deref()
                            == Some("merchant-wallet-disconnect")),
                );
            }
        }
        if let Some(e) = d
            .get_element_by_id("pay-wallet-send")
            .and_then(|n| n.dyn_into::<HtmlButtonElement>().ok())
        {
            let submitted = pending(string(v, "checkout_id")).is_some();
            e.set_disabled(
                busy || submitted
                    || v["status"] != "awaiting_payment"
                    || js_sys::Date::parse(string(v, "expires_at")) <= js_sys::Date::now(),
            );
            e.set_text_content(Some(&if submitted {
                "Waiting for confirmation…".into()
            } else {
                format!(
                    "Pay {} {}",
                    native_pay_display(
                        string(v, "amount"),
                        v["token_decimals"].as_u64().unwrap_or(18) as usize
                    ),
                    string(v, "token")
                )
            }));
        }
    }
    Ok(())
}
fn rerender() {
    let v = LAST.with(|s| s.borrow().clone());
    let _ = render(&v);
}
async fn sync(provider: &JsValue) -> Result<(), JsValue> {
    let accounts = wallet_request(provider, "eth_accounts", Array::new()).await?;
    ADDRESS.with(|a| {
        *a.borrow_mut() = Array::from(&accounts)
            .get(0)
            .as_string()
            .unwrap_or_default()
    });
    CHAIN.with(|c| c.set(0));
    let chain = provider_chain_id(provider).await?;
    CHAIN.with(|c| c.set(chain));
    rerender();
    Ok(())
}
fn subscribe(provider: &JsValue) {
    let Ok(on) =
        Reflect::get(provider, &JsValue::from_str("on")).and_then(|f| f.dyn_into::<Function>())
    else {
        return;
    };
    for event in ["accountsChanged", "chainChanged", "disconnect"] {
        let p = provider.clone();
        let epoch = EPOCH.with(Cell::get);
        let callback = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            if EPOCH.with(Cell::get) != epoch {
                return;
            }
            ADDRESS.with(|a| a.borrow_mut().clear());
            rerender();
            let p = p.clone();
            spawn_local(async move {
                let _ = sync(&p).await;
            });
        });
        let _ = on.call2(provider, &JsValue::from_str(event), callback.as_ref());
        callback.forget();
    }
}
pub async fn refresh(v: &Value) {
    if v["payment_method"] != "transfer" {
        return;
    }
    if CONNECTING.with(Cell::get) && USING_WC.with(Cell::get) {
        if let Ok(b) = bridge() {
            if let Ok(f) =
                Reflect::get(&b, &JsValue::from_str("state")).and_then(|f| f.dyn_into::<Function>())
            {
                if let Ok(s) = f.call0(&b) {
                    if let Ok(uri) = Reflect::get(&s, &JsValue::from_str("uri")) {
                        if let Some(uri) = uri.as_string().filter(|s| !s.is_empty()) {
                            if let Ok(code) = qrcode::QrCode::new(uri.as_bytes()) {
                                let svg = code
                                    .render::<qrcode::render::svg::Color>()
                                    .min_dimensions(240, 240)
                                    .build();
                                if let Some((_, d)) = window_document() {
                                    if let Some(e) = d.get_element_by_id("pay-wallet-pairing-qr") {
                                        let _ = e.set_attribute(
                                            "src",
                                            &format!(
                                                "data:image/svg+xml,{}",
                                                js_sys::encode_uri_component(&svg)
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let cs = string(v, "checkout_id");
    if let Some(hash) = pending(cs) {
        if v["status"] == "awaiting_payment" {
            feedback(&format!(
                "Transaction submitted: {hash}. Waiting for verified payment."
            ));
            let provider = PROVIDER.with(|p| p.borrow().clone());
            if let Some(provider) = provider {
                let args = Array::new();
                args.push(&JsValue::from_str(&hash));
                if let Ok(receipt) =
                    wallet_request(&provider, "eth_getTransactionReceipt", args).await
                {
                    if Reflect::get(&receipt, &JsValue::from_str("status"))
                        .ok()
                        .and_then(|s| s.as_string())
                        .as_deref()
                        == Some("0x0")
                    {
                        if let Some(s) = storage() {
                            let _ = s.remove_item(&tx_key(cs));
                        }
                        feedback(
                            "The transaction reverted. No payment was received; you can try again.",
                        );
                    }
                }
            }
        }
    }
    rerender();
}
pub async fn handle(action: &str) -> Result<(), JsValue> {
    if action.starts_with("merchant-method-") {
        USE_WALLET.with(|s| s.set(action.ends_with("wallet")));
        rerender();
        return Ok(());
    }
    if action == "merchant-wallet-disconnect" {
        if BUSY.with(Cell::get) && !CONNECTING.with(Cell::get) {
            return Ok(());
        }
        EPOCH.with(|e| e.set(e.get() + 1));
        PROVIDER.with(|p| *p.borrow_mut() = None);
        ADDRESS.with(|a| a.borrow_mut().clear());
        CONNECTING.with(|b| b.set(false));
        BUSY.with(|b| b.set(false));
        if USING_WC.with(Cell::get) {
            let _ = invoke("disconnect", &JsValue::UNDEFINED).await;
        }
        feedback("Wallet disconnected. You can still pay using QR transfer.");
        rerender();
        return Ok(());
    }
    if BUSY.with(|b| b.replace(true)) {
        return Ok(());
    }
    let epoch = EPOCH.with(|e| {
        if action != "merchant-wallet-send" {
            e.set(e.get() + 1);
        }
        e.get()
    });
    rerender();
    let result = async {
        if action != "merchant-wallet-send" {
            CONNECTING.with(|b| b.set(true));
            USING_WC.with(|b| b.set(action == "merchant-wallet-walletconnect"));
            rerender();
            feedback("Approve the connection in your wallet.");
            let provider = if USING_WC.with(Cell::get) {
                load_bridge().await?;
                let mut config = fetch_json("/api/wallet-config", "GET", None).await?;
                let chain = LAST
                    .with(|s| s.borrow()["chain_id"].as_u64())
                    .ok_or("Checkout is still loading")?;
                let (_, rpc, _) = chain_metadata(chain).ok_or("Unsupported checkout network")?;
                config["chainId"] = json!(chain);
                config["rpcUrl"] = json!(rpc);
                invoke("connect", &js_sys::JSON::parse(&config.to_string())?).await?
            } else {
                let p = injected_wallet_provider("metamask")?;
                request_provider_account(&p, false).await?;
                p
            };
            if EPOCH.with(Cell::get) != epoch {
                return Ok(());
            }
            PROVIDER.with(|p| *p.borrow_mut() = Some(provider.clone()));
            subscribe(&provider);
            sync(&provider).await?;
            feedback("Wallet connected. Review the amount, then choose Pay.");
            return Ok(());
        }
        let provider = PROVIDER
            .with(|p| p.borrow().clone())
            .ok_or("Connect your wallet first")?;
        let v = LAST.with(|s| s.borrow().clone());
        let cs = string(&v, "checkout_id");
        if pending(cs).is_some() {
            return Err(JsValue::from_str(
                "A transaction is already pending for this checkout.",
            ));
        }
        let chain = v["chain_id"]
            .as_u64()
            .ok_or("Checkout network unavailable")?;
        ensure_payment_chain(&provider, chain, &format!("0x{chain:x}")).await?;
        sync(&provider).await?;
        let address = ADDRESS.with(|a| a.borrow().clone());
        feedback("Checking token balance and network fees…");
        let prepared = call(
            &format!("checkout-sessions/{cs}/prepare-transfer"),
            "POST",
            Some(json!({"payer":address})),
            None,
            Some(cs),
        )
        .await?;
        let params = &prepared["transaction_parameters"];
        ensure_provider_account(&provider, string(params, "from")).await?;
        ensure_payment_chain(&provider, chain, &format!("0x{chain:x}")).await?;
        if js_sys::Date::parse(string(&prepared["payment"], "expires_at")) <= js_sys::Date::now() {
            return Err(JsValue::from_str(
                "Checkout expired. Choose a plan to start again.",
            ));
        }
        feedback("Confirm this token transfer in your wallet. No approval transaction is needed.");
        let _ = storage()
            .ok_or("Session storage unavailable")?
            .get_item(&tx_key(cs))?;
        let hash = native_pay_send(&provider, params).await?;
        storage()
            .ok_or("Session storage unavailable")?
            .set_item(&tx_key(cs), &hash)?;
        feedback(&format!(
            "Transaction submitted: {hash}. Waiting for verified payment."
        ));
        Ok::<(), JsValue>(())
    }
    .await;
    if EPOCH.with(Cell::get) == epoch {
        BUSY.with(|b| b.set(false));
        CONNECTING.with(|b| b.set(false));
        if let Err(e) = result {
            let message = e
                .as_string()
                .or_else(|| {
                    Reflect::get(&e, &JsValue::from_str("message"))
                        .ok()
                        .and_then(|v| v.as_string())
                })
                .unwrap_or_else(|| "Wallet request failed. Retry or use QR transfer.".into());
            feedback(match message.as_str() {
                "insufficient_token_balance" => {
                    "Not enough of the selected token. Add test tokens or use another wallet."
                }
                "insufficient_gas_balance" => {
                    "Not enough native coin for network fees. Add test ETH on Anvil."
                }
                "checkout_not_payable" => {
                    "This checkout is no longer payable. Refresh its status before continuing."
                }
                _ => &message,
            });
        }
        rerender();
    }
    Ok(())
}
