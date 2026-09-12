//! Admin-authorized plan purchase reads and reactive pagination.
use crate::{
    fullstack::LoadError,
    payment::purchases::{PurchaseData, PurchaseQuery, PurchasesNavigation},
};
use dioxus::prelude::*;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AdminOrdersProvider(pub AdminOrdersProviderCallback);

#[server(prefix = "/_server/admin", endpoint = "orders")]
pub async fn read_admin_orders(
    query: PurchaseQuery,
) -> Result<Result<PurchaseData, LoadError>, ServerFnError> {
    if let Err(error) = query.validate() {
        return Ok(Err(error));
    }
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AdminOrdersProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Purchases provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}

#[component]
pub fn HydratedAdminOrders(query: ReadSignal<PurchaseQuery>) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move {
            read_admin_orders(query)
                .await
                .map_err(|_| LoadError::Unavailable)?
        }
    })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut generation = use_signal(|| 0_u64);
    let mut last_query = use_signal(|| initial_query);
    let mut refresh = use_signal(|| 0_u64);
    let mut last_refresh = use_signal(|| 0_u64);
    let navigator = use_navigator();
    let navigate = use_callback(move |url: String| {
        navigator.push(url);
    });
    let refresh_handler = use_callback(move |()| {
        let next = *refresh.peek() + 1;
        refresh.set(next);
    });
    use_context_provider(|| PurchasesNavigation {
        navigate,
        refresh: refresh_handler,
        pending: pending.into(),
    });
    use_effect(move || {
        let requested = query();
        let revision = refresh();
        if requested == *last_query.peek() && revision == *last_refresh.peek() {
            return;
        }
        let ticket = *generation.peek() + 1;
        generation.set(ticket);
        if requested != *last_query.peek() {
            data.set(None);
        }
        last_query.set(requested.clone());
        last_refresh.set(revision);
        pending.set(true);
        error.set(None);
        spawn(async move {
            let result = read_admin_orders(requested)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if *generation.peek() != ticket {
                return;
            }
            pending.set(false);
            match result {
                Ok(value) => data.set(Some(value)),
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    rsx! {
      super::admin::AdminAnalyticsShell { authenticated:data().is_some(), current_path:"/payments/epsx".to_string(),title:"Plan purchases".to_string(),
        document::Title { "Plan purchases — EPSX Admin" }
        document::Meta { name: "description", content: "Your payment history and plan activation, in one place." }
        section { class:"container-x max-w-6xl mx-auto py-10 space-y-6", "data-dioxus-purchases": "true", aria_busy: pending(),
            if pending() { p { role: "status", class: "rounded-xl border bg-card p-6 space-y-4", "Updating purchases…" } }
            if let Some(failure) = error() {
                div { role: "status", class: "rounded-xl border bg-card p-6 space-y-4",
                    p { "{failure.message()}" }
                    button { r#type: "button", class: "btn btn-outline", disabled: pending(), onclick: move |_| refresh_handler.call(()), "Try again" }
                }
            }
            if let Some(snapshot) = data() {
                // The existing display component is shared with native admin SSR.
                // Transport and hydrated state are typed; conversion stays local to presentation.
                crate::payment::orders::OrdersPage { admin: true, data: match snapshot {
                    PurchaseData::List(list) => serde_json::to_value(list).unwrap_or_default(),
                    PurchaseData::Detail(order) => serde_json::to_value(order).unwrap_or_default(),
                } }
            }
        }
      }
    }
}

#[cfg(feature = "server")]
pub type AdminOrdersProviderCallback = std::sync::Arc<
    dyn Fn(
            PurchaseQuery,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<PurchaseData, LoadError>> + Send>,
        > + Send
        + Sync,
>;
