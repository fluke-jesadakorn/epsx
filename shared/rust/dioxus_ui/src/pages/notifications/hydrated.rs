use super::*;
use crate::fullstack::LoadError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationQuery {
    pub page: u32,
    pub status: Option<String>,
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
pub fn decode(
    value: serde_json::Value,
    query: NotificationQuery,
) -> Result<NotificationPage, LoadError> {
    let payload: ServiceNotificationList =
        serde_json::from_value(value).map_err(|_| LoadError::Malformed)?;
    let total = u64::try_from(payload.total).map_err(|_| LoadError::Malformed)?;
    let items = payload
        .items
        .into_iter()
        .map(Notification::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| LoadError::Malformed)?;
    if expected_page_items(total, query.page) != Some(items.len()) {
        return Err(LoadError::Malformed);
    }
    Ok(NotificationPage {
        items,
        total,
        page: query.page,
        total_pages: total_pages(total),
        status: NotificationStatusFilter::from_param(query.status.as_deref())
            .ok_or(LoadError::InvalidQuery)?,
        notification_type: NotificationTypeFilter::from_param(query.notification_type.as_deref())
            .ok_or(LoadError::InvalidQuery)?,
        priority: NotificationPriorityFilter::from_param(query.priority.as_deref())
            .ok_or(LoadError::InvalidQuery)?,
        start_date: query.start_date,
        end_date: query.end_date,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MutationKind {
    Read,
    Unread,
    Acknowledge,
    Dismiss,
    Delete,
    MarkAll,
    ClearAll,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationMutation {
    pub kind: MutationKind,
    pub id: Option<String>,
}
impl NotificationMutation {
    pub fn validate(&self) -> Result<(), LoadError> {
        if matches!(self.kind, MutationKind::MarkAll | MutationKind::ClearAll) {
            if self.id.is_none() {
                Ok(())
            } else {
                Err(LoadError::InvalidQuery)
            }
        } else if self.id.as_ref().is_some_and(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        }) {
            Ok(())
        } else {
            Err(LoadError::InvalidQuery)
        }
    }
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct NotificationsProvider(pub NotificationsProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct NotificationMutationProvider(pub NotificationMutationProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "notifications")]
pub async fn read_notifications(
    query: String,
) -> Result<Result<NotificationPage, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<NotificationsProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Notifications provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "notification-change")]
pub async fn change_notification(
    command: NotificationMutation,
) -> Result<Result<(), LoadError>, ServerFnError> {
    if let Err(error) = command.validate() {
        return Ok(Err(error));
    }
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<NotificationMutationProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Notifications provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(command, headers).await)
}
#[derive(Clone, Copy)]
pub(super) struct NotificationsControls {
    pub navigate: EventHandler<String>,
    pub mutate: EventHandler<NotificationMutation>,
    pub pending: ReadSignal<bool>,
    pub live: ReadSignal<String>,
}
#[component]
pub(super) fn NotificationLink(
    href: String,
    class: String,
    #[props(default)] rel: String,
    #[props(default)] aria_current: String,
    children: Element,
) -> Element {
    let control = try_use_context::<NotificationsControls>();
    let target = href.clone();
    rsx! { crate::navigation::AppLink { href, class, rel, aria_current, onclick: move |event: MouseEvent| {
        if event.modifiers().is_empty() { if let Some(control) = control { event.prevent_default(); control.navigate.call(target.clone()); } }
    }, {children} } }
}
#[component]
pub fn HydratedNotifications(query: ReadSignal<String>) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move { read_notifications(query).await }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Ok(Err(LoadError::Unavailable)))
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut mutation_pending = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut requested_query = use_signal(|| initial_query);
    let mut refresh = use_signal(|| 0_u64);
    let mut live = use_signal(|| "Live notification updates are connecting…".to_string());
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        if url == format!("/notifications?{}", query())
            || (url == "/notifications" && query().is_empty())
        {
            let next = *refresh.peek() + 1;
            refresh.set(next);
        } else {
            navigator.push(url);
        }
    });
    let mutate = use_callback(move |command: NotificationMutation| {
        if *mutation_pending.peek() {
            return;
        }
        mutation_pending.set(true);
        error.set(None);
        spawn(async move {
            let result = change_notification(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            mutation_pending.set(false);
            match result {
                Ok(()) => {
                    let next = *refresh.peek() + 1;
                    refresh.set(next);
                }
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    use_context_provider(|| NotificationsControls {
        navigate,
        mutate,
        pending: mutation_pending.into(),
        live: live.into(),
    });
    use_effect(move || {
        let requested = query();
        let revision = refresh();
        if requested == *requested_query.peek() && revision == 0 {
            return;
        }
        requested_query.set(requested.clone());
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = read_notifications(requested)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(page) => data.set(Some(page)),
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    let mut stream = use_signal(|| None::<document::Eval>);
    use_drop(move || {
        if let Some(stream) = *stream.peek() {
            let _ = stream.send("close");
        }
    });
    rsx! {
        document::Title { "Notifications — EPSX" }
        document::Meta { name: "description", content: "Read and manage your account notifications." }
        div { class: "container page-content notifications-page fe-page-layout", "data-dioxus-notifications": "true", aria_busy: pending() || mutation_pending(),
            onmounted: move |_| {
                if stream.peek().is_some() { return; }
                let mut evaluator = document::eval("const source = new EventSource('/api/v1/notifications/stream'); source.onopen = () => dioxus.send('connected'); source.onmessage = () => dioxus.send('changed'); source.addEventListener('notification', () => dioxus.send('changed')); source.onerror = () => dioxus.send('retrying'); try { await dioxus.recv(); } finally { source.close(); }");
                stream.set(Some(evaluator));
                spawn(async move { while let Ok(event) = evaluator.recv::<String>().await {
                    match event.as_str() { "changed" => { let next = *refresh.peek() + 1; refresh.set(next); live.set("New notification received. Updating…".into()); }, "connected" => live.set("Live notification updates connected.".into()), _ => live.set("Live updates interrupted; reconnecting. You can still refresh manually.".into()) }
                } });
            },
            PageHeader { title: "Notifications".to_string(), description: data.read().as_ref().map(NotificationPage::loaded_summary), icon: None }
            if let Some(failure) = error() { crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),  } }
            if error() != Some(LoadError::Unauthenticated) { button { r#type: "button", class: "btn btn-sm btn-outline", disabled: pending(), onclick: move |_| { let next = *refresh.peek() + 1; refresh.set(next); }, "Refresh notifications" } }
            if let Some(page) = data() { NotificationPageSection { page } }
        }
    }
}

#[cfg(feature = "server")]
pub type NotificationsProviderCallback = std::sync::Arc<
    dyn Fn(
            String,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<NotificationPage, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type NotificationMutationProviderCallback = std::sync::Arc<
    dyn Fn(
            NotificationMutation,
            http::HeaderMap,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), LoadError>> + Send>>
        + Send
        + Sync,
>;
