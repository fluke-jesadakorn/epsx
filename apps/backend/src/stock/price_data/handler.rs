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
        price_websocket_handler,
        candles_websocket_handler
    ),
    components(
        schemas(SubscribeParams)
    ),
    tags(
        (name = "Price Data", description = "Real-time price data streaming endpoints")
    )
)]
#[allow(dead_code)]
struct PriceDataApi;
use futures_util::SinkExt;
use serde::Deserialize;
use tokio_stream::wrappers::BroadcastStream;
use futures_util::StreamExt;

use crate::stock::common::StockServiceError;
use super::service::PriceDataService;

pub fn price_data_router(price_service: Arc<PriceDataService>) -> Router {
    Router::new()
        .route("/price/ws/price", get(price_websocket_handler))
        .route("/price/ws/candles", get(candles_websocket_handler))
        .route("/price/subscribe", get(subscribe_handler))
        .with_state(price_service)
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SubscribeParams {
    /// Comma-separated list of stock symbols e.g. AAPL,GOOGL
    #[schema(example = "AAPL,GOOGL")]
    symbols: String,
    /// Candlestick interval e.g. 1m, 5m, 15m, 30m, 1h, 4h, 1d
    #[schema(example = "5m")]
    interval: Option<String>,
}

/// Subscribe to real-time price and candlestick updates for specific symbols
#[utoipa::path(
    get,
    path = "/price/subscribe",
    params(
        ("symbols" = String, Query, description = "Comma-separated list of stock symbols e.g. AAPL,GOOGL"),
        ("interval" = Option<String>, Query, description = "Candlestick interval e.g. 1m, 5m, 15m, 30m, 1h, 4h, 1d")
    ),
    responses(
        (status = 200, description = "Successfully subscribed to symbols", body = String),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Price Data"
)]
async fn subscribe_handler(
    State(service): State<Arc<PriceDataService>>,
    Query(params): Query<SubscribeParams>,
) -> Result<Json<&'static str>, StockServiceError> {
    let symbols: Vec<String> = params.symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let interval = params.interval.unwrap_or_else(|| "1m".to_string());

    service.subscribe_to_symbols(symbols, interval).await?;
    
    Ok(Json("Subscribed successfully"))
}

/// WebSocket endpoint for real-time price data streaming
#[utoipa::path(
    get,
    path = "/price/ws/price",
    responses(
        (status = 101, description = "WebSocket connection upgraded successfully"),
        (status = 400, description = "Invalid WebSocket request"),
    ),
    tag = "Price Data"
)]
async fn price_websocket_handler(
    State(service): State<Arc<PriceDataService>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_price_websocket(socket, service))
}

/// WebSocket endpoint for real-time candlestick data streaming
#[utoipa::path(
    get,
    path = "/price/ws/candles",
    responses(
        (status = 101, description = "WebSocket connection upgraded successfully"),
        (status = 400, description = "Invalid WebSocket request"),
    ),
    tag = "Price Data"
)]
async fn candles_websocket_handler(
    State(service): State<Arc<PriceDataService>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_candles_websocket(socket, service))
}

async fn handle_price_websocket(
    socket: axum::extract::ws::WebSocket,
    service: Arc<PriceDataService>,
) {
    let (mut sender, _) = socket.split();
    
    let receiver = service.subscribe_to_price_updates();
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

async fn handle_candles_websocket(
    socket: axum::extract::ws::WebSocket,
    service: Arc<PriceDataService>,
) {
    let (mut sender, _) = socket.split();
    
    let receiver = service.subscribe_to_candlesticks();
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
