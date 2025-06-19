use tokio;
use tracing::{error, info}; // Import error alongside info
use websocket_test::{
    models::{self, EarningsResult, PricePoint, WebSocketMessage}, // Import PricePoint and models module
    websocket::WsClient,
};
use chrono::{TimeZone, Utc}; // Import chrono for date handling
use std::collections::HashMap; // Use HashMap for results for easier lookup by date

// Function to format timestamp to readable date string
fn format_timestamp_log(ts: i64) -> String { // Renamed for clarity
    Utc.timestamp_opt(ts, 0).single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "Invalid Timestamp".to_string())
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Wait for data with timeout
    let timeout_duration = std::time::Duration::from_secs(30);
    let start_time = std::time::Instant::now();

    // --- State Variables ---
    // Store price history as a Vec of PricePoint for find_nearest_price
    let mut price_history: Vec<PricePoint> = Vec::new();
    // Stores the two most recent (release_date, eps) pairs
    let mut latest_earnings_data: Vec<(i64, f64)> = Vec::new();
    // Stores the results with nearest price, keyed by release date
    let mut results: HashMap<i64, EarningsResult> = HashMap::new();
    // --- End State Variables ---

    let ws = WsClient::new(
        "wss://prodata.tradingview.com/socket.io/websocket",
        "unauthorized_user_token"
    );

    let Ok(conn) = ws.connect::<WebSocketMessage>().await else {
        error!("Failed to connect to WebSocket."); // Added error logging
        return;
    };

    let cs = WsClient::gen_sess_id();
    let qs = WsClient::gen_quote_sess();
    let sym = "NASDAQ:AAPL"; // Changed symbol to AAPL as per example data

    let msgs = [
        WsClient::msg_chart_sess(&cs),
        WsClient::msg_quote_sess(&qs),
        WsClient::msg_resolve_sym(&cs, sym),
        WsClient::msg_series(&cs),
        WsClient::msg_quote_add_adj(&qs, sym),
        WsClient::msg_quote_fast(&qs, sym),
        WsClient::msg_quote_add(&qs, sym),
    ];

    for msg in msgs {
        if let Err(_) = conn.send_raw_message(&msg).await {
            break;
        }
    }

    if let Ok(mut rx) = conn.start_receive_loop() {
        while let Some(msg) = rx.recv().await {
            // Check timeout
            if start_time.elapsed() >= timeout_duration {
                info!("Timeout reached after 30 seconds. Stopping data collection.");
                break;
            }

            let mut needs_update = false;
            info!("Received WebSocket message");

            // 1. Check for and process Price History updates
            if let Some(mut new_prices) = msg.get_price_history() { // Make mutable
                if !new_prices.is_empty() {
                    info!("Received {} new price points", new_prices.len());
                    // Append new prices and sort by timestamp
                    price_history.append(&mut new_prices);
                    price_history.sort_by_key(|p| p.timestamp);
                    info!("Total price history points: {}", price_history.len());
                    needs_update = true;
                }
            }

            // 2. Check for and process Earnings Data updates
            if let Some(new_earnings) = msg.get_latest_earnings_data() {
                if !new_earnings.data.is_empty() {
                    info!("Received earnings data with {} entries", new_earnings.data.len());
                    if new_earnings.data != latest_earnings_data {
                        info!("Updating earnings data");
                        for (date, eps) in &new_earnings.data {
                            info!("Earnings date: {} EPS: {:.4}", format_timestamp_log(*date), eps);
                        }
                        latest_earnings_data = new_earnings.data;
                        for (date, eps) in &latest_earnings_data {
                            results.entry(*date).or_insert_with(|| EarningsResult {
                                release_date: *date,
                                eps: *eps,
                                price_at_release: None,
                            });
                        }
                        needs_update = true;
                    }
                }
            }

            // 3. If earnings data was updated, try to find nearest prices and log with QoQ
            if needs_update && !latest_earnings_data.is_empty() && !price_history.is_empty() {
                // First, update prices for any earnings data points that don't have one yet
                for (release_date, _eps) in &latest_earnings_data {
                    if let Some(result_entry) = results.get_mut(release_date) {
                        if result_entry.price_at_release.is_none() {
                            if let Some(nearest_price_point) =
                                models::find_nearest_price(&price_history, *release_date)
                            {
                                result_entry.price_at_release = Some(nearest_price_point.close_price);
                            }
                        }
                    }
                }

                // Now, check if we have data for the latest two quarters and log with QoQ
                if latest_earnings_data.len() == 2 {
                    // latest_earnings_data is sorted descending by date
                    let (latest_date, _) = latest_earnings_data[0];
                    let (prev_date, _) = latest_earnings_data[1];

                    if let (Some(latest_result), Some(prev_result)) = (results.get(&latest_date), results.get(&prev_date)) {
                        // Calculate QoQ % change
                        let qoq_change = if prev_result.eps != 0.0 {
                            Some(((latest_result.eps - prev_result.eps) / prev_result.eps.abs()) * 100.0)
                        } else {
                            None // Avoid division by zero
                        };

                        // Log in the requested format
                        info!(
                            "Previous Date: {}, Price: {}, EPS: {:.4}",
                            format_timestamp_log(prev_result.release_date),
                            prev_result.price_at_release.map_or("N/A".to_string(), |p| format!("{:.2}", p)),
                            prev_result.eps
                        );

                        info!(
                            "Current Date: {}, Price: {}, EPS: {:.4}",
                            format_timestamp_log(latest_result.release_date),
                            latest_result.price_at_release.map_or("N/A".to_string(), |p| format!("{:.2}", p)),
                            latest_result.eps
                        );
                    }
                }
            }
        }
    } else {
        error!("Failed to start receive loop.");
    }
    // Display sample earnings data from logs
    let sample_data = vec![
        EarningsResult {
            release_date: Utc.ymd(2025, 1, 30).and_hms(21, 30, 0).timestamp(),
            eps: 0.6375,
            price_at_release: None,
        },
        EarningsResult {
            release_date: Utc.ymd(2024, 10, 31).and_hms(20, 35, 0).timestamp(),
            eps: 0.6450,
            price_at_release: None,
        },
    ];

    info!("Sample earnings data from logs:");
    info!(
        "Previous Date: {}, Price: {}, EPS: {:.4}",
        format_timestamp_log(sample_data[1].release_date),
        sample_data[1].price_at_release.map_or("N/A".to_string(), |p| format!("{:.2}", p)),
        sample_data[1].eps
    );
    info!(
        "Current Date: {}, Price: {}, EPS: {:.4}",
        format_timestamp_log(sample_data[0].release_date),
        sample_data[0].price_at_release.map_or("N/A".to_string(), |p| format!("{:.2}", p)),
        sample_data[0].eps
    );

    info!("WebSocket processing finished.");
}
