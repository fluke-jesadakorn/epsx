//! Settings commands remain validated and authorized by the native BFF/backend.
use super::{
    admin::{AdminAnalyticsShell, AdminNavigation},
    LoadError,
};
use crate::pages::admin_pages::settings::{AdminSettingValue, AdminSettingsSnapshot};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
    pub user: crate::auth::User,
    pub snapshot: AdminSettingsSnapshot,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SettingsCommand {
    Update {
        category: String,
        key: String,
        value: AdminSettingValue,
        expected_updated_at: Option<String>,
    },
    Reset,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsRequest {
    pub command: SettingsCommand,
    pub idempotency_key: String,
    pub return_tab: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SettingsOutcome {
    Success,
    Conflict,
    Invalid,
    Forbidden,
    Unauthenticated,
    Unavailable,
}
impl SettingsOutcome {
    pub fn state(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Conflict => "conflict",
            Self::Invalid => "invalid",
            Self::Forbidden => "forbidden",
            Self::Unauthenticated => "unauthorized",
            Self::Unavailable => "unavailable",
        }
    }
}
#[cfg(feature = "server")]
type ProviderFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct SettingsProvider {
    pub read: std::sync::Arc<
        dyn Fn(http::HeaderMap) -> ProviderFuture<Result<SettingsData, LoadError>> + Send + Sync,
    >,
    pub mutate: std::sync::Arc<
        dyn Fn(SettingsRequest, http::HeaderMap) -> ProviderFuture<SettingsOutcome> + Send + Sync,
    >,
}
#[server(prefix = "/_server/admin", endpoint = "settings_read")]
pub async fn read_settings() -> Result<Result<SettingsData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<SettingsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Settings provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request unavailable"))?;
    Ok((provider.read)(headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "settings_update")]
pub async fn update_settings(request: SettingsRequest) -> Result<SettingsOutcome, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<SettingsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Settings provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request unavailable"))?;
    Ok((provider.mutate)(request, headers).await)
}
#[derive(Clone, Copy)]
pub struct HydratedSettings(pub EventHandler<FormEvent>);
pub fn submit_form(event: FormEvent) {
    if let Some(handler) = try_consume_context::<HydratedSettings>() {
        event.stop_propagation();
        (handler.0).call(event);
    }
}

pub fn parse_submission(event: &FormEvent) -> Option<SettingsRequest> {
    let values = event.values();
    let get = |key: &str| match values
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
    {
        Some(dioxus::html::FormValue::Text(value)) => Some(value.clone()),
        _ => None,
    };
    let command = if let Some(category) = get("category") {
        let values = [
            get("value_text").map(AdminSettingValue::Text),
            get("value_bool")
                .and_then(|v| v.parse().ok())
                .map(AdminSettingValue::Bool),
            get("value_number")
                .and_then(|v| v.parse().ok())
                .map(AdminSettingValue::Number),
        ];
        let mut values = values.into_iter().flatten();
        let value = values.next()?;
        if values.next().is_some() {
            return None;
        }
        SettingsCommand::Update {
            category,
            key: get("key")?,
            value,
            expected_updated_at: get("expected_updated_at"),
        }
    } else {
        SettingsCommand::Reset
    };
    Some(SettingsRequest {
        command,
        idempotency_key: get("idempotency_key")?,
        return_tab: get("return_tab")?,
    })
}

#[component]
pub fn SettingsIdentity(prefix: String) -> Element {
    let key = use_server_future(move || {
        let prefix = prefix.clone();
        async move { format!("{prefix}.{}", uuid::Uuid::new_v4()) }
    })?;
    rsx! {input{r#type:"hidden",name:"idempotency_key",value:key.read().clone().unwrap_or_default()}}
}

#[component]
pub fn HydratedAdminSettings(query: ReadSignal<String>) -> Element {
    let initial = use_server_future(|| async {
        read_settings()
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
    let mut previous_request = use_signal(|| None::<SettingsRequest>);
    let mut outcome = use_signal(|| None::<SettingsOutcome>);
    let mut generation = use_signal(|| 0u64);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    use_context_provider(|| AdminNavigation(navigate));

    let tab = crate::pages::admin_pages::settings::AdminSettingsQuery::from_raw(&query())
        .map(|q| q.tab)
        .unwrap_or_else(|_| "general".into());
    let submit = use_callback(move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let Some(mut request) = parse_submission(&event) else {
            outcome.set(Some(SettingsOutcome::Invalid));
            return;
        };
        request.idempotency_key = format!("admin.settings.{}", uuid::Uuid::new_v4());
        if let Some(previous) = previous_request.peek().as_ref() {
            request.idempotency_key = if previous.command == request.command {
                previous.idempotency_key.clone()
            } else {
                format!("admin.settings.{}", uuid::Uuid::new_v4())
            };
        }
        previous_request.set(Some(request.clone()));
        pending.set(true);
        spawn(async move {
            let result = update_settings(request)
                .await
                .unwrap_or(SettingsOutcome::Unavailable);
            if result == SettingsOutcome::Success {
                data.set(
                    read_settings()
                        .await
                        .map_err(|_| LoadError::Unavailable)
                        .and_then(|v| v),
                );
                let next = *generation.peek() + 1;
                generation.set(next);
                previous_request.set(None);
            }
            if result == SettingsOutcome::Unauthenticated {
                data.set(Err(LoadError::Unauthenticated));
            }
            outcome.set(Some(result));
            pending.set(false);
        });
    });
    use_context_provider(|| HydratedSettings(submit));
    rsx! {
        AdminAnalyticsShell{authenticated:data().is_ok(),current_path:"/settings",title:"Settings",
            fieldset{disabled:pending(),aria_busy:pending(),
                if pending(){p{role:"status","Saving settings…"}}
                match data(){
                    Ok(snapshot)=>rsx!{crate::pages::admin_pages::settings::HydratedSettingsBody{key:"{generation}",data:snapshot,tab,mutation:outcome().map(|value|value.state().to_string())}},
                    Err(failure)=>rsx!{div{class:"p-6",p{role:"status","{failure.message()}"}
                        button{r#type:"button",class:"btn btn-primary",onclick:move |_|{pending.set(true);spawn(async move{data.set(read_settings().await.map_err(|_|LoadError::Unavailable).and_then(|v|v));pending.set(false);});},"Try again"}
                    }},
                }
            }
        }
    }
}
