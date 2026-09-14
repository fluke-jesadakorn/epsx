//! EPSX identities and access as returned by the canonical Rust backend.
use super::LoadError;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Wallet {
    pub wallet_address: String,
    pub is_active: bool,
    pub created_at: String,
    pub last_auth_at: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalletList {
    pub wallets: Vec<Wallet>,
    pub total: i64,
    pub pagination: Pagination,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub has_next: bool,
    pub has_prev: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    pub permission: String,
    pub source: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub wallet_address: String,
    pub plan_id: String,
    pub plan_name: String,
    pub expires_at: Option<String>,
    pub is_active: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanOption {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Detail {
    #[serde(flatten)]
    pub wallet: Wallet,
    pub permissions: Vec<Permission>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Data {
    List(WalletList),
    Detail {
        detail: Detail,
        assignments: Result<Vec<Assignment>, LoadError>,
        plans: Result<Vec<PlanOption>, LoadError>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Status {
        wallet: String,
        active: bool,
    },
    Assign {
        wallet: String,
        plan_id: String,
        expires_at: Option<String>,
        reason: String,
    },
    RevokePlan {
        assignment_id: String,
    },
    GrantPermission {
        wallet: String,
        permission: String,
        expires_at: Option<String>,
        reason: String,
    },
    RevokePermission {
        wallet: String,
        permission: String,
    },
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct Provider {
    pub read: std::sync::Arc<
        dyn Fn(Option<String>, String, http::HeaderMap) -> Future<Data> + Send + Sync,
    >,
    pub command: std::sync::Arc<dyn Fn(Command, http::HeaderMap) -> Future<()> + Send + Sync>,
}
#[server(prefix = "/_server/admin", endpoint = "core_wallets_read")]
pub async fn read_wallets(
    address: Option<String>,
    query: String,
) -> Result<Result<Data, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<Provider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((provider.read)(address, query, headers).await)
}
#[server(prefix = "/_server/admin", endpoint = "core_wallets_command")]
pub async fn command_wallet(command: Command) -> Result<Result<(), LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(provider) =
        dioxus_fullstack::FullstackContext::extract::<Extension<Provider>, _>().await?;
    let headers = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((provider.command)(command, headers).await)
}
#[component]
pub fn CoreWallets(address: Option<String>, query: String) -> Element {
    // Route props are not signals: explicitly subscribe so query-only navigation
    // cancels the previous request and reads the selected page.
    let mut initial =
        use_server_future(use_reactive!(|address, query| read_wallets(address, query)))?;
    let data = initial
        .read()
        .clone()
        .and_then(Result::ok)
        .unwrap_or(Err(LoadError::Unavailable));
    let mut pending = use_signal(|| false);
    let mut result = use_signal(|| None::<Result<(), LoadError>>);
    let execute = use_callback(move |command: Command| {
        spawn(async move {
            pending.set(true);
            let outcome = command_wallet(command)
                .await
                .unwrap_or(Err(LoadError::Unavailable));
            if outcome.is_ok() {
                initial.restart();
            }
            result.set(Some(outcome));
            pending.set(false);
        });
    });
    rsx! {
        document::Title { "Wallets & access | EPSX Admin" }
        document::Link { rel: "stylesheet", href: "/public/dist/tailwind.css" }
        document::Link { rel: "stylesheet", href: "/_ui/admin.css" }
        main { class: "container-x max-w-6xl mx-auto py-10 space-y-6",
            nav { class: "flex gap-5",
                Link { to: "/", "Admin home" }
                Link { to: "/wallet-management/wallets", "Wallets & access" }
                Link { to: "/plans", "EPSX Plans" }
                Link { to: "/wallet-management/credits", "Credits" }
                Link { to: "/payments/epsx", "Plan purchases" }
            }
            h1 { class: "text-3xl font-bold", "Wallets & access" }
            if let Some(outcome) = result() {
                match outcome {
                    Ok(()) => rsx! { p { role: "status", "Saved." } },
                    Err(e) => rsx! { crate::fullstack::load_error::LoadErrorNotice { error: e.clone(),  } },
                }
            }
            match data {
                Err(e) => rsx! { crate::fullstack::load_error::LoadErrorNotice { error: e.clone(),  } },
                Ok(Data::List(list)) => rsx! { WalletRows { list, query: query.clone() } },
                Ok(Data::Detail { detail, assignments, plans }) => rsx! {
                    WalletDetails { detail, assignments, plans, pending: pending(), execute }
                },
            }
        }
    }
}
#[component]
fn WalletRows(list: WalletList, query: String) -> Element {
    let filter = crate::pages::admin_pages::wallet_wallets::AdminWalletListQuery::from_raw(&query)
        .unwrap_or_default();
    let status = filter.status.as_deref().unwrap_or("all").to_owned();
    let nav = use_navigator();
    rsx! {
        form { class: "flex flex-wrap gap-3", onsubmit: move |event| {
            event.prevent_default();
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            serializer.append_pair("page", "1");
            for key in ["search", "status", "limit"] { serializer.append_pair(key, &event.values().iter().find(|(k, _)| k == key).and_then(|(_, v)| match v { dioxus::html::FormValue::Text(s) => Some(s.clone()), _ => None }).unwrap_or_default()); }
            nav.push(format!("/wallet-management/wallets?{}", serializer.finish()));
        },
            input { r#type: "hidden", name: "limit", value: "{list.pagination.limit}" }
            input { name: "search", value: filter.search.unwrap_or_default(), placeholder: "Search wallet address", maxlength: 42, class: "input input-bordered" }
            select { name: "status", value: status.clone(), class: "select select-bordered",
                option { value: "all", selected: status == "all", "All statuses" }
                option { value: "active", selected: status == "active", "Active" }
                option { value: "disabled", selected: status == "disabled", "Disabled" }
            }
            button { r#type: "submit", class: "btn btn-outline", "Search" }
        }
        p { "{list.total} wallets · Page {list.pagination.page}" }
        if list.wallets.is_empty() { p { "No wallets match these filters." } }
        for wallet in list.wallets {
            article { class: "rounded-xl border p-5 space-y-2",
                Link { to: format!("/wallet-management/{}", wallet.wallet_address), class: "font-mono underline break-all", "{wallet.wallet_address}" }
                p { if wallet.is_active { "Active" } else { "Disabled" } }
                p { class: "text-sm", "Created: {wallet.created_at}" }
                p { class: "text-sm", "Last sign-in: {wallet.last_auth_at.as_deref().unwrap_or(\"No sign-in recorded\")}" }
            }
        }
        div { class: "flex gap-4",
            if list.pagination.has_prev { Link { to: page_url(&query, list.pagination.page - 1, list.pagination.limit), "Previous" } }
            if list.pagination.has_next { Link { to: page_url(&query, list.pagination.page + 1, list.pagination.limit), "Next" } }
        }
    }
}
fn page_url(query: &str, page: u32, limit: u32) -> String {
    let mut params = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        if key == "search" || key == "status" {
            params.append_pair(&key, &value);
        }
    }
    params
        .append_pair("page", &page.to_string())
        .append_pair("limit", &limit.to_string());
    format!("/wallet-management/wallets?{}", params.finish())
}

#[component]
fn WalletDetails(
    detail: Detail,
    assignments: Result<Vec<Assignment>, LoadError>,
    plans: Result<Vec<PlanOption>, LoadError>,
    pending: bool,
    execute: Callback<Command>,
) -> Element {
    let address = detail.wallet.wallet_address.clone();
    let status_address = address.clone();
    let plan_address = address.clone();
    let permission_address = address.clone();
    let active = detail.wallet.is_active;
    rsx! {
        h2 { class: "font-mono break-all", "{address}" }
        p { if active { "Account active" } else { "Account disabled" } }
        button { class: "btn btn-outline", disabled: pending, onclick: move |_| execute.call(Command::Status { wallet: status_address.clone(), active: !active }),
            if active { "Disable account" } else { "Enable account" }
        }
        h2 { class: "text-xl font-semibold", "Plan assignments" }
        match assignments {
            Err(e) => rsx! { crate::fullstack::load_error::LoadErrorNotice { error: e.clone(),  } },
            Ok(items) => rsx! {
                if items.is_empty() { p { "No plan assignments." } }
                for row in items {
                    article { class: "border rounded-xl p-4 space-y-2",
                        strong { "{row.plan_name}" }
                        p { if row.is_active { "Enabled" } else { "Revoked" } }
                        p { "Expires: {row.expires_at.as_deref().unwrap_or(\"No expiry\")}" }
                        if row.is_active { button { class: "btn btn-outline", disabled: pending, onclick: move |_| execute.call(Command::RevokePlan { assignment_id: row.id.clone() }), "Revoke plan" } }
                    }
                }
            },
        }
        if let Ok(options) = plans {
            form { class: "grid gap-3 border rounded-xl p-4", onsubmit: move |event| {
                event.prevent_default();
                let values = event.values();
                let field = |key: &str| values.iter().find(|(k, _)| k == key).and_then(|(_, v)| match v { dioxus::html::FormValue::Text(s) => Some(s.clone()), _ => None }).unwrap_or_default();
                let expiry = field("expires_at");
                execute.call(Command::Assign { wallet: plan_address.clone(), plan_id: field("plan_id"), expires_at: (!expiry.is_empty()).then_some(expiry), reason: field("reason") });
            },
                label { "Assign EPSX plan", select { name: "plan_id", required: true, class: "select select-bordered", for plan in options { option { value: "{plan.id}", "{plan.name}" } } } }
                label { "Expiry (UTC; blank uses plan default)", input { name: "expires_at", placeholder: "2026-12-31T00:00:00Z", class: "input input-bordered" } }
                label { "Reason", input { name: "reason", required: true, maxlength: 1000, class: "input input-bordered" } }
                button { r#type: "submit", disabled: pending, class: "btn btn-primary", "Assign plan" }
            }
        }
        h2 { class: "text-xl font-semibold", "Permissions" }
        for permission in detail.permissions {
            article { class: "border rounded-xl p-4 space-y-2",
                code { "{permission.permission}" }
                p { "Source: {permission.source} · Expires: {permission.expires_at.as_deref().unwrap_or(\"No expiry\")}" }
                p { if permission.is_active { "Enabled" } else { "Revoked" } }
                if permission.source == "direct" && permission.is_active {
                    button { class: "btn btn-outline", disabled: pending, onclick: {
                        let address = permission_address.clone();
                        move |_| execute.call(Command::RevokePermission { wallet: address.clone(), permission: permission.permission.clone() })
                    }, "Revoke direct permission" }
                }
            }
        }
        form { class: "grid gap-3 border rounded-xl p-4", onsubmit: move |event| {
            event.prevent_default();
            let values = event.values();
            let field = |key: &str| values.iter().find(|(k, _)| k == key).and_then(|(_, v)| match v { dioxus::html::FormValue::Text(s) => Some(s.clone()), _ => None }).unwrap_or_default();
            let expiry = field("expires_at");
            execute.call(Command::GrantPermission { wallet: address.clone(), permission: field("permission"), expires_at: (!expiry.is_empty()).then_some(expiry), reason: field("reason") });
        },
            label { "Direct permission", input { name: "permission", placeholder: "epsx:analytics:read", required: true, maxlength: 256, class: "input input-bordered" } }
            label { "Expiry (UTC; blank has no expiry)", input { name: "expires_at", placeholder: "2026-12-31T00:00:00Z", class: "input input-bordered" } }
            label { "Reason", input { name: "reason", required: true, maxlength: 1000, class: "input input-bordered" } }
            button { r#type: "submit", disabled: pending, class: "btn btn-primary", "Grant permission" }
        }
    }
}
