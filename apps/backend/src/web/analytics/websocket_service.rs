// WebSocket Earnings Service
// Real-time earnings data fetching with intelligent caching

use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::time::{ SystemTime, UNIX_EPOCH };

use crate::infrastructure::adapters::services::tradingview_websocket::{
  TradingViewWebSocketService,
  QuarterlyEPSData,
  EPSWebSocketData,
};
use epsx_contracts::errors::AppError;

/// Simple in-memory cache for WebSocket results
struct EarningsCache {
  data: HashMap<String, (i64, i32, u64)>, // symbol -> (timestamp, days, cache_time)
  qoq_data: HashMap<String, (f64, u64)>, // symbol -> (qoq_growth, cache_time)
}

lazy_static::lazy_static! {
  static ref EARNINGS_CACHE: Arc<Mutex<EarningsCache>> = Arc::new(
    Mutex::new(EarningsCache {
      data: HashMap::new(),
      qoq_data: HashMap::new(),
    })
  );
}

const CACHE_DURATION_SECONDS: u64 = 3600; // 1 hour cache

/// WebSocket earnings date integration service with caching
pub struct WebSocketEarningsService;

impl WebSocketEarningsService {
  /// Fetch real earnings announcement dates using TradingView WebSocket with caching
  pub async fn fetch_earnings_dates(
    symbols: Vec<String>
  ) -> Result<HashMap<String, (i64, i32)>, AppError> {
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let mut result_map = HashMap::new();
    let mut symbols_to_fetch = Vec::new();

    // Check cache first
    {
      let cache = EARNINGS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
      for symbol in &symbols {
        if let Some((timestamp, days, cache_time)) = cache.data.get(symbol) {
          if current_time - cache_time < CACHE_DURATION_SECONDS {
            tracing::info!(
              "Using cached earnings data for {}: {} days",
              symbol,
              days
            );
            result_map.insert(symbol.clone(), (*timestamp, *days));
            continue;
          }
        }
        symbols_to_fetch.push(symbol.clone());
      }
    }

    // If all symbols are cached, return immediately
    if symbols_to_fetch.is_empty() {
      tracing::info!("All {} symbols served from cache", symbols.len());
      return Ok(result_map);
    }

    tracing::info!(
      "Fetching {} symbols via WebSocket (cached: {})",
      symbols_to_fetch.len(),
      result_map.len()
    );

    // Fetch with timeout
    let websocket_data = Self::fetch_with_timeout(
      symbols_to_fetch.clone()
    ).await?;

    // Process WebSocket data and update cache
    let current_timestamp = chrono::Utc::now().timestamp();
    for ws_data in websocket_data {
      if
        let Some(next_earnings) = Self::find_next_earnings_announcement(
          &ws_data.quarterly_data,
          current_timestamp
        )
      {
        let days_until = ((next_earnings - current_timestamp) / 86400).max(
          0
        ) as i32;
        result_map.insert(ws_data.symbol.clone(), (next_earnings, days_until));

        // Update cache
        {
          let mut cache = EARNINGS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
          cache.data.insert(ws_data.symbol.clone(), (
            next_earnings,
            days_until,
            current_time,
          ));
        }

        tracing::info!(
          "Real QoQ pattern earnings for {}: {} days",
          ws_data.symbol,
          days_until
        );
      } else {
        Self::handle_missing_quarterly_data(
          &ws_data.symbol,
          &ws_data.quarterly_data,
          current_timestamp,
          &mut result_map
        );
      }
    }

    // Add any remaining cached results
    Self::add_cached_results(&symbols, &mut result_map);

    // Validate all symbols have data
    let missing_symbols: Vec<String> = symbols
      .iter()
      .filter(|s| !result_map.contains_key(*s))
      .cloned()
      .collect();

    if !missing_symbols.is_empty() {
      tracing::error!(
        "Missing earnings data for {} symbols: {:?}",
        missing_symbols.len(),
        missing_symbols
      );
      return Err(
        AppError::validation_error(
          format!(
            "No earnings data available for symbols: {}",
            missing_symbols.join(", ")
          )
        )
      );
    }

    tracing::info!(
      "WebSocket earnings fetch complete: {}/{} symbols processed",
      result_map.len(),
      symbols.len()
    );

    Ok(result_map)
  }

