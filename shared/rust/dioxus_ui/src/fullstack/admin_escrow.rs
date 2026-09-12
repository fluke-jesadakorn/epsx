//! Admin escrow UI; only the payment service authorizes or confirms operations.
use super::{frontend_auth::browser, frontend_payment::WalletTransaction, LoadError};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscrowQuery {
    pub merchant: bool,
    pub id: Option<String>,
    pub environment: String,
}
impl EscrowQuery {
    pub fn url(&self, id: Option<&str>) -> String {
        format!(
            "/pay/{}{}?environment={}",
            if self.merchant {
                "merchant-escrows"
            } else {
                "escrows"
            },
            id.map(|v| format!("/{v}")).unwrap_or_default(),
            self.environment
        )
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Escrow {
    pub id: String,
    #[serde(deserialize_with = "nullable")]
    pub description: String,
    pub amount: String,
    #[serde(alias = "token_symbol")]
    pub token: String,
    pub token_decimals: Option<u32>,
    pub payee: String,
    pub status: String,
    pub available_actions: Vec<String>,
}
fn display_amount(item: &Escrow) -> String {
    let decimals = item.token_decimals.unwrap_or(0).min(36) as usize;
    if decimals == 0 {
        return item.amount.clone();
    }
    let padded = format!("{:0>width$}", item.amount, width = decimals + 1);
    let split = padded.len() - decimals;
    let tail = padded[split..].trim_end_matches('0');
    if tail.is_empty() {
        padded[..split].into()
    } else {
        format!("{}.{}", &padded[..split], tail)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EscrowData {
    pub items: Vec<Escrow>,
    pub selected: Option<Escrow>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EscrowCommand {
    Resolve {
        id: String,
        to_payee: bool,
    },
    Pause {
        mode: String,
        paused: bool,
    },
    Confirm {
        id: String,
        control: bool,
        hash: String,
    },
    Operation {
        id: String,
        control: bool,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EscrowOperation {
    pub id: String,
    pub status: String,
    #[serde(alias = "transaction")]
    pub transaction_parameters: Option<WalletTransaction>,
    pub approval_transaction: Option<WalletTransaction>,
    pub operation: Option<OperationId>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OperationId {
    pub id: String,
}
fn nullable<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Ok(Option::<String>::deserialize(d)?.unwrap_or_default())
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct EscrowProvider {
    pub read:
        std::sync::Arc<dyn Fn(EscrowQuery, http::HeaderMap) -> Future<EscrowData> + Send + Sync>,
    pub command: std::sync::Arc<
        dyn Fn(EscrowQuery, EscrowCommand, String, http::HeaderMap) -> Future<EscrowOperation>
            + Send
            + Sync,
    >,
}
#[server(prefix = "/_server/admin", endpoint = "escrow_read")]
pub async fn escrow_read(
    query: EscrowQuery,
) -> Result<Result<EscrowData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<EscrowProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(query, h).await)
}
#[server(prefix = "/_server/admin", endpoint = "escrow_action")]
pub async fn escrow_action(
    query: EscrowQuery,
    command: EscrowCommand,
    key: String,
) -> Result<Result<EscrowOperation, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<EscrowProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(query, command, key, h).await)
}
async fn action(
    query: EscrowQuery,
    command: EscrowCommand,
    key: String,
) -> Result<EscrowOperation, String> {
    escrow_action(query, command, key)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.message().to_string())
}
#[derive(Clone, Copy)]
struct Controls {
    busy: Signal<bool>,
    status: Signal<String>,
    revision: Signal<u64>,
}
#[component]
pub fn HydratedAdminEscrows(merchant: bool, id: Option<String>, query: String) -> Element {
    let environment = url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
        .find(|(key, _)| key == "environment")
        .map(|(_, v)| v.into_owned())
        .filter(|v| matches!(v.as_str(), "test" | "live"))
        .unwrap_or_else(|| "test".into());
    let request = EscrowQuery {
        merchant,
        id,
        environment,
    };
    let first = request.clone();
    let initial = use_server_future(move || escrow_read(first.clone()))?;
    let mut data = use_signal(|| {
        initial
            .read()
            .as_ref()
            .cloned()
            .and_then(Result::ok)
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut loading = use_signal(|| false);
    let mut failure = use_signal(|| None::<String>);
    let busy = use_signal(|| false);
    let status = use_signal(String::new);
    let revision = use_signal(|| 0u64);
    use_context_provider(|| Controls {
        busy,
        status,
        revision,
    });
    let refresh = request.clone();
    use_effect(move || {
        let version = revision();
        if version == 0 {
            return;
        }
        let refresh = refresh.clone();
        loading.set(true);
        spawn(async move {
            let value = escrow_read(refresh).await;
            if version != *revision.peek() {
                return;
            }
            match value {
                Ok(Ok(value)) => {
                    data.set(Ok(value));
                    failure.set(None);
                }
                Ok(Err(error)) => failure.set(Some(error.message().into())),
                Err(_) => failure.set(Some("Escrows could not be refreshed.".into())),
            }
            loading.set(false);
        });
    });
    if let Err(error) = data() {
        crate::pages::news::hydrated::response_status(match error {
            LoadError::Unauthenticated => 401,
            LoadError::Forbidden => 403,
            LoadError::NotFound => 404,
            _ => 502,
        });
    }
    let title = if merchant {
        "Merchant escrow disputes"
    } else {
        "Pay escrow disputes"
    };
    let nav = use_navigator();
    let path = request.url(request.id.as_deref());
    rsx! {super::admin::AdminAnalyticsShell{authenticated:data().is_ok(),current_path:path,title,
    main{class:"container-x max-w-5xl mx-auto p-6 space-y-6","data-dioxus-admin-escrows":"true",
     h1{class:"text-2xl font-semibold","{title}"}
     p{"Resolve disputed escrows using the configured Admin wallet. The payment service validates every operation."}
     div{class:"flex flex-wrap gap-3",Link{class:"btn btn-outline",to:if merchant{"/pay/escrows"}else{"/pay/merchant-escrows"},if merchant{"Native escrows"}else{"Merchant escrows"}}
      if merchant{select{aria_label:"Environment",class:"input",value:request.environment.clone(),disabled:busy(),onchange:{let request=request.clone();move|event|{let mut next=request.clone();next.environment=event.value();nav.push(next.url(next.id.as_deref()));}},option{value:"test","Test"}option{value:"live","Live"}}}
      button{class:"btn btn-outline",disabled:loading(),onclick:move |_|{let mut value=revision;value+=1;},"Refresh"}
     }
     if let Some(message)=failure(){p{role:"alert","{message}"}}
     p{role:"status","aria-live":"polite","{status}"}
     match data(){Ok(value)=>rsx!{
      if let Some(item)=value.selected{section{class:"border rounded-2xl p-6 space-y-3",h2{class:"text-xl font-semibold","{item.description}"}p{"{display_amount(&item)} {item.token}"}p{class:"break-all","Recipient: {item.payee}"}p{"Status: {item.status}"}
       div{class:"flex flex-wrap gap-3",for(to_payee,label,action_name)in[(true,"Release to recipient","resolve-release"),(false,"Refund payer","resolve-refund")]{if item.available_actions.iter().any(|a|a==action_name){EscrowActionButton{query:request.clone(),command:EscrowCommand::Resolve{id:item.id.clone(),to_payee},label}}}}
      }}
      section{class:"border rounded-2xl p-6 space-y-3",h2{class:"text-xl font-semibold","New payment controls"}p{"Pausing blocks new payments and deposits. Existing escrows can still be released or refunded."}
       for mode in if merchant{vec!["direct","escrow"]}else{vec!["escrow"]}{div{class:"flex flex-wrap items-center gap-3",strong{"{mode}"}for(paused,label)in[(true,"Pause"),(false,"Resume")]{EscrowActionButton{query:request.clone(),command:EscrowCommand::Pause{mode:mode.into(),paused},label}}}}
      }
      section{class:"border rounded-2xl p-6 space-y-3",h2{class:"text-xl font-semibold","Escrows"}if value.items.is_empty(){p{"No escrows in this environment."}}for item in value.items{Link{class:"block border-b py-3",to:request.url(Some(&item.id)),"{item.description} · {item.token} · {item.status}"}}}
     },Err(error)=>rsx!{section{role:"alert",p{"{error.message()}"}Link{class:"btn btn-primary",to:format!("/auth?return_url={}",url::form_urlencoded::byte_serialize(request.url(request.id.as_deref()).as_bytes()).collect::<String>()),"Sign in"}}}
    }
    }}}
}
#[component]
fn EscrowActionButton(query: EscrowQuery, command: EscrowCommand, label: String) -> Element {
    let mut controls = use_context::<Controls>();
    rsx! {button{class:"btn btn-outline",disabled:(controls.busy)(),onclick:move |_|{
    if (controls.busy)(){return;}controls.busy.set(true);controls.status.set("Preparing operation…".into());let query=query.clone();let command=command.clone();spawn(async move{
     let result:Result<(),String>=async{
      let context=serde_json::to_string(&(&query,&command)).map_err(|e|e.to_string())?;let key:String=browser("key",&context).await?;
      let _:String=browser("connect",serde_json::json!({"chain":0,"walletconnect":false})).await?;
      let control=matches!(command,EscrowCommand::Pause{..});let op=action(query.clone(),command,key.clone()).await?;
      let id=if op.id.is_empty(){op.operation.ok_or("Operation ID missing")?.id}else{op.id};
      let transaction=op.transaction_parameters.ok_or("Transaction unavailable")?;
      controls.status.set("Confirm the operation in your wallet…".into());
      let hash:String=browser("send",serde_json::json!({"transaction":transaction,"approval":op.approval_transaction,"storage_key":format!("epsx.admin.escrow.tx.{id}")})).await?;
      controls.status.set(format!("Transaction submitted: {hash}. Waiting for service confirmation…"));
      action(query.clone(),EscrowCommand::Confirm{id:id.clone(),control,hash},key.clone()).await?;
      for _ in 0..150{let op=action(query.clone(),EscrowCommand::Operation{id:id.clone(),control},key.clone()).await?;match op.status.as_str(){"confirmed"=>{let _:bool=browser("complete",&context).await?;controls.status.set("Transaction verified on chain.".into());controls.revision+=1;return Ok(());},"failed"=>{let _:bool=browser("complete",&context).await?;return Err("Transaction failed verification. No successful resolution was recorded.".into());},_=>{let _:bool=browser("pause",()).await?;}}}
      controls.status.set("Still confirming. Retry safely to resume this operation.".into());Ok(())
     }.await;if let Err(message)=result{controls.status.set(message);}controls.busy.set(false);
    });},"{label}"}}
}
