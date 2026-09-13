//! Typed SIWE UI. Signing stays in the browser; challenge/session verification
//! and cookie changes stay in the BFF's existing authentication handlers.
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
#[server(prefix = "/_server/frontend", endpoint = "auth_session")]
pub async fn auth_session() -> Result<Result<AuthSession, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AuthProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.session)(headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "auth_action")]
pub async fn auth_action(
    command: AuthCommand,
) -> Result<Result<AuthReply, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AuthProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(command, headers).await)
}
pub(crate) async fn browser<T: serde::de::DeserializeOwned>(
    operation: &str,
    input: impl Serialize,
) -> Result<T, String> {
    let mut eval = document::eval(include_str!("pay/wallet_adapter.js"));
    eval.send(serde_json::json!({"operation":operation,"input":input}))
        .map_err(|e| e.to_string())?;
    #[derive(Deserialize)]
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
async fn command(value: AuthCommand) -> Result<AuthReply, String> {
    auth_action(value)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.message().to_owned())
}
pub fn return_path(query: &str) -> String {
    url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
        .find(|(key, _)| key == "return_url")
        .map(|(_, v)| v.into_owned())
        .filter(|v| {
            v.starts_with('/')
                && !v.starts_with("//")
                && !v.contains('\\')
                && !v.chars().any(char::is_control)
                && !v.starts_with("/auth")
        })
        .unwrap_or_else(|| "/analytics".into())
}
fn invalidate() {
    if let Some(mut revision) = try_use_context::<super::shell::AuthRevision>() {
        let next = (revision.0)() + 1;
        revision.0.set(next);
    }
}
#[component]
pub fn HydratedAuth(query: String) -> Element {
    use crate::pages::auth_page::{AuthPageSessionState, RenderAuth};
    let destination = return_path(&query);
    let navigator = use_navigator();
    let initial = use_server_future(auth_session)?;
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut recovered = use_signal(|| false);
    let fallback = use_signal(|| true);
    let mut dark = try_use_context::<super::shell::ThemeSignal>()
        .map(|context| context.0)
        .unwrap_or(fallback);
    let redirect = destination.clone();
    use_effect(move || {
        let Some(Ok(Ok(session))) = initial.read().as_ref().cloned() else {
            return;
        };
        if session.authenticated {
            navigator.replace(redirect.clone());
            return;
        }
        if session.recover_session && !recovered() {
            recovered.set(true);
            busy.set(true);
            let redirect = redirect.clone();
            spawn(async move {
                match command(AuthCommand::Refresh).await {
                    Ok(v) if v.authenticated => {
                        invalidate();
                        navigator.replace(redirect);
                    }
                    _ => {
                        error.set(Some("Your session expired. Sign in again.".into()));
                    }
                }
                busy.set(false);
            });
        }
    });
    let state = match initial.read().as_ref() {
        Some(Ok(Ok(s))) if s.verifier_unavailable => AuthPageSessionState::VerifierUnavailable,
        _ => AuthPageSessionState::SignedOut,
    };
    rsx! {
     document::Title{"Sign in | EPSX"}
     div {class:if dark(){"dark"}else{"light"},
      RenderAuth {session_state:state,return_url:Some(destination.clone()),busy:busy(),error:error(),
       on_theme:move |_|{let value=!dark();dark.set(value);},
       on_sign_in:move |_|{if busy(){return;}busy.set(true);error.set(None);let destination=destination.clone();spawn(async move{
         let result:Result<(),String>=async{
          let address:String=browser("connect",serde_json::json!({"chain":0,"walletconnect":false})).await?;
          let challenge=command(AuthCommand::Challenge{address:address.clone()}).await?.challenge.ok_or("Wallet challenge unavailable")?;
          if !challenge.address.eq_ignore_ascii_case(&address){return Err("Wallet challenge does not match".into());}
          let signature:String=browser("sign",&challenge).await?;
          if !command(AuthCommand::Verify{challenge,signature}).await?.authenticated{return Err("Sign-in verification failed".into());}
          invalidate();navigator.replace(destination);Ok(())
         }.await;
         if let Err(message)=result{error.set(Some(message));}busy.set(false);
       });}
      }
     }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn return_urls_remain_local() {
        for value in [
            "//evil.test",
            "/\\evil.test",
            "/auth",
            "https://evil.test",
            "/a%0Ab",
        ] {
            let q = format!("return_url={value}");
            assert_eq!(return_path(&q), "/analytics");
        }
        assert_eq!(
            return_path("return_url=%2Faccount%3Ftab%3D1"),
            "/account?tab=1"
        );
    }
}
