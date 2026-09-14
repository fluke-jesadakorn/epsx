//! Fixed admin escrow operations. Verified admin session is forwarded to the
//! payment service, which owns dispute eligibility and chain confirmation.
use crate::AppState;
use axum::http::HeaderMap;
use epsx_dioxus_ui::fullstack::{admin_escrow::*, LoadError};
use serde::de::DeserializeOwned;
use serde_json::json;
fn identifier(value: &str) -> Result<&str, LoadError> {
    if !value.is_empty()
        && value.len() <= 100
        && value
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || matches!(v, b'-' | b'_'))
    {
        Ok(value)
    } else {
        Err(LoadError::InvalidQuery)
    }
}
fn validate(query: &EscrowQuery) -> Result<(), LoadError> {
    if !matches!(query.environment.as_str(), "test" | "live") {
        return Err(LoadError::InvalidQuery);
    }
    if let Some(id) = &query.id {
        identifier(id)?;
    }
    Ok(())
}
async fn call<T: DeserializeOwned>(
    state: &AppState,
    query: &EscrowQuery,
    headers: &HeaderMap,
    path: &str,
    body: Option<serde_json::Value>,
    key: Option<&str>,
) -> Result<T, LoadError> {
    let Some((token, _)) = state.session().verified_access_token(headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let mut request = state
        .payment
        .auth_client()
        .request(
            if body.is_some() {
                reqwest::Method::POST
            } else {
                reqwest::Method::GET
            },
            format!("{}{}", state.api_url.trim_end_matches('/'), path),
        )
        .bearer_auth(token);
    if query.merchant {
        request = request
            .header("x-pay-api-version", "2026-09-08")
            .header("x-pay-environment", &query.environment);
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await.map_err(|_| LoadError::Unavailable)?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 => LoadError::Unauthenticated,
            403 => LoadError::Forbidden,
            404 => LoadError::NotFound,
            _ => LoadError::Unavailable,
        });
    }
    response.json().await.map_err(|_| LoadError::Malformed)
}
pub async fn read(
    state: AppState,
    query: EscrowQuery,
    headers: HeaderMap,
) -> Result<EscrowData, LoadError> {
    validate(&query)?;
    let list = if query.merchant {
        "/api/v1/pay/escrows"
    } else {
        "/api/v1/admin/pay/escrows"
    };
    #[derive(serde::Deserialize)]
    struct Items {
        items: Vec<Escrow>,
    }
    let items: Items = call(&state, &query, &headers, list, None, None).await?;
    if items.items.len() > 5000 {
        return Err(LoadError::Malformed);
    }
    let selected = if let Some(id) = &query.id {
        let path = if query.merchant {
            format!("/api/v1/pay/intents/{}", identifier(id)?)
        } else {
            format!("{list}/{}", identifier(id)?)
        };
        Some(call(&state, &query, &headers, &path, None, None).await?)
    } else {
        None
    };
    Ok(EscrowData {
        items: items.items,
        selected,
    })
}
pub async fn command(
    state: AppState,
    query: EscrowQuery,
    command: EscrowCommand,
    key: String,
    headers: HeaderMap,
) -> Result<EscrowOperation, LoadError> {
    crate::auth_fullstack::same_origin(&headers)?;
    validate(&query)?;
    if key.len() != 64 || !key.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(LoadError::InvalidQuery);
    }
    let (path, body) = operation_request(query.merchant, command)?;
    call(&state, &query, &headers, &path, body, Some(&key)).await
}
fn operation_request(
    merchant: bool,
    command: EscrowCommand,
) -> Result<(String, Option<serde_json::Value>), LoadError> {
    Ok(match command {
        EscrowCommand::Resolve { id, to_payee } => {
            if merchant {
                (
                    format!(
                        "/api/v1/pay/intents/{}/{}",
                        identifier(&id)?,
                        if to_payee {
                            "resolve-release"
                        } else {
                            "resolve-refund"
                        }
                    ),
                    Some(json!({})),
                )
            } else {
                (
                    format!("/api/v1/admin/pay/escrows/{}/resolve", identifier(&id)?),
                    Some(json!({"to_payee":to_payee})),
                )
            }
        }
        EscrowCommand::Pause { mode, paused } => {
            if !matches!(mode.as_str(), "direct" | "escrow") {
                return Err(LoadError::InvalidQuery);
            }
            (
                if merchant {
                    format!("/api/v1/pay/contracts/{mode}/pause")
                } else {
                    "/api/v1/admin/pay/contract/pause".into()
                },
                Some(json!({"paused":paused})),
            )
        }
        EscrowCommand::Confirm { id, control, hash } => {
            if hash.len() != 66
                || !hash.starts_with("0x")
                || !hash[2..].bytes().all(|c| c.is_ascii_hexdigit())
            {
                return Err(LoadError::InvalidQuery);
            }
            (
                format!(
                    "{}/{}/confirm",
                    operation_base(merchant, control),
                    identifier(&id)?
                ),
                Some(json!({"tx_hash":hash})),
            )
        }
        EscrowCommand::Operation { id, control } => (
            format!("{}/{}", operation_base(merchant, control), identifier(&id)?),
            None,
        ),
    })
}
fn operation_base(merchant: bool, control: bool) -> &'static str {
    match (merchant, control) {
        (true, true) => "/api/v1/pay/contract-controls",
        (true, false) => "/api/v1/pay/operations",
        (false, true) => "/api/v1/admin/pay/contract/operations",
        (false, false) => "/api/v1/admin/pay/operations",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_operation_paths() {
        for merchant in [true, false] {
            assert!(operation_request(
                merchant,
                EscrowCommand::Resolve {
                    id: "../escape".into(),
                    to_payee: true
                }
            )
            .is_err());
            assert!(operation_request(
                merchant,
                EscrowCommand::Pause {
                    mode: "admin/unsafe".into(),
                    paused: true
                }
            )
            .is_err());
        }
        let (path, body) = operation_request(
            false,
            EscrowCommand::Resolve {
                id: "fixture".into(),
                to_payee: false,
            },
        )
        .unwrap();
        assert_eq!(path, "/api/v1/admin/pay/escrows/fixture/resolve");
        assert_eq!(body.unwrap(), json!({"to_payee":false}));
        assert_eq!(operation_base(true, true), "/api/v1/pay/contract-controls");
        assert_eq!(
            operation_base(false, true),
            "/api/v1/admin/pay/contract/operations"
        );
    }
}
