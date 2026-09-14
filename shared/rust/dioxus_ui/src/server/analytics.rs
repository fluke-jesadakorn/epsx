//! Shared analytics server fns — `A7 B3` insights + `A8 B1` admin dashboard.
//!
//! Phase 2 will move `pages/home.rs:get_home_rankings` + `pages/analytics.rs:get_analytics_rankings`
//! here and add `get_admin_analytics_dashboard` / `get_admin_audit_log` so
//! frontend `/:3000` + admin `/:3001` share 1 validated fetch.

use dioxus::prelude::{server, ServerFnError};

use super::{api_base, fetch_value};
use crate::pages::analytics::AnalyticsResponse;

/// Shared live fetch — `GET {API_URL}/api/analytics/rankings?page=&limit=`.
/// Validated once via `AnalyticsResponse::validated()` (`deny_unknown_fields`).
pub async fn fetch_rankings(page: u32, limit: u32) -> Result<AnalyticsResponse, ServerFnError> {
    let url = format!(
        "{}/api/analytics/rankings?page={}&limit={}",
        api_base(),
        page,
        limit
    );
    let value = fetch_value(url).await?;
    let response: AnalyticsResponse =
        serde_json::from_value(value).map_err(|e| ServerFnError::new(e.to_string()))?;
    response
        .clone()
        .validated()
        .map_err(|_| ServerFnError::new("validation failed".to_string()))?;
    Ok(response)
}

#[server]
pub async fn get_analytics_rankings_shared(
    page: u32,
    limit: u32,
) -> Result<AnalyticsResponse, ServerFnError> {
    fetch_rankings(page, limit).await
}

#[server]
pub async fn get_home_rankings_shared() -> Result<AnalyticsResponse, ServerFnError> {
    fetch_rankings(1, 3).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use wiremock::{matchers::*, Mock, MockServer, ResponseTemplate};

    fn ranking() -> serde_json::Value {
        serde_json::json!({
            "rank": 1,
            "symbol": "LIVE",
            "company_name": "LIVE Company",
            "latest_date": "2026-07-27",
            "value": 90.0,
            "active_status": "TRACK",
            "quarterly_performance": [{
                "quarter": "Q2",
                "date": "2026-06-30",
                "price": 1234.5,
                "eps": 2.5,
                "eps_growth": 1.0,
                "price_growth": 4.0,
                "announcement_date": "Jul 20, 2026",
                "announcement_timestamp": 1784505600,
                "is_estimated": false
            }],
            "next_quarter_estimate": {
                "quarter": "2026-Q3",
                "estimated_eps": 3.0,
                "announcement_date": "Oct 20, 2026",
                "announcement_timestamp": 1792454400,
                "days_until_announcement": 45,
                "estimated_price_target": 1300.0,
                "confidence": "High"
            },
            "next_earnings_date": 1792454400,
            "last_earnings_date": 1784505600,
            "next_earnings_date_formatted": "Oct 20, 2026",
            "days_until_next_earnings": 45,
            "progress_percentage": 50.0,
            "current_eps": 2.5,
            "growth_factor": 1.0,
            "price_current": 1234.5
        })
    }

    fn response_body() -> serde_json::Value {
        serde_json::json!({
            "success": true,
            "data": [ranking()],
            "pagination": {
                "page": 1,
                "limit": 10,
                "total": 1,
                "totalPages": 1,
                "hasNext": false,
                "hasPrev": false
            },
            "metadata": {
                "available_countries": ["america"],
                "available_sectors": ["Technology"],
                "request_timestamp": "2026-07-27T00:00:00Z",
                "data_source": "live"
            },
            "access_info": {
                "min_accessible_rank": 100,
                "locked_ranks_count": 99,
                "max_accessible_rank": null
            },
            "message": "live",
            "processing_time_ms": 3
        })
    }

    #[tokio::test]
    #[serial]
    async fn fetch_rankings_via_wiremock_validated() {
        let server = MockServer::start().await;
        crate::server::set_api_base_for_test(server.uri());
        let body = response_body();
        Mock::given(method("GET"))
            .and(path_regex(r"/api/analytics/rankings.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .mount(&server)
            .await;
        let res = fetch_rankings(1, 10)
            .await
            .expect("wiremock rankings must validate");
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.data[0].symbol, "LIVE");
        crate::server::clear_api_base_for_test();
    }
}
