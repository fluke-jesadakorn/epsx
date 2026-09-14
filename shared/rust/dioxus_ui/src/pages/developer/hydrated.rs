use super::*;
use crate::fullstack::LoadError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateInput {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateResult {
    pub api_key: DeveloperApiKey,
    pub secret: Option<String>,
    pub replayed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TryInput {
    pub operation_id: String,
    pub api_key: String,
    pub query: Option<String>,
    pub body: Option<serde_json::Value>,
    pub confirm_mutation: Option<bool>,
    pub idempotency_key: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TryResult {
    pub status: u16,
    pub content_type: String,
    pub body: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeveloperCommand {
    Create {
        input: CreateInput,
        idempotency_key: String,
    },
    Revoke {
        id: Uuid,
        reason: Option<String>,
        idempotency_key: String,
    },
    Try(TryInput),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DeveloperMutationResult {
    Created(CreateResult),
    Revoked {
        id: Uuid,
        status: String,
        replayed: bool,
    },
    Tried(TryResult),
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct DeveloperProvider(pub DeveloperProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct DeveloperMutationProvider(pub DeveloperMutationProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct DeveloperDocsProvider(pub DeveloperDocsProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "developer")]
pub async fn read_developer(
    days: i32,
) -> Result<Result<DeveloperOverview, LoadError>, ServerFnError> {
    if !matches!(days, 7 | 30 | 90) {
        return Ok(Err(LoadError::InvalidQuery));
    }
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<DeveloperProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Developer provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(days, headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "developer-change")]
pub async fn change_developer(
    command: DeveloperCommand,
) -> Result<Result<DeveloperMutationResult, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<DeveloperMutationProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Developer provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(command, headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "developer-docs")]
pub async fn read_developer_docs() -> Result<Result<serde_json::Value, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<DeveloperDocsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Developer docs provider unavailable"))?;
    Ok((provider.0)().await)
}
fn days(raw: &str) -> Result<i32, LoadError> {
    let mut value = None;
    for (key, text) in url::form_urlencoded::parse(raw.as_bytes()) {
        if key != "days" || value.is_some() {
            return Err(LoadError::InvalidQuery);
        }
        value = Some(text.parse::<i32>().map_err(|_| LoadError::InvalidQuery)?);
    }
    let value = value.unwrap_or(30);
    if matches!(value, 7 | 30 | 90) {
        Ok(value)
    } else {
        Err(LoadError::InvalidQuery)
    }
}
#[derive(Clone, Copy)]
pub(super) struct DeveloperControls {
    pub create: EventHandler<FormEvent>,
    pub revoke: EventHandler<(Uuid, FormEvent)>,
    pub pending: ReadSignal<bool>,
    pub secret: ReadSignal<Option<String>>,
    pub copy: EventHandler<()>,
    pub create_identity: ReadSignal<String>,
}
#[component]
pub fn HydratedDeveloper(query: ReadSignal<String>, #[props(default)] usage: bool) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let requested = days(&seed_query);
        async move {
            match requested {
                Ok(days) => read_developer(days)
                    .await
                    .unwrap_or(Err(LoadError::Unavailable)),
                Err(error) => Err(error),
            }
        }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut revision = use_signal(|| 0_u64);
    let mut generation = use_signal(|| 0_u64);
    let mut last_query = use_signal(|| initial_query);
    // Never initialized from a read or included in SSR; secrets live only in this mounted page.
    let mut secret = use_signal(|| None::<String>);
    let mut status = use_signal(String::new);
    let identity =
        dioxus_fullstack::use_server_cached(|| format!("developer.create.{}", Uuid::new_v4()));
    let mut create_identity = use_signal(|| identity);
    let create = use_callback(move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let values = event.values();
        let one = |name: &str| {
            values
                .iter()
                .find(|(key, _)| key == name)
                .and_then(|(_, value)| match value {
                    dioxus::html::FormValue::Text(value) => Some(value.clone()),
                    _ => None,
                })
        };
        let scopes = values
            .iter()
            .filter_map(|(key, value)| {
                if key == "scopes" {
                    if let dioxus::html::FormValue::Text(value) = value {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        let input = CreateInput {
            name: one("name").unwrap_or_default(),
            description: one("description").filter(|value| !value.is_empty()),
            scopes,
            expires_at: one("expires_at").filter(|value| !value.is_empty()),
        };
        let command = DeveloperCommand::Create {
            input,
            idempotency_key: create_identity(),
        };
        pending.set(true);
        error.set(None);
        secret.set(None);
        spawn(async move {
            match change_developer(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable))
            {
                Ok(DeveloperMutationResult::Created(value)) => {
                    secret.set(value.secret);
                    status.set(if value.replayed { "The previous request was already processed. The secret is not repeated." } else { "API key created. Save the secret now." }.into());
                    create_identity.set(format!("developer.create.{}", Uuid::new_v4()));
                    let next = *revision.peek() + 1;
                    revision.set(next);
                }
                Ok(_) => error.set(Some(LoadError::Malformed)),
                Err(failure) => error.set(Some(failure)),
            }
            pending.set(false);
        });
    });
    let revoke = use_callback(move |(id, event): (Uuid, FormEvent)| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let values = event.values();
        let value = |name: &str| {
            values
                .iter()
                .find(|(key, _)| key == name)
                .and_then(|(_, value)| match value {
                    dioxus::html::FormValue::Text(value) => Some(value.clone()),
                    _ => None,
                })
        };
        if value("confirm_revoke").as_deref() != Some("yes") {
            error.set(Some(LoadError::InvalidQuery));
            return;
        }
        let command = DeveloperCommand::Revoke {
            id,
            reason: value("reason"),
            idempotency_key: value("idempotency_key").unwrap_or_default(),
        };
        pending.set(true);
        error.set(None);
        spawn(async move {
            match change_developer(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable))
            {
                Ok(DeveloperMutationResult::Revoked { .. }) => {
                    status.set("API key revoked.".into());
                    let next = *revision.peek() + 1;
                    revision.set(next);
                }
                Ok(_) => error.set(Some(LoadError::Malformed)),
                Err(failure) => error.set(Some(failure)),
            }
            pending.set(false);
        });
    });
    let copy = use_callback(move |()| {
        if let Some(value) = secret() {
            spawn(async move {
                let script = format!("try {{ await navigator.clipboard.writeText({}); dioxus.send(true); }} catch (_) {{ dioxus.send(false); }}", serde_json::to_string(&value).unwrap_or_default());
                let success = document::eval(&script)
                    .recv::<bool>()
                    .await
                    .unwrap_or(false);
                status.set(
                    if success {
                        "Secret copied."
                    } else {
                        "Could not copy. Select and copy the secret manually."
                    }
                    .into(),
                );
            });
        }
    });
    use_context_provider(|| DeveloperControls {
        create,
        revoke,
        pending: pending.into(),
        secret: secret.into(),
        copy,
        create_identity: create_identity.into(),
    });
    use_effect(move || {
        let query = query();
        let revision = revision();
        if query == *last_query.peek() && revision == 0 {
            return;
        }
        last_query.set(query.clone());
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        spawn(async move {
            let response = match days(&query) {
                Ok(days) => read_developer(days)
                    .await
                    .unwrap_or(Err(LoadError::Unavailable)),
                Err(error) => Err(error),
            };
            if *generation.peek() != ticket {
                return;
            }
            match response {
                Ok(snapshot) => data.set(Some(snapshot)),
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                        secret.set(None);
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    rsx! {
        document::Title { if usage { "API usage — EPSX" } else { "Developer — EPSX" } }
        document::Meta { name: "description", content: "Manage API keys and review your API access and usage." }
        div { class: "container page-content space-y-6 fe-page-layout", "data-dioxus-developer": "true", aria_busy: pending(),
            PageHeader { title: if usage { "API usage".to_string() } else { "Developer portal".to_string() }, description: Some("Manage API keys and review the access included in your plan.".into()), icon: Some("code".into()) }
            if let Some(failure) = error() { crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),  button { class: "btn btn-outline", r#type: "button", disabled: pending(), onclick: move |_| { let next = *revision.peek()+1; revision.set(next); }, "Try again" } } }
            if !status().is_empty() { p { role: "status", "{status}" } }
            if let Some(data) = data() { if usage { UsageReady { data } } else { OverviewReady { data } } }
        }
    }
}

#[component]
pub fn HydratedDeveloperDocs() -> Element {
    let mut resource = use_server_future(|| async {
        read_developer_docs()
            .await
            .unwrap_or(Err(LoadError::Unavailable))
    })?;
    let mut api_key = use_signal(String::new);
    let result = resource
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    rsx! {
        document::Title { "API documentation — EPSX" }
        document::Meta { name: "description", content: "Browse API operations, parameters, and responses." }
        div { class: "container page-content space-y-6 fe-page-layout",
            PageHeader { title: "API documentation", description: Some("Browse API operations, parameters, and responses.".to_string()), icon: Some("book-open".to_string()) }
            match result.and_then(|spec| decode_openapi(spec.clone()).map(|operations|(spec,operations)).ok_or(LoadError::Malformed)) {
                Ok((spec,operations)) => rsx! {
                    reference::Introduction { operations: operations.clone(), spec: serde_json::to_string(&spec).unwrap_or_default() }
                    div { class: "space-y-6", "data-developer-docs-state": "ready",
                        section { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl fe-surface",
                            label { class: "grid gap-2 text-sm font-medium text-foreground fe-tone-text", "API key for Try It",
                                input { class: "input font-mono", r#type: "password", autocomplete: "off", spellcheck: "false", placeholder: "epsx_…", value: api_key(), oninput: move |event| api_key.set(event.value()) }
                            }
                            p { class: "mt-2 text-xs text-muted-foreground fe-tone-muted", "Kept only in this page's memory and cleared when you leave." }
                        }
                        for operation in operations { TryOperation { key: "{operation.operation_id}", operation, api_key } }
                    }
                },
                Err(error) => rsx! { crate::fullstack::load_error::LoadErrorNotice { error: error.clone(),  button { class: "btn btn-outline", onclick: move |_| resource.restart(), "Try again" } } }
            }
        }
    }
}

#[component]
fn TryOperation(operation: DeveloperOperation, api_key: ReadSignal<String>) -> Element {
    let mut query = use_signal(String::new);
    let mut body = use_signal(|| "{}".to_string());
    let mut confirmed = use_signal(|| false);
    let mut pending = use_signal(|| false);
    let mut response = use_signal(String::new);
    let seed = dioxus_fullstack::use_server_cached(|| format!("developer.try.{}", Uuid::new_v4()));
    let mut identity = use_signal(|| seed);
    let operation_id = operation.operation_id.clone();
    let mutation = operation.mutation;
    let run = move |_| {
        if *pending.peek() {
            return;
        }
        if mutation && !confirmed() {
            response.set("Confirm this mutation before sending it.".into());
            return;
        }
        let parsed_body = if mutation {
            match serde_json::from_str(&body()) {
                Ok(value) => Some(value),
                Err(_) => {
                    response.set("Enter a valid JSON body.".into());
                    return;
                }
            }
        } else {
            None
        };
        let input = TryInput {
            operation_id: operation_id.clone(),
            api_key: api_key(),
            query: Some(query()).filter(|value| !value.is_empty()),
            body: parsed_body,
            confirm_mutation: Some(confirmed()),
            idempotency_key: mutation.then(|| identity.read().clone()),
        };
        pending.set(true);
        spawn(async move {
            match change_developer(DeveloperCommand::Try(input))
                .await
                .unwrap_or(Err(LoadError::Unavailable))
            {
                Ok(DeveloperMutationResult::Tried(value)) => {
                    response.set(format!(
                        "HTTP {}\n{}\n\n{}",
                        value.status, value.content_type, value.body
                    ));
                    identity.set(format!("developer.try.{}", Uuid::new_v4()));
                }
                Ok(_) => response.set(LoadError::Malformed.message().into()),
                Err(error) => response.set(error.message().into()),
            }
            pending.set(false);
        });
    };
    rsx! {
        article { class: "rounded-2xl border border-border/20 bg-card p-6 shadow-xl fe-surface", id: format!("operation-{}",operation.operation_id),
            div { class: "flex flex-wrap items-center gap-3",
                span { class: "rounded-lg bg-purple-500/15 px-2 py-1 text-xs font-bold text-purple-700 dark:text-purple-300 fe-tone-accent", "{operation.method}" }
                code { class: "font-mono text-sm text-foreground fe-tone-text", "{operation.path}" }
            }
            h2 { class: "mt-3 text-lg font-semibold text-foreground fe-tone-text", "{operation.summary}" }
            div { class: "mt-3 flex flex-wrap gap-2", for scope in &operation.required_scopes { code { class: "rounded bg-background px-2 py-1 text-xs", "{scope}" } } }
            reference::OperationDetails { operation: operation.clone() }
            if operation.api_key_callable {
                div { class: "mt-5 grid gap-3",
                    label { class: "grid gap-1 text-xs text-muted-foreground fe-tone-muted", "Query string (optional)", input { class: "input font-mono", disabled: pending(),value:query(),oninput:move |event|query.set(event.value()) } }
                    if mutation {
                        label { class: "grid gap-1 text-xs text-muted-foreground fe-tone-muted", "JSON body", textarea { class: "input min-h-28 font-mono", disabled:pending(),value:body(),oninput:move |event|body.set(event.value()) } }
                        label { class: "flex items-center gap-2 text-sm", input { r#type:"checkbox",disabled:pending(),checked:confirmed(),onchange:move |event|confirmed.set(event.checked()) } "I confirm sending this mutation." }
                    }
                    button { class: "btn btn-primary justify-self-start",r#type:"button",disabled:pending(),onclick:run,if pending() { "Sending…" } else { "Try It" } }
                    if !response().is_empty() { pre { class: "max-h-96 overflow-auto rounded-xl bg-slate-950 p-4 text-xs text-slate-200 fe-fill-neutral",role:"status","{response}" } }
                }
            } else { p { class:"mt-4 text-sm text-muted-foreground fe-tone-muted","Browser-session operation; Try It is disabled for API keys." } }
        }
    }
}

#[cfg(feature = "server")]
pub type DeveloperProviderCallback = std::sync::Arc<
    dyn Fn(
            i32,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<DeveloperOverview, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type DeveloperMutationProviderCallback = std::sync::Arc<
    dyn Fn(
            DeveloperCommand,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<DeveloperMutationResult, LoadError>> + Send,
            >,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type DeveloperDocsProviderCallback = std::sync::Arc<
    dyn Fn() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<serde_json::Value, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn usage_query_keeps_only_supported_backend_windows() {
        assert_eq!(days(""), Ok(30));
        for value in [7, 30, 90] {
            assert_eq!(days(&format!("days={value}")), Ok(value));
        }
        for query in ["days=1", "days=30&days=7", "wallet=other", "days=all"] {
            assert!(days(query).is_err());
        }
    }
}
