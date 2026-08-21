// Cache Management for EPS Analytics
// Focused module handling caching logic and cache-related endpoints

use axum::{
    extract::{Extension, Query},
    response::Json,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::domain::market_analytics::repository_ports::{
    MarketRankingsPage, MarketRankingsProviderPort, MarketRankingsRequest,
};
use crate::domain::shared_kernel::entities::eps_growth::EPSRanking;
use crate::domain::shared_kernel::entities::market_data::StockScreeningResult;
use epsx_contracts::errors::{AppError, ErrorKind};
use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
// wave12(track-b): EPSCacheService import removed — the dead `get_cache_stats`
// and `force_cache_refresh` handlers were deleted (option b decision).
use super::{
    metadata::{get_available_countries_static, get_available_sectors_static},
    transform::{transform_ranking_to_unified_format, transform_unified_to_card_format},
    types::*,
};
use crate::infrastructure::cache::Cache;
use crate::web::analytics::convert_screening_result_to_eps_ranking;
use crate::web::middleware::bearer_middleware::OpenIDUserContext;

/// Minimal request context inserted only after a transport has verified the
/// caller. The standalone market service uses this instead of fabricating the
/// monolith-specific `OpenIDUserContext` fields it does not receive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsWalletContext {
    wallet_address: String,
}

impl AnalyticsWalletContext {
    pub fn new(wallet_address: String) -> Self {
        Self { wallet_address }
    }

    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }
}

