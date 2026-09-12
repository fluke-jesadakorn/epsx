//! Browser capabilities only. No element selection, event delegation, HTML or UI
//! mutation belongs here. Dioxus owns everything visible, including pairing QR.
use super::types::*;
use dioxus::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

async fn adapter<T: DeserializeOwned>(operation: &str, input: impl Serialize) -> Result<T, String> {
    let mut eval = document::eval(include_str!("wallet_adapter.js"));
    eval.send(serde_json::json!({"operation":operation,"input":input}))
        .map_err(|e| e.to_string())?;
    #[derive(serde::Deserialize)]
    struct Reply<T> {
        value: Option<T>,
        error: Option<String>,
    }
    let reply: Reply<T> = eval.recv().await.map_err(|e| e.to_string())?;
    reply.value.ok_or_else(|| {
        reply
            .error
            .unwrap_or_else(|| "Browser request failed".into())
    })
}
pub async fn credentials(
    checkout: Option<String>,
    environment: Environment,
) -> Result<Credentials, String> {
    adapter(
        "credentials",
        Credentials {
            checkout,
            environment,
            ..Default::default()
        },
    )
    .await
}
pub async fn key(context: &str) -> Result<String, String> {
    adapter("key", context).await
}
pub async fn complete(context: &str) -> Result<bool, String> {
    adapter("complete", context).await
}
pub async fn copy(text: &str) -> Result<bool, String> {
    adapter("copy", text).await
}
pub async fn connect(chain: u64, walletconnect: bool) -> Result<String, String> {
    adapter(
        "connect",
        serde_json::json!({"chain":chain,"walletconnect":walletconnect}),
    )
    .await
}
pub async fn pairing() -> Result<String, String> {
    adapter("pairing", ()).await
}
pub async fn disconnect() -> Result<bool, String> {
    adapter("disconnect", ()).await
}
async fn auth(command: PayAuthCommand) -> Result<ActionResult, String> {
    let key = key("pay-auth").await?;
    super::act_pay(
        super::types::Action::Auth(command),
        Credentials::default(),
        key,
    )
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.message().to_owned())
}
pub async fn sign_in() -> Result<bool, String> {
    let address = connect(0, false).await?;
    let challenge = auth(PayAuthCommand::Challenge {
        address: address.clone(),
    })
    .await?
    .challenge
    .ok_or("Wallet challenge unavailable")?;
    if !challenge.address.eq_ignore_ascii_case(&address) {
        return Err("Wallet challenge does not match".into());
    }
    let signature: String = adapter("sign", &challenge).await?;
    auth(PayAuthCommand::Verify {
        address,
        message: challenge.message,
        nonce: challenge.nonce,
        signature,
    })
    .await?;
    Ok(true)
}
pub async fn refresh() -> Result<bool, String> {
    auth(PayAuthCommand::Refresh).await?;
    Ok(true)
}
pub async fn logout() -> Result<bool, String> {
    auth(PayAuthCommand::Logout).await?;
    Ok(true)
}
pub async fn send(
    tx: Transaction,
    storage_key: String,
    approval: Option<Transaction>,
) -> Result<String, String> {
    adapter(
        "send",
        serde_json::json!({"transaction":tx,"storage_key":storage_key,"approval":approval}),
    )
    .await
}
pub async fn pause() -> Result<bool, String> {
    adapter("pause", ()).await
}
pub async fn local_checkout(url: &str, context: &str) -> Result<String, String> {
    adapter("checkout", serde_json::json!({"url":url,"context":context})).await
}
pub async fn finish_checkout(id: &str) -> Result<bool, String> {
    adapter("finish_checkout", id).await
}

pub async fn theme(value: Option<bool>) -> Result<bool, String> {
    adapter("theme", value).await
}

pub async fn remaining(expires: &str) -> Result<u64, String> {
    adapter("remaining", expires).await
}
