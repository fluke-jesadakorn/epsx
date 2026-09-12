//! Wallet management UI contracts. Versions, permissions and credit mutations
//! remain backend-owned; Dioxus transports explicit user commands only.
use super::LoadError;
pub use crate::pages::admin_pages::{
    wallet_access::AdminAccessProjection,
    wallet_credits::AdminCreditStatsProjection,
    wallet_plans::{AdminPlanListProjection, AdminPlanProjection},
    wallet_wallets::{
        AdminWalletDetailProjection, AdminWalletListProjection, AdminWalletStatsSummary,
    },
};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
mod ui;
pub use ui::HydratedAdminWallets;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletPage {
    List,
    Detail(String),
    Disable(String),
    Access,
    Credits,
    Plans,
    Plan(String),
    InvalidQuery,
    NotFound,
}
impl WalletPage {
    pub fn from_path(path: &str) -> Self {
        match path {
            "/wallet-management" | "/wallet-management/wallets" => Self::List,
            "/wallet-management/access" => Self::Access,
            "/wallet-management/credits" => Self::Credits,
            "/wallet-management/access/plans" => Self::Plans,
            _ => {
                let parts = path.trim_matches('/').split('/').collect::<Vec<_>>();
                match parts.as_slice() {
                    ["wallet-management", "access", "plans", id] => Self::Plan((*id).into()),
                    ["wallet-management", "wallets", address, "disable"] => {
                        Self::Disable((*address).into())
                    }
                    ["wallet-management", address] => Self::Detail((*address).into()),
                    ["wallet-management", "wallets", address] => Self::Detail((*address).into()),
                    _ => Self::NotFound,
                }
            }
        }
    }
    pub fn path(&self) -> String {
        match self {
            Self::List => "/wallet-management/wallets".into(),
            Self::Detail(v) => format!("/wallet-management/{v}"),
            Self::Disable(v) => format!("/wallet-management/wallets/{v}/disable"),
            Self::Access => "/wallet-management/access".into(),
            Self::Credits => "/wallet-management/credits".into(),
            Self::Plans => "/wallet-management/access/plans".into(),
            Self::Plan(v) => format!("/wallet-management/access/plans/{v}"),
            Self::InvalidQuery => "/wallet-management/wallets".into(),
            Self::NotFound => "/wallet-management/not-found".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletFilter {
    pub search: Option<String>,
    pub status: Option<String>,
    pub page: i64,
    pub limit: i64,
}
impl Default for WalletFilter {
    fn default() -> Self {
        Self {
            search: None,
            status: None,
            page: 1,
            limit: 10,
        }
    }
}
impl WalletFilter {
    pub fn url(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        if let Some(s) = &self.search {
            q.append_pair("search", s);
        }
        if let Some(s) = &self.status {
            q.append_pair("status", s);
        }
        q.append_pair("page", &self.page.to_string());
        q.append_pair("limit", &self.limit.to_string());
        format!("/wallet-management/wallets?{}", q.finish())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletRequest {
    pub page: WalletPage,
    pub filter: WalletFilter,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalletData {
    pub stats: Result<AdminWalletStatsSummary, LoadError>,
    pub wallets: Option<AdminWalletListProjection>,
    pub detail: Option<AdminWalletDetailProjection>,
    pub access: Option<AdminAccessProjection>,
    pub credits: Option<AdminCreditStatsProjection>,
    pub plans: Option<AdminPlanListProjection>,
    pub plan: Option<AdminPlanProjection>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WalletCommand {
    Access {
        revoke: bool,
        wallet_address: String,
        plan_id: String,
        permission: String,
        expected_version: i64,
    },
    Credit {
        revoke: bool,
        wallet_address: String,
        amount_minor: i64,
        reason: String,
        expected_version: i64,
    },
    Plan {
        plan_id: Option<String>,
        merchant_id: Option<String>,
        name: String,
        description: String,
        amount: String,
        currency: String,
        chain_id: String,
        interval: i32,
        active: Option<bool>,
        expected_version: Option<i64>,
    },
    Disable {
        address: String,
        reason: String,
        expected_version: i64,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalletMutation {
    pub committed: bool,
    pub message: String,
}
#[cfg(feature = "server")]
type Future<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LoadError>> + Send>>;
#[cfg(feature = "server")]
#[derive(Clone)]
pub struct WalletProvider {
    pub read:
        std::sync::Arc<dyn Fn(WalletRequest, http::HeaderMap) -> Future<WalletData> + Send + Sync>,
    pub command: std::sync::Arc<
        dyn Fn(WalletCommand, String, http::HeaderMap) -> Future<WalletMutation> + Send + Sync,
    >,
}
#[server(prefix = "/_server/admin", endpoint = "wallets_read")]
pub async fn wallets_read(
    request: WalletRequest,
) -> Result<Result<WalletData, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<WalletProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.read)(request, h).await)
}
#[server(prefix = "/_server/admin", endpoint = "wallets_action")]
pub async fn wallets_action(
    command: WalletCommand,
    key: String,
) -> Result<Result<WalletMutation, LoadError>, ServerFnError> {
    use dioxus_server::axum::Extension;
    let Extension(p) =
        dioxus_fullstack::FullstackContext::extract::<Extension<WalletProvider>, _>().await?;
    let h = dioxus_fullstack::FullstackContext::extract::<http::HeaderMap, _>().await?;
    Ok((p.command)(command, key, h).await)
}
