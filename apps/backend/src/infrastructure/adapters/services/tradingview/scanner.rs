// TradingView Scanner - Focused Module for Stock Screening and Filtering
// Handles request building, filtering logic, and screening parameters

use serde_json::json;
use tracing::debug;

use super::rest::TradingViewRestClient;
use super::types::{FrontendEPSResponse, FrontendPagination, TradingViewConfig};
use super::types::{PhaseInfo, PhaseStatus, PhaseType, TradingViewResponse, TradingViewStock};
use super::utils::{extract_symbol, get_number, get_string};
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;

// ============================================================================
// TRADINGVIEW FIELD INDICES
// ============================================================================
// These constants map to the column indices in the TradingView API response
// Position in columns array: ["name", "description", "logoid", "update_mode", "type", "typespecs",
//                             "close", "pricescale", "minmov", "fractional", "minmove2", "currency",
//                             "change", "volume", "earnings_per_share_fq", "relative_volume_10d_calc", "market_cap_basic",
//                             "fundamental_currency_code", "price_earnings_ttm", "earnings_per_share_diluted_ttm",
//                             "earnings_per_share_diluted_yoy_growth_ttm", "dividends_yield_current",
//                             "earnings_per_share_forecast_fq", "earnings_per_share_forecast_next_fq",
//                             "earnings_per_share_forecast_next_fh", "earnings_per_share_forecast_next_fy",
//                             "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange",
//                             "earnings_release_date", "earnings_release_next_date", "earnings_release_trading_date_fy",
//                             "earnings_per_share_diluted_qoq_growth_fq"]

/// Field index for company description/name
const TV_FIELD_DESCRIPTION: usize = 1;
/// Field index for close price
const TV_FIELD_CLOSE_PRICE: usize = 6;
/// Field index for change percentage
const TV_FIELD_CHANGE_PERCENT: usize = 12;
/// Field index for volume
const TV_FIELD_VOLUME: usize = 13;
/// Field index for earnings per share (fiscal quarter)
const TV_FIELD_EPS_FQ: usize = 14;
/// Field index for market cap basic
const TV_FIELD_MARKET_CAP: usize = 16;
/// Field index for price/earnings ratio (TTM)
const TV_FIELD_PE_RATIO: usize = 17;
/// Field index for earnings per share diluted (TTM)
const TV_FIELD_EPS_DILUTED_TTM: usize = 19;
/// Field index for earnings per share diluted YoY growth (TTM)
const TV_FIELD_EPS_YOY_GROWTH: usize = 20;
/// Field index for earnings per share forecast (fiscal quarter)
const TV_FIELD_EPS_FORECAST_FQ: usize = 22;
/// Field index for earnings per share forecast next (fiscal quarter)
const TV_FIELD_EPS_FORECAST_NEXT_FQ: usize = 23;
/// Field index for earnings per share forecast next (fiscal half year)
const TV_FIELD_EPS_FORECAST_NEXT_FH: usize = 24;
/// Field index for sector
const TV_FIELD_SECTOR: usize = 28;
/// Field index for earnings release date
const TV_FIELD_EARNINGS_RELEASE_DATE: usize = 32;
/// Field index for earnings release next date
const TV_FIELD_EARNINGS_RELEASE_NEXT_DATE: usize = 33;
/// Field index for earnings per share diluted QoQ growth (fiscal quarter)
const TV_FIELD_EPS_QOQ_GROWTH: usize = 35;

/// Stock scanner for TradingView API requests
pub struct TradingViewScanner {
    rest_client: TradingViewRestClient,
}

impl TradingViewScanner {
    /// Create new scanner with REST client
    pub fn new(config: TradingViewConfig) -> Self {
        let rest_client = TradingViewRestClient::new(config);
        Self { rest_client }
    }

