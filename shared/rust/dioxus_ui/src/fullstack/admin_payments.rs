//! Admin payment-service inventory, access projections and lifecycle commands.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::{auth::user::User, pages::admin_pages::payments::*};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaymentTab {
    Payments,
    Links,
    Access,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentsQuery {
    pub tab: PaymentTab,
    pub payer: String,
    pub status: String,
    pub limit: u32,
    pub offset: u64,
    pub page: u32,
    pub search: String,
}
impl Default for PaymentsQuery {
    fn default() -> Self {
        Self {
            tab: PaymentTab::Payments,
            payer: String::new(),
            status: String::new(),
            limit: 20,
            offset: 0,
            page: 1,
            search: String::new(),
        }
    }
}
impl PaymentsQuery {
    pub fn parse(raw: &str) -> Result<Self, LoadError> {
        let mut q = Self::default();
        let mut seen = std::collections::HashSet::new();
        for (k, v) in url::form_urlencoded::parse(raw.trim_start_matches('?').as_bytes()) {
            if !seen.insert(k.to_string()) {
                return Err(LoadError::InvalidQuery);
            }
            match k.as_ref() {
                "tab" => {
                    q.tab = match v.as_ref() {
                        "payments" => PaymentTab::Payments,
                        "payment-links" => PaymentTab::Links,
                        "user-access" => PaymentTab::Access,
                        _ => return Err(LoadError::InvalidQuery),
                    }
                }
                "payer" => q.payer = v.into_owned(),
                "status" => q.status = v.into_owned(),
                "search" => q.search = v.into_owned(),
                "limit" => q.limit = v.parse().map_err(|_| LoadError::InvalidQuery)?,
                "offset" => q.offset = v.parse().map_err(|_| LoadError::InvalidQuery)?,
                "page" => q.page = v.parse().map_err(|_| LoadError::InvalidQuery)?,
                "mutation" => {}
                _ => return Err(LoadError::InvalidQuery),
            }
        }
        if !(1..=100).contains(&q.limit)
            || q.offset > 10_000_000
            || !(1..=500001).contains(&q.page)
            || q.payer.len() > 128
            || q.status.len() > 32
            || q.search.len() > 42
        {
            return Err(LoadError::InvalidQuery);
        }
        Ok(q)
    }
    pub fn raw(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        q.append_pair(
            "tab",
            match self.tab {
                PaymentTab::Payments => "payments",
                PaymentTab::Links => "payment-links",
                PaymentTab::Access => "user-access",
            },
        );
        match self.tab {
            PaymentTab::Payments => {
                q.append_pair("limit", &self.limit.to_string())
                    .append_pair("offset", &self.offset.to_string());
                if !self.payer.is_empty() {
                    q.append_pair("payer", &self.payer);
                }
                if !self.status.is_empty() {
                    q.append_pair("status", &self.status);
                }
            }
            PaymentTab::Access => {
                q.append_pair("page", &self.page.to_string())
                    .append_pair("limit", &self.limit.to_string());
                if !self.search.is_empty() {
                    q.append_pair("search", &self.search);
                }
                if !self.status.is_empty() {
                    q.append_pair("status", &self.status);
                }
            }
            PaymentTab::Links => {}
        }
        q.finish()
    }
    pub fn href(&self) -> String {
        format!("/payments?{}", self.raw())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaymentContents {
    Payments(AdminPaymentIntentList),
    Links(Option<AdminPaymentLinkListProjection>),
    Access(Option<AdminPaymentUserAccessProjection>),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentsData {
    pub user: User,
    pub contents: PaymentContents,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaymentCommand {
    Cancel {
        id: String,
        version: i64,
    },
    DisableLink {
        id: String,
        version: i64,
    },
    CreateLink {
        intent: String,
        max_uses: String,
        expires_in: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub command: PaymentCommand,
    pub idempotency_key: String,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PaymentsProvider {
    pub read: PaymentsProviderReadCallback,
    pub command: PaymentsProviderCommandCallback,
}
#[server(prefix = "/_server/admin", endpoint = "payments_read")]
pub async fn read_payments(
    query: PaymentsQuery,
) -> Result<Result<PaymentsData, LoadError>, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<PaymentsProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(query, h).await)
}
#[server(prefix = "/_server/admin", endpoint = "payments_command")]
pub async fn payments_command(request: PaymentRequest) -> Result<String, ServerFnError> {
    let dioxus_server::axum::Extension(p) = dioxus_fullstack::FullstackContext::extract::<
        dioxus_server::axum::Extension<PaymentsProvider>,
        _,
    >()
    .await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(request, h).await)
}
#[derive(Clone, Copy)]
pub struct PaymentEvents {
    pub submit: EventHandler<FormEvent>,
    pub filter: EventHandler<FormEvent>,
    pub navigate: EventHandler<String>,
}
pub fn submit(event: FormEvent) {
    if let Some(c) = try_consume_context::<PaymentEvents>() {
        c.submit.call(event)
    }
}
pub fn filter(event: FormEvent) {
    if let Some(c) = try_consume_context::<PaymentEvents>() {
        c.filter.call(event)
    }
}
pub fn follow(event: MouseEvent, url: String) {
    if event.modifiers().is_empty() {
        if let Some(c) = try_consume_context::<PaymentEvents>() {
            event.prevent_default();
            c.navigate.call(url)
        }
    }
}
async fn load(query: Result<PaymentsQuery, LoadError>) -> Result<PaymentsData, LoadError> {
    match query {
        Ok(q) => read_payments(q)
            .await
            .map_err(|_| LoadError::Unavailable)
            .and_then(|v| v),
        Err(e) => Err(e),
    }
}
#[component]
pub fn HydratedAdminPayments(query: ReadSignal<String>) -> Element {
    let parsed = use_memo(move || PaymentsQuery::parse(&query()));
    let iq = parsed();
    let initial = use_server_future(move || {
        let q = iq.clone();
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
    let mut result = use_signal(|| None::<String>);
    let mut previous = use_signal(|| None::<PaymentRequest>);
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
            let response = load(q).await;
            if *sequence.peek() == id {
                data.set(response);
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
        let command = match get("operation").as_str() {
            "payment_intent_cancel" => {
                let Ok(version) = get("expected_version").parse() else {
                    return;
                };
                PaymentCommand::Cancel {
                    id: get("intent_id"),
                    version,
                }
            }
            "payment_link_disable" => {
                let Ok(version) = get("expected_version").parse() else {
                    return;
                };
                PaymentCommand::DisableLink {
                    id: get("link_id"),
                    version,
                }
            }
            "payment_link_create" => PaymentCommand::CreateLink {
                intent: get("intent_id"),
                max_uses: get("max_uses"),
                expires_in: get("expires_in"),
            },
            _ => return,
        };
        let key = previous
            .peek()
            .as_ref()
            .filter(|r| r.command == command)
            .map(|r| r.idempotency_key.clone())
            .unwrap_or_else(|| format!("admin.payment.{}", uuid::Uuid::new_v4()));
        let request = PaymentRequest {
            command,
            idempotency_key: key,
        };
        previous.set(Some(request.clone()));
        pending.set(true);
        let q = parsed();
        spawn(async move {
            let outcome = payments_command(request)
                .await
                .unwrap_or("unavailable".into());
            if outcome == "success" {
                previous.set(None);
                let next = *generation.peek() + 1;
                generation.set(next);
                data.set(load(q).await);
            }
            result.set(Some(outcome));
            pending.set(false);
        });
    });
    let navigate = use_callback(move |url: String| {
        if url == parsed().unwrap_or_default().href() {
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
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in event.values() {
            if let dioxus::html::FormValue::Text(v) = v {
                q.append_pair(&k, &v);
            }
        }
        nav.push(format!("/payments?{}", q.finish()));
    });
    use_context_provider(move || PaymentEvents {
        submit,
        filter,
        navigate,
    });
    use_context_provider(move || AdminNavigation(navigate));
    rsx! {AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/payments",title:"Payments",document::Title{"Payments | EPSX Admin"}fieldset{disabled:pending(),aria_busy:pending(),if pending(){p{class:"p-4",role:"status","Loading payments…"}}match data(){Ok(snapshot)=>rsx!{crate::pages::admin_pages::payments::HydratedPaymentsBody{key:"{generation}",data:snapshot,query:parsed().unwrap_or_default(),mutation:result()}},Err(error)=>rsx!{div{class:"p-6 space-y-4",crate::fullstack::load_error::LoadErrorNotice { error: error.clone(), button{r#type:"button",class:"btn btn-outline",onclick:move |_|navigate.call(parsed().unwrap_or_default().href()),"Try again"} }}}}}}
    }
}

#[cfg(feature = "server")]
pub type PaymentsProviderReadCallback = std::sync::Arc<
    dyn Fn(PaymentsQuery, http::HeaderMap) -> Future<Result<PaymentsData, LoadError>> + Send + Sync,
>;

#[cfg(feature = "server")]
pub type PaymentsProviderCommandCallback =
    std::sync::Arc<dyn Fn(PaymentRequest, http::HeaderMap) -> Future<String> + Send + Sync>;
