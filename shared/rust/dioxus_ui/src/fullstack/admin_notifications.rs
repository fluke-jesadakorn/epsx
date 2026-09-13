//! Typed notification inventory and explicit, backend-authorized commands.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::{
    auth::user::User,
    pages::admin_pages::notifications::{
        AdminNotificationCreateResult, AdminNotificationList, AdminNotificationMetrics,
    },
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NotificationQuery {
    pub page: u32,
    pub status: String,
    pub notification_type: String,
    pub priority: String,
}
impl NotificationQuery {
    pub fn parse(raw: &str) -> Result<Self, LoadError> {
        let mut out = Self {
            page: 1,
            ..Self::default()
        };
        let mut seen = std::collections::HashSet::new();
        for (k, v) in url::form_urlencoded::parse(raw.trim_start_matches('?').as_bytes()) {
            if !seen.insert(k.to_string()) {
                return Err(LoadError::InvalidQuery);
            }
            match k.as_ref() {
                "page" => {
                    out.page = v
                        .parse()
                        .ok()
                        .filter(|n| (1..=50001).contains(n))
                        .ok_or(LoadError::InvalidQuery)?
                }
                "status" => out.status = v.into_owned(),
                "type" => out.notification_type = v.into_owned(),
                "priority" => out.priority = v.into_owned(),
                "mutation" => {}
                _ => return Err(LoadError::InvalidQuery),
            }
        }
        if !["", "pending", "sent", "failed", "read", "unread"].contains(&out.status.as_str())
            || !["", "low", "normal", "high", "critical", "urgent"].contains(&out.priority.as_str())
            || out.notification_type.len() > 50
            || !out
                .notification_type
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        {
            return Err(LoadError::InvalidQuery);
        }
        Ok(out)
    }
    pub fn raw(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        q.append_pair("page", &self.page.max(1).to_string());
        for (k, v) in [
            ("status", &self.status),
            ("type", &self.notification_type),
            ("priority", &self.priority),
        ] {
            if !v.is_empty() {
                q.append_pair(k, v);
            }
        }
        q.finish()
    }
    pub fn href(&self) -> String {
        format!("/notifications/manage?{}", self.raw())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationData {
    pub user: User,
    pub list: Result<AdminNotificationList, LoadError>,
    pub metrics: Result<AdminNotificationMetrics, LoadError>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NotificationCommand {
    Send {
        wallet: String,
        title: String,
        message: String,
    },
    Read(String),
    Delete(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationRequest {
    pub command: NotificationCommand,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationReply {
    pub outcome: String,
    pub created: Option<AdminNotificationCreateResult>,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct NotificationProvider {
    pub read: NotificationProviderReadCallback,
    pub command: NotificationProviderCommandCallback,
}
#[server(prefix = "/_server/admin", endpoint = "notifications_read")]
pub async fn read_notifications(
    query: NotificationQuery,
) -> Result<Result<NotificationData, LoadError>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<NotificationProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(query, h).await)
}
#[server(prefix = "/_server/admin", endpoint = "notifications_command")]
pub async fn notifications_command(
    request: NotificationRequest,
) -> Result<NotificationReply, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<NotificationProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(request, h).await)
}
#[derive(Clone, Copy)]
pub struct NotificationDraft(pub Signal<Option<NotificationRequest>>);
#[derive(Clone, Copy)]
pub struct NotificationEvents {
    pub submit: EventHandler<FormEvent>,
    pub filter: EventHandler<FormEvent>,
    pub navigate: EventHandler<String>,
}
pub fn submit(event: FormEvent) {
    if let Some(c) = try_consume_context::<NotificationEvents>() {
        c.submit.call(event)
    }
}
pub fn filter(event: FormEvent) {
    if let Some(c) = try_consume_context::<NotificationEvents>() {
        c.filter.call(event)
    }
}
pub fn follow(event: MouseEvent, url: String) {
    if event.modifiers().is_empty() {
        if let Some(c) = try_consume_context::<NotificationEvents>() {
            event.prevent_default();
            c.navigate.call(url)
        }
    }
}
async fn load(query: Result<NotificationQuery, LoadError>) -> Result<NotificationData, LoadError> {
    match query {
        Ok(q) => read_notifications(q)
            .await
            .map_err(|_| LoadError::Unavailable)
            .and_then(|v| v),
        Err(e) => Err(e),
    }
}
#[component]
pub fn HydratedAdminNotifications(
    query: ReadSignal<String>,
    #[props(default)] create: bool,
) -> Element {
    let parsed = use_memo(move || NotificationQuery::parse(&query()));
    let initial_query = parsed();
    let initial = use_server_future(move || {
        let q = initial_query.clone();
        async move { load(q).await }
    })?;
    let mut data = use_signal(|| {
        initial
            .read()
            .clone()
            .unwrap_or(Err(LoadError::Unavailable))
    });
    let mut loaded = use_signal(move || parsed.read().clone());
    let mut sequence = use_signal(|| 0u64);
    let mut pending = use_signal(|| false);
    let mut reply = use_signal(|| None::<NotificationReply>);
    let mut previous = use_signal(|| None::<NotificationRequest>);
    let mut generation = use_signal(|| 0u64);
    let nav = use_navigator();
    use_effect(move || {
        let q = parsed();
        if q == *loaded.peek() {
            return;
        }
        loaded.set(q.clone());
        let id = *sequence.peek() + 1;
        sequence.set(id);
        pending.set(true);
        spawn(async move {
            let result = load(q).await;
            if *sequence.peek() == id {
                data.set(result);
                pending.set(false);
            }
        });
    });
    let submit = use_callback(move |event: FormEvent| {
        event.prevent_default();
        event.stop_propagation();
        if *pending.peek() {
            return;
        }
        let values = event.values();
        let get = |name: &str| {
            values
                .iter()
                .find(|(k, _)| k == name)
                .and_then(|(_, v)| match v {
                    dioxus::html::FormValue::Text(v) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let command = if create {
            NotificationCommand::Send {
                wallet: get("recipient_wallet_address"),
                title: get("title"),
                message: get("message"),
            }
        } else {
            match get("action").as_str() {
                "read" => NotificationCommand::Read(get("id")),
                "delete" => NotificationCommand::Delete(get("id")),
                _ => return,
            }
        };
        let key = previous
            .peek()
            .as_ref()
            .filter(|r| r.command == command)
            .map(|r| r.idempotency_key.clone())
            .unwrap_or_else(|| format!("admin.notify.{}", uuid::Uuid::new_v4()));
        let request = NotificationRequest {
            command,
            idempotency_key: key,
        };
        previous.set(Some(request.clone()));
        pending.set(true);
        let q = parsed();
        spawn(async move {
            let result = notifications_command(request)
                .await
                .unwrap_or(NotificationReply {
                    outcome: "unavailable".into(),
                    created: None,
                });
            if result.outcome == "committed" || result.created.is_some() {
                previous.set(None);
                let next = *generation.peek() + 1;
                generation.set(next);
                if !create {
                    data.set(load(q).await);
                }
            }
            reply.set(Some(result));
            pending.set(false);
        });
    });
    let navigate = use_callback(move |url: String| {
        if url == "/notifications/create" && create {
            reply.set(None);
            previous.set(None);
            let next = *generation.peek() + 1;
            generation.set(next);
        } else if url == "/notifications/manage" && !create {
            pending.set(true);
            let q = parsed();
            spawn(async move {
                data.set(load(q).await);
                pending.set(false);
            });
        } else {
            nav.push(url);
        }
    });
    let filter = use_callback(move |event: FormEvent| {
        event.prevent_default();
        let values = event.values();
        let get = |name: &str| {
            values
                .iter()
                .find(|(k, _)| k == name)
                .and_then(|(_, v)| match v {
                    dioxus::html::FormValue::Text(v) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let q = NotificationQuery {
            page: 1,
            status: get("status"),
            notification_type: get("type"),
            priority: get("priority"),
        };
        nav.push(q.href());
    });
    use_context_provider(move || NotificationDraft(previous));
    use_context_provider(move || NotificationEvents {
        submit,
        filter,
        navigate,
    });
    use_context_provider(move || AdminNavigation(navigate));
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/notifications/manage",title:"Notifications",document::Title{"Notifications | EPSX Admin"}fieldset{disabled:pending(),aria_busy:pending(),if pending(){p{class:"p-4",role:"status","Loading notifications…"}}match data(){Ok(snapshot)=>rsx!{crate::pages::admin_pages::notifications::HydratedNotificationBody{key:"{generation}",data:snapshot,query:parsed().unwrap_or_default(),create,reply:reply()}},Err(error)=>rsx!{div{class:"p-6 space-y-4",p{role:"status","{error.message()}"}button{r#type:"button",class:"btn btn-outline",onclick:move |_|navigate.call("/notifications/manage".into()),"Try again"}}}}}}
    }
}

#[cfg(feature = "server")]
pub type NotificationProviderReadCallback = std::sync::Arc<
    dyn Fn(NotificationQuery, http::HeaderMap) -> Future<Result<NotificationData, LoadError>>
        + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type NotificationProviderCommandCallback = std::sync::Arc<
    dyn Fn(NotificationRequest, http::HeaderMap) -> Future<NotificationReply> + Send + Sync,
>;
