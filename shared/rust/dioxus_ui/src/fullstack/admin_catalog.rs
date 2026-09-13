//! Typed catalog editing. Pricing and promotion decisions remain backend-owned.
fn default_on_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

use super::{admin::AdminNavigation, LoadError};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DecimalText {
    Text(String),
    Number(serde_json::Number),
    Null(()),
}
impl Default for DecimalText {
    fn default() -> Self {
        Self::Text("0".into())
    }
}
impl std::fmt::Display for DecimalText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(value) => f.write_str(value),
            Self::Number(value) => write!(f, "{value}"),
            Self::Null(()) => f.write_str("0"),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Promotion {
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub value: DecimalText,
    #[serde(default)]
    pub price: DecimalText,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    pub duration_days: Option<i64>,
    #[serde(default)]
    pub pay_prices: BTreeMap<String, DecimalText>,
    #[serde(default)]
    pub pay_use_catalog_promotion: bool,
    #[serde(default, deserialize_with = "default_on_null")]
    pub promotion: Promotion,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: uuid::Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub is_active: bool,
    #[serde(default, deserialize_with = "default_on_null")]
    pub billing_model: String,
    #[serde(default, deserialize_with = "default_on_null")]
    pub permissions: Vec<String>,
    #[serde(default, deserialize_with = "default_on_null")]
    pub metadata: Metadata,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CatalogData {
    List(Vec<Plan>),
    Detail(Box<Plan>),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanEdit {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub usdt: String,
    pub usdc: String,
    pub use_promotion: bool,
    pub promotion_enabled: bool,
    pub promotion_type: String,
    pub promotion_value: String,
    pub promotion_price: String,
    pub promotion_start: String,
    pub promotion_end: String,
    pub duration_days: String,
    pub active: bool,
    pub permissions: String,
}
#[cfg(feature = "server")]
type ProviderFuture<T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct CatalogProvider {
    pub read: std::sync::Arc<
        dyn Fn(Option<uuid::Uuid>, http::HeaderMap) -> ProviderFuture<CatalogData> + Send + Sync,
    >,
    pub save: std::sync::Arc<dyn Fn(PlanEdit, http::HeaderMap) -> ProviderFuture<()> + Send + Sync>,
}
#[server(prefix = "/_server/admin", endpoint = "catalog_read")]
pub async fn read_catalog(
    id: Option<uuid::Uuid>,
) -> Result<Result<CatalogData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<CatalogProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Catalog unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request unavailable"))?;
    Ok((provider.read)(id, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "catalog_save")]
pub async fn save_catalog(edit: PlanEdit) -> Result<Result<(), LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<CatalogProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Catalog unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request unavailable"))?;
    Ok((provider.save)(edit, headers).await)
}
#[component]
pub fn HydratedCatalog(id: Option<String>) -> Element {
    let parsed = id.as_deref().map(uuid::Uuid::parse_str).transpose();
    let initial = use_server_future(move || {
        let parsed = parsed.clone();
        async move {
            read_catalog(parsed.map_err(|_| LoadError::NotFound)?)
                .await
                .map_err(|_| LoadError::Unavailable)?
        }
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut pending = use_signal(|| false);
    let mut saved = use_signal(|| false);
    let mut error = use_signal(|| None::<LoadError>);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| AdminNavigation(navigate));
    rsx! {
        document::Title{"EPSX Plans | EPSX Admin"}
        document::Link{rel:"stylesheet",href:"/public/dist/tailwind.css"}
        document::Link{rel:"stylesheet",href:"/_ui/admin.css"}
        main{class:"container-x max-w-5xl mx-auto py-10 space-y-6",
            nav{class:"flex gap-5",a{href:"/",class:"underline","Admin home"}Link{to:"/plans",class:"underline","EPSX Plans"}Link{to:"/payments/epsx",class:"underline","Plan purchases"}a{href:"/pay/merchant-escrows",class:"underline","Escrow disputes"}}
            h1{class:"text-3xl font-bold","EPSX Plan catalog"}
            p{class:"text-muted-foreground","These plans appear on the main EPSX website. Set regular token prices, then enable a catalog promotion for checkout. Existing orders keep their reserved price."}
            if saved(){p{role:"status",class:"text-emerald-600","Plan saved."}}
            if let Some(failure)=error(){p{role:"alert",if failure == LoadError::InvalidQuery {"Plan was not saved. Check prices, duration and permissions."} else {"{failure.message()}"}}}
            match data(){
                Ok(CatalogData::List(plans))=>rsx!{for plan in plans{article{class:"rounded-2xl border bg-card p-6 flex flex-wrap justify-between gap-5",
                    div{h2{class:"text-xl font-semibold","{plan.name}"}p{if let Some(days)=plan.metadata.duration_days{"{days} days"}else{"{plan.billing_model}"}}p{if plan.is_active{"Enabled"}else{"Disabled"}}}
                    Link{to:format!("/plans/{}",plan.id),class:"btn btn-outline","Edit plan"}
                }}},
                Ok(CatalogData::Detail(plan))=>rsx!{fieldset{disabled:pending(),aria_busy:pending(),
                    PlanForm{plan:(*plan).clone(),onsubmit:move|event:FormEvent|{
                        event.prevent_default();if *pending.peek(){return;}
                        let plan_id=plan.id;
                        let Some(edit)=parse_edit(plan_id,&event)else{error.set(Some(LoadError::InvalidQuery));return;};
                        pending.set(true);saved.set(false);error.set(None);
                        spawn(async move{
                            match save_catalog(edit).await.map_err(|_|LoadError::Unavailable).and_then(|v|v){
                                Ok(())=>{saved.set(true);match read_catalog(Some(plan_id)).await.map_err(|_|LoadError::Unavailable).and_then(|v|v){Ok(value)=>data.set(Ok(value)),Err(failure)=>error.set(Some(failure))}},
                                Err(failure)=>{if failure==LoadError::Unauthenticated{data.set(Err(failure.clone()));}error.set(Some(failure));},
                            }
                            pending.set(false);
                        });
                    }}
                }},
                Err(failure)=>rsx!{div{class:"rounded-xl border bg-card p-6 space-y-4",p{role:"status","{failure.message()}"}button{r#type:"button",class:"btn btn-outline",disabled:pending(),onclick:move |_|{
                    if *pending.peek(){return;} pending.set(true);
                    let requested = id.as_deref().map(uuid::Uuid::parse_str).transpose();
                    spawn(async move {let result = match requested {Ok(id)=>read_catalog(id).await.map_err(|_|LoadError::Unavailable).and_then(|v|v),Err(_)=>Err(LoadError::NotFound)};data.set(result);pending.set(false);});
                },"Try again"}}},
            }
        }
    }
}
fn parse_edit(id: uuid::Uuid, event: &FormEvent) -> Option<PlanEdit> {
    let values = event.values();
    let get = |key: &str| {
        values
            .iter()
            .find(|(name, _)| name == key)
            .and_then(|(_, value)| match value {
                dioxus::html::FormValue::Text(value) => Some(value.clone()),
                _ => None,
            })
    };
    Some(PlanEdit {
        id,
        name: get("name")?,
        description: get("description")?,
        usdt: get("USDT")?,
        usdc: get("USDC")?,
        use_promotion: get("pay_use_catalog_promotion")? == "true",
        promotion_enabled: get("promotion_enabled")? == "true",
        promotion_type: get("promotion_type")?,
        promotion_value: get("promotion_value")?,
        promotion_price: get("promotion_price")?,
        promotion_start: get("promotion_start_date")?,
        promotion_end: get("promotion_end_date")?,
        duration_days: get("duration_days")?,
        active: get("is_active")? == "true",
        permissions: get("permissions")?,
    })
}
#[component]
fn PlanForm(plan: Plan, onsubmit: EventHandler<FormEvent>) -> Element {
    let meta = plan.metadata;
    rsx! {form{method:"post",action:format!("/plans/{}",plan.id),onsubmit,class:"grid gap-5 rounded-2xl border bg-card p-6",
        label{class:"grid gap-2","Name",input{name:"name",value:plan.name,required:true,class:"input input-bordered"}}
        label{class:"grid gap-2","Description",super::admin_textarea::TextArea{name:"description",class:"textarea textarea-bordered",value:plan.description.unwrap_or_default()}}
        for symbol in ["USDT","USDC"]{label{class:"grid gap-2","{symbol} regular price",input{name:symbol,value:meta.pay_prices.get(symbol).map(ToString::to_string).unwrap_or_default(),required:true,inputmode:"decimal",class:"input input-bordered"}}}
        fieldset{class:"grid gap-4 rounded-xl border p-5",legend{class:"px-2 font-semibold","Sale promotion"}
            BooleanSelect{name:"pay_use_catalog_promotion",label:"Apply catalog promotion to token checkout",value:meta.pay_use_catalog_promotion}
            BooleanSelect{name:"promotion_enabled",label:"Promotion status",value:meta.promotion.enabled}
            label{class:"grid gap-2","Discount type",select{name:"promotion_type",class:"select select-bordered",option{value:"percentage",selected:meta.promotion.kind=="percentage","Percentage"}option{value:"fixed",selected:meta.promotion.kind!="percentage","Fixed amount off"}}}
            label{class:"grid gap-2","Discount value",input{name:"promotion_value",inputmode:"decimal",value:meta.promotion.value.to_string(),class:"input input-bordered"}}
            label{class:"grid gap-2","Optional sale price (overrides discount; blank to use discount)",input{name:"promotion_price",inputmode:"decimal",value:meta.promotion.price.to_string(),class:"input input-bordered"}}
            label{class:"grid gap-2","Starts at (UTC, e.g. 2026-09-10T00:00:00Z)",input{name:"promotion_start_date",value:meta.promotion.start_date.unwrap_or_default(),class:"input input-bordered"}}
            label{class:"grid gap-2","Ends at (UTC; blank for no end date)",input{name:"promotion_end_date",value:meta.promotion.end_date.unwrap_or_default(),class:"input input-bordered"}}
            p{class:"text-sm text-muted-foreground","The backend applies the sale once. Scheduled and expired promotions charge the regular price."}
        }
        label{class:"grid gap-2","Duration in days (blank for lifetime)",input{name:"duration_days",r#type:"number",min:1,max:3650,value:meta.duration_days.map(|n|n.to_string()).unwrap_or_default(),class:"input input-bordered"}}
        BooleanSelect{name:"is_active",label:"Available for new purchases",value:plan.is_active}
        label{class:"grid gap-2","Permissions (one per line)",super::admin_textarea::TextArea{name:"permissions",rows:8,class:"textarea textarea-bordered font-mono",value:plan.permissions.join("\n")}}
        button{r#type:"submit",class:"btn btn-primary","Save plan"}
    }}
}
#[component]
fn BooleanSelect(name: String, label: String, value: bool) -> Element {
    rsx! {label{class:"grid gap-2","{label}",select{name,class:"select select-bordered",option{value:"true",selected:value,"Enabled"}option{value:"false",selected:!value,"Disabled"}}}}
}
