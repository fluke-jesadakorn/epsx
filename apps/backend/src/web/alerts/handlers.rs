// Alert handlers - Stubbed for Diesel migration
// TODO: Implement with Diesel

use axum::{
    extract::{State},
    http::StatusCode,
    response::Json,
};
use tracing::warn;
use crate::infra::container::AppContainer;
use super::models::*;

// Stub alert handlers
pub async fn list_alerts(
    State(_container): State<AppContainer>,
) -> Result<Json<Vec<AlertResponse>>, (StatusCode, Json<ApiError>)> {
    warn!("Alert handlers stubbed - implement with Diesel");
    Ok(Json(vec![]))
}

pub async fn get_alert(
    State(_container): State<AppContainer>,
) -> Result<Json<AlertResponse>, (StatusCode, Json<ApiError>)> {
    warn!("Alert handlers stubbed - implement with Diesel");
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiError {
            error: "Alert handlers stubbed during migration".to_string(),
            message: "This functionality needs to be reimplemented with Diesel".to_string(),
            code: "NOT_IMPLEMENTED".to_string(),
        })
    ))
}

// Add other stub handlers as needed
pub fn stub_function() {
    warn!("Alert handlers stubbed - implement with Diesel");
}