use super::{frontend_auth::browser, LoadError};
use crate::pages::payment::{
    CheckoutContent, CheckoutErrorContent, PaymentEntryContent, PlanCheckoutData,
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentData {
    pub wallet: String,
    pub checkout: Option<PlanCheckoutData>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub from: String,
    pub to: String,
    pub data: String,
    pub value: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaymentCommand {
    Prepare {
        plan_id: String,
    },
    Hosted {
        plan_id: String,
        token: String,
        key: String,
    },
    Submit {
        plan_id: String,
        hash: String,
    },
    Status {
        hash: String,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaymentReply {
    pub transaction: Option<WalletTransaction>,
    pub pay_url: Option<String>,
    pub status: Option<String>,
    pub hash: Option<String>,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PaymentProvider {
    pub read: std::sync::Arc<
        dyn Fn(Option<String>, http::HeaderMap) -> Future<PaymentData> + Send + Sync,
    >,
    pub command: std::sync::Arc<
        dyn Fn(PaymentCommand, http::HeaderMap) -> Future<PaymentReply> + Send + Sync,
    >,
}
#[server(prefix = "/_server/frontend", endpoint = "payment_read")]
pub async fn payment_read(
    plan_id: Option<String>,
) -> Result<Result<PaymentData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PaymentProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(plan_id, h).await)
}
#[server(prefix = "/_server/frontend", endpoint = "payment_action")]
pub async fn payment_action(
    command: PaymentCommand,
) -> Result<Result<PaymentReply, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PaymentProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(command, h).await)
}
async fn command(value: PaymentCommand) -> Result<PaymentReply, String> {
    payment_action(value)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.message().to_owned())
}
#[component]
pub fn HydratedPayment(plan_id: Option<String>) -> Element {
    let return_url = format!(
        "/auth?return_url={}",
        url::form_urlencoded::byte_serialize(
            format!(
                "/payment{}",
                plan_id
                    .as_ref()
                    .map(|id| format!("/plan/{id}"))
                    .unwrap_or_default()
            )
            .as_bytes()
        )
        .collect::<String>()
    );
    let request_context = format!("frontend-plan-{}", plan_id.as_deref().unwrap_or_default());
    let mut initial = use_server_future(move || payment_read(plan_id.clone()))?;
    let mut busy = use_signal(|| false);
    let mut status = use_signal(|| None::<String>);
    let mut hash = use_signal(|| None::<String>);
    let mut done = use_signal(|| false);
    use_future(move || {
        let request_context = request_context.clone();
        async move {
            loop {
                let _: Result<bool, _> = browser("pause", ()).await;
                if !done() {
                    if let Some(value) = hash() {
                        if let Ok(reply) = command(PaymentCommand::Status { hash: value }).await {
                            let value = reply.status.unwrap_or_default();
                            status.set(Some(format!("Payment {value}")));
                            if value == "confirmed" {
                                done.set(true);
                                let _: Result<bool, _> =
                                    browser("complete", &request_context).await;
                            } else if matches!(value.as_str(), "failed" | "expired") {
                                hash.set(None);
                            }
                        }
                    }
                }
            }
        }
    });
    let data = initial.read().as_ref().cloned();
    if let Some(result) = data.as_ref() {
        let code = match result {
            Ok(Ok(_)) => 200,
            Ok(Err(LoadError::NotFound)) => 404,
            Ok(Err(LoadError::Unauthenticated)) => 401,
            Ok(Err(LoadError::Forbidden)) => 403,
            _ => 502,
        };
        if code != 200 {
            crate::pages::news::hydrated::response_status(code);
        }
    }
    rsx! {document::Title{"Payment | EPSX"}
    match data{
     Some(Ok(Ok(data)))=>rsx!{if let Some(checkout)=data.checkout {
      CheckoutContent{checkout:checkout.clone(),session_wallet:data.wallet,busy:busy()||done(),status:status(),on_pay:move |_|{
       if busy()||done(){return;}busy.set(true);status.set(Some("Preparing checkout…".into()));let checkout=checkout.clone();spawn(async move{
        let result:Result<(),String>=async{
         let key:String=browser("key",format!("frontend-plan-{}",checkout.plan.id)).await?;
         if checkout.hosted_pay{
          let context=format!("frontend-plan-{}",checkout.plan.id);
          for _ in 0..2 {
           let key:String=browser("key",&context).await?;
           let reply=command(PaymentCommand::Hosted{plan_id:checkout.plan.id.clone(),token:checkout.plan.settlement_currency.clone(),key}).await?;
           if matches!(reply.status.as_deref(),Some("succeeded"|"refunded"|"expired")){let _:bool=browser("complete",&context).await?;continue;}
           let _:bool=browser("external_checkout",reply.pay_url.ok_or("Checkout URL missing")?).await?;return Ok(());
          }
          return Err("Please retry to start a new checkout.".into());
         }else{
          let _:String=browser("connect",serde_json::json!({"chain":checkout.chain_id,"walletconnect":false})).await?;
          let tx=command(PaymentCommand::Prepare{plan_id:checkout.plan.id.clone()}).await?.transaction.ok_or("Transaction unavailable")?;
          status.set(Some("Confirm the transfer in your wallet…".into()));
          let value:String=browser("send",serde_json::json!({"transaction":tx,"storage_key":format!("epsx.frontend.tx.{key}"),"approval":null})).await?;
          hash.set(Some(value.clone()));
          let reply=command(PaymentCommand::Submit{plan_id:checkout.plan.id,hash:value}).await?;
          status.set(Some(format!("Payment {}. Waiting for verified confirmation.",reply.status.unwrap_or_else(||"pending".into()))));
         }Ok(())
        }.await;if let Err(message)=result{status.set(Some(message));}busy.set(false);
       });}
      }
     }else{PaymentEntryContent{}}},
     Some(Ok(Err(LoadError::Unauthenticated)))=>rsx!{super::load_error::SessionNotice{actions:rsx!{super::shell::ShellLink{href:return_url.clone(),class:"epsx-session-primary", "Connect wallet →"}}}},
     Some(Ok(Err(LoadError::NotFound)))=>rsx!{CheckoutErrorContent{title:"Plan not found".to_string(),body:"Choose another current plan.".to_string()}},
     Some(_)=>rsx!{section{class:"fe-state-card",role:"alert",p{"Checkout could not be loaded."}button{onclick:move |_|initial.restart(),"Try again"}}},
     None=>rsx!{p{role:"status","Loading checkout…"}},
    }
    }
}