  /// Fetch WebSocket data with timeout
  async fn fetch_with_timeout(
    symbols: Vec<String>
  ) -> Result<Vec<EPSWebSocketData>, AppError> {
    let fetch_future = async {
      let mut websocket_service = TradingViewWebSocketService::new();
      websocket_service.connect_and_fetch_eps_data(symbols.clone()).await
    };

    match
      tokio::time::timeout(
        std::time::Duration::from_secs(epsx_contracts::constants::MINUTE as u64 / 4), // 15 seconds = 1/4 minute
        fetch_future
      ).await
    {
      Ok(Ok(data)) => Ok(data),
      Ok(Err(e)) => {
        tracing::error!("WebSocket earnings fetch failed: {}", e);
        Err(e)
      }
      Err(_) => {
        tracing::warn!("WebSocket earnings fetch timed out after 15s");
        Err(AppError::network_error("WebSocket timeout".to_string()))
      }
    }
  }

  /// Handle cases where quarterly data is missing or incomplete
  fn handle_missing_quarterly_data(
    symbol: &str,
    quarterly_data: &[QuarterlyEPSData],
    current_timestamp: i64,
    result_map: &mut HashMap<String, (i64, i32)>
  ) {
    tracing::warn!(
      "No quarterly pattern available for {}, trying last announcement",
      symbol
    );

    // Try to calculate from last known announcement
    if let Some(last_quarter) = quarterly_data.first() {
      let estimated_next = last_quarter.timestamp + 90 * 86400;
      if estimated_next > current_timestamp {
        let days_until = ((estimated_next - current_timestamp) / 86400).max(
          0
        ) as i32;
        result_map.insert(symbol.to_string(), (estimated_next, days_until));
        tracing::info!(
          "Estimated from last quarter for {}: {} days",
          symbol,
          days_until
        );
        return;
      }
    }

    tracing::error!("No valid earnings data available for {}", symbol);
  }

  /// Add cached results for symbols not fetched from WebSocket
  fn add_cached_results(
    symbols: &[String],
    result_map: &mut HashMap<String, (i64, i32)>
  ) {
    for symbol in symbols {
      if !result_map.contains_key(symbol) {
        let cache = EARNINGS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((timestamp, days, _)) = cache.data.get(symbol) {
          result_map.insert(symbol.clone(), (*timestamp, *days));
        }
      }
    }
  }

  /// Find next earnings announcement from quarterly data using intelligent QoQ pattern analysis
  fn find_next_earnings_announcement(
    quarterly_data: &[QuarterlyEPSData],
    current_timestamp: i64
  ) -> Option<i64> {
    if quarterly_data.is_empty() {
      return None;
    }

    // PRIORITY 1: Look for future earnings announcements in WebSocket data
    for quarter in quarterly_data {
      if let Some(announcement_date) = quarter.estimated_earnings_date {
        if announcement_date > current_timestamp {
          tracing::info!(
            "Found future announcement: {} days from now",
            (announcement_date - current_timestamp) / 86400
          );
          return Some(announcement_date);
        }
      }
    }

    // PRIORITY 2: Calculate from quarterly intervals (QoQ pattern)
    if
      let Some(estimated) = Self::calculate_from_intervals(
        quarterly_data,
        current_timestamp
      )
    {
      return Some(estimated);
    }

    // PRIORITY 3: Use standard 90-day cycle
    if let Some(latest_quarter) = quarterly_data.first() {
      let estimated_next = latest_quarter.timestamp + 90 * 86400;
      if estimated_next > current_timestamp {
        tracing::info!("Using 90-day standard cycle from last announcement");
        return Some(estimated_next);
      }
    }

    None
  }

