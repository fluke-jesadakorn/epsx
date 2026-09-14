use crate::AppState;
use epsx_dioxus_ui::{
    fullstack::{admin_orders::AdminOrdersProvider, LoadError},
    payment::purchases::{Purchase, PurchaseData, PurchaseList, PurchaseQuery},
};
use std::sync::Arc;
pub fn provider(state: AppState) -> AdminOrdersProvider {
    AdminOrdersProvider(Arc::new(move |query, headers| {
        let state = state.clone();
        Box::pin(async move { read(&state, query, headers).await })
    }))
}
async fn read(
    state: &AppState,
    query: PurchaseQuery,
    headers: http::HeaderMap,
) -> Result<PurchaseData, LoadError> {
    query.validate()?;
    let path = match query.order_id {
        Some(id) => format!("/api/admin/pay-orders/{id}"),
        None => format!("/api/admin/pay-orders?offset={}", query.offset),
    };
    let value = crate::plan_catalog::read(state, &headers, &path)
        .await
        .map_err(|r| match r.status() {
            http::StatusCode::UNAUTHORIZED | http::StatusCode::SEE_OTHER => {
                LoadError::Unauthenticated
            }
            http::StatusCode::FORBIDDEN => LoadError::Forbidden,
            http::StatusCode::NOT_FOUND => LoadError::NotFound,
            _ => LoadError::Unavailable,
        })?;
    if let Some(id) = query.order_id {
        let order: Purchase = serde_json::from_value(value).map_err(|_| LoadError::Malformed)?;
        if order.order_id != id {
            return Err(LoadError::Malformed);
        }
        Ok(PurchaseData::Detail(Box::new(order)))
    } else {
        let list: PurchaseList = serde_json::from_value(value).map_err(|_| LoadError::Malformed)?;
        if list
            .next_offset
            .is_some_and(|next| next <= query.offset || next > 1_000_000)
        {
            return Err(LoadError::Malformed);
        }
        Ok(PurchaseData::List(list))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn admin_orders_need_verified_session() {
        assert_eq!(
            read(
                &crate::routing_tests::test_state(),
                PurchaseQuery {
                    order_id: None,
                    offset: 0
                },
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn admin_orders_reject_invalid_query() {
        assert_eq!(
            read(
                &crate::routing_tests::test_state(),
                PurchaseQuery {
                    order_id: Some(uuid::Uuid::nil()),
                    offset: 10
                },
                http::HeaderMap::new()
            )
            .await,
            Err(LoadError::InvalidQuery)
        );
    }
    #[tokio::test]
    async fn admin_orders_ssr_preserves_statuses_and_cursor() {
        use axum::{
            body::{to_bytes, Body},
            http::Request,
            Extension, Router,
        };
        use tower::ServiceExt;
        let provider = AdminOrdersProvider(Arc::new(|query, _| {
            Box::pin(async move {
                assert_eq!(query.offset, 10);
                Ok(PurchaseData::List(serde_json::from_value(serde_json::json!({"orders":[{"order_id":uuid::Uuid::nil(),"plan_name":"Fixture plan","wallet_address":"0x1234","amount":"5000000","token":"USDT","token_decimals":6,"status":"succeeded","fulfillment_status":"pending","created_at":"2026-09-09T03:00:00Z"}],"next_offset":20})).unwrap()))
            })
        }));
        let state = dioxus_server::FullstackState::new(
            dioxus_server::ServeConfig::new(),
            epsx_dioxus_ui::app::AdminRoot,
        );
        let app = Router::new()
            .route(
                "/payments/epsx",
                axum::routing::get(dioxus_server::FullstackState::render_handler),
            )
            .with_state(state)
            .layer(Extension(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/payments/epsx?offset=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::OK);
        let html = String::from_utf8(
            to_bytes(response.into_body(), 2 * 1024 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("Fixture plan"));
        assert!(html.contains("5.00 USDT"));
        assert!(html.contains("succeeded"));
        assert!(html.contains("pending"));
        assert!(html.contains("/payments/epsx?offset=20"));
        if cfg!(debug_assertions) {
            assert!(html.contains("PurchaseData"));
        }
    }
}
