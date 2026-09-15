//! Deterministic, loopback-only browser fixture. No wallet or production service
//! is contacted. Run explicitly with the ignored test and built Dioxus assets.
use axum::{extract::Path, Extension, Json, Router};
use epsx_dioxus_ui::fullstack::pay::Action;
use epsx_dioxus_ui::fullstack::{pay::*, LoadError, Surface};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::test]
#[ignore = "interactive browser fixture; binds 127.0.0.1:43127 until interrupted"]
async fn pay_browser_fixture() {
    super::fullstack::validate_assets().unwrap();
    let payment=Payment{id:"pi_fixture".into(),checkout_id:"cs_fixture".into(),merchant_id:"m_fixture".into(),environment:Environment::Test,description:"Research pass".into(),amount:"5000000".into(),token:"USDT".into(),token_decimals:Some(6),payee:"0x2222222222222222222222222222222222222222".into(),mode:"direct".into(),status:"awaiting_payment".into(),payment_method:"transfer".into(),deposit_address:"0x3333333333333333333333333333333333333333".into(),chain_id:31337,expires_at:"2099-01-01T00:00:00Z".into(),qr_svg:"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"240\" height=\"240\"><rect width=\"240\" height=\"240\" fill=\"white\"/><path d=\"M30 30h60v60H30zM150 30h60v60h-60zM30 150h60v60H30z\"/></svg>".into(),checkout_snapshot:Snapshot{kind:"epsx_plan".into(),merchant_name:"Fixture shop".into(),item_name:"Research pass".into(),description:"30 days of research".into(),..Default::default()},..Default::default()};
    let data = PageData {
        signed_in: true,
        frontend_origin: "http://127.0.0.1:43127".into(),
        merchant: Some(Merchant {
            merchant_id: "m_fixture".into(),
            name: "Fixture shop".into(),
            wallet: "0x1111111111111111111111111111111111111111".into(),
        }),
        config: Config {
            environments: vec![NetworkConfig {
                environment: Environment::Test,
                tokens: [
                    ("USDT".into(), TokenConfig { decimals: 6 }),
                    ("USDC".into(), TokenConfig { decimals: 6 }),
                ]
                .into(),
            }],
            ..Default::default()
        },
        products: vec![Product {
            id: "pkg_fixture".into(),
            name: "Research pass".into(),
            description: "Thirty days of research".into(),
            enabled: true,
            duration_days: Some(30),
            prices: [("USDT".into(), "5000000".into())].into(),
            ..Default::default()
        }],
        payments: vec![payment.clone()],
        payment: Some(payment),
        ..Default::default()
    };
    let shared = Arc::new(Mutex::new(data));
    let settings = Arc::new(Mutex::new(
        json!({"delay_ms":0,"reject":false,"approval":false,"operation_status":"pending","sent":0}),
    ));
    let controls = shared.clone();
    let action_settings = settings.clone();
    let wallet_settings = settings.clone();
    let sent_settings = settings.clone();
    let reset_settings = settings.clone();
    let read_data = shared.clone();
    let read_settings = settings.clone();
    let action_data = shared;
    let provider = PayProvider {
        read: Arc::new(move |page, credentials, _| {
            let mut data = read_data.lock().unwrap().clone();
            if matches!(page, Page::Checkout(_)) && credentials.capability.is_none() {
                let data = PageData {
                    frontend_origin: data.frontend_origin,
                    ..Default::default()
                };
                return Box::pin(async move { Ok(data) });
            }
            if read_settings.lock().unwrap()["read_error"]
                .as_bool()
                .unwrap_or(false)
            {
                return Box::pin(async { Err(LoadError::Unavailable) });
            }
            if let Page::Checkout(id) = page {
                data.payment.as_mut().unwrap().checkout_id = id;
            }
            Box::pin(async move { Ok(data) })
        }),
        action: Arc::new(move |action, _, _, _| {
            let shared = action_data.clone();
            let settings = action_settings.clone();
            Box::pin(async move {
                match action{
                Action::Profile{name}=>{shared.lock().unwrap().merchant.as_mut().unwrap().name=name;Ok(ActionResult::default())},
                Action::SaveProduct{id,input}=>{let mut data=shared.lock().unwrap();let id=id.unwrap_or_else(||format!("pkg_{}",data.products.len()+1));data.products.retain(|p|p.id!=id);data.products.push(Product{id,name:input.name,description:input.description,prices:input.prices.into_iter().map(|(t,a)|(t,(a.parse::<f64>().unwrap()*1_000_000.0)as u64)).map(|(t,a)|(t,a.to_string())).collect(),enabled:input.enabled,duration_days:input.duration_days,..Default::default()});Ok(ActionResult::default())},
                Action::PrepareOperation{id,payer,..}=>{
                    let approval=settings.lock().unwrap()["approval"].as_bool().unwrap_or(false);
                    let tx=Transaction{from:payer,to:"0x3333333333333333333333333333333333333333".into(),chain_id:"0x38".into(),data:"0x".into(),value:"0x0".into()};
                    Ok(ActionResult{id:format!("mop_{id}"),transaction_parameters:Some(tx.clone()),approval_transaction:approval.then_some(tx),..Default::default()})
                },
                Action::ConfirmOperation{..}|Action::ReadOperation{..}=>Ok(ActionResult{status:settings.lock().unwrap()["operation_status"].as_str().unwrap_or("pending").into(),..Default::default()}),
                Action::PrepareTransfer{payer,..}=>Ok(ActionResult{transaction_parameters:Some(Transaction{from:payer,to:"0x3333333333333333333333333333333333333333".into(),chain_id:"0x7a69".into(),data:"0x".into(),value:"0x0".into()}),payment:shared.lock().unwrap().payment.clone(),..Default::default()}),
                Action::BuyProduct{..}|Action::RedeemLink{..}=>Ok(ActionResult{pay_url:Some("http://127.0.0.1:43127/checkout/cs_fixture#token=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into()),..Default::default()}),
                Action::CreateKey{..}=>Ok(ActionResult{key:Some("fixture_only_secret".into()),..Default::default()}),
                _=>Ok(ActionResult::default()),
            }
            })
        }),
    };
    let public = std::env::var("DIOXUS_PUBLIC_PATH").unwrap();
    let fullstack = dioxus_server::FullstackState::new(dioxus_server::ServeConfig::new(), PayApp);
    let app = Router::new()
        .route("/fixture/scenario/{scenario}",axum::routing::post(move |Path(scenario):Path<String>| {
            let data=controls.clone();let settings=reset_settings.clone();
            async move {
                let mut d=data.lock().unwrap();
                d.completion_available=true;
                d.completion=Some(CheckoutCompletion{order_id:uuid::Uuid::nil(),payment_status:"awaiting_payment".into(),fulfillment_status:"pending".into(),return_url:format!("http://127.0.0.1:43127/account/payments/{}",uuid::Uuid::nil())});
                let p=d.payment.as_mut().unwrap();p.chain_id=56;p.mode="direct".into();p.payment_method="contract".into();p.available_actions=vec!["pay".into()];p.status="awaiting_payment".into();p.tx_hash=None;
                p.checkout_snapshot.kind="merchant".into();
                if matches!(scenario.as_str(),"paid"|"granted"|"merchant") {p.status="succeeded".into();p.tx_hash=Some(format!("0x{}","a".repeat(64)));}
                if scenario=="expired" {p.status="expired".into();}
                if scenario=="outage" {d.completion_available=false;}
                if matches!(scenario.as_str(),"paid"|"granted"|"merchant") {d.completion.as_mut().unwrap().payment_status="succeeded".into();}
                if scenario=="granted" {d.completion.as_mut().unwrap().fulfillment_status="granted".into();}
                if scenario=="merchant" {d.completion=None;}
                let mut cfg=settings.lock().unwrap();
                cfg["read_error"]=json!(scenario=="outage");cfg["reject"]=json!(scenario=="rejected");cfg["approval"]=json!(scenario=="approval");cfg["delay_ms"]=json!(if scenario=="wallet"||scenario=="approval"{3000}else{0});cfg["operation_status"]=json!(if scenario=="failed"{"failed"}else{"pending"});
                Json(json!({"scenario":scenario,"sent":cfg["sent"]}))
            }
        }))
        .route("/fixture/wallet",axum::routing::get(move || {let settings=wallet_settings.clone();async move {Json(settings.lock().unwrap().clone())}}))
        .route("/fixture/sent",axum::routing::post(move || {let settings=sent_settings.clone();async move {let mut settings=settings.lock().unwrap();let n=settings["sent"].as_u64().unwrap_or(0)+1;settings["sent"]=json!(n);Json(json!({"hash":format!("0x{}","a".repeat(64))}))}}))
        .route("/account/payments/{id}",axum::routing::get(|Path(id):Path<String>|async move {axum::response::Html(format!("<!doctype html><title>Fixture purchase</title><main><h1>Purchase details</h1><p>Paid · Granted</p><p>{id}</p></main>"))}))
        .route("/fixture-wallet.js",axum::routing::get(||async { ([("content-type","text/javascript")],r#"
            // Isolated fixture only: no extension, RPC, key or real funds.
            const fixtureProvider={request:async({method})=>{
                const settings=await(await fetch('/fixture/wallet')).json();
                if(method==='eth_requestAccounts'||method==='eth_accounts')return ['0x1111111111111111111111111111111111111111'];
                if(method==='eth_chainId')return '0x38';
                if(method==='eth_getTransactionReceipt')return settings.approval?{status:'0x1'}:null;
                if(method==='eth_sendTransaction'){
                    await new Promise(resolve=>setTimeout(resolve,settings.delay_ms));
                    if(settings.reject)throw Error('You declined the payment in your wallet. You can try again.');
                    return (await(await fetch('/fixture/sent',{method:'POST'})).json()).hash;
                }
                throw Error('Unexpected fixture method '+method);
            }};
            Object.defineProperty(window,'__epsxPayProvider',{get:()=>fixtureProvider,set:()=>{},configurable:false});
            window.ethereum=fixtureProvider;
        "#) }))
        .route(
            "/merchant.css",
            axum::routing::get(|| async {
                ([("content-type", "text/css")], include_str!("merchant.css"))
            }),
        )
        .route(
            "/checkout.css",
            axum::routing::get(|| async {
                ([("content-type", "text/css")], include_str!("checkout.css"))
            }),
        )
        .route(
            "/brand-icon.svg",
            axum::routing::get(|| async {
                (
                    [("content-type", "image/svg+xml")],
                    include_str!("../../frontend/public/logos/epsx-icon.svg"),
                )
            }),
        )
        .route(
            "/walletconnect-2.24.0.js",
            axum::routing::get(|| async { ([("content-type", "text/javascript")], "") }),
        )
        .nest_service(
            "/assets",
            tower_http::services::ServeDir::new(format!("{public}/assets")),
        )
        .nest_service(
            "/wasm",
            tower_http::services::ServeDir::new(format!("{public}/wasm")),
        )
        .nest_service(
            "/public",
            tower_http::services::ServeDir::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../frontend/public"
            )),
        )
        .fallback(dioxus_server::FullstackState::render_handler)
        .with_state(fullstack.clone())
        .merge(epsx_bff::fullstack::server_functions(
            Surface::Pay,
            fullstack,
        ))
        .layer(Extension(provider))
        .layer(axum::middleware::from_fn(fixture_wallet_script));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:43127")
        .await
        .unwrap();
    println!("Pay fixture ready at http://127.0.0.1:43127");
    axum::serve(listener, app).await.unwrap();
}

// Inject only into the fixture HTML, keeping the real server/client component
// tree identical. The application bundle never contains the simulated wallet.
async fn fixture_wallet_script(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let response = next.run(request).await;
    if !response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.starts_with("text/html"))
    {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, 16 * 1024 * 1024).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap().replace(
        "<head>",
        "<head><script src=\"/fixture-wallet.js\"></script>",
    );
    parts.headers.remove("content-length");
    axum::response::Response::from_parts(parts, axum::body::Body::from(html))
}
