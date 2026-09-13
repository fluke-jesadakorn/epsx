//! Dioxus-owned support inbox, query navigation and explicit operator commands.
use super::{admin::AdminAnalyticsShell, LoadError};
use crate::{
    auth::user::User,
    pages::admin_pages::chat::{AdminChatDetail, AdminChatInbox},
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatQuery {
    pub id: Option<uuid::Uuid>,
    pub page: u32,
    pub limit: u32,
    pub status: Option<String>,
    pub topic_id: Option<String>,
    pub agent: Option<String>,
}
impl ChatQuery {
    pub fn parse(id: Option<String>, raw: &str) -> Result<Self, LoadError> {
        let mut q = Self {
            id: id
                .map(|v| uuid::Uuid::parse_str(&v))
                .transpose()
                .map_err(|_| LoadError::NotFound)?,
            page: 1,
            limit: 20,
            status: None,
            topic_id: None,
            agent: None,
        };
        let mut seen = std::collections::BTreeSet::new();
        for (k, v) in url::form_urlencoded::parse(raw.as_bytes()) {
            if !seen.insert(k.to_string()) {
                return Err(LoadError::InvalidQuery);
            }
            match k.as_ref() {
                "page" => q.page = v.parse().map_err(|_| LoadError::InvalidQuery)?,
                "limit" => q.limit = v.parse().map_err(|_| LoadError::InvalidQuery)?,
                "status" => q.status = (!v.is_empty()).then(|| v.to_string()),
                "topic_id" => q.topic_id = (!v.is_empty()).then(|| v.to_string()),
                "agent" => q.agent = (!v.is_empty()).then(|| v.to_string()),
                _ => return Err(LoadError::InvalidQuery),
            }
        }
        if q.page == 0
            || q.page > 50_001
            || q.limit == 0
            || q.limit > 50
            || u64::from(q.page - 1) * u64::from(q.limit) > 1_000_000
        {
            return Err(LoadError::InvalidQuery);
        }
        Ok(q)
    }
    pub fn raw_query(&self) -> String {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.append_pair("page", &self.page.to_string())
            .append_pair("limit", &self.limit.to_string());
        for (k, v) in [
            ("status", &self.status),
            ("topic_id", &self.topic_id),
            ("agent", &self.agent),
        ] {
            if let Some(v) = v {
                form.append_pair(k, v);
            }
        }
        form.finish()
    }
    pub fn href(&self) -> String {
        self.id
            .map(|id| format!("/chat/{id}"))
            .unwrap_or_else(|| format!("/chat?{}", self.raw_query()))
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChatContents {
    Inbox(AdminChatInbox),
    Detail(AdminChatDetail),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatData {
    pub user: User,
    pub contents: ChatContents,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChatOperation {
    Reply(String),
    Status(String),
    Assign(String),
    Read,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatMutation {
    pub id: uuid::Uuid,
    pub operation: ChatOperation,
    pub idempotency_key: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChatOutcome {
    Success,
    Invalid,
    Forbidden,
    Unauthenticated,
    Unavailable,
    Conflict,
}
impl ChatOutcome {
    pub fn label(&self) -> &'static str {
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
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct ChatProvider {
    pub read: ChatProviderReadCallback,
    pub mutate: ChatProviderMutateCallback,
}
#[server(prefix = "/_server/admin", endpoint = "chat_read")]
pub async fn read_chat(query: ChatQuery) -> Result<Result<ChatData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<ChatProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(query, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "chat_mutate")]
pub async fn mutate_chat(command: ChatMutation) -> Result<ChatOutcome, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<ChatProvider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.mutate)(command, headers).await)
}
#[derive(Clone, Copy)]
pub struct ChatControls {
    pub navigate: EventHandler<String>,
    pub submit: EventHandler<FormEvent>,
    pub filter: EventHandler<FormEvent>,
}
pub fn submit(event: FormEvent) {
    if let Some(c) = try_consume_context::<ChatControls>() {
        c.submit.call(event);
    }
}
pub fn filter(event: FormEvent) {
    if let Some(c) = try_consume_context::<ChatControls>() {
        c.filter.call(event);
    }
}
pub fn follow(event: MouseEvent, url: String) {
    if event.modifiers().is_empty() {
        if let Some(c) = try_consume_context::<ChatControls>() {
            event.prevent_default();
            c.navigate.call(url);
        }
    }
}
#[component]
pub fn ChatIdentity(prefix: String) -> Element {
    let identity = use_server_future(move || {
        let prefix = prefix.clone();
        async move { format!("{prefix}.{}", uuid::Uuid::new_v4()) }
    })?;
    rsx! {input{r#type:"hidden",name:"idempotency_key",value:identity.read().clone().unwrap_or_default()}}
}
fn field(values: &[(String, dioxus::html::FormValue)], key: &str) -> Option<String> {
    values
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| match v {
            dioxus::html::FormValue::Text(v) => Some(v.clone()),
            _ => None,
        })
}
#[component]
pub fn HydratedAdminChat(id: ReadSignal<Option<String>>, query: ReadSignal<String>) -> Element {
    let location = use_memo(move || ChatQuery::parse(id(), &query()));
    let initial_location = use_hook(|| location.read().clone());
    let seed = initial_location.clone();
    let initial = use_server_future(move || {
        let query = seed.clone();
        async move {
            read_chat(query?)
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
    let mut last = use_signal(|| initial_location);
    let mut revision = use_signal(|| 0u64);
    let mut requested_revision = use_signal(|| 0u64);
    let mut generation = use_signal(|| 0u64);
    let mut render_generation = use_signal(|| 0u64);
    let mut outcome = use_signal(|| None::<ChatOutcome>);
    let mut previous = use_signal(|| None::<ChatMutation>);
    let nav = use_navigator();
    let location_signal = location;
    let navigate = use_callback(move |url: String| {
        if location_signal
            .peek()
            .as_ref()
            .is_ok_and(|q| q.href() == url)
        {
            let next = *revision.peek() + 1;
            revision.set(next);
        } else {
            nav.push(url);
        }
    });
    let submit = use_callback(move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let Ok(q) = location_signal.peek().clone() else {
            return;
        };
        let Some(id) = q.id else {
            return;
        };
        let fields = event.values();
        let op = match field(&fields, "operation").as_deref() {
            Some("reply") => field(&fields, "content").map(ChatOperation::Reply),
            Some("status") => field(&fields, "status").map(ChatOperation::Status),
            Some("assign") => Some(ChatOperation::Assign(
                field(&fields, "agent_address").unwrap_or_default(),
            )),
            Some("read") => Some(ChatOperation::Read),
            _ => None,
        };
        let Some(operation) = op else {
            outcome.set(Some(ChatOutcome::Invalid));
            return;
        };
        let Some(key) = field(&fields, "idempotency_key") else {
            outcome.set(Some(ChatOutcome::Invalid));
            return;
        };
        let mut command = ChatMutation {
            id,
            operation,
            idempotency_key: key,
        };
        if let Some(old) = previous.peek().as_ref() {
            command.idempotency_key = if old.id == id && old.operation == command.operation {
                old.idempotency_key.clone()
            } else {
                format!("admin.chat.{}", uuid::Uuid::new_v4())
            };
        }
        previous.set(Some(command.clone()));
        pending.set(true);
        spawn(async move {
            let result = mutate_chat(command)
                .await
                .unwrap_or(ChatOutcome::Unavailable);
            if result == ChatOutcome::Success {
                data.set(
                    read_chat(q)
                        .await
                        .map_err(|_| LoadError::Unavailable)
                        .and_then(|v| v),
                );
                previous.set(None);
                let next = *render_generation.peek() + 1;
                render_generation.set(next);
            }
            if result == ChatOutcome::Unauthenticated {
                data.set(Err(LoadError::Unauthenticated));
            }
            outcome.set(Some(result));
            pending.set(false);
        });
    });
    let filter = use_callback(move |event: FormEvent| {
        event.prevent_default();
        let fields = event.values();
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in fields {
            if let dioxus::html::FormValue::Text(value) = value {
                form.append_pair(&key, &value);
            }
        }
        nav.push(format!("/chat?{}", form.finish()));
    });
    use_context_provider(|| ChatControls {
        navigate,
        submit,
        filter,
    });
    use_effect(move || {
        let next = location_signal();
        let rev = revision();
        if next == *last.peek() && rev == *requested_revision.peek() {
            return;
        }
        last.set(next.clone());
        requested_revision.set(rev);
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        spawn(async move {
            let result = match next {
                Ok(q) => read_chat(q)
                    .await
                    .map_err(|_| LoadError::Unavailable)
                    .and_then(|v| v),
                Err(e) => Err(e),
            };
            if *generation.peek() == ticket {
                data.set(result);
                pending.set(false);
            }
        });
    });
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/chat",title:"Chat support",document::Title{"Chat Support | EPSX Admin"}fieldset{disabled:pending(),aria_busy:pending(),if pending(){p{class:"p-4",role:"status","Updating conversations…"}}match (data(),location()){(Ok(data),Ok(query))=>rsx!{crate::pages::admin_pages::chat::HydratedChatBody{key:"{render_generation}",data,query,mutation:outcome().map(|v|v.label().to_string())}},(Err(error),_)=>rsx!{div{class:"p-6 space-y-4",p{role:"status","{error.message()}"}button{r#type:"button",class:"btn btn-outline",onclick:move |_|{let next=*revision.peek()+1;revision.set(next);},"Try again"}}},_=>rsx!{p{"Invalid chat URL"}}}}}}
}

#[cfg(feature = "server")]
pub type ChatProviderReadCallback = std::sync::Arc<
    dyn Fn(ChatQuery, http::HeaderMap) -> Future<Result<ChatData, LoadError>> + Send + Sync,
>;

#[cfg(feature = "server")]
pub type ChatProviderMutateCallback =
    std::sync::Arc<dyn Fn(ChatMutation, http::HeaderMap) -> Future<ChatOutcome> + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chat_query_bounds_and_roundtrip() {
        for raw in ["page=0", "limit=51", "page=2&page=3", "unknown=true"] {
            assert!(ChatQuery::parse(None, raw).is_err());
        }
        let query = ChatQuery::parse(None, "status=open&limit=25&page=2").unwrap();
        assert_eq!(ChatQuery::parse(None, &query.raw_query()).unwrap(), query);
    }
}
