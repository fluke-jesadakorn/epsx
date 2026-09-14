//! Deterministic, loopback-only browser fixture. No wallet or production service
//! is contacted. Run explicitly with the ignored test and built Dioxus assets.
use axum::{Extension, Router};
use epsx_dioxus_ui::fullstack::{pay::*, Surface};
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
    let read_data = shared.clone();
    let action_data = shared;
    let provider = PayProvider {
        read: Arc::new(move |_, _, _| {
            let data = read_data.lock().unwrap().clone();
            Box::pin(async move { Ok(data) })
        }),
        action: Arc::new(move |action, _, _, _| {
            let shared = action_data.clone();
            Box::pin(async move {
                match action{
                Action::Profile{name}=>{shared.lock().unwrap().merchant.as_mut().unwrap().name=name;Ok(ActionResult::default())},
                Action::SaveProduct{id,input}=>{let mut data=shared.lock().unwrap();let id=id.unwrap_or_else(||format!("pkg_{}",data.products.len()+1));data.products.retain(|p|p.id!=id);data.products.push(Product{id,name:input.name,description:input.description,prices:input.prices.into_iter().map(|(t,a)|(t,(a.parse::<f64>().unwrap()*1_000_000.0)as u64)).map(|(t,a)|(t,a.to_string())).collect(),enabled:input.enabled,duration_days:input.duration_days,..Default::default()});Ok(ActionResult::default())},
                Action::PrepareTransfer{payer,..}=>Ok(ActionResult{transaction_parameters:Some(Transaction{from:payer,to:"0x3333333333333333333333333333333333333333".into(),chain_id:"0x7a69".into(),data:"0x".into(),value:"0x0".into()}),payment:shared.lock().unwrap().payment.clone(),..Default::default()}),
                Action::BuyProduct{..}|Action::RedeemLink{..}=>Ok(ActionResult{pay_url:Some("http://127.0.0.1:43127/checkout/cs_fixture#token=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into()),..Default::default()}),
                Action::CreateKey{..}=>Ok(ActionResult{key:Some("fixture_only_secret".into()),..Default::default()}),
                _=>Ok(ActionResult::default()),
            }
            })
        }),
    };
    let fullstack = dioxus_server::FullstackState::new(dioxus_server::ServeConfig::new(), PayApp);
    let public = std::env::var("DIOXUS_PUBLIC_PATH").unwrap();
    let app = Router::new()
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
        .layer(Extension(provider));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:43127")
        .await
        .unwrap();
    println!("Pay fixture ready at http://127.0.0.1:43127");
    axum::serve(listener, app).await.unwrap();
}