  /// Calculate next announcement from historical intervals
  fn calculate_from_intervals(
    quarterly_data: &[QuarterlyEPSData],
    current_timestamp: i64
  ) -> Option<i64> {
    if quarterly_data.len() < 2 {
      return None;
    }

    let mut intervals = Vec::new();
    let max_lookback = std::cmp::min(4, quarterly_data.len() - 1);

    for i in 0..max_lookback {
      let interval =
        quarterly_data[i].timestamp - quarterly_data[i + 1].timestamp;
      intervals.push(interval);
    }

    if intervals.is_empty() {
      return None;
    }

    // Use median interval for robust estimation
    intervals.sort();
    let median_interval = if intervals.len() % 2 == 0 {
      (intervals[intervals.len() / 2 - 1] + intervals[intervals.len() / 2]) / 2
    } else {
      intervals[intervals.len() / 2]
    };

    let estimated_next = quarterly_data[0].timestamp + median_interval;

    if estimated_next > current_timestamp {
      let days_until = (estimated_next - current_timestamp) / 86400;
      tracing::info!(
        "QoQ pattern: {} days median interval → {} days until next",
        median_interval / 86400,
        days_until
      );
      Some(estimated_next)
    } else {
      None
    }
  }

  /// Fetch real QoQ growth data using TradingView WebSocket with caching
  pub async fn fetch_qoq_data(
    symbols: Vec<String>
  ) -> Result<HashMap<String, f64>, AppError> {
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let mut result_map = HashMap::new();
    let mut symbols_to_fetch = Vec::new();

    // Check cache first
    {
      let cache = EARNINGS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
      for symbol in &symbols {
        if let Some((qoq_growth, cache_time)) = cache.qoq_data.get(symbol) {
          if current_time - cache_time < CACHE_DURATION_SECONDS {
            tracing::info!(
              "Using cached QoQ data for {}: {:.2}%",
              symbol,
              qoq_growth
            );
            result_map.insert(symbol.clone(), *qoq_growth);
            continue;
          }
        }
        symbols_to_fetch.push(symbol.clone());
      }
    }

    if symbols_to_fetch.is_empty() {
      tracing::info!("All {} QoQ symbols served from cache", symbols.len());
      return Ok(result_map);
    }

    tracing::info!(
      "Fetching {} QoQ symbols via WebSocket",
      symbols_to_fetch.len()
    );

    let websocket_data = Self::fetch_with_timeout(symbols_to_fetch).await?;

    // Process and cache QoQ data
    for ws_data in websocket_data {
      if
        let Some(qoq_growth) = Self::calculate_real_qoq_growth(
          &ws_data.quarterly_data
        )
      {
        result_map.insert(ws_data.symbol.clone(), qoq_growth);

        let mut cache = EARNINGS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        cache.qoq_data.insert(ws_data.symbol.clone(), (
          qoq_growth,
          current_time,
        ));

        tracing::info!(
          "Real WebSocket QoQ for {}: {:.2}%",
          ws_data.symbol,
          qoq_growth
        );
      }
    }

    tracing::info!(
      "WebSocket QoQ fetch complete: {}/{} symbols processed",
      result_map.len(),
      symbols.len()
    );

    Ok(result_map)
  }

  /// Calculate real quarter-over-quarter growth from quarterly data
  fn calculate_real_qoq_growth(
    quarterly_data: &[QuarterlyEPSData]
  ) -> Option<f64> {
    if quarterly_data.len() < 2 {
      return None;
    }

    let current_eps = quarterly_data[0].eps;
    let previous_eps = quarterly_data[1].eps;

    if previous_eps != 0.0 {
      let qoq_growth = ((current_eps - previous_eps) / previous_eps) * 100.0;
      Some(qoq_growth)
    } else {
      None
    }
  }
}
