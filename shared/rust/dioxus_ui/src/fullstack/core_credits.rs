//! Canonical EPSX credit balances and commands; all ledger rules live in backend.
use super::LoadError;
use crate::pages::account_credits::{CreditBalanceProjection, CreditHistoryProjection};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub total_credits_outstanding: String,
    pub total_credits_granted_today: String,
    pub total_credits_used_today: String,
    pub active_users_with_credits: i64,
    pub total_transactions_today: i64,
    pub average_balance: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Detail {
    pub balance: CreditBalanceProjection,
    pub history: CreditHistoryProjection,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub stats: Stats,
    pub detail: Option<Detail>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub wallet: String,
    pub amount: String,
    pub reason: String,
    pub grant: bool,
    pub idempotency_key: String,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct Provider {
    pub read: std::sync::Arc<dyn Fn(Option<String>, http::HeaderMap) -> Future<Data> + Send + Sync>,
    pub command: std::sync::Arc<dyn Fn(Command, http::HeaderMap) -> Future<()> + Send + Sync>,
}
#[server(prefix = "/_server/admin", endpoint = "core_credits_read")]
pub async fn read_credits(
    wallet: Option<String>,
) -> Result<Result<Data, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<Provider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((provider.read)(wallet, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "core_credits_command")]
pub async fn command_credit(command: Command) -> Result<Result<(), LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<Provider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((provider.command)(command, headers).await)
}
#[component]
pub fn CoreCredits(query: String) -> Element {
    let wallet = url::form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == "wallet")
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.is_empty());
    let mut initial = use_server_future(use_reactive!(|wallet| read_credits(wallet)))?;
    let data = initial
        .read()
        .clone()
        .and_then(Result::ok)
        .unwrap_or(Err(LoadError::Unavailable));
    let mut pending = use_signal(|| false);
    let mut result = use_signal(|| None::<Result<(), LoadError>>);
    let mut previous = use_signal(|| None::<Command>);
    let nav = use_navigator();
    let execute = move |event: FormEvent| {
        event.prevent_default();
        if pending() {
            return;
        }
        let values = event.values();
        let field = |name: &str| {
            values
                .iter()
                .find(|(key, _)| key == name)
                .and_then(|(_, v)| match v {
                    dioxus::html::FormValue::Text(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let mut command = Command {
            wallet: field("wallet"),
            amount: field("amount"),
            reason: field("reason"),
            grant: field("action") == "grant",
            idempotency_key: String::new(),
        };
        if let Some(mut last) = previous() {
            let key = std::mem::take(&mut last.idempotency_key);
            if command == last {
                command.idempotency_key = key;
            }
        }
        if command.idempotency_key.is_empty() {
            command.idempotency_key = uuid::Uuid::new_v4().to_string();
        }
        previous.set(Some(command.clone()));
        pending.set(true);
        spawn(async move {
            let outcome = command_credit(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if outcome.is_ok() {
                previous.set(None);
                initial.restart();
            }
            result.set(Some(outcome));
            pending.set(false);
        });
    };
    rsx! {
        document::Title { "Credits | EPSX Admin" }
        document::Link { rel: "stylesheet", href: "/public/dist/tailwind.css" }
        document::Link { rel: "stylesheet", href: "/_ui/admin.css" }
        main { class: "container-x max-w-6xl mx-auto py-10 space-y-6",
            nav { class: "flex gap-5", Link { to: "/", "Admin home" } Link { to: "/wallet-management/wallets", "Wallets & access" } Link { to: "/plans", "EPSX Plans" } }
            h1 { class: "text-3xl font-bold", "Wallet credits" }
            if let Some(outcome) = result() { match outcome {
                Ok(()) => rsx!{p{role:"status","Credit adjustment saved."}},
                Err(error) => rsx!{crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  }},
            } }
            match data {
                Err(error) => rsx!{crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  }},
                Ok(value) => rsx! {
                    dl { class: "grid gap-4 sm:grid-cols-3 rounded-xl border p-5",
                        div { dt { "Outstanding credits" } dd { "{value.stats.total_credits_outstanding}" } }
                        div { dt { "Granted today" } dd { "{value.stats.total_credits_granted_today}" } }
                        div { dt { "Used today" } dd { "{value.stats.total_credits_used_today}" } }
                        div { dt { "Wallets with credits" } dd { "{value.stats.active_users_with_credits}" } }
                        div { dt { "Transactions today" } dd { "{value.stats.total_transactions_today}" } }
                        div { dt { "Average balance" } dd { "{value.stats.average_balance}" } }
                    }
                    if let Some(detail) = value.detail {
                        section { class: "rounded-xl border p-5 space-y-3",
                            h2 { class:"font-mono break-all", "{detail.balance.wallet_address}" }
                            p { "Balance: {detail.balance.balance} · Pending: {detail.balance.pending_balance} · Available: {detail.balance.available_balance}" }
                            h3 { class:"font-semibold", "Latest 100 transactions" }
                            if detail.history.transactions.is_empty() { p { "No credit transactions." } }
                            for row in detail.history.transactions { article { key:"{row.id}", class:"border-t py-3",
                                p { "{row.created_at} · {row.tx_type} · {row.amount} credits · Balance: {row.balance_after}" }
                                p { "{row.reason.as_deref().unwrap_or(\"\")}" }
                            } }
                        }
                    }
                },
            }
            form { class:"flex gap-3", onsubmit: move |event| {
                event.prevent_default();
                let value = event.values().into_iter().find(|(k,_)| k == "wallet").and_then(|(_,v)| match v { dioxus::html::FormValue::Text(s) => Some(s), _ => None }).unwrap_or_default();
                let query = url::form_urlencoded::Serializer::new(String::new()).append_pair("wallet", &value).finish();
                nav.push(format!("/wallet-management/credits?{query}"));
            },
                input { name:"wallet", value: wallet.clone().unwrap_or_default(), maxlength:42, placeholder:"Wallet 0x…", required:true, class:"input input-bordered grow" }
                button { r#type:"submit", class:"btn btn-outline", "View balance & history" }
            }
            form { class:"grid gap-3 rounded-xl border p-5", onsubmit: execute,
                h2 { class:"text-xl font-semibold", "Adjust credits" }
                label { "Wallet", input { name:"wallet", value:wallet.unwrap_or_default(), maxlength:42, required:true, class:"input input-bordered w-full" } }
                label { "Action", select { name:"action", class:"select select-bordered w-full", option { value:"grant", "Grant credits" } option { value:"revoke", "Revoke credits" } } }
                label { "Credits", input { name:"amount", inputmode:"decimal", placeholder:"1.25", maxlength:12, required:true, class:"input input-bordered w-full" } }
                label { "Reason", input { name:"reason", maxlength:500, required:true, class:"input input-bordered w-full" } }
                button { r#type:"submit", disabled:pending(), class:"btn btn-primary", if pending() { "Saving…" } else { "Save credit adjustment" } }
            }
        }
    }
}
