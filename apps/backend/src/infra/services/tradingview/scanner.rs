// TradingView Scanner - Focused Module for Stock Screening and Filtering
// Handles request building, filtering logic, and screening parameters

use serde_json::json;
use tracing::debug;

use crate::dom::entities::market_data::{
    TradingViewResponse, TradingViewStock, StockScreeningResult, StockDataField,
    PhaseInfo, PhaseStatus, PhaseType, FinancialFormatter
};
use super::types::{TradingViewConfig, FrontendEPSResponse, FrontendPagination};
use super::rest::TradingViewRestClient;

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
                "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange"
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
                "sector.tr", "market", "sector", "AnalystRating", "AnalystRating.tr", "exchange"
            ],
            "filter": filters,
            "ignore_unknown_fields": false,
            "options": { "lang": "en" },
            "range": [range_start, range_end],
            "sort": { "sortBy": sort_field, "sortOrder": sort_order },
            "symbols": {},
            "markets": markets,
            "filter2": self.build_stock_type_filters()
        })
    }

    /// Build markets filter based on country parameter
    fn build_markets_filter(&self, country: Option<&String>) -> Vec<String> {
        if let Some(country) = country {
            vec![country.clone()]
        } else {
            // All available markets from TradingView
            vec![
                "america".to_string(), "argentina".to_string(), "australia".to_string(), 
                "austria".to_string(), "bahrain".to_string(), "bangladesh".to_string(),
                "belgium".to_string(), "brazil".to_string(), "canada".to_string(), 
                "chile".to_string(), "china".to_string(), "colombia".to_string(), 
                "cyprus".to_string(), "czech".to_string(), "denmark".to_string(), 
                "egypt".to_string(), "estonia".to_string(), "finland".to_string(), 
                "france".to_string(), "germany".to_string(), "greece".to_string(), 
                "hongkong".to_string(), "hungary".to_string(), "iceland".to_string(), 
                "india".to_string(), "indonesia".to_string(), "ireland".to_string(), 
                "israel".to_string(), "italy".to_string(), "japan".to_string(), 
                "kenya".to_string(), "kuwait".to_string(), "latvia".to_string(), 
                "lithuania".to_string(), "luxembourg".to_string(), "malaysia".to_string(), 
                "mexico".to_string(), "morocco".to_string(), "netherlands".to_string(), 
                "newzealand".to_string(), "nigeria".to_string(), "norway".to_string(), 
                "pakistan".to_string(), "peru".to_string(), "philippines".to_string(), 
                "poland".to_string(), "portugal".to_string(), "qatar".to_string(), 
                "romania".to_string(), "russia".to_string(), "ksa".to_string(), 
                "serbia".to_string(), "singapore".to_string(), "slovakia".to_string(), 
                "rsa".to_string(), "korea".to_string(), "spain".to_string(), 
                "srilanka".to_string(), "sweden".to_string(), "switzerland".to_string(), 
                "taiwan".to_string(), "thailand".to_string(), "tunisia".to_string(), 
                "turkey".to_string(), "uae".to_string(), "uk".to_string(), 
                "venezuela".to_string(), "vietnam".to_string()
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
            })
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
            request_payload["filter"].as_array_mut().unwrap().push(json!({
                "left": "name",
                "operation": "in_range",
                "right": symbols
            }));
        }
        
        request_payload
    }

    /// Process TradingView API response to screening results
    pub fn process_trading_view_response(&self, response: TradingViewResponse) -> Vec<StockScreeningResult> {
        response.data
            .into_iter()
            .map(|stock| self.convert_to_stock_screening_result(stock))
            .collect()
    }

    /// Convert TradingView stock data to stock screening result
    fn convert_to_stock_screening_result(&self, stock: TradingViewStock) -> StockScreeningResult {
        let get_number = |data: &[StockDataField], idx: usize| -> f64 {
            match data.get(idx) {
                Some(StockDataField::Number(n)) => *n,
                Some(StockDataField::Integer(i)) => *i as f64,
                _ => 0.0
            }
        };

        let get_string = |data: &[StockDataField], idx: usize, default: &str| -> String {
            data.get(idx)
                .map(|field| match field {
                    StockDataField::String(s) => s.clone(),
                    StockDataField::Number(n) => n.to_string(),
                    StockDataField::Integer(i) => i.to_string(),
                    StockDataField::Boolean(b) => b.to_string(),
                    StockDataField::Array(_) => "Array".to_string(),
                    StockDataField::Object(_) => "Object".to_string(),
                    StockDataField::Null => default.to_string(),
                })
                .unwrap_or_else(|| default.to_string())
        };

        let last = get_number(&stock.d, 17);
        let next = get_number(&stock.d, 18);
        let (entry_phase, phase_status) = if last != 0.0 && next != 0.0 {
            StockScreeningResult::get_analysis_phases(last as i64, next as i64)
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
                }
            )
        };

        StockScreeningResult {
            symbol: stock.s.split(':').nth(1).unwrap_or(&stock.s).to_string(),
            name: get_string(&stock.d, 0, ""),
            value_index: StockScreeningResult::format_number(Some(get_number(&stock.d, 6))), // close
            growth_rate: StockScreeningResult::format_number(Some(get_number(&stock.d, 12))), // change
            activity_score: StockScreeningResult::format_large_number(get_number(&stock.d, 13)), // volume
            market_size: StockScreeningResult::format_large_number(get_number(&stock.d, 15)), // market_cap_basic
            growth_factor: StockScreeningResult::format_number(Some(get_number(&stock.d, 17))), // price_earnings_ttm
            sector: get_string(&stock.d, 21, "N/A"), // sector.tr (correct sector field)
            country: get_string(&stock.d, 22, "N/A"), // market (correct country field)
            exchange: get_string(&stock.d, 26, "N/A"), // exchange (correct exchange field)
            currency: get_string(&stock.d, 11, "N/A"), // currency
            metric_score: StockScreeningResult::format_number(Some(get_number(&stock.d, 18))), // earnings_per_share_diluted_ttm
            growth_indicator: StockScreeningResult::format_number(Some(get_number(&stock.d, 19))), // earnings_per_share_diluted_yoy_growth_ttm
            current_metric: StockScreeningResult::format_number(Some(get_number(&stock.d, 14))), // earnings_per_share_fq
            predicted_metric: StockScreeningResult::format_number(Some(get_number(&stock.d, 16))), // earnings_per_share_forecast_next_fq
            last_analysis_date: StockScreeningResult::format_date(if last != 0.0 { Some(last as i64) } else { None }), // earnings_release_date
            next_analysis_date: StockScreeningResult::format_date(if next != 0.0 { Some(next as i64) } else { None }), // earnings_release_next_date
            entry_phase,
            phase_status,
            start_buy: None,
            start_action: None,
            eps_growth: None,
            last_earnings_date: None,
        }
    }

    /// Convert TradingView response to frontend format
    pub fn convert_to_frontend_format(&self, response: TradingViewResponse, page: i32, limit: i32) -> FrontendEPSResponse {
        let frontend_data = response.data
            .into_iter()
            .map(|stock| super::mapper::TradingViewMapper::map_to_frontend_eps_data(stock))
            .collect();

        let total = response.total_count;
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

    /// Get REST client reference
    pub fn get_rest_client(&self) -> &TradingViewRestClient {
        &self.rest_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_scanner_creation() {
        let config = Config::default();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);
        
        let request = scanner.build_screener_request();
        assert!(request["columns"].as_array().unwrap().len() > 0);
    }

    #[test]
    fn test_markets_filter() {
        let config = Config::default();
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
        let config = Config::default();
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
        let config = Config::default();
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
    fn test_request_with_params() {
        let config = Config::default();
        let tv_config = TradingViewConfig::from(&config);
        let scanner = TradingViewScanner::new(tv_config);
        
        let request = scanner.build_screener_request_with_params(
            0, 10,
            Some("america".to_string()),
            Some("Technology".to_string()),
            Some("eps_growth".to_string())
        );
        
        assert_eq!(request["range"].as_array().unwrap()[0], 0);
        assert_eq!(request["range"].as_array().unwrap()[1], 10);
        assert_eq!(request["markets"].as_array().unwrap()[0], "america");
        assert_eq!(request["sort"]["sortBy"], "earnings_per_share_diluted_qoq_growth_fq");
    }
}