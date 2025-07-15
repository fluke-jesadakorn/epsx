use serde::{ Deserialize, Serialize };
use tracing::{ debug, warn }; // Added info
use chrono::{ Datelike, NaiveDateTime }; // For DateTime operations

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PricePoint {
    pub timestamp: i64,
    pub close_price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EarningsResult {
    pub release_date: i64,
    pub eps: f64,
    pub price_at_release: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuarterlyEarnings {
    #[serde(rename = "Actual")]
    pub actual: Option<f64>,
    #[serde(rename = "Estimate")]
    pub estimate: f64,
    #[serde(rename = "FiscalPeriod")]
    pub fiscal_period: String,
    #[serde(rename = "IsReported")]
    pub is_reported: bool,
    #[serde(rename = "Type")]
    pub report_type: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuarterlyEarningsResponse {
    pub earnings_fq_h: Vec<QuarterlyEarnings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviousEPS {
    pub previous_quarter: Option<f64>, // Consider removing if not used elsewhere
    pub previous_quarter_2: Option<f64>, // Consider removing if not used elsewhere
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LatestEarningsData {
    // Vec of (release_timestamp, actual_eps)
    pub data: Vec<(i64, f64)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpsData {
    pub actual: f64,
    pub estimate: f64,
    pub period: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockData {
    pub price: f64,
    pub eps_history: Vec<EpsData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteSymbolPayload {
    pub n: String,
    pub s: String,
    pub v: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteSymbolMessage {
    pub m: String,
    pub p: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteData {
    pub n: String,
    pub s: String,
    pub v: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    #[serde(rename = "m")]
    pub message_type: Option<String>,
    #[serde(rename = "p")]
    pub payload: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl WebSocketMessage {
    // Parses QSD messages for the two most recent earnings release dates and actual EPS
    pub fn get_latest_earnings_data(&self) -> Option<LatestEarningsData> {
        let payload = self.payload.as_ref()?;
        let p_array = payload.as_array()?;
        if p_array.len() < 2 {
            return None;
        } // Ensure payload has at least two elements

        // Check if it's a QSD message
        if self.message_type.as_deref() != Some("qsd") {
            debug!("Not a QSD message: {:?}", self.message_type);
            return None;
        }

        let second_element = p_array.get(1)?;
        debug!("Full payload second element: {:?}", second_element);
        let v_obj = second_element.get("v")?.as_object()?;

        // Log all available keys and values in v_obj for debugging
        debug!("QSD message full v_obj content: {:?}", v_obj);

        let earnings_fq_h = match v_obj.get("earnings_fq_h") {
            Some(e) => match e.as_array() {
                Some(arr) => arr,
                None => {
                    debug!("earnings_fq_h is not an array: {:?}", e);
                    return None;
                }
            },
            None => {
                debug!("earnings_fq_h not found in v_obj");
                return None;
            }
        };

        let release_dates_fq_h = match v_obj.get("earnings_release_date_fq_h") {
            Some(d) => match d.as_array() {
                Some(arr) => arr,
                None => {
                    debug!("earnings_release_date_fq_h is not an array: {:?}", d);
                    return None;
                }
            },
            None => {
                debug!("earnings_release_date_fq_h not found in v_obj");
                return None;
            }
        };

        debug!("Found {} earnings entries and {} release dates", 
            earnings_fq_h.len(), release_dates_fq_h.len());


        if earnings_fq_h.len() != release_dates_fq_h.len() {
            warn!(
                "Mismatch between earnings count ({}) and release date count ({})",
                earnings_fq_h.len(),
                release_dates_fq_h.len()
            );
            // Allow processing to continue even if counts mismatch
            // return None; // Removed this line
        }

        let mut combined_data: Vec<(i64, f64)> = Vec::new();
        
        // Process each earnings entry
        for earnings_entry in earnings_fq_h.iter().filter_map(|e| e.as_object()) {
            // Only process reported earnings with actual EPS values
            if !earnings_entry.get("IsReported").and_then(|v| v.as_bool()).unwrap_or(false) {
                continue;
            }

            let actual_eps = match earnings_entry.get("Actual").and_then(|v| v.as_f64()) {
                Some(eps) => eps,
                None => continue,
            };

            let fiscal_period = match earnings_entry.get("FiscalPeriod").and_then(|v| v.as_str()) {
                Some(period) => period,
                None => continue,
            };

            // Find the nearest release date for this earnings entry
            if let Some(release_date) = find_nearest_release_date(release_dates_fq_h, fiscal_period) {
                combined_data.push((release_date, actual_eps));
            }
        }

        // Sort by date descending (most recent first)
        combined_data.sort_by(|a, b| b.0.cmp(&a.0));

        // Take the top 4 (last 4 EPS values)
        combined_data.truncate(4);

        if combined_data.is_empty() {
            debug!("No valid earnings data found in QSD message");
            None
        } else {
            debug!("Successfully parsed {} earnings data points", combined_data.len());
            Some(LatestEarningsData { data: combined_data })
        }
    }

    // Parses timescale_update messages for price history
    pub fn get_price_history(&self) -> Option<Vec<PricePoint>> {
        let payload = self.payload.as_ref()?;
        let p_array = payload.as_array()?;
        if p_array.len() < 2 {
            return None;
        } // Ensure payload has at least two elements

        // Check if it's a timescale_update message
        if self.message_type.as_deref() != Some("timescale_update") {
            return None;
        }

        // The price data seems nested within the second element 'p[1]' -> 'sds_1' -> 's'
        let series_data = p_array
            .get(1)?
            .as_object()?
            .values() // Iterate over values like 'sds_1'
            .find_map(|v| v.get("s").and_then(|s| s.as_array()))?;

        let mut price_points = Vec::new();
        for item in series_data {
            if let Some(v_array) = item.get("v").and_then(|v| v.as_array()) {
                if v_array.len() >= 5 {
                    // Ensure timestamp and close price exist
                    if
                        let (Some(timestamp_float), Some(close_price)) = (
                            v_array[0].as_f64(),
                            v_array[4].as_f64(), // Index 4 is close price based on example
                        )
                    {
                        // Convert float timestamp to i64 (seconds)
                        let timestamp_sec = timestamp_float as i64;
                        price_points.push(PricePoint {
                            timestamp: timestamp_sec,
                            close_price,
                        });
                    }
                }
            }
        }

        if price_points.is_empty() {
            None
        } else {
            Some(price_points)
        }
    }

    pub fn parse_tradingview(text: &str) -> Option<serde_json::Value> {
        // TradingView format: ~m~LENGTH~m~JSON
        if !text.starts_with("~m~") {
            return None;
        }

        // Extract the actual JSON message
        let msg_start = text.rfind("~m~")? + 3;
        let json_str = &text[msg_start..];

        // Parse and return the JSON part
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    }
}

// Helper function to find the PricePoint with the timestamp closest to the target_timestamp
pub fn find_nearest_price(
    price_history: &Vec<PricePoint>,
    target_timestamp: i64
) -> Option<PricePoint> {
    price_history
        .iter()
        .min_by_key(|pp| (pp.timestamp - target_timestamp).abs())
        .cloned() // Clone the PricePoint to return an owned value
}

// Helper function to find the nearest release date for a given earnings entry
fn find_nearest_release_date(release_dates: &[serde_json::Value], earnings_quarter: &str) -> Option<i64> {
    // Helper function to extract quarter from timestamp
    fn get_quarter(timestamp: i64) -> String {
        let dt = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap_or_default();
        format!("{}Q{}", dt.year(), (dt.month() - 1) / 3 + 1)
    }
    
    // Extract year and quarter from earnings_quarter (e.g., "2025Q1")
    let target_year: i32 = earnings_quarter[..4].parse().ok()?;
    let target_quarter: i32 = earnings_quarter[5..].parse().ok()?;

    release_dates
        .iter()
        .filter_map(|d| d.as_i64())
        .min_by_key(|&timestamp| {
            let quarter = get_quarter(timestamp);
            let year: i32 = quarter[..4].parse().unwrap_or(0);
            let q: i32 = quarter[5..].parse().unwrap_or(0);
            
            // Calculate distance in quarters
            let year_diff = (year - target_year).abs() * 4;  // Convert year difference to quarters
            let quarter_diff = (q - target_quarter).abs();
            year_diff + quarter_diff
        })
}
