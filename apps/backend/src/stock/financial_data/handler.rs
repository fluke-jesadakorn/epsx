use std::sync::Arc;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
    Json,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        subscribe_handler,
        websocket_handler,
    ),
    components(
        schemas(SubscribeParams)
    ),
    tags(
        (name = "Financial Data", description = "Financial data streaming endpoints")
    )
)]
#[allow(dead_code)]
struct FinancialDataApi;
use futures_util::SinkExt;
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;

use crate::stock::common::StockServiceError;
use super::service::FinancialDataService;

pub fn financial_data_router(financial_service: Arc<FinancialDataService>) -> Router {
    Router::new()
        .route("/financial/ws", get(websocket_handler))
        .route("/financial/subscribe", get(subscribe_handler))
        .with_state(financial_service)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SubscribeParams {
    /// Comma-separated list of stock symbols e.g. AAPL,GOOGL
    #[schema(example = "AAPL,GOOGL")]
    symbols: String,
}

/// Subscribe to financial data updates for specific symbols
#[utoipa::path(
    get,
    path = "/financial/subscribe",
    params(
        ("symbols" = String, Query, description = "Comma-separated list of stock symbols e.g. AAPL,GOOGL")
    ),
    responses(
        (status = 200, description = "Successfully subscribed to symbols", body = String),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Financial Data"
)]
async fn subscribe_handler(
    State(service): State<Arc<FinancialDataService>>,
    Query(params): Query<SubscribeParams>,
) -> Result<Json<&'static str>, StockServiceError> {
    let symbols: Vec<String> = params.symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    service.subscribe_to_symbols(symbols).await?;
    
    Ok(Json("Subscribed successfully"))
}

/// WebSocket endpoint for real-time financial data streaming
#[utoipa::path(
    get,
    path = "/financial/ws",
    responses(
        (status = 101, description = "WebSocket connection upgraded successfully"),
        (status = 400, description = "Invalid WebSocket request"),
    ),
    tag = "Financial Data"
)]
async fn websocket_handler(
    State(service): State<Arc<FinancialDataService>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket_connection(socket, service))
}

async fn handle_websocket_connection(
    socket: axum::extract::ws::WebSocket,
    service: Arc<FinancialDataService>,
) {
    let (mut sender, _) = socket.split();
    
    let receiver = service.subscribe_to_updates();
    let mut stream = BroadcastStream::new(receiver);

    while let Some(Ok(msg)) = stream.next().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            if sender
                .send(axum::extract::ws::Message::Text(json))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}
