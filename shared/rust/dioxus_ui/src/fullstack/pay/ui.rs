use super::types::Action;
use super::{act_pay, read_pay, types::*, wallet};
use crate::fullstack::LoadError;
use dioxus::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Routable)]
enum PayRoute {
    #[route("/?:..query")]
    Home { query: String },
    #[route("/:..segments?:..query")]
    View {
        segments: Vec<String>,
        query: String,
    },
}
#[component]
fn Home(query: String) -> Element {
    let PayTheme(dark) = use_context::<PayTheme>();
    rsx! { super::home::Landing { environment: environment_for(&query), dark } }
}
fn environment_for(query: &str) -> Environment {
    if url::form_urlencoded::parse(query.as_bytes()).any(|(k, v)| k == "environment" && v == "live")
    {
        Environment::Live
    } else {
        Environment::Test
    }
}
pub(super) fn page_for(path: &str) -> Option<Page> {
    Some(match path {
        "/dashboard" => Page::Dashboard,
        "/payments" => Page::Payments,
        "/packages" => Page::Packages,
        "/payment-links" => Page::Links,
        "/webhooks" => Page::Webhooks,
        "/settings" => Page::Settings,
        "/escrow" => Page::NativeDashboard,
        _ => {
            let parts: Vec<_> = path.trim_matches('/').split('/').collect();
            match parts.as_slice() {
                ["packages", id, "edit"] => Page::EditPackage((*id).into()),
                ["packages", id] => Page::Product((*id).into()),
                ["m", id] => Page::Store((*id).into()),
                ["checkout", id] if id.starts_with("cs_") => Page::Checkout((*id).into()),
                ["checkout" | "intent", id] => Page::NativeIntent((*id).into()),
                ["payments", id] => Page::Payment((*id).into()),
                ["r", id] if id.starts_with("plink_") => Page::Link((*id).into()),
                ["r", id] => Page::NativeLink((*id).into()),
                _ => return None,
            }
        }
    })
}
#[derive(Clone, Copy)]
struct PayTheme(Signal<bool>);
#[component]
pub fn PayApp() -> Element {
    let mut dark = use_signal(|| false);
    let mut theme_ready = use_signal(|| false);
    use_context_provider(|| PayTheme(dark));
    use_future(move || async move {
        if let Ok(value) = wallet::theme(None).await {
            dark.set(value);
        }
        theme_ready.set(true);
    });
    use_effect(move || {
        let value = dark();
        if theme_ready() {
            spawn(async move {
                let _ = wallet::theme(Some(value)).await;
            });
        }
    });
    rsx! {
        document::Meta{name:"viewport",content:"width=device-width,initial-scale=1"}
        document::Link{rel:"stylesheet",href:"/public/dist/tailwind.css"}
        document::Link{rel:"stylesheet",href:"/merchant.css"}
        document::Link{rel:"stylesheet",href:"/checkout.css"}
        super::home::LandingStyles{}
        document::Script{src:"/walletconnect-2.24.0.js"}
        Router::<PayRoute>{}
    }
}
#[component]
fn View(segments: Vec<String>, query: String) -> Element {
    let path = format!("/{}", segments.join("/"));
    let env = environment_for(&query);
    if matches!(path.as_str(), "/docs" | "/docs/merchant") {
        return rsx! {Docs{merchant:path=="/docs/merchant",environment:env}};
    }
    match page_for(&path) {
        Some(page) => rsx! {PayPage{key:"{path}:{env:?}",page,environment:env,path}},
        None => {
            rsx! {main{class:"container-x py-10",document::Title{"Page not found · EPSX Pay"}h1{"Page not found"}Link{to:"/","Back to Pay"}}}
        }
    }
}
#[derive(Clone, Copy)]
struct Controller {
    data: Signal<Option<PageData>>,
    credentials: Signal<Credentials>,
    busy: Signal<bool>,
    message: Signal<String>,
    secret: Signal<Option<String>>,
    revision: Signal<u64>,
    page: Signal<Page>,
}
impl Controller {
    async fn perform(mut self, action: Action) -> Result<ActionResult, String> {
        let credentials = (self.credentials)();
        let context = serde_json::to_string(&(credentials.environment, &action))
            .map_err(|e| e.to_string())?;
        let key = wallet::key(&context).await?;
        let mut result = act_pay(action.clone(), credentials.clone(), key.clone())
            .await
            .map_err(|_| "Pay is unavailable. Retry safely.".to_string())?;
        if matches!(result, Err(LoadError::Unauthenticated)) && wallet::refresh().await.is_ok() {
            result = act_pay(action, credentials, key)
                .await
                .map_err(|_| "Pay is unavailable. Retry safely.".to_string())?;
        }
        let result = result.map_err(|e| e.message().to_string())?;
        if let Some(secret) = result.key.as_ref().or(result.signing_secret.as_ref()) {
            self.secret.set(Some(secret.clone()));
        }
        if let Some(url) = &result.pay_url {
            let local = wallet::local_checkout(url, &context).await?;
            dioxus_router::navigator().push(local);
        }
        // An operation key remains until chain confirmation. Creation requests
        // clear only after the successful response, so failed retries are safe.
        if result.transaction_parameters.is_none() && result.pay_url.is_none() {
            wallet::complete(&context).await?;
        }
        self.revision += 1;
        Ok(result)
    }
    fn dispatch(mut self, action: Action) {
        if *(self.busy).peek() {
            return;
        }
        self.busy.set(true);
        self.message.set("Saving…".into());
        spawn(async move {
            match self.perform(action).await {
                Ok(v) => {
                    self.message.set(if !v.attempts.is_empty() {
                        v.attempts
                            .iter()
                            .map(|a| {
                                format!(
                                    "{} · HTTP {}",
                                    a.attempted_at.as_deref().unwrap_or(""),
                                    a.status.map(|v| v.to_string()).unwrap_or_default()
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("; ")
                    } else {
                        v.url.unwrap_or_else(|| "Saved.".into())
                    });
                }
                Err(e) => self.message.set(e),
            }
            self.busy.set(false);
        });
    }
}
#[component]
fn PayPage(page: Page, environment: Environment, path: String) -> Element {
    let seed_page = page.clone();
    let seed_c = Credentials {
        environment,
        checkout: match &page {
            Page::Checkout(id) => Some(id.clone()),
            _ => None,
        },
        ..Default::default()
    };
    let ssr_c = seed_c.clone();
    let initial = use_server_future(move || {
        let page = seed_page.clone();
        let c = ssr_c.clone();
        async move {
            read_pay(page, c)
                .await
                .map_err(|_| LoadError::Unavailable)?
        }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    if let Err(error) = &seed {
        crate::pages::news::hydrated::response_status(match error {
            LoadError::NotFound => 404,
            LoadError::Unauthenticated => 401,
            LoadError::Forbidden => 403,
            LoadError::InvalidQuery => 400,
            _ => 502,
        });
    }
    let data = use_signal(|| seed.clone().ok());
    let message = use_signal(|| {
        seed.err()
            .map(|e| e.message().to_string())
            .unwrap_or_default()
    });
    let credentials = use_signal(|| seed_c);
    let mut controller = Controller {
        data,
        credentials,
        busy: use_signal(|| false),
        message,
        secret: use_signal(|| None),
        revision: use_signal(|| 0),
        page: use_signal(|| page.clone()),
    };
    use_context_provider(|| controller);
    let mut ready = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut first_client_read = use_signal(|| true);
    use_future(move || async move {
        let current = (controller.credentials)();
        match wallet::credentials(current.checkout, current.environment).await {
            Ok(c) => {
                controller.credentials.set(c);
                if controller
                    .data
                    .peek()
                    .as_ref()
                    .is_some_and(|d| d.recover_session)
                {
                    let _ = wallet::refresh().await;
                }
                ready.set(true);
            }
            Err(e) => controller.message.set(e),
        }
    });
    use_effect(move || {
        let revision = (controller.revision)();
        if !ready() {
            return;
        }
        if *first_client_read.peek() {
            first_client_read.set(false);
            if revision == 0
                && controller.credentials.peek().checkout.is_none()
                && controller
                    .data
                    .peek()
                    .as_ref()
                    .is_some_and(|d| !d.recover_session)
            {
                return;
            }
        }
        let c = (controller.credentials)();
        let page = (controller.page)();
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        spawn(async move {
            let mut result = read_pay(page.clone(), c.clone())
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if matches!(result, Err(LoadError::Unauthenticated)) && wallet::refresh().await.is_ok()
            {
                result = read_pay(page, c)
                    .await
                    .unwrap_or(Err(LoadError::Unavailable));
            }
            if ticket != *generation.peek() {
                return;
            }
            match result {
                Ok(data) => {
                    if let Some(p) = &data.payment {
                        if p.terminal() {
                            let _ = wallet::finish_checkout(&p.checkout_id).await;
                        }
                    }
                    if [
                        LoadError::InvalidQuery,
                        LoadError::Unauthenticated,
                        LoadError::Forbidden,
                        LoadError::Unavailable,
                        LoadError::Malformed,
                        LoadError::NotFound,
                    ]
                    .iter()
                    .any(|e| e.message() == controller.message.peek().as_str())
                    {
                        controller.message.set(String::new());
                    }
                    controller.data.set(Some(data));
                }
                Err(e) => controller.message.set(e.message().into()),
            }
        });
    });
    // Scoped future is cancelled when the route unmounts; no detached timers or
    // global listeners accumulate across navigation.
    use_future(move || async move {
        loop {
            if wallet::pause().await.is_err() {
                break;
            }
            if !*(controller.busy).peek() {
                controller.revision += 1;
            }
        }
    });
    let PayTheme(mut dark) = use_context::<PayTheme>();
    let current = data();
    let signed_in = current.as_ref().is_some_and(|v| v.signed_in);
    if matches!(page, Page::Checkout(_)) {
        return rsx! {div{class:if dark(){"dark epsx-checkout"}else{"epsx-checkout"},Checkout{dark}}};
    }
    rsx! {
        document::Title{"{page.title()} · EPSX Pay"}
        div{class:if dark(){"dark epsx-merchant"}else{"epsx-merchant"},
            main{class:if page.public(){"md-app md-public"}else{"md-app"},
                aside{class:"md-sidebar",Link{class:"md-brand",to:format!("/?environment={}",environment.as_str()),img{src:"/brand-icon.svg",alt:""}"EPSX" small{"Pay"}}
                    if !page.public(){nav{for (href,label) in [("/dashboard","Overview"),("/payments","Payments"),("/packages","Packages"),("/payment-links","Payment links"),("/webhooks","Webhooks"),("/settings","Settings")]{Link{to:format!("{href}?environment={}",environment.as_str()),class:if label==page.title(){"active"}else{""},"{label}"}}}}
                    div{class:"md-sidebar-bottom",Link{to:format!("/docs?environment={}",environment.as_str()),"Developer guide ↗"}p{"Crypto payments, simply."}}
                }
                div{class:"md-workspace",
                    header{class:"md-topbar",span{if let Some(m)=current.as_ref().and_then(|d|d.merchant.as_ref()){"{m.name}"}else{"Your business"}}
                        div{class:"md-tools",
                            if !page.public(){EnvironmentSelect{path,environment}}
                            button{class:"md-quiet",onclick:move |_|controller.revision+=1,"Refresh"}
                            button{class:"md-quiet","aria-label":"Toggle theme",onclick:move |_|dark.toggle(),"◐"}
                            if signed_in{button{class:"md-quiet",disabled:(controller.busy)(),onclick:move |_|{spawn(async move{match wallet::logout().await{Ok(_)=>controller.revision+=1,Err(e)=>controller.message.set(e)}});},"Sign out"}}
                            else if !page.public(){SignIn{}}
                        }
                    }
                    div{class:"md-content",
                        div{class:"md-heading",div{p{class:"md-eyebrow","YOUR MERCHANT WORKSPACE"}h1{"{page.title()}"}p{class:"md-muted","A clear view of every payment, from checkout to your wallet."}}
                            if let Some(m)=current.as_ref().and_then(|v|v.merchant.as_ref()){Link{class:"md-secondary",to:format!("/m/{}?environment={}",m.merchant_id,environment.as_str()),"View storefront ↗"}}
                        }
                        Feedback{}
                        if let Some(data)=current{
                            if !page.public()&&data.merchant.is_none()&&!matches!(page,Page::Payment(_)|Page::NativeIntent(_)|Page::NativeDashboard){Onboarding{signed_in}}
                            else{match page{
                                Page::Dashboard=>rsx!{OverviewPanel{data:data.clone()}PaymentRows{payments:data.payments}},
                                Page::Payments=>rsx!{PaymentRows{payments:data.payments}},
                                Page::NativeDashboard=>rsx!{if signed_in{LinkEditor{}PaymentRows{payments:data.payments}}else{SignIn{}}},
                                Page::Packages|Page::EditPackage(_)=>rsx!{ProductEditor{product:if matches!(page,Page::EditPackage(_)){data.products.first().cloned()}else{None},config:data.config.clone(),environment}Products{products:data.products,config:data.config,environment,public:false}},
                                Page::Store(_)|Page::Product(_)=>rsx!{Products{products:data.products,config:data.config,environment,public:true}},
                                Page::Links=>rsx!{LinkEditor{}Links{links:data.links,config:data.config,environment}},
                                Page::Settings=>rsx!{Settings{merchant:data.merchant.unwrap_or_default(),keys:data.keys}},
                                Page::Webhooks=>rsx!{Webhooks{webhooks:data.webhooks,deliveries:data.deliveries}},
                                Page::Payment(_)|Page::NativeIntent(_)=>rsx!{if let Some(payment)=data.payment{PaymentDetail{payment}}},
                                Page::Link(_)|Page::NativeLink(_)=>rsx!{if let Some(link)=data.link{section{class:"md-panel",h2{"{link.description}"}p{"{display_amount(&link.amount,data.config.decimals(environment,&link.token).unwrap_or(0))} {link.token}"}button{class:"md-primary",disabled:(controller.busy)(),onclick:move |_|controller.dispatch(if matches!((controller.page)(),Page::NativeLink(_)){Action::NativeRedeem{id:link.id.clone()}}else{Action::RedeemLink{id:link.id.clone()}}),"Continue to checkout"}}}},
                                Page::Checkout(_)=>rsx!{},
                            }}
                        }else{button{class:"md-secondary",onclick:move |_|controller.revision+=1,"Try again"}}
                        Secret{}
                    }
                }
            }
        }
    }
}
#[component]
fn EnvironmentSelect(path: String, environment: Environment) -> Element {
    let navigator = use_navigator();
    rsx! {select{"aria-label":"Environment",value:environment.as_str(),onchange:move|e|{navigator.push(format!("{path}?environment={}",if e.value()=="live"{"live"}else{"test"}));},option{value:"test",selected:environment==Environment::Test,"Test environment"}option{value:"live",selected:environment==Environment::Live,"Live environment"}}}
}
#[component]
fn SignIn() -> Element {
    let mut c = use_context::<Controller>();
    rsx! {button{class:"md-primary",disabled:(c.busy)(),onclick:move |_|{if *(c.busy).peek(){return}c.busy.set(true);spawn(async move{match wallet::sign_in().await{Ok(_)=>{c.message.set("Signed in.".into());c.revision+=1;},Err(e)=>c.message.set(e)}c.busy.set(false);});},"Connect MetaMask"}}
}
#[component]
fn Feedback() -> Element {
    let c = use_context::<Controller>();
    rsx! {p{class:"md-feedback",role:"status","aria-live":"polite","{(c.message)()}"}}
}
#[component]
fn Secret() -> Element {
    let mut c = use_context::<Controller>();
    rsx! {if let Some(secret)=(c.secret)(){section{class:"md-panel",h2{"Save this secret now"}p{"Shown once. Store it securely in your server configuration."}code{style:"overflow-wrap:anywhere","{secret}"}button{class:"md-secondary",onclick:move |_|c.secret.set(None),"I saved it"}}}}
}
#[component]
fn Field(
    label: String,
    value: Signal<String>,
    #[props(default="text".into())] kind: String,
) -> Element {
    rsx! {label{class:"md-field",span{"{label}"}input{r#type:kind,value:value(),oninput:move|e|value.set(e.value())}}}
}
#[component]
fn Onboarding(signed_in: bool) -> Element {
    let c = use_context::<Controller>();
    let name = use_signal(String::new);
    rsx! {section{class:"md-panel",h2{"Your wallet. Your business."}p{class:"md-muted","Connect MetaMask and sign in. Choose a shop name to start accepting payments."}if signed_in{Field{label:"Shop name",value:name}button{class:"md-primary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::Register{name:name()}),"Create shop"}}else{SignIn{}}}}
}
#[component]
fn OverviewPanel(data: PageData) -> Element {
    rsx! {div{class:"md-stats",if data.overview.is_empty(){p{"Your first payment starts here. Create a package to get going."}}for v in data.overview{for (label,value)in[("Customer payments confirmed",v.paid),("Ready to collect · after fee",v.ready),("Processing fees · estimated until collection",v.fees)]{div{class:"md-stat",small{"{label}"}strong{"{display_amount(&value,v.decimals)} {v.token}"}}}}section{class:"md-panel md-hero",h2{"Turn a great idea into your next sale."}p{"Create a package or share a payment link."}div{class:"md-tools",Link{class:"md-primary",to:"/packages","Create a package"}Link{class:"md-secondary",to:"/payment-links","Create a payment link"}}}}}
}
#[component]
fn PaymentRows(payments: Vec<Payment>) -> Element {
    rsx! {section{class:"md-panel",h2{"Payment activity"}if payments.is_empty(){p{"No payments yet."}}for p in payments{div{class:"md-payment-row",div{strong{"{p.checkout_snapshot.item_name}"}small{"{p.status} · {p.settlement_status}"}}strong{"{display_amount(&p.amount,p.token_decimals.unwrap_or(0))} {p.token}"}Link{class:"md-secondary",to:format!("/{}/{}",if p.id.starts_with("pi_"){"payments"}else{"checkout"},p.id),"View payment →"}}}}}
}
#[component]
fn Products(
    products: Vec<Product>,
    config: Config,
    environment: Environment,
    public: bool,
) -> Element {
    let c = use_context::<Controller>();
    rsx! {section{class:"md-panel",h2{if public{"Choose a package"}else{"Your catalog"}}div{class:"md-products",if products.is_empty(){p{"No packages available yet."}}for p in products{article{class:"md-product",h3{"{p.name}"}p{"{p.description}"}p{if let Some(days)=p.duration_days{"{days} days of service · one-time payment"}else{"One-time payment · no automatic renewal"}}for(token,amount)in p.prices{strong{"{display_amount(&amount,config.decimals(environment,&token).unwrap_or(0))} {token}"}if public{button{class:"md-primary",disabled:(c.busy)(),onclick:{let id=p.id.clone();move |_|c.dispatch(Action::BuyProduct{id:id.clone(),token:token.clone()})},"Pay"}}}if !public{Link{class:"md-secondary",to:format!("/packages/{}/edit?environment={}",p.id,environment.as_str()),"Edit package"}}}}}}}
}
#[component]
fn ProductEditor(product: Option<Product>, config: Config, environment: Environment) -> Element {
    let p = product.unwrap_or_default();
    let editing = !p.id.is_empty();
    let id = p.id.clone();
    let name = use_signal(|| p.name);
    let description = use_signal(|| p.description);
    let duration = use_signal(|| p.duration_days.map(|d| d.to_string()).unwrap_or_default());
    let usdt = use_signal(|| {
        p.prices
            .get("USDT")
            .map(|v| display_amount(v, config.decimals(environment, "USDT").unwrap_or(0)))
            .unwrap_or_default()
    });
    let usdc = use_signal(|| {
        p.prices
            .get("USDC")
            .map(|v| display_amount(v, config.decimals(environment, "USDC").unwrap_or(0)))
            .unwrap_or_default()
    });
    let mut enabled = use_signal(|| !editing || p.enabled);
    let mut c = use_context::<Controller>();
    rsx! {section{class:"md-panel",h2{if editing{"Edit package"}else{"Create a package"}}form{onsubmit:move|e|{e.prevent_default();let duration_days=if duration().trim().is_empty(){None}else{match duration().parse(){Ok(v)=>Some(v),Err(_)=>{c.message.set("Enter a whole number of days.".into());return}}};let prices=[("USDT",usdt()),("USDC",usdc())].into_iter().filter(|(_,v)|!v.trim().is_empty()).map(|(k,v)|(k.into(),v.trim().into())).collect::<BTreeMap<_,_>>();c.dispatch(Action::SaveProduct{id:editing.then(||id.clone()),input:ProductInput{name:name(),description:description(),prices,duration_days,enabled:enabled()}});},div{class:"md-form-grid",Field{label:"Package name",value:name}Field{label:"Service duration in days (optional)",value:duration,kind:"number"}Field{label:"USDT price (optional)",value:usdt}Field{label:"USDC price (optional)",value:usdc}}Field{label:"Description and service terms",value:description}label{class:"md-field",span{"Availability"}select{value:if enabled(){"true"}else{"false"},onchange:move|e|enabled.set(e.value()=="true"),option{value:"true","Open for sale"}option{value:"false","Hidden from sale"}}}p{class:"md-muted","One-time payment. Existing checkouts keep their original terms."}button{class:"md-primary",r#type:"submit",disabled:(c.busy)(),"Save package"}}}}
}
#[component]
fn LinkEditor() -> Element {
    let mut c = use_context::<Controller>();
    let description = use_signal(String::new);
    let amount = use_signal(String::new);
    let uses = use_signal(String::new);
    let days = use_signal(|| "30".to_string());
    let mut token = use_signal(|| "USDT".to_string());
    let mut mode = use_signal(|| "direct".to_string());
    rsx! {section{class:"md-panel",h2{"Create payment link"}form{onsubmit:move|e|{e.prevent_default();let max_uses=if uses().is_empty(){None}else{match uses().parse::<u32>(){Ok(v)=>Some(v),Err(_)=>{c.message.set("Enter valid maximum payments.".into());return}}};let expires_in=match days().parse::<u32>().ok().and_then(|v|v.checked_mul(86400)){Some(v)=>v,None=>{c.message.set("Enter valid expiry days.".into());return}};let input=LinkInput{mode:mode(),token:token(),amount:amount(),description:description(),max_uses,expires_in};c.dispatch(if matches!((c.page)(),Page::NativeDashboard){Action::NativeCreateLink(input)}else{Action::CreateLink(input)});},div{class:"md-form-grid",Field{label:"Item name",value:description}Field{label:"Amount",value:amount}Field{label:"Maximum payments (optional)",value:uses,kind:"number"}Field{label:"Link expires in days",value:days,kind:"number"}label{class:"md-field",span{"Token"}select{value:token(),onchange:move|e|token.set(e.value()),for value in ["USDT","USDC","BNB"]{option{value,"{value}"}}}}label{class:"md-field",span{"Settlement"}select{value:mode(),onchange:move|e|mode.set(e.value()),option{value:"direct","Direct · 0.5%"}option{value:"escrow","Escrow · 1% on release"}}}}button{class:"md-primary",r#type:"submit",disabled:(c.busy)(),"Create link"}}}}
}
#[component]
fn Links(links: Vec<PaymentLink>, config: Config, environment: Environment) -> Element {
    let c = use_context::<Controller>();
    rsx! {section{class:"md-panel",h2{"Your payment links"}if links.is_empty(){p{"No records yet."}}for link in links{div{class:"md-payment-row",span{"{link.description} · {display_amount(&link.amount,config.decimals(environment,&link.token).unwrap_or(0))} {link.token} · {link.mode}"}Link{class:"md-secondary",to:format!("/r/{}?environment={}",link.id,environment.as_str()),"Open link"}if !link.disabled{button{class:"md-secondary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::DisableLink{id:link.id.clone()}),"Disable"}}}}}}
}
#[component]
fn Settings(merchant: Merchant, keys: Vec<ApiKey>) -> Element {
    let name = use_signal(|| merchant.name);
    let key_name = use_signal(String::new);
    let c = use_context::<Controller>();
    rsx! {section{class:"md-panel",h2{"Shop details"}Field{label:"Shop name",value:name}p{class:"md-muted","Owner and receiving wallet"}code{"{merchant.wallet}"}button{class:"md-primary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::Profile{name:name()}),"Save shop name"}}section{class:"md-panel",h2{"Server API keys"}p{"Keep keys and signing secrets on your server."}Field{label:"Key name",value:key_name}button{class:"md-primary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::CreateKey{name:key_name()}),"Create API key"}for key in keys{div{class:"md-payment-row",span{"{key.name} · {key.prefix}"}if !key.revoked{button{class:"md-secondary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::RevokeKey{id:key.id.clone()}),"Revoke"}}else{span{"Revoked"}}}}}}
}
#[component]
fn Webhooks(webhooks: Vec<Webhook>, deliveries: Vec<Delivery>) -> Element {
    let url = use_signal(String::new);
    let c = use_context::<Controller>();
    rsx! {section{class:"md-panel",h2{"Keep your business in sync"}Field{label:"HTTPS endpoint URL",value:url,kind:"url"}button{class:"md-primary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::CreateWebhook{url:url()}),"Add endpoint"}for w in webhooks{div{class:"md-payment-row",span{"{w.url}"}if w.enabled{button{class:"md-secondary",disabled:(c.busy)(),onclick:{let id=w.id.clone();move |_|c.dispatch(Action::RotateWebhook{id:id.clone()})},"Rotate secret"}button{class:"md-secondary",disabled:(c.busy)(),onclick:{let id=w.id.clone();move |_|c.dispatch(Action::DisableWebhook{id:id.clone()})},"Disable"}}else{button{class:"md-secondary",disabled:(c.busy)(),onclick:{let id=w.id.clone();move |_|c.dispatch(Action::EnableWebhook{id:id.clone()})},"Enable"}}button{class:"md-secondary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::ReplaceWebhook{id:w.id.clone(),url:url()}),"Replace with URL above"}}}section{class:"md-panel",h2{"Delivery history"}for d in deliveries{div{class:"md-payment-row",span{"{d.event_id} · {d.status} · attempts {d.attempts}"}button{class:"md-secondary",disabled:(c.busy)(),onclick:{let id=d.id.clone();move |_|c.dispatch(Action::ReplayDelivery{id:id.clone()})},"Replay"}button{class:"md-secondary",disabled:(c.busy)(),onclick:move |_|c.dispatch(Action::DeliveryDetails{id:d.id.clone()}),"Attempts"}}}}}}
}

#[component]
fn PaymentDetail(payment: Payment) -> Element {
    rsx! {section{class:"md-panel",h2{"{payment.description}"}dl{dt{"Amount"}dd{"{display_amount(&payment.amount,payment.token_decimals.unwrap_or(0))} {payment.token}"}dt{"Recipient"}dd{style:"overflow-wrap:anywhere","{payment.payee}"}dt{"Payment"}dd{"{payment.status}"}dt{"Collection"}dd{"{payment.settlement_status}"}}OperationButtons{payment}}}
}
#[component]
fn OperationButtons(payment: Payment) -> Element {
    let mut c = use_context::<Controller>();
    rsx! {div{class:"md-tools",for (kind,label)in[(OperationKind::Pay,"Pay with wallet"),(OperationKind::Deposit,"Deposit into escrow"),(OperationKind::Release,"Release funds"),(OperationKind::Refund,"Refund full amount"),(OperationKind::Dispute,"Open dispute"),(OperationKind::Collect,"Settle to merchant wallet")]{if payment.available_actions.iter().any(|a|a==kind.as_str()){
        button{class:"md-primary",disabled:(c.busy)(),onclick:{let payment=payment.clone();move |_|{if *(c.busy).peek(){return}c.busy.set(true);c.message.set("Review and confirm the transaction in your wallet.".into());let payment=payment.clone();spawn(async move{let result=async{
            let payer=wallet::connect(payment.chain_id,false).await?;let checkout=matches!((c.page)(),Page::Checkout(_));let id=if checkout{payment.checkout_id.clone()}else{payment.id.clone()};let native=matches!((c.page)(),Page::NativeIntent(_));let prepare=if native{Action::NativePrepare{id,kind}}else{Action::PrepareOperation{id,checkout,kind,payer}};let request_context=serde_json::to_string(&((c.credentials)().environment,&prepare)).map_err(|e|e.to_string())?;let op=c.perform(prepare).await?;
            let tx=op.transaction_parameters.ok_or("Transaction unavailable")?;let hash=wallet::send(tx,format!("epsx.merchant.tx.{}",op.id),op.approval_transaction).await?;
            c.message.set(format!("Transaction submitted: {hash}. Waiting for chain confirmations…"));
            c.perform(if native{Action::NativeConfirm{id:op.id.clone(),tx_hash:hash}}else{Action::ConfirmOperation{id:op.id.clone(),tx_hash:hash}}).await?;
            for _ in 0..120{wallet::pause().await?;let state=c.perform(if native{Action::NativeRead{id:op.id.clone()}}else{Action::ReadOperation{id:op.id.clone()}}).await?;if state.status=="confirmed"{wallet::complete(&request_context).await?;return Ok("Transaction confirmed on chain.".to_string())}
            if state.status=="failed"{wallet::complete(&request_context).await?;return Err("Transaction failed. No successful payment was recorded.".to_string())}}
            Ok::<_,String>("Still confirming. Reopen this checkout safely later.".into())
        }.await;c.message.set(result.unwrap_or_else(|e|e));c.busy.set(false);c.revision+=1;});}},"{label}"}
    }}}}
}
#[component]
fn Checkout(mut dark: Signal<bool>) -> Element {
    let mut c = use_context::<Controller>();
    let mut method_wallet = use_signal(|| false);
    let mut address = use_signal(String::new);
    let mut connecting = use_signal(|| false);
    let mut pairing = use_signal(String::new);
    use_future(move || async move {
        loop {
            if wallet::pause().await.is_err() {
                break;
            }
            if connecting() {
                if let Ok(uri) = wallet::pairing().await {
                    pairing.set(uri)
                }
            }
        }
    });
    let data = (c.data)();
    let origin = data
        .as_ref()
        .map(|d| d.frontend_origin.clone())
        .unwrap_or_default();
    let p = data.and_then(|d| d.payment);
    rsx! {document::Title{"Checkout · EPSX Pay"}div{class:"pay-shell",header{class:"pay-header",a{class:"pay-brand",href:origin.clone(),img{class:"pay-brand-icon",src:"/brand-icon.svg",alt:""}"EPSX" small{"Pay"}}div{class:"pay-header-actions",button{class:"pay-theme-toggle","aria-label":"Toggle theme",onclick:move |_|dark.toggle(),"◐"}a{class:"pay-back",href:format!("{origin}/plans"),"← Back to plans"}}}
        if let Some(payment)=p{div{class:"pay-layout",section{class:"pay-summary",p{class:"pay-eyebrow","{payment.checkout_snapshot.merchant_name}"}h1{"A simple way to pay."}div{class:"pay-total","{display_amount(&payment.amount,payment.token_decimals.unwrap_or(0))} {payment.token}"}
            if payment.checkout_snapshot.pricing.promotion_active{div{class:"pay-sale",div{class:"pay-sale-row",span{"Regular price"}del{"{payment.checkout_snapshot.pricing.original_price} {payment.token}"}}div{class:"pay-sale-row pay-sale-saving",span{"Sale applied"}strong{"Save {payment.checkout_snapshot.pricing.savings} {payment.token}"}}p{"One-time payment · price reserved for this checkout"}}}
            p{class:"pay-subtitle","One-time crypto payment. No automatic renewal."}div{class:"pay-item",span{class:"pay-item-icon","↗"}div{strong{"{payment.description}"}p{"{payment.checkout_snapshot.description}"}}}p{class:"pay-note",if payment.checkout_snapshot.kind=="epsx_plan"{"Payment confirmation and plan access are tracked separately. Check both in your EPSX account."}else{"Keep this checkout link as your receipt. Your merchant provides the purchased service after payment confirmation."}}
            div{class:"pay-steps",for(label,n)in[("Send the exact amount from your wallet",1),("We verify payment on the network",2),("Your merchant receives payment confirmation",3)]{div{class:"pay-step",span{"{n}"}"{label}"}}}
        }
        section{class:"pay-card","aria-label":"Crypto checkout",div{class:"pay-card-head",h2{"Pay with crypto"}p{"Scan a QR code or connect your wallet."}}
            if payment.terminal(){div{class:"pay-result",div{class:"pay-result-icon",if payment.status=="succeeded"{"✓"}else{"!"}}h3{match payment.status.as_str(){"succeeded"=>"Payment received","expired"=>"Checkout expired","refunded"=>"Payment refunded",_=>"Payment needs review"}}p{if payment.status=="expired"{"Do not send funds to this address. Start a new checkout."}else if payment.status=="succeeded"{"Payment confirmed on the network. Contact your merchant for service delivery."}else{"Contact support with your payment reference."}}if payment.checkout_snapshot.kind=="epsx_plan"{a{class:"pay-button",href:format!("{origin}/account/payments"),"View plan access"}}else{Link{class:"pay-button",to:format!("/m/{}?environment={}",payment.merchant_id,payment.environment.as_str()),"Back to merchant"}}if let Some(hash)=&payment.tx_hash{p{class:"pay-transaction","{hash}"}}}}
            else if payment.payment_method=="transfer"{div{class:"pay-card-body",if payment.environment==Environment::Test{span{class:"pay-test","TEST PAYMENT · SIMULATED FUNDS"}}div{class:"pay-network-row",div{class:"pay-asset",span{class:"pay-coin","₮"}div{strong{"{payment.token}"}small{"Chain {payment.chain_id}"}}}Expiry{expires:payment.expires_at.clone()}}
                div{class:"pay-methods","aria-label":"Payment method",button{"aria-pressed":(!method_wallet()).to_string(),onclick:move |_|method_wallet.set(false),"QR / Transfer"}button{"aria-pressed":method_wallet().to_string(),onclick:move |_|method_wallet.set(true),"Connect wallet"}}
                if !method_wallet(){div{class:"pay-qr-wrap",img{class:"pay-qr",src:svg_uri(&payment.qr_svg),alt:"Payment QR with token, network, amount and recipient"}span{class:"pay-qr-caption","Scan with a compatible crypto wallet"}}CopyField{label:"Amount to send",text:display_amount(&payment.amount,payment.token_decimals.unwrap_or(0))}CopyField{label:"Payment address · unique to this checkout",text:payment.deposit_address.clone()}}
                else{
                    if address().is_empty(){div{class:"pay-wallet-options",for (wc,label)in[(false,"MetaMask"),(true,"WalletConnect")]{button{class:"pay-wallet-option",disabled:connecting()||(c.busy)(),onclick:move |_|{connecting.set(true);spawn(async move{match wallet::connect(payment.chain_id,wc).await{Ok(a)=>address.set(a),Err(e)=>c.message.set(e)}connecting.set(false);pairing.set(String::new());});},"{label}"}}}
                        if connecting(){p{"Approve the connection in your wallet."}if !pairing().is_empty(){img{class:"pay-qr",src:pairing_qr(&pairing()),alt:"WalletConnect pairing QR — not a payment QR"}}button{class:"pay-wallet-option",onclick:move |_|{spawn(async move{let _=wallet::disconnect().await;connecting.set(false);pairing.set(String::new());});},"Cancel connection"}}
                    }else{div{class:"pay-field",label{"Connected wallet"}code{class:"pay-wallet-address","{address()}"}}button{class:"pay-wallet-change",disabled:(c.busy)(),onclick:move |_|{spawn(async move{let _=wallet::disconnect().await;address.set(String::new());});},"Disconnect"}
                        button{class:"pay-button",disabled:(c.busy)(),onclick:{let payment=payment.clone();move |_|{if *(c.busy).peek(){return}c.busy.set(true);c.message.set("Review and confirm the transfer in your wallet.".into());let payment=payment.clone();spawn(async move{let result=async{let prepared=c.perform(Action::PrepareTransfer{id:payment.checkout_id.clone(),payer:address()}).await?;let current=prepared.payment.ok_or("Checkout unavailable")?;if current.terminal(){return Err("Checkout is no longer payable. Refresh for its status.".into())}let tx=prepared.transaction_parameters.ok_or("Transaction unavailable")?;let hash=wallet::send(tx,format!("epsx.checkout.transfer.{}",payment.checkout_id),None).await?;Ok::<_,String>(format!("Transaction submitted: {hash}. Waiting for verified payment."))}.await;c.message.set(result.unwrap_or_else(|e|e));c.busy.set(false);c.revision+=1;});}},"Pay"}
                    }
                }
                p{class:"pay-instruction","Send exactly the amount shown on chain {payment.chain_id}. Network fees are paid separately."}
                if payment.chain_id==31337{p{class:"pay-instruction","Local test network: use a wallet on this Mac Mini. A phone cannot reach this chain."}}
            }}else{div{class:"pay-card-body",OperationButtons{payment:payment.clone()}}}
            div{class:"pay-status",role:"status",span{class:"pay-status-dot"}span{"{payment.status}"}}
        }}}
        else{p{class:"pay-feedback","Preparing your checkout…"}button{class:"pay-button",onclick:move |_|c.revision+=1,"Retry"}}
        Feedback{}footer{class:"pay-footer",span{"Payments by EPSX"}div{class:"pay-footer-links",a{href:format!("{origin}/contact"),"Support"}a{href:format!("{origin}/terms"),"Terms"}a{href:format!("{origin}/privacy"),"Privacy"}}}
    }}
}
fn pairing_qr(uri: &str) -> String {
    qrcode::QrCode::new(uri.as_bytes())
        .map(|code| {
            svg_uri(
                &code
                    .render::<qrcode::render::svg::Color>()
                    .min_dimensions(240, 240)
                    .build(),
            )
        })
        .unwrap_or_default()
}
fn svg_uri(svg: &str) -> String {
    format!(
        "data:image/svg+xml,{}",
        url::form_urlencoded::byte_serialize(svg.as_bytes())
            .collect::<String>()
            .replace('+', "%20")
    )
}
#[component]
fn CopyField(label: String, text: String) -> Element {
    let mut c = use_context::<Controller>();
    rsx! {div{class:"pay-field",label{"{label}"}div{class:"pay-copy-row",code{"{text}"}button{onclick:move |_|{let text=text.clone();spawn(async move{c.message.set(match wallet::copy(&text).await{Ok(_)=>"Copied.".into(),Err(e)=>e});});},"Copy"}}}}
}
#[component]
fn Docs(merchant: bool, environment: Environment) -> Element {
    let source = if merchant {
        include_str!("../../../../../../docs/pay/merchant-dashboard-dev.md")
    } else {
        include_str!("../../../../../../docs/pay/merchant-api.md")
    };
    let mut html = String::new();
    pulldown_cmark::html::push_html(
        &mut html,
        pulldown_cmark::Parser::new_ext(source, pulldown_cmark::Options::ENABLE_TABLES),
    );
    rsx! {document::Title{"Integration guide · EPSX Pay"}main{class:"container-x max-w-5xl mx-auto py-10 prose",Link{to:format!("/?environment={}",environment.as_str()),"Back to Pay"}article{dangerous_inner_html:html}}}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_cover_native_and_merchant_pages() {
        for path in [
            "/dashboard",
            "/payments",
            "/packages",
            "/packages/pkg_a/edit",
            "/packages/pkg_a",
            "/payment-links",
            "/settings",
            "/webhooks",
            "/m/m_a",
            "/checkout/cs_a",
            "/payments/pi_a",
            "/r/plink_a",
            "/r/old",
            "/intent/old",
        ] {
            assert!(page_for(path).is_some(), "{path}");
        }
        assert!(page_for("/bogus").is_none());
        assert!(super::super::known_page_path("/"));
        assert!(
            page_for("/").is_none(),
            "Home must not use the payment data loader"
        );
        assert_eq!(page_for("/dashboard"), Some(Page::Dashboard));
    }
    #[test]
    fn landing_and_workspace_share_environment_selection() {
        assert_eq!(environment_for(""), Environment::Test);
        assert_eq!(environment_for("environment=test"), Environment::Test);
        assert_eq!(environment_for("environment=invalid"), Environment::Test);
        assert_eq!(environment_for("environment=live"), Environment::Live);
        assert_eq!(
            environment_for("other=value&environment=live"),
            Environment::Live
        );
    }
    #[test]
    fn units_remain_exact() {
        assert_eq!(display_amount("5125000000000000000", 18), "5.125");
        assert_eq!(display_amount("1", 6), "0.000001");
    }
}

#[component]
fn Expiry(expires: String) -> Element {
    let mut remaining = use_signal(|| None::<u64>);
    use_future(move || {
        let expires = expires.clone();
        async move {
            loop {
                if let Ok(seconds) = wallet::remaining(&expires).await {
                    remaining.set(Some(seconds));
                }
                if wallet::pause().await.is_err() {
                    break;
                }
            }
        }
    });
    rsx! {span{class:"pay-timer",if let Some(seconds)=remaining(){"{seconds/60:02}:{seconds%60:02} remaining"}else{"Time remaining"}}}
}
