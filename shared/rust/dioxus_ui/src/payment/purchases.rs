//! Owner-scoped purchase data and reactive Dioxus page.
use crate::fullstack::LoadError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseQuery {
    pub order_id: Option<uuid::Uuid>,
    pub offset: u64,
}
impl PurchaseQuery {
    pub fn validate(&self) -> Result<(), LoadError> {
        if self.offset > 1_000_000 || (self.order_id.is_some() && self.offset != 0) {
            Err(LoadError::InvalidQuery)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub order_id: uuid::Uuid,
    pub plan_name: String,
    pub wallet_address: String,
    pub amount: String,
    pub token: String,
    pub token_decimals: u32,
    pub status: String,
    pub fulfillment_status: String,
    pub created_at: String,
    pub payment_id: Option<String>,
    pub contract_address: Option<String>,
    pub payment_status: Option<String>,
    pub payment_available: Option<bool>,
    pub tx_hash: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PurchaseList {
    pub orders: Vec<Purchase>,
    pub next_offset: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PurchaseData {
    List(PurchaseList),
    Detail(Box<Purchase>),
}

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct PurchasesProvider(pub PurchasesProviderCallback);

#[server(prefix = "/_server/frontend", endpoint = "purchases")]
pub async fn read_purchases(
    query: PurchaseQuery,
) -> Result<Result<PurchaseData, LoadError>, ServerFnError> {
    if let Err(error) = query.validate() {
        return Ok(Err(error));
    }
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<PurchasesProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Purchases provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(query, headers).await)
}

#[derive(Clone, Copy)]
pub struct PurchasesNavigation {
    pub navigate: EventHandler<String>,
    pub refresh: EventHandler<()>,
    pub pending: ReadSignal<bool>,
}
pub fn follow(event: MouseEvent, navigation: Option<PurchasesNavigation>, url: String) {
    if event.modifiers().is_empty() {
        if let Some(navigation) = navigation {
            event.prevent_default();
            navigation.navigate.call(url);
        }
    }
}

#[component]
pub fn HydratedPurchases(query: ReadSignal<PurchaseQuery>) -> Element {
    let initial_query = use_hook(|| query.read().clone());
    let seed_query = initial_query.clone();
    let initial = use_server_future(move || {
        let query = seed_query.clone();
        async move {
            read_purchases(query)
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
            let result = read_purchases(requested)
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
        document::Title { "Plan purchases — EPSX" }
        document::Meta { name: "description", content: "Your payment history and plan activation, in one place." }
        section { "data-dioxus-purchases": "true", aria_busy: pending(),
            if pending() { p { role: "status", class: "fe-purchase-note", "Updating purchases…" } }
            if let Some(failure) = error() {
                div {
                    crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),
                    button { r#type: "button", class: "fe-button", disabled: pending(), onclick: move |_| refresh_handler.call(()), "Try again" } }
                }
            }
            if let Some(snapshot) = data() {
                // The existing display component is shared with native admin SSR.
                // Transport and hydrated state are typed; conversion stays local to presentation.
                crate::payment::orders::OrdersPage { admin: false, data: match snapshot {
                    PurchaseData::List(list) => serde_json::to_value(list).unwrap_or_default(),
                    PurchaseData::Detail(order) => serde_json::to_value(order).unwrap_or_default(),
                } }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type PurchasesProviderCallback = std::sync::Arc<
    dyn Fn(
            PurchaseQuery,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<PurchaseData, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn purchase_query_rejects_invalid_windows() {
        assert!(PurchaseQuery {
            order_id: None,
            offset: 1_000_000
        }
        .validate()
        .is_ok());
        assert!(PurchaseQuery {
            order_id: None,
            offset: 1_000_001
        }
        .validate()
        .is_err());
        assert!(PurchaseQuery {
            order_id: Some(uuid::Uuid::nil()),
            offset: 1
        }
        .validate()
        .is_err());
    }
}
