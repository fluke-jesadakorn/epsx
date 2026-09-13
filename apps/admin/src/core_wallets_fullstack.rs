//! Transport canonical EPSX wallet/access commands. Authorization and policy stay
//! in the core Rust backend, never in the commerce wallet/subscription services.
use crate::AppState;
use epsx_dioxus_ui::fullstack::{core_wallets::*, LoadError};
use http::{HeaderMap, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::Arc;

pub fn provider(state: AppState) -> Provider {
    let read_state = state.clone();
    Provider {
        read: Arc::new(move |address, query, headers| {
            let state = read_state.clone();
            Box::pin(async move { read(&state, address, &query, &headers).await })
        }),
        command: Arc::new(move |command, headers| {
            let state = state.clone();
            Box::pin(async move { write(&state, command, &headers).await })
        }),
    }
}
fn classify(status: StatusCode) -> LoadError {
    match status.as_u16() {
        401 | 303 => LoadError::Unauthenticated,
        403 => LoadError::Forbidden,
        404 => LoadError::NotFound,
        400 | 409 | 422 => LoadError::InvalidQuery,
        _ => LoadError::Unavailable,
    }
}
fn address(value: &str) -> Result<String, LoadError> {
    if value.len() == 42
        && value.starts_with("0x")
        && value[2..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        Ok(value.to_ascii_lowercase())
    } else {
        Err(LoadError::InvalidQuery)
    }
}
fn decode<T: DeserializeOwned>(value: Value) -> Result<T, LoadError> {
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(LoadError::Malformed);
    }
    serde_json::from_value(value.get("data").cloned().ok_or(LoadError::Malformed)?)
        .map_err(|_| LoadError::Malformed)
}
async fn get<T: DeserializeOwned>(
    state: &AppState,
    path: &str,
    headers: &HeaderMap,
) -> Result<T, LoadError> {
    decode(
        crate::plan_catalog::read(state, headers, path)
            .await
            .map_err(|r| classify(r.status()))?,
    )
}
async fn read(
    state: &AppState,
    wallet: Option<String>,
    raw_query: &str,
    headers: &HeaderMap,
) -> Result<Data, LoadError> {
    if let Some(wallet) = wallet {
        let wallet = address(&wallet)?;
        let detail: Detail = get(state, &format!("/api/admin/wallets/{wallet}"), headers).await?;
        if detail.wallet.wallet_address.to_ascii_lowercase() != wallet {
            return Err(LoadError::Malformed);
        }
        let assignment_path =
            format!("/api/permissions/assignments?wallet_address={wallet}&limit=100");
        let (assignments, plans) = tokio::join!(
            get::<Vec<Assignment>>(state, &assignment_path, headers),
            get::<Vec<PlanOption>>(state, "/api/permissions/plans?limit=100", headers),
        );
        Ok(Data::Detail {
            detail,
            assignments,
            plans,
        })
    } else {
        let query =
            epsx_dioxus_ui::pages::admin_pages::wallet_wallets::AdminWalletListQuery::from_raw(
                raw_query,
            )
            .map_err(|_| LoadError::InvalidQuery)?;
        let path = {
            let mut params = url::form_urlencoded::Serializer::new(String::new());
            params
                .append_pair("page", &query.page.to_string())
                .append_pair("limit", &query.limit.to_string());
            if let Some(search) = query.search {
                params.append_pair("search", &search);
            }
            if let Some(status) = query.status {
                params.append_pair(
                    "status",
                    if status == "disabled" {
                        "inactive"
                    } else {
                        &status
                    },
                );
            }
            format!("/api/admin/wallets?{}", params.finish())
        };
        let list = get(state, &path, headers).await?;
        Ok(Data::List(list))
    }
}
fn command_request(command: Command) -> Result<(reqwest::Method, String, Value), LoadError> {
    use reqwest::Method;
    let uuid = |value: &str| {
        uuid::Uuid::parse_str(value)
            .map(|id| id.to_string())
            .map_err(|_| LoadError::InvalidQuery)
    };
    let expiry = |value: Option<String>| -> Result<Option<String>, LoadError> {
        value
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.to_rfc3339())
                    .map_err(|_| LoadError::InvalidQuery)
            })
            .transpose()
    };
    Ok(match command {
        Command::Status { wallet, active } => (
            Method::PUT,
            format!("/api/admin/wallets/{}", address(&wallet)?),
            json!({"is_active":active}),
        ),
        Command::Assign {
            wallet,
            plan_id,
            expires_at,
            reason,
        } => (
            Method::POST,
            "/api/permissions/assignments".into(),
            json!({"wallet_address":address(&wallet)?,"plan_id":uuid(&plan_id)?,"assignment_source":"manual","assignment_reason":reason,"expires_at":expiry(expires_at)?}),
        ),
        Command::RevokePlan { assignment_id } => (
            Method::DELETE,
            format!("/api/permissions/assignments/{}", uuid(&assignment_id)?),
            Value::Null,
        ),
        Command::GrantPermission {
            wallet,
            permission,
            expires_at,
            reason,
        } => (
            Method::POST,
            "/api/admin/permissions/direct/grant".into(),
            json!({"wallet_address":address(&wallet)?,"permission_string":permission,"expires_at":expiry(expires_at)?,"reason":reason}),
        ),
        Command::RevokePermission { wallet, permission } => (
            Method::DELETE,
            "/api/admin/permissions/direct/revoke".into(),
            json!({"wallet_address":address(&wallet)?,"permission_string":permission}),
        ),
    })
}
async fn write(state: &AppState, command: Command, headers: &HeaderMap) -> Result<(), LoadError> {
    crate::auth_fullstack::same_origin(headers)?;
    let Some((token, _)) = state.session().verified_access_token(headers).await else {
        return Err(LoadError::Unauthenticated);
    };
    let (method, path, body) = command_request(command)?;
    if body.to_string().len() > 8192 {
        return Err(LoadError::InvalidQuery);
    }
    let response = state
        .content
        .auth_client()
        .request(method, format!("{}{path}", state.api_url))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|_| LoadError::Unavailable)?;
    if !response.status().is_success() {
        return Err(classify(response.status()));
    }
    let value: Value = response.json().await.map_err(|_| LoadError::Malformed)?;
    if value.get("success") != Some(&Value::Bool(true)) {
        return Err(LoadError::Malformed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_envelope_requires_success_and_original_wallet_shape() {
        let source = json!({"success":true,"data":{"wallets":[{"wallet_address":"0x1111111111111111111111111111111111111111","is_active":true,"created_at":"2026-09-01T00:00:00Z","last_auth_at":null,"metadata":{},"plans_count":2}],"total":1,"pagination":{"page":1,"limit":10,"has_next":false,"has_prev":false}}});
        let list: WalletList = decode(source.clone()).unwrap();
        assert_eq!(list.total, 1);
        assert!(list.wallets[0].is_active);
        let mut failed = source;
        failed["success"] = json!(false);
        assert_eq!(decode::<WalletList>(failed), Err(LoadError::Malformed));
        assert_eq!(
            decode::<WalletList>(json!({"items":[],"total":0})),
            Err(LoadError::Malformed)
        );
    }
    #[test]
    fn wallet_commands_target_core_and_reject_path_injection() {
        let (method, path, body) = command_request(Command::Status {
            wallet: "0x1111111111111111111111111111111111111111".into(),
            active: false,
        })
        .unwrap();
        assert_eq!(method, reqwest::Method::PUT);
        assert_eq!(
            path,
            "/api/admin/wallets/0x1111111111111111111111111111111111111111"
        );
        assert_eq!(body, json!({"is_active":false}));
        assert!(command_request(Command::Status {
            wallet: "../plans".into(),
            active: false
        })
        .is_err());
        let (_, path, body) = command_request(Command::Assign {
            wallet: "0x1111111111111111111111111111111111111111".into(),
            plan_id: uuid::Uuid::nil().to_string(),
            expires_at: None,
            reason: "Rehearsal".into(),
        })
        .unwrap();
        assert_eq!(path, "/api/permissions/assignments");
        assert_eq!(body["assignment_source"], "manual");
        assert!(body.get("merchant_id").is_none());
    }
    #[tokio::test]
    async fn canonical_reads_require_session_and_writes_require_same_origin() {
        let state = crate::routing_tests::test_state();
        assert_eq!(
            read(&state, None, "", &HeaderMap::new()).await,
            Err(LoadError::Unauthenticated)
        );
        let mut headers = HeaderMap::new();
        headers.insert("host", "admin.test".parse().unwrap());
        headers.insert("origin", "https://attacker.test".parse().unwrap());
        assert_eq!(
            write(
                &state,
                Command::Status {
                    wallet: "0x1111111111111111111111111111111111111111".into(),
                    active: false
                },
                &headers
            )
            .await,
            Err(LoadError::Forbidden)
        );
    }
    #[tokio::test]
    async fn canonical_wallet_ssr_uses_imported_identity_fields() {
        use axum::{
            body::{to_bytes, Body},
            http::Request,
            Extension, Router,
        };
        use tower::ServiceExt;
        let provider = Provider {
            read: Arc::new(|wallet, _, _| {
                Box::pin(async move {
                    assert_eq!(
                        wallet,
                        Some("0x1111111111111111111111111111111111111111".into())
                    );
                    Ok(Data::Detail {
                        detail: Detail {
                            wallet: Wallet {
                                wallet_address: wallet.unwrap(),
                                is_active: true,
                                created_at: "2026-09-01T00:00:00Z".into(),
                                last_auth_at: None,
                            },
                            permissions: vec![Permission {
                                permission: "epsx:analytics:read".into(),
                                source: "direct".into(),
                                expires_at: None,
                                is_active: true,
                            }],
                        },
                        assignments: Ok(vec![Assignment {
                            id: uuid::Uuid::nil().to_string(),
                            wallet_address: "0x1111111111111111111111111111111111111111".into(),
                            plan_id: uuid::Uuid::nil().to_string(),
                            plan_name: "Imported production plan".into(),
                            expires_at: None,
                            is_active: true,
                        }]),
                        plans: Ok(vec![]),
                    })
                })
            }),
            command: Arc::new(|_, _| Box::pin(async { panic!("SSR must never mutate access") })),
        };
        let state = dioxus_server::FullstackState::new(
            dioxus_server::ServeConfig::new(),
            epsx_dioxus_ui::app::AdminRoot,
        );
        let app = Router::new()
            .route(
                "/wallet-management/{address}",
                axum::routing::get(dioxus_server::FullstackState::render_handler),
            )
            .with_state(state)
            .layer(Extension(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/wallet-management/0x1111111111111111111111111111111111111111")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        for expected in [
            "Imported production plan",
            "epsx:analytics:read",
            "Disable account",
            "Revoke plan",
            "Revoke direct permission",
        ] {
            assert!(html.contains(expected), "missing {expected}");
        }
    }
}