/// GET /api/analytics/rankings - bounded market-provider card dashboard endpoint
#[utoipa::path(
    get,
    path = "/api/analytics/rankings",
    tag = "analytics",
    responses(
        (status = 200, description = "Successfully retrieved analytics rankings", body = CardDashboardResponse),
        (status = 400, description = "Unsupported sort or pagination range"),
        (status = 502, description = "Market provider returned a permanent or invalid response"),
        (status = 503, description = "Ranking access authority or market provider is unavailable, or the process concurrency budget is saturated"),
        (status = 504, description = "Market provider request exceeded the total deadline")
    ),
    params(
        ("page" = Option<i32>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i32>, Query, description = "Items per page (default: 10; anonymous max: 10; authenticated max: 100)"),
        ("country" = Option<String>, Query, description = "Filter by country code (e.g., 'america', 'uk')"),
        ("sector" = Option<String>, Query, description = "Filter by sector (e.g., 'Technology', 'Healthcare')"),
        ("sort_by" = Option<String>, Query, description = "Sort field (default: 'eps_growth'; aliases: qoq_growth, growth_factor, ranking_position)"),
        ("min_eps" = Option<f64>, Query, description = "Reserved minimum EPS filter; not yet enforced by the canonical provider"),
        ("min_growth" = Option<f64>, Query, description = "Reserved minimum growth filter; not yet enforced by the canonical provider")
    )
)]
pub async fn get_unified_analytics_rankings_cached(
    Query(params): Query<EPSRankingQueryParams>,
    Extension(_cache): Extension<Arc<dyn Cache>>,
    Extension(permission_service): Extension<Arc<dyn WalletRankingOffsetQuery>>,
    Extension(rankings_provider): Extension<Arc<dyn MarketRankingsProviderPort>>,
    user_context_ext: Option<Extension<OpenIDUserContext>>,
    analytics_wallet_ext: Option<Extension<AnalyticsWalletContext>>,
) -> Result<Json<CardDashboardResponse>, AppError> {
    debug!(
        "Direct TradingView analytics rankings API called with params: {:?}",
        params
    );

    let user_context = user_context_ext.map(|ext| ext.0);
    let analytics_wallet = analytics_wallet_ext.map(|ext| ext.0);

    // Both transports insert server-owned request extensions only after token
    // verification. The standalone service context wins when present because it
    // is produced by its direct `epsx-service-auth` boundary.
    let wallet_address = analytics_wallet
        .as_ref()
        .map(|context| context.wallet_address().to_lowercase())
        .or_else(|| {
            user_context
                .as_ref()
                .map(|ctx| ctx.wallet_address.to_lowercase())
        });
    let is_authenticated = wallet_address.is_some();

    // Anonymous rankings deliberately use the free-tier input without calling
    // plan authority. Once a verified wallet exists, however, only a successful
    // authority decision may select its rank range. Treating an authority
    // outage as free access would turn an operational failure into a false plan
    // decision and still amplify the request into market-provider work.
    let access =
        resolve_market_ranking_access(permission_service.as_ref(), wallet_address.as_deref())
            .await?;

    debug!(
        "Rankings permission config resolved: offset={}, limit_cap={}",
        access.rank_offset, access.rankings_limit
    );

    // `min_eps` and `min_growth` remain explicit A2.5 residuals until
    // the provider contract can enforce them instead of silently ignoring them.
    let prepared = prepare_market_rankings_request(
        &params,
        access.rank_offset,
        access.rankings_limit,
        is_authenticated,
    )?;
    let page = prepared.page;
    let limit = prepared.page_size;
    let skip = prepared.request.skip;
    let rank_start = prepared.rank_start;

    // Generate cache key for this request (includes rank_offset so different plans get separate caches)
    let cache_key = generate_cache_key(&params, access.rank_offset, access.rankings_limit);
    debug!("Generated cache key: {}", cache_key);

    // CACHE DISABLED FOR SECURITY CONTROL (Always fetch fresh from DB/TradingView)
    debug!("Development environment or security override - skipping cache lookup");

    debug!("Cache miss for analytics rankings - fetching fresh data");

    // Log request details for debugging
    info!(
    "Processing direct TradingView analytics rankings - Country: {:?}, Sort: {:?}, Page: {}, Limit: {}",
    params.country,
    params.sort_by,
    page,
    limit
  );

    // Fetch data through the injected provider boundary. Construction, retries,
    // concurrency limits, and upstream error details stay outside this handler.
    let start_time = std::time::Instant::now();
    let MarketRankingsPage { items, total } =
        fetch_market_rankings(rankings_provider.as_ref(), prepared.request).await?;
    let (total_count, total_pages, has_next, has_prev) =
        accessible_pagination(total, rank_start, access.rankings_limit, page, limit);
    let card_data = map_market_rankings_to_cards(items, skip);

    // Prepare metadata - using direct TradingView API
    let metadata = CardDashboardMetadata {
        available_countries: get_available_countries_static(),
        available_sectors: get_available_sectors_static(),
        request_timestamp: chrono::Utc::now(),
        data_source: "live_tradingview_api".to_string(),
    };

    let duration = start_time.elapsed();

    // DEBUG: Capture final DTO structure before JSON serialization
    let _dto_debug = card_data
        .iter()
        .take(3)
        .map(|card| {
            let quarters_debug = card
                .quarterly_performance
                .iter()
                .take(2)
                .map(|q| {
                    format!(
                        "  Quarter: '{}', Date: '{}', EPS: {:.2}, Price: {:.2}",
                        q.quarter, q.date, q.eps, q.price
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "Symbol: {}, Rank: {}, Status: '{}', Value: {:.2}\nQuarterly Performance:\n{}",
                card.symbol, card.rank, card.active_status, card.value, quarters_debug
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Build card dashboard response
    let data_len = card_data.len();
    let card_response = CardDashboardResponse {
        success: true,
        data: card_data,
        pagination: EPSPaginationResponse {
            page,
            limit,
            total: total_count as i64,
            total_pages,
            has_next,
            has_prev,
        },
        metadata,
        access_info: Some(AccessInfo {
            min_accessible_rank: access.rank_offset.max(0).saturating_add(1),
            locked_ranks_count: access.rank_offset.max(0),
        }),
        message: Some(format!(
            "Fetched {} card dashboard rankings successfully from TradingView API",
            data_len
        )),
        processing_time_ms: duration.as_millis() as u64,
    };

    info!(
        "Direct TradingView API card dashboard completed in {:?} - {} items returned",
        duration, data_len
    );

    // Store response in cache with 1-hour TTL (3600 seconds)
    // Only cache in non-development environments
    if !crate::config::env::is_development() {
        // CACHE WRITE DISABLED FOR SECURITY CONTROL
        debug!("Cache write skipped due to security settings");
    }

    Ok(Json(card_response))
}

const RANKING_AUTHORITY_UNAVAILABLE_MESSAGE: &str = "Ranking access authority unavailable";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MarketRankingAccess {
    rank_offset: i32,
    rankings_limit: i32,
}

async fn resolve_market_ranking_access(
    permission_service: &dyn WalletRankingOffsetQuery,
    wallet_address: Option<&str>,
) -> Result<MarketRankingAccess, AppError> {
    let Some(wallet) = wallet_address else {
        return Ok(MarketRankingAccess {
            rank_offset: epsx_contracts::constants::FREE_PLAN_RANKING_OFFSET,
            rankings_limit: epsx_contracts::constants::FREE_PLAN_RANKINGS_LIMIT,
        });
    };

    let rank_offset = permission_service
        .get_wallet_ranking_offset(wallet)
        .await
        .map(|offset| offset.value())
        .map_err(|_| {
            warn!("Analytics ranking offset authority unavailable");
            AppError::new(
                ErrorKind::ServiceUnavailable,
                RANKING_AUTHORITY_UNAVAILABLE_MESSAGE,
            )
        })?;
    let rankings_limit = permission_service
        .get_wallet_rankings_limit(wallet)
        .await
        .map_err(|_| {
            warn!("Analytics ranking limit authority unavailable");
            AppError::new(
                ErrorKind::ServiceUnavailable,
                RANKING_AUTHORITY_UNAVAILABLE_MESSAGE,
            )
        })?;
    if rankings_limit != -1 && !(1..=10_000).contains(&rankings_limit) {
        warn!("Analytics ranking limit authority returned invalid data");
        return Err(AppError::new(
            ErrorKind::ServiceUnavailable,
            RANKING_AUTHORITY_UNAVAILABLE_MESSAGE,
        ));
    }

    Ok(MarketRankingAccess {
        rank_offset,
        rankings_limit,
    })
}

#[derive(Debug, PartialEq)]
struct PreparedMarketRankingsQuery {
    page: i32,
    page_size: i32,
    rank_start: i32,
    request: MarketRankingsRequest,
}

fn prepare_market_rankings_request(
    params: &EPSRankingQueryParams,
    rank_offset: i32,
    rankings_limit: i32,
    is_authenticated: bool,
) -> Result<PreparedMarketRankingsQuery, AppError> {
    let transport_cap = if is_authenticated { 100 } else { 10 };
    let entitlement_cap = if rankings_limit == -1 {
        transport_cap
    } else {
        rankings_limit.clamp(1, transport_cap)
    };
    let page_size = params
        .limit
        .unwrap_or(10)
        .clamp(1, transport_cap)
        .min(entitlement_cap);
    let page = params.page.unwrap_or(1).max(1);
    let sort_by = normalize_market_rankings_sort(params.sort_by.as_deref())?;

    // Catalog `ranking_offset` is the number of leading ranks hidden from the
    // wallet: 5 means the first visible result is rank 6, while 0 means rank 1.
    let rank_start = rank_offset.max(0);
    let page_index = page
        .checked_sub(1)
        .ok_or_else(rankings_pagination_overflow)?;
    let page_skip = page_index
        .checked_mul(page_size)
        .ok_or_else(rankings_pagination_overflow)?;
    if rankings_limit != -1 && page_skip >= rankings_limit {
        return Err(AppError::validation_error(
            "Analytics rankings page exceeds plan access",
        ));
    }
    let skip = rank_start
        .checked_add(page_skip)
        .ok_or_else(rankings_pagination_overflow)?;
    let request_limit = if rankings_limit == -1 {
        page_size
    } else {
        page_size.min(rankings_limit.saturating_sub(page_skip))
    };

    Ok(PreparedMarketRankingsQuery {
        page,
        page_size,
        rank_start,
        request: MarketRankingsRequest {
            skip,
            limit: request_limit,
            country: params.country.clone(),
            sector: params.sector.clone(),
            sort_by: Some(sort_by),
        },
    })
}

fn normalize_market_rankings_sort(sort_by: Option<&str>) -> Result<String, AppError> {
    let normalized = sort_by.unwrap_or("eps_growth").trim().to_ascii_lowercase();

    match normalized.as_str() {
        "qoq_growth" | "growth_factor" | "ranking_position" | "eps_growth" => {
            Ok("eps_growth".to_string())
        }
        "current_eps" | "market_cap" | "volume" | "price" | "symbol" | "name" => Ok(normalized),
        _ => Err(AppError::validation_error(
            "Unsupported analytics rankings sort field",
        )),
    }
}

fn rankings_pagination_overflow() -> AppError {
    AppError::validation_error("Analytics rankings pagination exceeds supported range")
}

fn accessible_pagination(
    provider_total: i32,
    rank_start: i32,
    rankings_limit: i32,
    page: i32,
    limit: i32,
) -> (i32, i32, bool, bool) {
    let provider_accessible = provider_total.saturating_sub(rank_start).max(0);
    let accessible_total = if rankings_limit == -1 {
        provider_accessible
    } else {
        provider_accessible.min(rankings_limit.max(0))
    };
    let total_pages =
        ((i64::from(accessible_total) + i64::from(limit) - 1) / i64::from(limit)) as i32;

    (accessible_total, total_pages, page < total_pages, page > 1)
}

async fn fetch_market_rankings(
    provider: &dyn MarketRankingsProviderPort,
    request: MarketRankingsRequest,
) -> Result<MarketRankingsPage, AppError> {
    provider
        .fetch_rankings(request)
        .await
        .map_err(sanitize_market_rankings_provider_error)
}

fn sanitize_market_rankings_provider_error(error: AppError) -> AppError {
    let kind = match error.kind {
        ErrorKind::ServiceUnavailable => ErrorKind::ServiceUnavailable,
        ErrorKind::TimeoutError => ErrorKind::TimeoutError,
        ErrorKind::RateLimitExceeded => ErrorKind::RateLimitExceeded,
        _ => ErrorKind::ExternalServiceError,
    };

    AppError::new(kind, "Market rankings provider request failed")
}

fn map_market_rankings_to_cards(
    screening_results: Vec<StockScreeningResult>,
    skip: i32,
) -> Vec<SymbolCardData> {
    let rankings_with_quarterly: Vec<(EPSRanking, StockScreeningResult)> = screening_results
        .into_iter()
        .map(|result| {
            let ranking = convert_screening_result_to_eps_ranking(result.clone());
            (ranking, result)
        })
        .collect();

    let unified_rankings: Vec<UnifiedRankingItem> = rankings_with_quarterly
        .iter()
        .map(|(ranking, _)| ranking.clone())
        .enumerate()
        .map(|(index, ranking)| {
            transform_ranking_to_unified_format(ranking, index + (skip as usize) + 1)
        })
        .collect();

    unified_rankings
        .into_iter()
        .enumerate()
        .map(|(index, unified_ranking)| {
            let mut card_data = transform_unified_to_card_format(&unified_ranking);

            if let Some((_, screening_result)) = rankings_with_quarterly.get(index) {
                card_data.eps_quarterly =
                    super::transform::transform_stock_screening_to_quarterly_data(screening_result);
            }

            card_data
        })
        .collect()
}

/// Generate a cache key from query parameters and the resolved plan inventory.
pub fn generate_cache_key(
    params: &EPSRankingQueryParams,
    rank_offset: i32,
    rankings_limit: i32,
) -> String {
    let mut hasher = DefaultHasher::new();

    // Hash rank_offset so different plan tiers get separate caches
    rank_offset.hash(&mut hasher);
    rankings_limit.hash(&mut hasher);

    // Hash relevant parameters
    params.country.hash(&mut hasher);
    params.sector.hash(&mut hasher);
    params.sort_by.hash(&mut hasher);
    params.page.unwrap_or(1).hash(&mut hasher);
    params.limit.unwrap_or(10).hash(&mut hasher);

    // Handle f64 fields by converting to strings (to avoid NaN hash issues)
    if let Some(min_eps) = params.min_eps {
        min_eps.to_string().hash(&mut hasher);
    }
    if let Some(min_growth) = params.min_growth {
        min_growth.to_string().hash(&mut hasher);
    }

    let hash = hasher.finish();
    format!("analytics:rankings:{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epsx_contracts::value_objects::ranking_offset::RankingOffset;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    fn a2_5_params(
        page: Option<i32>,
        limit: Option<i32>,
        sort_by: Option<&str>,
    ) -> EPSRankingQueryParams {
        EPSRankingQueryParams {
            page,
            limit,
            country: Some("america".to_string()),
            sector: Some("Technology".to_string()),
            sort_by: sort_by.map(str::to_string),
            min_eps: None,
            min_growth: None,
        }
    }

    struct A2_5FailingProvider {
        calls: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl MarketRankingsProviderPort for A2_5FailingProvider {
        async fn fetch_rankings(
            &self,
            _request: MarketRankingsRequest,
        ) -> Result<MarketRankingsPage, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err(AppError::external_service_error(
                "upstream secret response body and bearer token",
            ))
        }
    }

    struct A2_6Authority {
        calls: AtomicUsize,
        limit_calls: AtomicUsize,
        wallet: Mutex<Option<String>>,
        result: Result<RankingOffset, AppError>,
        limit_result: Result<i32, AppError>,
    }

    #[async_trait::async_trait]
    impl WalletRankingOffsetQuery for A2_6Authority {
        async fn get_wallet_ranking_offset(
            &self,
            wallet_address: &str,
        ) -> Result<RankingOffset, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            *self.wallet.lock().expect("wallet recorder lock") = Some(wallet_address.to_string());
            self.result.clone()
        }

        async fn get_wallet_rankings_limit(&self, _wallet_address: &str) -> Result<i32, AppError> {
            self.limit_calls.fetch_add(1, Ordering::SeqCst);
            self.limit_result.clone()
        }
    }

    struct A2_6RecordingProvider {
        calls: AtomicUsize,
        requests: Mutex<Vec<MarketRankingsRequest>>,
        total: i32,
    }

    #[async_trait::async_trait]
    impl MarketRankingsProviderPort for A2_6RecordingProvider {
        async fn fetch_rankings(
            &self,
            request: MarketRankingsRequest,
        ) -> Result<MarketRankingsPage, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.requests
                .lock()
                .expect("provider request recorder lock")
                .push(request);
            Ok(MarketRankingsPage {
                items: Vec::new(),
                total: self.total,
            })
        }
    }

    async fn call_a2_6_handler(
        params: EPSRankingQueryParams,
        authority: Arc<A2_6Authority>,
        provider: Arc<A2_6RecordingProvider>,
        wallet: Option<&str>,
    ) -> Result<Json<CardDashboardResponse>, AppError> {
        let cache: Arc<dyn Cache> = Arc::new(crate::infrastructure::cache::MemoryCache::new());
        let permission_service: Arc<dyn WalletRankingOffsetQuery> = authority;
        let rankings_provider: Arc<dyn MarketRankingsProviderPort> = provider;
        let analytics_wallet_ext =
            wallet.map(|wallet| Extension(AnalyticsWalletContext::new(wallet.to_string())));

        get_unified_analytics_rankings_cached(
            Query(params),
            Extension(cache),
            Extension(permission_service),
            Extension(rankings_provider),
            None,
            analytics_wallet_ext,
        )
        .await
    }

    #[test]
    fn test_cache_key_generation() {
        let params = EPSRankingQueryParams {
            page: Some(1),
            limit: Some(10),
            country: Some("america".to_string()),
            sector: None,
            sort_by: None,
            min_eps: None,
            min_growth: None,
        };

        let cache_key = generate_cache_key(&params, 100, 5);
        assert!(cache_key.starts_with("analytics:rankings:"));
        assert!(cache_key.len() > 20); // Should be a hex hash

        // Same params + offset should generate same key
        let cache_key2 = generate_cache_key(&params, 100, 5);
        assert_eq!(cache_key, cache_key2);

        // Different offset should generate different key
        let cache_key3 = generate_cache_key(&params, 0, -1);
        assert_ne!(cache_key, cache_key3);

        // Plans with the same starting rank but different inventory must
        // never share a cached entitlement response.
        let different_inventory = generate_cache_key(&params, 100, 25);
        assert_ne!(cache_key, different_inventory);
    }

    #[test]
    fn a2_5_anonymous_limit_is_capped_by_free_entitlement() {
        let prepared =
            prepare_market_rankings_request(&a2_5_params(Some(1), Some(100), None), 100, 5, false)
                .expect("anonymous request should be valid");

        assert_eq!(prepared.page, 1);
        assert_eq!(prepared.request.limit, 5);
        assert_eq!(prepared.request.skip, 100);
    }

    #[test]
    fn a2_5_authenticated_limit_is_capped_at_one_hundred() {
        let prepared =
            prepare_market_rankings_request(&a2_5_params(Some(1), Some(1_000), None), 1, -1, true)
                .expect("authenticated request should be valid");

        assert_eq!(prepared.request.limit, 100);
        assert_eq!(prepared.request.skip, 1);
    }

    #[test]
    fn a2_5_plan_inventory_caps_results_and_rejects_later_pages() {
        let first_page =
            prepare_market_rankings_request(&a2_5_params(Some(1), Some(10), None), 5, 5, true)
                .expect("first entitlement page should be valid");
        assert_eq!(first_page.page_size, 5);
        assert_eq!(first_page.request.skip, 5);
        assert_eq!(first_page.request.limit, 5);

        let error =
            prepare_market_rankings_request(&a2_5_params(Some(2), Some(10), None), 5, 5, true)
                .expect_err("a second page would exceed the five-row plan inventory");
        assert_eq!(error.kind, ErrorKind::ValidationError);
        assert_eq!(error.message, "Analytics rankings page exceeds plan access");
    }

    #[test]
    fn a2_5_partial_final_page_never_exposes_rows_past_plan_inventory() {
        let final_page =
            prepare_market_rankings_request(&a2_5_params(Some(3), Some(10), None), 1, 25, true)
                .expect("the partial final page should be valid");
        assert_eq!(final_page.request.skip, 21);
        assert_eq!(final_page.request.limit, 5);
        assert_eq!(final_page.page_size, 10);
    }

    #[tokio::test]
    async fn a2_6_anonymous_bypasses_authority_and_keeps_free_input() {
        let authority = Arc::new(A2_6Authority {
            calls: AtomicUsize::new(0),
            limit_calls: AtomicUsize::new(0),
            wallet: Mutex::new(None),
            result: Err(AppError::database_error(
                "authority details must be unreachable for anonymous input",
            )),
            limit_result: Err(AppError::database_error(
                "authority details must be unreachable for anonymous input",
            )),
        });
        let provider = Arc::new(A2_6RecordingProvider {
            calls: AtomicUsize::new(0),
            requests: Mutex::new(Vec::new()),
            total: 150,
        });

        let Json(response) = call_a2_6_handler(
            a2_5_params(Some(1), Some(100), None),
            authority.clone(),
            provider.clone(),
            None,
        )
        .await
        .expect("anonymous rankings should remain available");

        assert_eq!(authority.calls.load(Ordering::SeqCst), 0);
        assert_eq!(authority.limit_calls.load(Ordering::SeqCst), 0);
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        let requests = provider.requests.lock().expect("provider requests lock");
        assert_eq!(
            requests.as_slice(),
            &[MarketRankingsRequest {
                skip: 100,
                limit: 5,
                country: Some("america".to_string()),
                sector: Some("Technology".to_string()),
                sort_by: Some("eps_growth".to_string()),
            }]
        );
        let access = response.access_info.expect("anonymous access info");
        assert_eq!(access.min_accessible_rank, 101);
        assert_eq!(access.locked_ranks_count, 100);
    }

    #[tokio::test]
    async fn a2_6_authenticated_authoritative_offset_proceeds() {
        let authority = Arc::new(A2_6Authority {
            calls: AtomicUsize::new(0),
            limit_calls: AtomicUsize::new(0),
            wallet: Mutex::new(None),
            result: Ok(RankingOffset::new(5).expect("valid offset")),
            limit_result: Ok(-1),
        });
        let provider = Arc::new(A2_6RecordingProvider {
            calls: AtomicUsize::new(0),
            requests: Mutex::new(Vec::new()),
            total: 250,
        });

        let Json(response) = call_a2_6_handler(
            a2_5_params(Some(2), Some(1_000), None),
            authority.clone(),
            provider.clone(),
            Some("0xAbC"),
        )
        .await
        .expect("authoritative offset should proceed");

        assert_eq!(authority.calls.load(Ordering::SeqCst), 1);
        assert_eq!(authority.limit_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            authority
                .wallet
                .lock()
                .expect("authority wallet lock")
                .as_deref(),
            Some("0xabc")
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        let requests = provider.requests.lock().expect("provider requests lock");
        assert_eq!(requests[0].skip, 105);
        assert_eq!(requests[0].limit, 100);
        let access = response.access_info.expect("authenticated access info");
        assert_eq!(access.min_accessible_rank, 6);
        assert_eq!(access.locked_ranks_count, 5);
    }

    #[tokio::test]
    async fn a2_6_authenticated_no_plan_is_explicit_free_success() {
        let authority = Arc::new(A2_6Authority {
            calls: AtomicUsize::new(0),
            limit_calls: AtomicUsize::new(0),
            wallet: Mutex::new(None),
            result: Ok(RankingOffset::free_plan()),
            limit_result: Ok(epsx_contracts::constants::FREE_PLAN_RANKINGS_LIMIT),
        });
        let provider = Arc::new(A2_6RecordingProvider {
            calls: AtomicUsize::new(0),
            requests: Mutex::new(Vec::new()),
            total: 150,
        });

        let Json(response) = call_a2_6_handler(
            a2_5_params(Some(1), Some(10), None),
            authority.clone(),
            provider.clone(),
            Some("0xNoPlan"),
        )
        .await
        .expect("explicit free-plan authority success should proceed");

        assert_eq!(authority.calls.load(Ordering::SeqCst), 1);
        assert_eq!(authority.limit_calls.load(Ordering::SeqCst), 1);
        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            provider.requests.lock().expect("provider requests lock")[0].skip,
            100
        );
        assert_eq!(
            provider.requests.lock().expect("provider requests lock")[0].limit,
            5
        );
        let access = response.access_info.expect("free-plan access info");
        assert_eq!(access.min_accessible_rank, 101);
        assert_eq!(access.locked_ranks_count, 100);
    }

    #[tokio::test]
    async fn a2_6_authenticated_authority_errors_stop_before_provider_work() {
        let failures = [
            AppError::not_found("sensitive missing-plan authority detail"),
            AppError::database_error("sensitive authority database detail"),
            AppError::new(
                ErrorKind::TimeoutError,
                "sensitive authority timeout detail",
            ),
        ];

        for failure in failures {
            let authority = Arc::new(A2_6Authority {
                calls: AtomicUsize::new(0),
                limit_calls: AtomicUsize::new(0),
                wallet: Mutex::new(None),
                result: Err(failure),
                limit_result: Ok(-1),
            });
            let provider = Arc::new(A2_6RecordingProvider {
                calls: AtomicUsize::new(0),
                requests: Mutex::new(Vec::new()),
                total: 150,
            });

            let error = call_a2_6_handler(
                a2_5_params(Some(1), Some(10), None),
                authority.clone(),
                provider.clone(),
                Some("0xAuthenticated"),
            )
            .await
            .expect_err("authority failure must fail closed");

            assert_eq!(authority.calls.load(Ordering::SeqCst), 1);
            assert_eq!(authority.limit_calls.load(Ordering::SeqCst), 0);
            assert_eq!(error.kind, ErrorKind::ServiceUnavailable);
            assert_eq!(error.message, RANKING_AUTHORITY_UNAVAILABLE_MESSAGE);
            assert!(!error.message.contains("sensitive"));
            assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
            assert!(provider
                .requests
                .lock()
                .expect("provider requests lock")
                .is_empty());
        }
    }

    #[tokio::test]
    async fn a2_5_checked_pagination_overflow_fails_before_provider_call() {
        let provider = A2_5FailingProvider {
            calls: AtomicUsize::new(0),
        };
        let prepared = prepare_market_rankings_request(
            &a2_5_params(Some(i32::MAX), Some(100), None),
            100,
            -1,
            true,
        );

        let error = match prepared {
            Ok(value) => fetch_market_rankings(&provider, value.request)
                .await
                .expect_err("overflow must be rejected before this branch"),
            Err(error) => error,
        };

        assert_eq!(error.kind, ErrorKind::ValidationError);
        assert_eq!(
            error.message,
            "Analytics rankings pagination exceeds supported range"
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a2_5_sort_aliases_normalize_and_supported_fields_are_preserved() {
        for alias in [
            "qoq_growth",
            "growth_factor",
            "ranking_position",
            "eps_growth",
        ] {
            assert_eq!(
                normalize_market_rankings_sort(Some(alias)).expect("alias should be supported"),
                "eps_growth"
            );
        }
        assert_eq!(
            normalize_market_rankings_sort(None).expect("default should be supported"),
            "eps_growth"
        );

        for field in [
            "current_eps",
            "market_cap",
            "volume",
            "price",
            "symbol",
            "name",
        ] {
            assert_eq!(
                normalize_market_rankings_sort(Some(field)).expect("field should be supported"),
                field
            );
        }
    }

    #[test]
    fn a2_5_unknown_sort_is_rejected_before_provider_call() {
        let provider = A2_5FailingProvider {
            calls: AtomicUsize::new(0),
        };
        let error = prepare_market_rankings_request(
            &a2_5_params(Some(1), Some(10), Some("unsupported")),
            100,
            5,
            false,
        )
        .expect_err("unknown sort must be rejected");

        assert_eq!(error.kind, ErrorKind::ValidationError);
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a2_5_accessible_pagination_excludes_locked_ranks() {
        let (total, total_pages, has_next, has_prev) = accessible_pagination(105, 100, 5, 1, 5);

        assert_eq!(total, 5);
        assert_eq!(total_pages, 1);
        assert!(!has_next);
        assert!(!has_prev);

        for provider_total in [99, 25, 0] {
            let (total, total_pages, has_next, has_prev) =
                accessible_pagination(provider_total, 100, 5, 1, 5);
            assert_eq!(total, 0);
            assert_eq!(total_pages, 0);
            assert!(!has_next);
            assert!(!has_prev);
        }
    }

    #[tokio::test]
    async fn a2_5_provider_error_is_sanitized() {
        let provider = A2_5FailingProvider {
            calls: AtomicUsize::new(0),
        };
        let request =
            prepare_market_rankings_request(&a2_5_params(Some(1), Some(10), None), 100, 5, false)
                .expect("request should be valid")
                .request;

        let error = fetch_market_rankings(&provider, request)
            .await
            .expect_err("fake provider should fail");

        assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
        assert_eq!(error.kind, ErrorKind::ExternalServiceError);
        assert_eq!(error.message, "Market rankings provider request failed");
        assert!(!error.message.contains("secret"));
        assert!(!error.message.contains("bearer"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a2_5_successful_mapping_preserves_quarterly_dto() {
        let mut result =
            StockScreeningResult::new("A2FIVE".to_string(), "A2.5 Test Company".to_string(), 42.5);
        result.eps_q_minus_2 = Some(1.0);
        result.eps_q_minus_1 = Some(1.5);
        result.eps_q_current = Some(2.0);
        result.current_eps = Some(2.0);

        let cards = map_market_rankings_to_cards(vec![result], 99);

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].rank, 100);
        assert_eq!(cards[0].symbol, "A2FIVE");
        let quarterly = cards[0]
            .eps_quarterly
            .as_ref()
            .expect("quarterly DTO should be present");
        assert_eq!(quarterly.eps_q_minus_2, Some(1.0));
        assert_eq!(quarterly.eps_q_minus_1, Some(1.5));
        assert_eq!(quarterly.eps_q_current, Some(2.0));
    }

    // wave12(track-b) option b: the dead route decision test. The
    // 'get_cache_stats' and 'force_cache_refresh' HTTP handlers were
    // deleted (audit-analytics §7d, ROADMAP §4 item 5). They are
    // intentionally NOT exported. This compile-time check guards
    // against silent reintroduction.
    //
    // If a future change re-adds them to the public API, this test
    // (and the matching sentinel in `web/routes/unified_router.rs`
    // and the 3 openapi_*.rs files) will need to be revisited — at
    // which point the author must choose option (a) wiring or keep
    // option (b) and accept the dead code.
    #[allow(dead_code)]
    const _WAVE12_DEAD_ROUTE_OPTION_B: () = ();
}
