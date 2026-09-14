use super::*;
use crate::components::account::PayHistory;
use crate::fullstack::LoadError;
use crate::pages::account_credits::CreditBalanceProjection;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountData {
    pub user: User,
    pub profile: Result<AccountProfileProjection, LoadError>,
    pub access: Result<AccountAccessProjection, LoadError>,
    pub credits: Result<CreditBalanceProjection, LoadError>,
    pub plan_payments: Result<AccountPlanPaymentsProjection, LoadError>,
    pub payments: Result<PayHistory, LoadError>,
    pub preferences: Result<NotificationPreferencesPayload, LoadError>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreferencesInput {
    pub channels: BTreeMap<String, bool>,
    pub quiet_hours: Option<NotificationQuietHours>,
    pub timezone: Option<String>,
}
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AccountProvider(pub AccountProviderCallback);
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct AccountMutationProvider(pub AccountMutationProviderCallback);
#[server(prefix = "/_server/frontend", endpoint = "account")]
pub async fn read_account() -> Result<Result<AccountData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AccountProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Account provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(headers).await)
}
#[server(prefix = "/_server/frontend", endpoint = "account-preferences")]
pub async fn save_preferences(
    input: PreferencesInput,
) -> Result<Result<NotificationPreferencesPayload, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<AccountMutationProvider>, _>()
            .await
            .map_err(|_| ServerFnError::new("Account provider unavailable"))?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("Request context unavailable"))?;
    Ok((provider.0)(input, headers).await)
}
#[derive(Clone, Copy)]
pub(super) struct PreferencesEditor {
    pub submit: EventHandler<FormEvent>,
    pub pending: ReadSignal<bool>,
}
fn form_input(event: &FormEvent) -> Result<PreferencesInput, LoadError> {
    let values = event.values();
    let text = |name: &str| {
        values
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
            .and_then(|value| match value {
                dioxus::html::FormValue::Text(value) => Some(value.clone()),
                _ => None,
            })
            .ok_or(LoadError::InvalidQuery)
    };
    let flag = |name| match text(name)?.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(LoadError::InvalidQuery),
    };
    Ok(PreferencesInput {
        channels: ["email", "in_app", "push"]
            .into_iter()
            .map(|key| flag(key).map(|value| (key.into(), value)))
            .collect::<Result<_, _>>()?,
        quiet_hours: Some(NotificationQuietHours {
            start: text("quiet_start")?,
            end: text("quiet_end")?,
            enabled: Some(flag("quiet_enabled")?),
        }),
        timezone: Some(text("timezone")?),
    })
}
#[derive(Clone, Copy)]
pub struct AccountRefresh(pub EventHandler<()>);
#[component]
pub fn AccountLink(href: String, class: String, children: Element) -> Element {
    let refresh = try_use_context::<AccountRefresh>();
    if href == "/account" && refresh.is_some() {
        rsx! { crate::navigation::AppLink { href, class, onclick: move |event: MouseEvent| { if event.modifiers().is_empty() { event.prevent_default(); if let Some(refresh) = refresh { refresh.0.call(()); } } }, {children} } }
    } else {
        rsx! { crate::fullstack::shell::ShellLink { href, class, {children} } }
    }
}