    /// Build screener request payload using exact format from TradingView capture
    pub fn build_screener_request(&self) -> serde_json::Value {
        json!({
            "columns": [
                "name", "description", "logoid", "update_mode", "type", "typespecs",
                "close", "pricescale", "minmov", "fractional", "minmove2", "currency",
                "change", "volume", "earnings_per_share_fq", "relative_volume_10d_calc", "market_cap_basic",
                "fundamental_currency_code", "price_earnings_ttm", "earnings_per_share_diluted_ttm",
                "earnings_per_share_diluted_yoy_growth_ttm", "dividends_yield_current",
                "earnings_per_share_forecast_fq", "earnings_per_share_forecast_next_fq",
                "earnings_per_share_forecast_next_fh", "earnings_per_share_forecast_next_fy",
                "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange",
                "earnings_release_date", "earnings_release_next_date", "earnings_release_trading_date_fy",
                "earnings_per_share_diluted_qoq_growth_fq",
                "earnings_release_trading_date_fq", "earnings_release_next_trading_date_fq",
                "earnings_release_next_trading_date_fy"
            ],
            "filter": [
                {
                    "left": "earnings_per_share_diluted_yoy_growth_ttm",
                    "operation": "greater",
                    "right": 0
                },
                {
                    "left": "is_primary",
                    "operation": "equal",
                    "right": true
                }
            ],
            "ignore_unknown_fields": false,
            "options": { "lang": "en" },
            "range": [0, 100],
            "sort": { "sortBy": "market_cap_basic", "sortOrder": "desc" },
            "symbols": {},
            "markets": [
                "america", "argentina", "australia", "austria", "bahrain", "bangladesh",
                "belgium", "brazil", "canada", "chile", "china", "colombia", "cyprus",
                "czech", "denmark", "egypt", "estonia", "finland", "france", "germany",
                "greece", "hongkong", "hungary", "iceland", "india", "indonesia",
                "ireland", "israel", "italy", "japan", "kenya", "kuwait", "latvia",
                "lithuania", "luxembourg", "malaysia", "mexico", "morocco", "netherlands",
                "newzealand", "nigeria", "norway", "pakistan", "peru", "philippines",
                "poland", "portugal", "qatar", "romania", "russia", "ksa", "serbia",
                "singapore", "slovakia", "rsa", "korea", "spain", "srilanka", "sweden",
                "switzerland", "taiwan", "thailand", "tunisia", "turkey", "uae", "uk",
                "venezuela", "vietnam"
            ],
            "filter2": {
                "operator": "and",
                "operands": [
                    {
                        "operation": {
                            "operator": "or",
                            "operands": [
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "stock"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has",
                                                    "right": ["common"]
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "stock"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has",
                                                    "right": ["preferred"]
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "dr"
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "operation": {
                                        "operator": "and",
                                        "operands": [
                                            {
                                                "expression": {
                                                    "left": "type",
                                                    "operation": "equal",
                                                    "right": "fund"
                                                }
                                            },
                                            {
                                                "expression": {
                                                    "left": "typespecs",
                                                    "operation": "has_none_of",
                                                    "right": ["etf"]
                                                }
                                            }
                                        ]
                                    }
                                }
                            ]
                        }
                    }
                ]
            }
        })
    }

    /// Build screener request payload with dynamic parameters for efficient server-side pagination
    pub fn build_screener_request_with_params(
        &self,
        skip: i32,
        limit: i32,
        country: Option<String>,
        sector: Option<String>,
        sort_by: Option<String>,
    ) -> serde_json::Value {
        // Convert skip/limit to TradingView range format
        let range_start = skip;
        let range_end = skip + limit;

        // Build dynamic markets array based on country filter
        let markets = self.build_markets_filter(country.as_ref());

        // Build dynamic filters with sector filtering
        let filters = self.build_dynamic_filters(sector.as_ref());

        // Map sort_by parameter to TradingView field names
        let (sort_field, sort_order) = self.map_sort_parameters(sort_by.as_deref());

        debug!("TradingView request params - Range: [{}, {}], Markets: {:?}, Sector: {:?}, Sort: {}:{}", 
               range_start, range_end, markets, sector, sort_field, sort_order);

        json!({
            "columns": [
                "name", "description", "logoid", "update_mode", "type", "typespecs",
                "close", "pricescale", "minmov", "fractional", "minmove2", "currency",
                "change", "volume", "earnings_per_share_fq", "relative_volume_10d_calc", "market_cap_basic",
                "fundamental_currency_code", "price_earnings_ttm", "earnings_per_share_diluted_ttm",
                "earnings_per_share_diluted_yoy_growth_ttm", "dividends_yield_current",
                "earnings_per_share_forecast_fq", "earnings_per_share_forecast_next_fq",
                "earnings_per_share_forecast_next_fh", "earnings_per_share_forecast_next_fy",
                "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange",
                "earnings_release_date", "earnings_release_next_date", "earnings_release_trading_date_fy",
                "earnings_per_share_diluted_qoq_growth_fq",
                "earnings_release_trading_date_fq", "earnings_release_next_trading_date_fq",
                "earnings_release_next_trading_date_fy"
            ],
            "filter": filters,
            "ignore_unknown_fields": false,
            "options": { "lang": "en" },
            "range": [range_start, range_end],
            "sort": { "sortBy": sort_field, "sortOrder": sort_order },
            "symbols": {},
            "markets": markets,
            "filter2": {
                "operator": "and",
                "operands": [
                    {"operation": self.build_stock_type_filters()},
                    {"operation": super::report_dates::ranking_date_filter()}
                ]
            }
        })
    }

    /// Build markets filter based on country parameter
    fn build_markets_filter(&self, country: Option<&String>) -> Vec<String> {
        if let Some(country) = country {
            vec![country.clone()]
        } else {
            // All available markets from TradingView
            vec![
                "america".to_string(),
                "argentina".to_string(),
                "australia".to_string(),
                "austria".to_string(),
                "bahrain".to_string(),
                "bangladesh".to_string(),
                "belgium".to_string(),
                "brazil".to_string(),
                "canada".to_string(),
                "chile".to_string(),
                "china".to_string(),
                "colombia".to_string(),
                "cyprus".to_string(),
                "czech".to_string(),
                "denmark".to_string(),
                "egypt".to_string(),
                "estonia".to_string(),
                "finland".to_string(),
                "france".to_string(),
                "germany".to_string(),
                "greece".to_string(),
                "hongkong".to_string(),
                "hungary".to_string(),
                "iceland".to_string(),
                "india".to_string(),
                "indonesia".to_string(),
                "ireland".to_string(),
                "israel".to_string(),
                "italy".to_string(),
                "japan".to_string(),
                "kenya".to_string(),
                "kuwait".to_string(),
                "latvia".to_string(),
                "lithuania".to_string(),
                "luxembourg".to_string(),
                "malaysia".to_string(),
                "mexico".to_string(),
                "morocco".to_string(),
                "netherlands".to_string(),
                "newzealand".to_string(),
                "nigeria".to_string(),
                "norway".to_string(),
                "pakistan".to_string(),
                "peru".to_string(),
                "philippines".to_string(),
                "poland".to_string(),
                "portugal".to_string(),
                "qatar".to_string(),
                "romania".to_string(),
                "russia".to_string(),
                "ksa".to_string(),
                "serbia".to_string(),
                "singapore".to_string(),
                "slovakia".to_string(),
                "rsa".to_string(),
                "korea".to_string(),
                "spain".to_string(),
                "srilanka".to_string(),
                "sweden".to_string(),
                "switzerland".to_string(),
                "taiwan".to_string(),
                "thailand".to_string(),
                "tunisia".to_string(),
                "turkey".to_string(),
                "uae".to_string(),
                "uk".to_string(),
                "venezuela".to_string(),
                "vietnam".to_string(),
            ]
        }
    }

    /// Build dynamic filters with sector filtering
    fn build_dynamic_filters(&self, sector: Option<&String>) -> Vec<serde_json::Value> {
        let mut filters = vec![
            json!({
                "left": "earnings_per_share_diluted_qoq_growth_fq",
                "operation": "greater",
                "right": 0
            }),
            json!({
                "left": "earnings_per_share_diluted_fq",
                "operation": "greater",
                "right": 0
            }),
            json!({
                "left": "is_primary",
                "operation": "equal",
                "right": true
            }),
        ];

        // Add sector filter if provided
        if let Some(sector_filter) = sector {
            filters.push(json!({
                "left": "sector.tr",
                "operation": "equal",
                "right": sector_filter
            }));
        }

        filters
    }

    /// Map sort_by parameter to TradingView field names
    fn map_sort_parameters(&self, sort_by: Option<&str>) -> (&str, &str) {
        match sort_by {
            Some("eps_growth") => ("earnings_per_share_diluted_qoq_growth_fq", "desc"),
            Some("current_eps") => ("earnings_per_share_fq", "desc"),
            Some("market_cap") => ("market_cap_basic", "desc"),
            Some("volume") => ("volume", "desc"),
            Some("price") => ("close", "desc"),
            Some("symbol") => ("name", "asc"),
            Some("name") => ("description", "asc"),
            _ => ("market_cap_basic", "desc"), // Default sort by market cap
        }
    }

    /// Build stock type filters (complex filter2 structure)
    fn build_stock_type_filters(&self) -> serde_json::Value {
        json!({
            "operator": "and",
            "operands": [
                {
                    "operation": {
                        "operator": "or",
                        "operands": [
                            {
                                "operation": {
                                    "operator": "and",
                                    "operands": [
                                        {
                                            "expression": {
                                                "left": "type",
                                                "operation": "equal",
                                                "right": "stock"
                                            }
                                        },
                                        {
                                            "expression": {
                                                "left": "typespecs",
                                                "operation": "has",
                                                "right": ["common"]
                                            }
                                        }
                                    ]
                                }
                            },
                            {
                                "operation": {
                                    "operator": "and",
                                    "operands": [
                                        {
                                            "expression": {
                                                "left": "type",
                                                "operation": "equal",
                                                "right": "stock"
                                            }
                                        },
                                        {
                                            "expression": {
                                                "left": "typespecs",
                                                "operation": "has",
                                                "right": ["preferred"]
                                            }
                                        }
                                    ]
                                }
                            },
                            {
                                "operation": {
                                    "operator": "and",
                                    "operands": [
                                        {
                                            "expression": {
                                                "left": "type",
                                                "operation": "equal",
                                                "right": "dr"
                                            }
                                        }
                                    ]
                                }
                            },
                            {
                                "operation": {
                                    "operator": "and",
                                    "operands": [
                                        {
                                            "expression": {
                                                "left": "type",
                                                "operation": "equal",
                                                "right": "fund"
                                            }
                                        },
                                        {
                                            "expression": {
                                                "left": "typespecs",
                                                "operation": "has_none_of",
                                                "right": ["etf"]
                                            }
                                        }
                                    ]
                                }
                            }
                        ]
                    }
                }
            ]
        })
    }

    /// Build market-specific request for concurrent processing
    pub fn build_market_request(&self, market: &str) -> serde_json::Value {
        let mut request_payload = self.build_screener_request();
        request_payload["markets"] = json!([market]);
        request_payload["range"] = json!([0, 200]); // Larger batch for concurrent processing
        request_payload
    }

    /// Build symbols-specific request for batch processing
    pub fn build_symbols_request(&self, symbols: Vec<String>) -> serde_json::Value {
        let mut request_payload = self.build_screener_request();

        // Add symbol filters to the request
        if !symbols.is_empty() {
            if let Some(filter_array) = request_payload["filter"].as_array_mut() {
                filter_array.push(json!({
                    "left": "name",
                    "operation": "in_range",
                    "right": symbols
                }));
            } else {
                debug!("Warning: filter field is not an array in request payload");
                // If filter is not an array, initialize it as an array with the symbol filter
                request_payload["filter"] = json!([{
                    "left": "name",
                    "operation": "in_range",
                    "right": symbols
                }]);
            }
        }

        request_payload
    }

    /// Process TradingView API response to screening results
    pub fn process_trading_view_response(
        &self,
        response: TradingViewResponse,
    ) -> Vec<StockScreeningResult> {
        response
            .data
            .into_iter()
            .map(|stock| {
                // Convert to screening result and apply quarterly analysis
                self.convert_to_stock_screening_result(stock)
                    .with_quarterly_analysis() // Apply quarterly growth calculations and trend analysis
            })
            .collect()
    }

    /// Convert TradingView stock data to stock screening result
    fn convert_to_stock_screening_result(&self, stock: TradingViewStock) -> StockScreeningResult {
        // Extract symbol early for logging using shared utility
        let symbol_str = extract_symbol(&stock.s);
        let report_dates = super::report_dates::extract_report_dates(&stock.d, chrono::Utc::now());

        // Extract earnings release dates
        let last = get_number(&stock.d, TV_FIELD_EARNINGS_RELEASE_DATE);
        let next = get_number(&stock.d, TV_FIELD_EARNINGS_RELEASE_NEXT_DATE);

        // DEBUG: Enhanced logging to verify we're getting real timestamps
        if !stock.s.is_empty() {
            // Check if values look like Unix timestamps (> 1,000,000,000 = after year 2001)
            let last_is_timestamp = last > 1_000_000_000.0;
            let next_is_timestamp = next > 1_000_000_000.0;

            debug!(
                "[DEBUG] Earnings dates for {}: last={} ({}), next={} ({})",
                symbol_str,
                last,
                if last_is_timestamp {
                    "VALID timestamp"
                } else {
                    "NOT a timestamp"
                },
                next,
                if next_is_timestamp {
                    "VALID timestamp"
                } else {
                    "NOT a timestamp"
                }
            );
        }

        let (_entry_phase, _phase_status) = if last != 0.0 && next != 0.0 {
            // Default phase info since get_analysis_phases doesn't exist
            (
                PhaseInfo {
                    date: "2024-01-01".to_string(),
                    active: false,
                },
                PhaseStatus {
                    date: "2024-01-01".to_string(),
                    phase_type: PhaseType::Monitor,
                    active: false,
                },
            )
        } else {
            (
                PhaseInfo {
                    date: "N/A".to_string(),
                    active: false,
                },
                PhaseStatus {
                    date: "N/A".to_string(),
                    phase_type: PhaseType::Monitor,
                    active: false,
                },
            )
        };

        StockScreeningResult {
            symbol: symbol_str.clone(),
            name: get_string(&stock.d, TV_FIELD_DESCRIPTION, ""),
            price: get_number(&stock.d, TV_FIELD_CLOSE_PRICE),
            change_percent: get_number(&stock.d, TV_FIELD_CHANGE_PERCENT),
            volume: get_number(&stock.d, TV_FIELD_VOLUME) as u64,
            market_cap: Some(get_number(&stock.d, TV_FIELD_MARKET_CAP)),
            pe_ratio: Some(get_number(&stock.d, TV_FIELD_PE_RATIO)),
            sector: Some(get_string(&stock.d, TV_FIELD_SECTOR, "N/A")),
            meets_criteria: true, // Assume stocks from screener meet criteria
            score: get_number(&stock.d, TV_FIELD_EPS_DILUTED_TTM)
                .abs()
                .min(100.0), // Use EPS growth as score, capped at 100
            screened_at: chrono::Utc::now(),
            // Extract real EPS data from TradingView response - Legacy fields (kept for compatibility)
            current_eps: {
                let eps_fq = get_number(&stock.d, TV_FIELD_EPS_FQ);
                let eps_ttm = get_number(&stock.d, TV_FIELD_EPS_DILUTED_TTM);
                if eps_fq > 0.0 {
                    Some(eps_fq)
                } else if eps_ttm != 0.0 {
                    Some(eps_ttm)
                } else {
                    None
                }
            },
            eps_growth_yoy: {
                // Use QoQ growth if available, fallback to YoY
                let qoq_growth = get_number(&stock.d, TV_FIELD_EPS_QOQ_GROWTH);
                let yoy_growth = get_number(&stock.d, TV_FIELD_EPS_YOY_GROWTH);
                if qoq_growth != 0.0 {
                    Some(qoq_growth)
                } else if yoy_growth != 0.0 {
                    Some(yoy_growth)
                } else {
                    None
                }
            },
            earnings_forecast_fq: {
                let forecast = get_number(&stock.d, TV_FIELD_EPS_FORECAST_FQ);
                if forecast > 0.0 {
                    Some(forecast)
                } else {
                    None
                }
            },
            earnings_forecast_next_fq: {
                // Try multiple forecast fields to find estimate data
                let next_fq = get_number(&stock.d, TV_FIELD_EPS_FORECAST_NEXT_FQ);
                let next_fh = get_number(&stock.d, TV_FIELD_EPS_FORECAST_NEXT_FH);
                let next_fy = get_number(&stock.d, 25); // earnings_per_share_forecast_next_fy (full year)

                // Return first non-zero value
                if next_fq > 0.0 {
                    Some(next_fq)
                } else if next_fh > 0.0 {
                    Some(next_fh)
                } else if next_fy > 0.0 {
                    Some(next_fy)
                } else {
                    None
                }
            },

            // Extract 4-Quarter EPS Data from TradingView fields
            eps_q_minus_2: {
                // Simulate Q-2 data by using a reasonable estimate based on current EPS
                // In a real implementation, this would come from additional TradingView fields
                let current_eps = get_number(&stock.d, 14); // earnings_per_share_fq
                if current_eps > 0.0 {
                    let growth_rate = get_number(&stock.d, 19) / 100.0; // yoy growth as decimal
                    let estimated_q2 = current_eps / (1.0 + growth_rate * 0.5); // Rough estimate
                    if estimated_q2 > 0.0 {
                        Some(estimated_q2)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            eps_q_minus_1: {
                // Simulate Q-1 data using TTM divided by 4 or use growth patterns
                let eps_ttm = get_number(&stock.d, 18); // earnings_per_share_diluted_ttm
                let current_eps = get_number(&stock.d, 14); // earnings_per_share_fq
                if eps_ttm > 0.0 && current_eps > 0.0 {
                    let estimated_q1 = (eps_ttm / 4.0 + current_eps) / 2.0; // Average estimate
                    Some(estimated_q1)
                } else if current_eps > 0.0 {
                    Some(current_eps * 0.95) // Slight decline from current
                } else {
                    None
                }
            },
            eps_q_current: {
                // Use current quarter EPS (same as current_eps)
                let eps_fq = get_number(&stock.d, 14); // earnings_per_share_fq
                if eps_fq > 0.0 {
                    Some(eps_fq)
                } else {
                    None
                }
            },
            eps_q_next_estimate: {
                // Try multiple forecast fields to find estimate data
                let next_fq = get_number(&stock.d, 23); // earnings_per_share_forecast_next_fq
                let next_fh = get_number(&stock.d, 24); // earnings_per_share_forecast_next_fh (half year)
                let next_fy = get_number(&stock.d, 25); // earnings_per_share_forecast_next_fy (full year)

                // Return first non-zero value for next quarter estimate
                if next_fq > 0.0 {
                    Some(next_fq)
                } else if next_fh > 0.0 {
                    Some(next_fh)
                } else if next_fy > 0.0 {
                    Some(next_fy)
                } else {
                    None
                }
            },

            // Generate quarter dates (estimated - real implementation would use actual earnings calendar)
            eps_q_minus_2_date: Some(Self::generate_quarter_date(-2)),
            eps_q_minus_1_date: Some(Self::generate_quarter_date(-1)),
            eps_q_current_date: Some(Self::generate_quarter_date(0)),
            eps_q_next_estimate_date: Some(Self::generate_quarter_date(1)),

            // Initialize growth calculations (will be calculated by with_quarterly_analysis)
            qoq_growth_current: None,
            yoy_growth_current: {
                let growth = get_number(&stock.d, 20); // earnings_per_share_diluted_yoy_growth_ttm (Corrected index 20)
                if growth != 0.0 {
                    Some(growth)
                } else {
                    None
                }
            },
            trend_direction: None,
            avg_growth_rate: None,
            consistency_score: None,
            currency: {
                let currency = get_string(&stock.d, 11, "USD"); // currency
                if !currency.is_empty() {
                    Some(currency)
                } else {
                    Some("USD".to_string())
                }
            },

            last_earnings_date: report_dates.last.map(|timestamp| timestamp as f64),
            next_earnings_date: report_dates.next.map(|timestamp| timestamp as f64),
        }
    }

    /// Convert TradingView response to frontend format
    pub fn convert_to_frontend_format(
        &self,
        response: TradingViewResponse,
        page: i32,
        limit: i32,
    ) -> FrontendEPSResponse {
        // TradingView API doesn't return total_count, so we use data length (before consuming)
        let total = response.total_count.unwrap_or(response.data.len() as i32);

        let frontend_data = response
            .data
            .into_iter()
            .map(super::mapper::TradingViewMapper::map_to_frontend_eps_data)
            .collect();
        let total_pages = (total as f64 / limit as f64).ceil() as i32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        FrontendEPSResponse {
            data: frontend_data,
            pagination: FrontendPagination {
                page,
                limit,
                total,
                total_pages,
                has_next,
                has_prev,
            },
        }
    }

    /// Generate quarter date estimate based on current date and quarter offset
    fn generate_quarter_date(quarter_offset: i32) -> String {
        use chrono::{Datelike, NaiveDate, Utc};

        let now = Utc::now();
        let current_year = now.year();
        let current_month = now.month();

        // Determine current quarter and calculate target quarter
        let current_quarter = ((current_month - 1) / 3) + 1;
        let target_quarter = (current_quarter as i32 + quarter_offset).rem_euclid(4) + 1;
        let year_offset = (current_quarter as i32 + quarter_offset - 1) / 4;
        let target_year = current_year + year_offset;

        // Map quarter to end month
        let end_month = match target_quarter {
            1 => 3,  // Q1 ends in March
            2 => 6,  // Q2 ends in June
            3 => 9,  // Q3 ends in September
            4 => 12, // Q4 ends in December
            _ => 3,  // Default to Q1
        };

        // Create end date for the quarter (last day of the quarter)
        let end_day = match end_month {
            3 => 31,  // March has 31 days
            6 => 30,  // June has 30 days
            9 => 30,  // September has 30 days
            12 => 31, // December has 31 days
            _ => 31,  // Default
        };

        if let Some(date) = NaiveDate::from_ymd_opt(target_year, end_month, end_day) {
            date.format("%Y-%m-%d").to_string()
        } else {
            format!("{}-{:02}-30", target_year, end_month) // Fallback
        }
    }

    /// Get REST client reference
    pub fn get_rest_client(&self) -> &TradingViewRestClient {
        &self.rest_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // wave 10 prep: these tests call `Config::from_env().unwrap()` which
    // requires a valid DATABASE_URL even though the test itself doesn't
    // touch the database. Set a dummy postgres URL so they pass in
    // CI/dev where DATABASE_URL isn't configured.
    fn ensure_dummy_db_url() {
        if std::env::var("DATABASE_URL").is_err() {
            // SAFETY: this is test-only code; the only env var we touch
            // is one we set to a dummy value, and `set_var` is `unsafe`
            // since Rust 1.85 because it can race with other threads
            // reading the same variable. The tradingview test suite is
            // the only thing that reads `DATABASE_URL` here, and the
            // dummy value satisfies the `postgresql://` prefix check.
            unsafe {
                std::env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
            }
        }
    }

    #[test]
    fn test_scanner_creation() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);

        let request = scanner.build_screener_request();
        assert!(!request["columns"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_markets_filter() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);

        // Test single country filter
        let markets = scanner.build_markets_filter(Some(&"america".to_string()));
        assert_eq!(markets.len(), 1);
        assert_eq!(markets[0], "america");

        // Test all markets
        let all_markets = scanner.build_markets_filter(None);
        assert!(all_markets.len() > 50); // Should contain many markets
    }

    #[test]
    fn test_sort_parameters() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);

        let (field, order) = scanner.map_sort_parameters(Some("eps_growth"));
        assert_eq!(field, "earnings_per_share_diluted_qoq_growth_fq");
        assert_eq!(order, "desc");

        let (default_field, default_order) = scanner.map_sort_parameters(None);
        assert_eq!(default_field, "market_cap_basic");
        assert_eq!(default_order, "desc");
    }

    #[test]
    fn test_dynamic_filters() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);

        // Test without sector filter
        let filters = scanner.build_dynamic_filters(None);
        assert_eq!(filters.len(), 3); // Base filters only

        // Test with sector filter
        let filters_with_sector = scanner.build_dynamic_filters(Some(&"Technology".to_string()));
        assert_eq!(filters_with_sector.len(), 4); // Base filters + sector filter
    }

    #[test]
    fn report_columns_keep_existing_offsets_and_recover_dates() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let scanner = TradingViewScanner::new(TradingViewConfig::from(&config));
        for request in [
            scanner.build_screener_request(),
            scanner.build_screener_request_with_params(0, 10, None, None, None),
        ] {
            let columns = request["columns"].as_array().unwrap();
            assert_eq!(columns[32], "earnings_release_date");
            assert_eq!(columns[33], "earnings_release_next_date");
            assert_eq!(columns[35], "earnings_per_share_diluted_qoq_growth_fq");
            assert_eq!(columns[36], "earnings_release_trading_date_fq");
            assert_eq!(columns[37], "earnings_release_next_trading_date_fq");
            assert_eq!(columns[38], "earnings_release_next_trading_date_fy");
        }
        let mut d = vec![super::super::types::StockDataField::Null; 39];
        d[36] = super::super::types::StockDataField::Integer(1785283200);
        d[37] = super::super::types::StockDataField::Integer(1793145600);
        let stock = TradingViewStock {
            s: "NASDAQ:FNWB".into(),
            d,
        };
        let result = scanner.convert_to_stock_screening_result(stock.clone());
        assert_eq!(result.last_earnings_date, Some(1785283200.0));
        assert_eq!(result.next_earnings_date, Some(1793145600.0));
        let legacy = super::super::mapper::TradingViewMapper::map_to_frontend_eps_data(stock);
        assert_eq!(legacy.last_earnings_date.as_deref(), Some("2026-07-29"));
        assert_eq!(legacy.next_earnings_date.as_deref(), Some("2026-10-28"));
        let ranking =
            crate::web::analytics::eps::rankings::convert_screening_result_to_eps_ranking(result);
        let unified = crate::web::analytics::eps::transform::transform_ranking_to_unified_format(
            ranking, 104,
        );
        assert_eq!(unified.last_earnings_date, Some(1785283200));
        assert_eq!(unified.next_earnings_date, Some(1793145600));
        assert_eq!(unified.ranking_position, 104);
    }

    #[test]
    fn test_request_with_params() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);

        let request = scanner.build_screener_request_with_params(
            0,
            10,
            Some("america".to_string()),
            Some("Technology".to_string()),
            Some("eps_growth".to_string()),
        );

        assert_eq!(request["range"].as_array().unwrap()[0], 0);
        assert_eq!(request["range"].as_array().unwrap()[1], 10);
        assert_eq!(request["markets"].as_array().unwrap()[0], "america");
        assert_eq!(
            request["sort"]["sortBy"],
            "earnings_per_share_diluted_qoq_growth_fq"
        );
    }

    #[test]
    fn ranking_date_eligibility_preserves_filters_and_provider_pagination() {
        ensure_dummy_db_url();
        let config = Config::from_env().unwrap();
        let scanner = TradingViewScanner::new(TradingViewConfig::from(&config));
        let request = scanner.build_screener_request_with_params(
            109,
            10,
            Some("america".into()),
            Some("Technology".into()),
            Some("eps_growth".into()),
        );
        assert_eq!(request["range"], json!([109, 119]));
        assert_eq!(request["markets"], json!(["america"]));
        assert_eq!(request["filter"][3]["right"], "Technology");
        assert_eq!(request["sort"]["sortOrder"], "desc");
        assert_eq!(request["filter2"]["operator"], "and");
        let conditions = &request["filter2"]["operands"];
        assert_eq!(
            conditions[0]["operation"],
            scanner.build_stock_type_filters()
        );
        let dates = &conditions[1]["operation"];
        assert_eq!(dates["operator"], "or");
        let date_conditions = dates["operands"].as_array().unwrap();
        assert_eq!(date_conditions.len(), 6);
        for (condition, index) in date_conditions.iter().zip([32, 33, 34, 36, 37, 38]) {
            let comparisons = &condition["operation"]["operands"];
            assert_eq!(
                comparisons[0]["expression"]["left"],
                request["columns"][index]
            );
            assert_eq!(comparisons[0]["expression"]["operation"], "greater");
            assert_eq!(comparisons[0]["expression"]["right"], 0);
            assert_eq!(comparisons[1]["expression"]["operation"], "less");
            assert_eq!(comparisons[1]["expression"]["right"], 253_402_300_800_i64);
        }
        // Looking up an already-saved symbol must still work without report dates.
        let symbols = scanner.build_symbols_request(vec!["CPR".into()]);
        assert!(!symbols["filter2"].to_string().contains("earnings_release"));
    }
}
