//! Reuse existing strict commerce projections and validated/versioned mutation
//! handlers. No permissions or credit accounting rules are moved into the UI.
use crate::{commerce_adapter as adapter, AppState};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request},
};
use epsx_dioxus_ui::fullstack::{admin_wallets::*, LoadError};
fn required<T>(value: adapter::AdminCommerceLoad<T>) -> Result<T, LoadError> {
    match value {
        adapter::AdminCommerceLoad::Ready(value) => Ok(value),
        adapter::AdminCommerceLoad::Forbidden => Err(LoadError::Forbidden),
        adapter::AdminCommerceLoad::Unauthorized => Err(LoadError::Unauthenticated),
        adapter::AdminCommerceLoad::Malformed => Err(LoadError::Malformed),
        adapter::AdminCommerceLoad::Empty => Err(LoadError::NotFound),
        adapter::AdminCommerceLoad::Unavailable => Err(LoadError::Unavailable),
    }
}
fn empty_plans() -> AdminPlanListProjection {
    AdminPlanListProjection {
        items: vec![],
        total: 0,
        limit: 100,
        offset: 0,
    }
}
pub async fn read(
    state: AppState,
    request: WalletRequest,
    headers: HeaderMap,
) -> Result<WalletData, LoadError> {
    if request.page == WalletPage::InvalidQuery {
        return Err(LoadError::InvalidQuery);
    }
    if request.page == WalletPage::NotFound {
        return Err(LoadError::NotFound);
    }
    let Some((token, _)) = state.session().verified_access_token(&headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut context = epsx_client::RequestContext::from_headers(&headers);
    context.auth_token = Some(token);
    let mut stats = adapter::load_wallet_stats(&state.wallet, &context).await;
    if matches!(stats, adapter::AdminCommerceLoad::Unavailable) {
        stats = adapter::load_wallet_stats(&state.identity, &context).await;
    }
    let mut data = WalletData {
        stats: required(stats),
        wallets: None,
        detail: None,
        access: None,
        credits: None,
        plans: None,
        plan: None,
    };
    match request.page {
        WalletPage::List => {
            let url = request.filter.url();
            let raw = url.split_once('?').map(|(_, q)| q).unwrap_or_default();
            let query =
                epsx_dioxus_ui::pages::admin_pages::wallet_wallets::AdminWalletListQuery::from_raw(
                    raw,
                )
                .map_err(|_| LoadError::InvalidQuery)?;
            let mut value = adapter::load_wallet_list(&state.wallet, &query, &context).await;
            if matches!(value, adapter::AdminCommerceLoad::Unavailable) {
                value = adapter::load_wallet_list(&state.identity, &query, &context).await;
            }
            data.wallets = Some(if matches!(value, adapter::AdminCommerceLoad::Empty) {
                AdminWalletListProjection {
                    items: vec![],
                    total: 0,
                    limit: query.limit,
                    offset: 0,
                }
            } else {
                required(value)?
            });
        }
        WalletPage::Detail(address) | WalletPage::Disable(address) => {
            data.detail = Some(required(
                adapter::load_wallet_detail(&state.wallet, &address, &context).await,
            )?);
            data.access = required(
                adapter::load_wallet_access(&state.subscription, &address, &context).await,
            )
            .ok();
            let plans = adapter::load_plans(&state.subscription, &context).await;
            data.plans = if matches!(plans, adapter::AdminCommerceLoad::Empty) {
                Some(empty_plans())
            } else {
                required(plans).ok()
            };
        }
        WalletPage::Access => {
            data.access = Some(required(
                adapter::load_access(&state.subscription, &context).await,
            )?);
        }
        WalletPage::Credits => {
            data.credits = Some(required(
                adapter::load_credit_stats(&state.wallet, &context).await,
            )?)
        }
        WalletPage::Plans => {
            let value = adapter::load_plans(&state.subscription, &context).await;
            data.plans = Some(if matches!(value, adapter::AdminCommerceLoad::Empty) {
                empty_plans()
            } else {
                required(value)?
            });
        }
        WalletPage::Plan(id) => {
            data.plan = Some(required(
                adapter::load_plan_detail(&state.subscription, &id, &context).await,
            )?)
        }
        WalletPage::InvalidQuery => return Err(LoadError::InvalidQuery),
        WalletPage::NotFound => return Err(LoadError::NotFound),
    }
    Ok(data)
}
pub async fn command(
    state: AppState,
    command: WalletCommand,
    key: String,
    headers: HeaderMap,
) -> Result<WalletMutation, LoadError> {
    crate::auth_fullstack::same_origin(&headers)?;
    let (mut fields, path, disable) = fields(command)?;
    fields.push(("idempotency_key".into(), key));
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(fields)
        .finish();
    let mut request = Request::builder()
        .method("POST")
        .uri(&path)
        .body(Body::from(body))
        .map_err(|_| LoadError::Malformed)?;
    *request.headers_mut() = headers;
    request.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/x-www-form-urlencoded".parse().unwrap(),
    );
    let response = if disable {
        crate::submit_wallet_disable_form(State(state), request).await
    } else {
        crate::submit_commerce_mutation_form(State(state), request).await
    };
    if !response.status().is_redirection() {
        return Err(match response.status().as_u16() {
            401 => LoadError::Unauthenticated,
            403 => LoadError::Forbidden,
            _ => LoadError::Unavailable,
        });
    }
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(LoadError::Malformed)?;
    let status = url::form_urlencoded::parse(
        location
            .split_once('?')
            .map(|(_, q)| q)
            .unwrap_or_default()
            .as_bytes(),
    )
    .find(|(key, _)| key == "mutation")
    .map(|(_, value)| value.into_owned())
    .unwrap_or_default();
    Ok(WalletMutation {
        committed: matches!(status.as_str(), "success" | "committed"),
        message: match status.as_str() {
            "success" | "committed" => "Saved.",
            "conflict" => "This record changed. Refresh it before trying again.",
            "forbidden" => "Your account cannot perform this operation.",
            "unauthorized" => "Your session expired. Sign in again.",
            "malformed" => "Check the form values and try again.",
            _ => "The operation could not be completed. Retry with the same request.",
        }
        .into(),
    })
}
type WalletForm = (Vec<(String, String)>, String, bool);
fn fields(command: WalletCommand) -> Result<WalletForm, LoadError> {
    let mut fields = vec![];
    let mut add = |key: &str, value: String| fields.push((key.into(), value));
    let (path, disable) = match command {
        WalletCommand::Access {
            revoke,
            wallet_address,
            plan_id,
            permission,
            expected_version,
        } => {
            add(
                "operation",
                if revoke {
                    "access_revoke"
                } else {
                    "access_assign"
                }
                .into(),
            );
            add("wallet_address", wallet_address);
            add("plan_id", plan_id);
            add("permission", permission);
            add("expected_version", expected_version.to_string());
            ("/wallet-management/access".into(), false)
        }
        WalletCommand::Credit {
            revoke,
            wallet_address,
            amount_minor,
            reason,
            expected_version,
        } => {
            add(
                "operation",
                if revoke {
                    "credit_revoke"
                } else {
                    "credit_grant"
                }
                .into(),
            );
            add("wallet_address", wallet_address);
            add("amount_minor", amount_minor.to_string());
            add("reason", reason);
            add("expected_version", expected_version.to_string());
            ("/wallet-management/credits".into(), false)
        }
        WalletCommand::Plan {
            plan_id,
            merchant_id,
            name,
            description,
            amount,
            currency,
            chain_id,
            interval,
            active,
            expected_version,
        } => {
            let path = if let Some(id) = plan_id.as_ref() {
                format!(
                    "/wallet-management/access/plans/{}",
                    uuid::Uuid::parse_str(id).map_err(|_| LoadError::InvalidQuery)?
                )
            } else {
                "/wallet-management/access/plans".into()
            };
            add(
                "operation",
                if plan_id.is_some() {
                    "plan_update"
                } else {
                    "plan_create"
                }
                .into(),
            );
            if let Some(id) = plan_id {
                add("plan_id", id);
            }
            if let Some(id) = merchant_id {
                add("merchant_id", id);
            }
            add("name", name);
            add("description", description);
            add("amount", amount);
            add("currency", currency);
            add("chain_id", chain_id);
            add("interval", interval.to_string());
            if let Some(value) = active {
                add("active", value.to_string());
            }
            if let Some(value) = expected_version {
                add("expected_version", value.to_string());
            }
            (path, false)
        }
        WalletCommand::Disable {
            address,
            reason,
            expected_version,
        } => {
            if address.len() != 42
                || !address.starts_with("0x")
                || !address[2..].bytes().all(|c| c.is_ascii_hexdigit())
            {
                return Err(LoadError::InvalidQuery);
            }
            add("reason", reason);
            add("expected_version", expected_version.to_string());
            (
                format!("/wallet-management/wallets/{address}/disable"),
                true,
            )
        }
    };
    Ok((fields, path, disable))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wallet_disable_cannot_escape_path() {
        assert!(fields(WalletCommand::Disable {
            address: "../../access".into(),
            reason: "Fixture".into(),
            expected_version: 1
        })
        .is_err());
    }
    #[test]
    fn versioned_mutations_keep_existing_contract() {
        let (values, path, _) = fields(WalletCommand::Access {
            revoke: true,
            wallet_address: "wallet".into(),
            plan_id: "plan".into(),
            permission: "read".into(),
            expected_version: 7,
        })
        .unwrap();
        assert_eq!(path, "/wallet-management/access");
        assert!(values.contains(&("expected_version".into(), "7".into())));
        assert!(values.contains(&("operation".into(), "access_revoke".into())));
    }
}