#[component]
pub fn HydratedAccount() -> Element {
    let initial = use_server_future(move || async move { read_account().await })?;
    let seed = initial
        .read()
        .clone()
        .unwrap_or(Ok(Err(LoadError::Unavailable)))
        .unwrap_or(Err(LoadError::Unavailable));
    let mut data = use_signal(|| seed.clone().ok());
    let mut error = use_signal(|| seed.err());
    let mut pending = use_signal(|| false);
    let mut form_state = use_signal(|| NotificationPreferencesFormState::None);
    let refresh = use_callback(move |()| {
        if *pending.peek() {
            return;
        }
        pending.set(true);
        error.set(None);
        spawn(async move {
            let response = read_account().await.unwrap_or(Err(LoadError::Unavailable));
            pending.set(false);
            match response {
                Ok(snapshot) => data.set(Some(snapshot)),
                Err(failure) => {
                    if matches!(failure, LoadError::Unauthenticated | LoadError::Forbidden) {
                        data.set(None);
                    }
                    error.set(Some(failure));
                }
            }
        });
    });
    use_context_provider(|| AccountRefresh(refresh));
    let submit = use_callback(move |event: FormEvent| {
        event.prevent_default();
        if *pending.peek() {
            return;
        }
        let input = match form_input(&event) {
            Ok(input) => input,
            Err(_) => {
                form_state.set(NotificationPreferencesFormState::Error);
                return;
            }
        };
        pending.set(true);
        form_state.set(NotificationPreferencesFormState::None);
        spawn(async move {
            let result = save_preferences(input)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            pending.set(false);
            match result {
                Ok(preferences) => {
                    if let Some(snapshot) = data.write().as_mut() {
                        snapshot.preferences = Ok(preferences);
                    }
                    form_state.set(NotificationPreferencesFormState::Saved);
                }
                Err(LoadError::Unauthenticated) => {
                    data.set(None);
                    error.set(Some(LoadError::Unauthenticated));
                }
                Err(_) => form_state.set(NotificationPreferencesFormState::Error),
            }
        });
    });
    use_context_provider(|| PreferencesEditor {
        submit,
        pending: pending.into(),
    });
    rsx! {
        document::Title { "Account — EPSX" }
        document::Meta { name: "description", content: "Manage your access, billing, and notification preferences." }
        section { "data-dioxus-account": "true", aria_busy: pending(),
            if let Some(failure) = error() { crate::fullstack::load_error::LoadErrorNotice { error: failure.clone(),  button { r#type: "button", class: "fe-button", disabled: pending(), onclick: move |_| refresh.call(()), "Try again" } } }
            if let Some(snapshot) = data() {
                AccountBody {
                    session_user: Some(snapshot.user.clone()), payment_history_address: Some(snapshot.user.address),
                    profile_load: match snapshot.profile { Ok(value) => AccountProfileLoad::Ready(value), Err(LoadError::Malformed) => AccountProfileLoad::Malformed, Err(_) => AccountProfileLoad::Unavailable },
                    access_load: match snapshot.access { Ok(value) => AccountAccessLoad::Ready(value), Err(LoadError::Malformed) => AccountAccessLoad::Malformed, Err(_) => AccountAccessLoad::Unavailable },
                    credit_balance: match snapshot.credits { Ok(value) => CreditBalanceLoad::Ready(value), Err(LoadError::Malformed) => CreditBalanceLoad::Malformed, Err(_) => CreditBalanceLoad::Unavailable },
                    plan_payments_load: match snapshot.plan_payments { Ok(value) if value.payments.is_empty() => AccountPlanPaymentsLoad::Empty, Ok(value) => AccountPlanPaymentsLoad::Ready(value), Err(LoadError::Malformed) => AccountPlanPaymentsLoad::Malformed, Err(_) => AccountPlanPaymentsLoad::Unavailable },
                    payment_history_load: match snapshot.payments { Ok(value) if value.intents.is_empty() && value.escrows.is_empty() => PaymentHistoryLoad::Empty, Ok(value) => PaymentHistoryLoad::Ready(value), Err(LoadError::Malformed) => PaymentHistoryLoad::Malformed, Err(_) => PaymentHistoryLoad::Unavailable },
                    notification_preferences_load: match snapshot.preferences { Ok(value) => NotificationPreferencesLoad::Ready(value), Err(LoadError::Malformed) => NotificationPreferencesLoad::Malformed, Err(_) => NotificationPreferencesLoad::Unavailable },
                    notification_preferences_form_state: form_state(),
                }
            }
        }
    }
}

#[cfg(feature = "server")]
pub type AccountProviderCallback = std::sync::Arc<
    dyn Fn(
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<AccountData, LoadError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(feature = "server")]
pub type AccountMutationProviderCallback = std::sync::Arc<
    dyn Fn(
            PreferencesInput,
            http::HeaderMap,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<NotificationPreferencesPayload, LoadError>>
                    + Send,
            >,
        > + Send
        + Sync,
>;
