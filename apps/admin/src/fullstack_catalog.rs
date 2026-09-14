use crate::AppState;
use epsx_dioxus_ui::fullstack::{
    admin_catalog::{CatalogData, CatalogProvider, Plan, PlanEdit},
    LoadError,
};
use std::{collections::HashMap, sync::Arc};
pub fn provider(state: AppState) -> CatalogProvider {
    let read_state = state.clone();
    CatalogProvider {
        read: Arc::new(move |id, headers| {
            let state = read_state.clone();
            Box::pin(async move { read(&state, id, &headers).await })
        }),
        save: Arc::new(move |edit, headers| {
            let state = state.clone();
            Box::pin(async move { save(state, edit, headers).await })
        }),
    }
}
async fn read(
    state: &AppState,
    id: Option<uuid::Uuid>,
    headers: &http::HeaderMap,
) -> Result<CatalogData, LoadError> {
    let path = id
        .map(|id| format!("/api/admin/plans/{id}"))
        .unwrap_or_else(|| "/api/admin/plans".into());
    let data = crate::plan_catalog::read(state, headers, &path)
        .await
        .map_err(|response| classify(response.status()))?;
    if let Some(id) = id {
        let plan: Plan = serde_json::from_value(data).map_err(|_| LoadError::Malformed)?;
        if plan.id != id {
            return Err(LoadError::Malformed);
        }
        Ok(CatalogData::Detail(Box::new(plan)))
    } else {
        #[derive(serde::Deserialize)]
        struct Envelope {
            data: List,
        }
        #[derive(serde::Deserialize)]
        struct List {
            plans: Vec<Plan>,
        }
        let envelope: Envelope = serde_json::from_value(data).map_err(|_| LoadError::Malformed)?;
        Ok(CatalogData::List(envelope.data.plans))
    }
}
fn classify(status: http::StatusCode) -> LoadError {
    match status {
        http::StatusCode::UNAUTHORIZED | http::StatusCode::SEE_OTHER => LoadError::Unauthenticated,
        http::StatusCode::FORBIDDEN => LoadError::Forbidden,
        http::StatusCode::NOT_FOUND => LoadError::NotFound,
        http::StatusCode::BAD_REQUEST
        | http::StatusCode::UNPROCESSABLE_ENTITY
        | http::StatusCode::CONFLICT => LoadError::InvalidQuery,
        _ => LoadError::Unavailable,
    }
}
async fn save(state: AppState, edit: PlanEdit, headers: http::HeaderMap) -> Result<(), LoadError> {
    let fields: HashMap<String, String> = [
        ("name", edit.name),
        ("description", edit.description),
        ("USDT", edit.usdt),
        ("USDC", edit.usdc),
        ("pay_use_catalog_promotion", edit.use_promotion.to_string()),
        ("promotion_enabled", edit.promotion_enabled.to_string()),
        ("promotion_type", edit.promotion_type),
        ("promotion_value", edit.promotion_value),
        ("promotion_price", edit.promotion_price),
        ("promotion_start_date", edit.promotion_start),
        ("promotion_end_date", edit.promotion_end),
        ("duration_days", edit.duration_days),
        ("is_active", edit.active.to_string()),
        ("permissions", edit.permissions),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect();
    if fields.values().map(String::len).sum::<usize>() > 64 * 1024 {
        return Err(LoadError::InvalidQuery);
    }
    let response = crate::plan_catalog::save(
        axum::extract::State(state),
        headers,
        axum::extract::Path(edit.id),
        axum::extract::Form(fields),
    )
    .await;
    if response.status() == http::StatusCode::SEE_OTHER {
        Ok(())
    } else {
        Err(classify(response.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn edit() -> PlanEdit {
        PlanEdit {
            id: uuid::Uuid::nil(),
            name: "One day".into(),
            description: String::new(),
            usdt: "5".into(),
            usdc: "5".into(),
            use_promotion: false,
            promotion_enabled: false,
            promotion_type: "fixed".into(),
            promotion_value: "0".into(),
            promotion_price: String::new(),
            promotion_start: String::new(),
            promotion_end: String::new(),
            duration_days: "1".into(),
            active: true,
            permissions: "epsx:analytics:read".into(),
        }
    }
    #[tokio::test]
    async fn catalog_read_requires_verified_session() {
        assert_eq!(
            read(
                &crate::routing_tests::test_state(),
                None,
                &http::HeaderMap::new()
            )
            .await,
            Err(LoadError::Unauthenticated)
        );
    }
    #[tokio::test]
    async fn catalog_save_rejects_cross_origin_before_upstream() {
        let mut headers = http::HeaderMap::new();
        headers.insert("host", "admin.test".parse().unwrap());
        headers.insert("origin", "https://attacker.test".parse().unwrap());
        assert_eq!(
            save(crate::routing_tests::test_state(), edit(), headers).await,
            Err(LoadError::Forbidden)
        );
    }
    #[tokio::test]
    async fn catalog_detail_ssr_serializes_data_without_saving() {
        use axum::{
            body::{to_bytes, Body},
            http::Request,
            Extension, Router,
        };
        use tower::ServiceExt;
        let plan: Plan = serde_json::from_value(serde_json::json!({"id":uuid::Uuid::nil(),"name":"One day fixture","is_active":true,"permissions":["epsx:analytics:read"],"metadata":{"duration_days":1,"pay_prices":{"USDT":"5.00","USDC":"5.00"},"promotion":null}})).unwrap();
        let provider = CatalogProvider {
            read: Arc::new(move |id, _| {
                let plan = plan.clone();
                Box::pin(async move {
                    assert_eq!(id, Some(plan.id));
                    Ok(CatalogData::Detail(Box::new(plan)))
                })
            }),
            save: Arc::new(|_, _| Box::pin(async { panic!("SSR must not save a plan") })),
        };
        let state = dioxus_server::FullstackState::new(
            dioxus_server::ServeConfig::new(),
            epsx_dioxus_ui::app::AdminRoot,
        );
        let app = Router::new()
            .route(
                "/plans/{id}",
                axum::routing::get(dioxus_server::FullstackState::render_handler),
            )
            .with_state(state)
            .layer(Extension(provider));
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/plans/{}", uuid::Uuid::nil()))
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
        assert!(html.contains("One day fixture"));
        assert!(html.contains("5.00"));
        assert!(html.contains("Save plan"));
        assert!(html.contains("epsx:analytics:read"));
        assert!(
            html.contains("value=\"fixed\" selected"),
            "SSR must select a real default discount option"
        );
        assert!(
            html.contains("value=\"false\" selected"),
            "SSR must preserve disabled promotion"
        );
        assert!(
            html.split("<textarea")
                .skip(1)
                .any(
                    |part| part.split_once('>').is_some_and(|(_, content)| content
                        .split("</textarea>")
                        .next()
                        .unwrap_or_default()
                        .contains("epsx:analytics:read"))
                ),
            "Textarea content must survive direct SSR"
        );
        if cfg!(debug_assertions) {
            assert!(html.contains("CatalogData"));
        }
    }
}
