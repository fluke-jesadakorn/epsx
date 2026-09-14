//! Typed API-key inventory and explicit one-time-secret creation responses.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::{
    auth::user::User,
    pages::admin_pages::developer_portal::{
        AdminDeveloperPortalProjection, AdminDeveloperSecretOnceProjection,
    },
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeveloperData {
    pub user: User,
    pub projection: AdminDeveloperPortalProjection,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateKey {
    pub name: String,
    pub description: String,
    pub email: String,
    pub expires_at: String,
    pub ip_restrictions: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeveloperCommand {
    Create(CreateKey),
    Revoke { id: uuid::Uuid, reason: String },
    Expire { id: uuid::Uuid, expires_at: String },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeveloperRequest {
    pub command: DeveloperCommand,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeveloperOutcome {
    Success,
    Invalid,
    Forbidden,
    Unauthenticated,
    Unavailable,
    Conflict,
}
impl DeveloperOutcome {
    pub fn state(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Invalid => "malformed",
            Self::Forbidden => "forbidden",
            Self::Unauthenticated => "unauthorized",
            Self::Unavailable => "unavailable",
            Self::Conflict => "conflict",
        }
    }
}
// Intentionally no Debug: the mutation-only result may contain a plaintext key.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperReply {
    pub outcome: DeveloperOutcome,
    pub created: Option<AdminDeveloperSecretOnceProjection>,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct DeveloperProvider {
    pub read: std::sync::Arc<
        dyn Fn(http::HeaderMap) -> Future<Result<DeveloperData, LoadError>> + Send + Sync,
    >,
    pub command: std::sync::Arc<
        dyn Fn(DeveloperRequest, http::HeaderMap) -> Future<DeveloperReply> + Send + Sync,
    >,
}
#[server(prefix = "/_server/admin", endpoint = "developer_read")]
pub async fn read_developer() -> Result<Result<DeveloperData, LoadError>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<DeveloperProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(h).await)
}
#[server(prefix = "/_server/admin", endpoint = "developer_command")]
pub async fn developer_command(request: DeveloperRequest) -> Result<DeveloperReply, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<DeveloperProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(request, h).await)
}
#[derive(Clone, Copy)]
pub struct DeveloperEvents {
    pub submit: EventHandler<FormEvent>,
    pub navigate: EventHandler<String>,
}
pub fn submit(event: FormEvent) {
    if let Some(handler) = try_consume_context::<DeveloperEvents>() {
        handler.submit.call(event);
    }
}
pub fn follow(event: MouseEvent, url: String) {
    if event.modifiers().is_empty() {
        if let Some(handler) = try_consume_context::<DeveloperEvents>() {
            event.prevent_default();
            handler.navigate.call(url);
        }
    }
}
fn fields(event: &FormEvent, create: bool) -> Option<DeveloperRequest> {
    let values = event.values();
    let get = |key: &str| {
        values
            .iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| match v {
                dioxus::html::FormValue::Text(v) => Some(v.clone()),
                _ => None,
            })
    };
    let command = if create {
        DeveloperCommand::Create(CreateKey {
            name: get("client_name")?,
            description: get("client_description").unwrap_or_default(),
            email: get("client_contact_email").unwrap_or_default(),
            expires_at: get("expires_at").unwrap_or_default(),
            ip_restrictions: get("ip_restrictions").unwrap_or_default(),
        })
    } else {
        let id = uuid::Uuid::parse_str(&get("api_key_id")?).ok()?;
        match get("operation")?.as_str() {
            "revoke" => DeveloperCommand::Revoke {
                id,
                reason: get("reason")?,
            },
            "expiration" => DeveloperCommand::Expire {
                id,
                expires_at: get("expires_at").unwrap_or_default(),
            },
            _ => return None,
        }
    };
    Some(DeveloperRequest {
        command,
        idempotency_key: get("idempotency_key")?,
    })
}
#[component]
pub fn HydratedAdminDeveloper(
    query: ReadSignal<String>,
    #[props(default)] create: bool,
) -> Element {
    let initial = use_server_future(|| async {
        read_developer()
            .await
            .map_err(|_| LoadError::Unavailable)
            .and_then(|v| v)
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut pending = use_signal(|| false);
    let mut result = use_signal(|| None::<DeveloperReply>);
    let mut previous = use_signal(|| None::<DeveloperRequest>);
    let mut generation = use_signal(|| 0u64);
    let nav = use_navigator();
    let submit = use_callback(move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let Some(mut request) = fields(&event, create) else {
            result.set(Some(DeveloperReply {
                outcome: DeveloperOutcome::Invalid,
                created: None,
            }));
            return;
        };
        request.idempotency_key = format!("admin.developer.{}", uuid::Uuid::new_v4());
        if let Some(old) = previous.peek().as_ref() {
            request.idempotency_key = if old.command == request.command {
                old.idempotency_key.clone()
            } else {
                format!("admin.developer.{}", uuid::Uuid::new_v4())
            };
        }
        previous.set(Some(request.clone()));
        pending.set(true);
        spawn(async move {
            let reply = developer_command(request).await.unwrap_or(DeveloperReply {
                outcome: DeveloperOutcome::Unavailable,
                created: None,
            });
            if reply.outcome == DeveloperOutcome::Success {
                previous.set(None);
                if !create {
                    data.set(
                        read_developer()
                            .await
                            .map_err(|_| LoadError::Unavailable)
                            .and_then(|v| v),
                    );
                }
                let next = *generation.peek() + 1;
                generation.set(next);
            }
            if reply.outcome == DeveloperOutcome::Unauthenticated {
                data.set(Err(LoadError::Unauthenticated));
            }
            result.set(Some(reply));
            pending.set(false);
        });
    });
    let navigate = use_callback(move |url: String| {
        if url == "/developer-portal" && !create {
            pending.set(true);
            spawn(async move {
                data.set(
                    read_developer()
                        .await
                        .map_err(|_| LoadError::Unavailable)
                        .and_then(|v| v),
                );
                pending.set(false);
            });
        } else {
            nav.push(url);
        }
    });
    use_context_provider(|| DeveloperEvents { submit, navigate });
    use_context_provider(|| AdminNavigation(navigate));
    let tab = url::form_urlencoded::parse(query().as_bytes())
        .find(|(k, _)| k == "tab")
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| "overview".into());
    let page_title = if create {
        "Create API Key | EPSX Admin"
    } else {
        "Developer Portal | EPSX Admin"
    };
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/developer-portal",title:"Developer portal",document::Title{"{page_title}"}fieldset{disabled:pending(),aria_busy:pending(),if pending(){p{class:"p-4",role:"status","Updating API keys…"}}match data(){Ok(snapshot)=>rsx!{crate::pages::admin_pages::developer_portal::HydratedDeveloperBody{key:"{generation}",data:snapshot,tab,create,result:result()}},Err(error)=>rsx!{div{class:"p-6 space-y-4",crate::fullstack::load_error::LoadErrorNotice { error: error.clone(), button{class:"btn btn-outline",r#type:"button",onclick:move |_|{pending.set(true);spawn(async move{data.set(read_developer().await.map_err(|_|LoadError::Unavailable).and_then(|v|v));pending.set(false);});},"Try again"} }}}}}}
    }
}
