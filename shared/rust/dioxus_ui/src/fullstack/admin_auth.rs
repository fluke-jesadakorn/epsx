//! Typed SIWE UI. Signing stays in the browser; challenge/session verification
//! and cookie changes stay in the BFF's existing authentication handlers.
use super::frontend_auth::browser;
use super::LoadError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    pub authenticated: bool,
    pub recover_session: bool,
    pub verifier_unavailable: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Challenge {
    pub address: String,
    pub message: String,
    pub nonce: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AuthCommand {
    Logout,
    Challenge {
        address: String,
    },
    Verify {
        challenge: Challenge,
        signature: String,
    },
    Refresh,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthReply {
    pub challenge: Option<Challenge>,
    pub authenticated: bool,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AuthProvider {
    pub session: std::sync::Arc<dyn Fn(http::HeaderMap) -> Future<AuthSession> + Send + Sync>,
    pub command:
        std::sync::Arc<dyn Fn(AuthCommand, http::HeaderMap) -> Future<AuthReply> + Send + Sync>,
}
#[server(prefix = "/_server/admin", endpoint = "auth_session")]
pub async fn auth_session() -> Result<Result<AuthSession, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AuthProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.session)(headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "auth_action")]
pub async fn auth_action(
    command: AuthCommand,
) -> Result<Result<AuthReply, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AuthProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(command, headers).await)
}
async fn command(value: AuthCommand) -> Result<AuthReply, String> {
    auth_action(value)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.message().to_owned())
}
#[derive(Clone, Copy)]
pub struct AdminRecoveryOwned;
#[component]
pub fn HydratedAdminAuth(query: String) -> Element {
    let destination = if url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
        .any(|(key, _)| key == "return_url")
    {
        super::frontend_auth::return_path(&query)
    } else {
        "/dashboard".into()
    };
    let navigator = use_navigator();
    let initial = use_server_future(auth_session)?;
    let mut busy = use_signal(|| false);
    let mut status = use_signal(|| None::<String>);
    let mut recovered = use_signal(|| false);
    let recovery_owned = try_consume_context::<AdminRecoveryOwned>().is_some();
    let target = destination.clone();
    use_effect(move || {
        let Some(Ok(Ok(session))) = initial.read().as_ref().cloned() else {
            return;
        };
        if session.authenticated {
            navigator.replace(target.clone());
            return;
        }
        if session.recover_session && !recovered() && !recovery_owned {
            recovered.set(true);
            busy.set(true);
            status.set(Some("Restoring your admin session…".into()));
            let target = target.clone();
            spawn(async move {
                match command(AuthCommand::Refresh).await {
                    Ok(v) if v.authenticated => {
                        navigator.replace(target);
                    }
                    _ => status.set(Some("Your session expired. Sign in again.".into())),
                }
                busy.set(false);
            });
        }
    });
    let unavailable = matches!(initial.read().as_ref(),Some(Ok(Ok(s)))if s.verifier_unavailable);
    rsx! {
     document::Title{"Admin sign in | EPSX"}
      document::Link {rel:"stylesheet",href:"/public/dist/tailwind.css"}
      document::Link {rel:"stylesheet",href:"/_ui/admin.css"}
     div {class:"dark",crate::components::admin::auth_page_overlay::AuthPageOverlay{
      return_url:destination.clone(),busy:busy()||unavailable,status:if unavailable{Some("Session verification is temporarily unavailable. Try again later.".into())}else{status()},
      on_sign_in:move |_|{if busy(){return;}busy.set(true);status.set(Some("Connect your wallet and sign the verification message…".into()));let destination=destination.clone();spawn(async move{
       let result:Result<(),String>=async{
        let address:String=browser("connect",serde_json::json!({"chain":0,"walletconnect":false})).await?;
        let challenge=command(AuthCommand::Challenge{address:address.clone()}).await?.challenge.ok_or("Wallet challenge unavailable")?;
        if !challenge.address.eq_ignore_ascii_case(&address){return Err("Wallet challenge does not match".into());}
        let signature:String=browser("sign",&challenge).await?;
        if !command(AuthCommand::Verify{challenge,signature}).await?.authenticated{return Err("Admin sign-in failed".into());}
        navigator.replace(destination);Ok(())
       }.await;if let Err(message)=result{status.set(Some(message));}busy.set(false);
      });}
     }}
    }
}
#[component]
pub fn AdminLogoutButton(#[props(default)] class: String) -> Element {
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let navigator = use_navigator();
    rsx! {button{class,disabled:busy(),onclick:move |_|{if busy(){return;}busy.set(true);spawn(async move{match command(AuthCommand::Logout).await{Ok(_)=>{navigator.replace("/auth");},Err(message)=>error.set(Some(message))}busy.set(false);});},"Sign out"}if let Some(message)=error(){p{role:"alert","{message}"}}}
}
