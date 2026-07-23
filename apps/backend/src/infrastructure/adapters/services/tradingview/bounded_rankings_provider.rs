use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use epsx_contracts::errors::{AppError, ErrorKind};
use tokio::{sync::Semaphore, time};

use super::types::constants::{DEFAULT_TIMEOUT_SECONDS, MAX_CONCURRENT_REQUESTS, MAX_PAGE_SIZE};
use crate::domain::market_analytics::repository_ports::{
    MarketRankingsPage, MarketRankingsProviderPort, MarketRankingsRequest,
};

const MAX_TOTAL_ATTEMPTS: usize = 3;
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);

const INVALID_PAGINATION_MESSAGE: &str = "Invalid market rankings pagination";
const PROVIDER_FAILURE_MESSAGE: &str = "Market rankings provider request failed";
const PROVIDER_UNAVAILABLE_MESSAGE: &str = "Market rankings provider is unavailable";
const PROVIDER_TIMEOUT_MESSAGE: &str = "Market rankings provider request timed out";

/// Applies the production resource and retry policy to a rankings provider.
pub struct BoundedMarketRankingsProvider {
    inner: Arc<dyn MarketRankingsProviderPort>,
    semaphore: Arc<Semaphore>,
    max_attempts: usize,
    total_deadline: Duration,
    initial_backoff: Duration,
}

impl BoundedMarketRankingsProvider {
    pub fn new(inner: Arc<dyn MarketRankingsProviderPort>) -> Self {
        Self {
            inner,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
            max_attempts: MAX_TOTAL_ATTEMPTS,
            total_deadline: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            initial_backoff: INITIAL_BACKOFF,
        }
    }

    #[cfg(test)]
    fn with_policy(
        inner: Arc<dyn MarketRankingsProviderPort>,
        total_deadline: Duration,
        initial_backoff: Duration,
    ) -> Self {
        Self {
            inner,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS)),
            max_attempts: MAX_TOTAL_ATTEMPTS,
            total_deadline,
            initial_backoff,
        }
    }

    fn validate_request(request: &MarketRankingsRequest) -> Result<(), AppError> {
        if request.skip < 0
            || !(1..=MAX_PAGE_SIZE).contains(&request.limit)
            || request.skip.checked_add(request.limit).is_none()
        {
            return Err(AppError::validation_error(INVALID_PAGINATION_MESSAGE));
        }

        Ok(())
    }

    async fn fetch_with_retries(
        &self,
        request: MarketRankingsRequest,
    ) -> Result<MarketRankingsPage, AppError> {
        let mut attempt = 0;

        loop {
            attempt += 1;
            match self.inner.fetch_rankings(request.clone()).await {
                Ok(page) => {
                    validate_provider_page(&page, &request)?;
                    return Ok(page);
                }
                Err(error) if error.is_retryable() && attempt < self.max_attempts => {
                    let multiplier = 1_u32 << (attempt - 1);
                    time::sleep(self.initial_backoff.saturating_mul(multiplier)).await;
                }
                Err(error) => return Err(sanitize_provider_error(error)),
            }
        }
    }
}

fn validate_provider_page(
    page: &MarketRankingsPage,
    request: &MarketRankingsRequest,
) -> Result<(), AppError> {
    let page_len = i32::try_from(page.items.len())
        .map_err(|_| AppError::external_service_error(PROVIDER_FAILURE_MESSAGE))?;
    let minimum_total = request
        .skip
        .checked_add(page_len)
        .ok_or_else(|| AppError::external_service_error(PROVIDER_FAILURE_MESSAGE))?;

    if page.total < 0
        || page.items.len() > request.limit as usize
        || (!page.items.is_empty() && page.total < minimum_total)
    {
        return Err(AppError::external_service_error(PROVIDER_FAILURE_MESSAGE));
    }

    Ok(())
}

#[async_trait]
impl MarketRankingsProviderPort for BoundedMarketRankingsProvider {
    async fn fetch_rankings(
        &self,
        request: MarketRankingsRequest,
    ) -> Result<MarketRankingsPage, AppError> {
        Self::validate_request(&request)?;

        let _permit = Arc::clone(&self.semaphore)
            .try_acquire_owned()
            .map_err(|_| {
                AppError::new(ErrorKind::ServiceUnavailable, PROVIDER_UNAVAILABLE_MESSAGE)
            })?;

        match time::timeout(self.total_deadline, self.fetch_with_retries(request)).await {
            Ok(result) => result,
            Err(_) => Err(AppError::new(
                ErrorKind::TimeoutError,
                PROVIDER_TIMEOUT_MESSAGE,
            )),
        }
    }
}

fn sanitize_provider_error(error: AppError) -> AppError {
    match error.kind {
        ErrorKind::TimeoutError => AppError::new(ErrorKind::TimeoutError, PROVIDER_TIMEOUT_MESSAGE),
        ErrorKind::NetworkError | ErrorKind::ServiceUnavailable | ErrorKind::ResourceExhausted => {
            AppError::new(ErrorKind::ServiceUnavailable, PROVIDER_UNAVAILABLE_MESSAGE)
        }
        _ => AppError::external_service_error(PROVIDER_FAILURE_MESSAGE),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        future::pending,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Mutex,
        },
    };

    use tokio::{sync::Semaphore as TokioSemaphore, task::JoinSet};

    use super::*;

    enum FakeOutcome {
        Page,
        Error(ErrorKind, &'static str),
        Pending,
    }

    struct SequenceProvider {
        calls: AtomicUsize,
        outcomes: Mutex<VecDeque<FakeOutcome>>,
    }

    impl SequenceProvider {
        fn new(outcomes: impl IntoIterator<Item = FakeOutcome>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                outcomes: Mutex::new(outcomes.into_iter().collect()),
            }
        }
    }

    #[async_trait]
    impl MarketRankingsProviderPort for SequenceProvider {
        async fn fetch_rankings(
            &self,
            _request: MarketRankingsRequest,
        ) -> Result<MarketRankingsPage, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let outcome = self.outcomes.lock().unwrap().pop_front().unwrap();

            match outcome {
                FakeOutcome::Page => Ok(empty_page()),
                FakeOutcome::Error(kind, message) => Err(AppError::new(kind, message)),
                FakeOutcome::Pending => pending().await,
            }
        }
    }

    struct BlockingProvider {
        calls: AtomicUsize,
        active: AtomicUsize,
        peak: AtomicUsize,
        entered: TokioSemaphore,
        release: TokioSemaphore,
    }

    impl BlockingProvider {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                active: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                entered: TokioSemaphore::new(0),
                release: TokioSemaphore::new(0),
            }
        }

        fn record_peak(&self, active: usize) {
            let mut peak = self.peak.load(Ordering::SeqCst);
            while active > peak {
                match self
                    .peak
                    .compare_exchange(peak, active, Ordering::SeqCst, Ordering::SeqCst)
                {
                    Ok(_) => break,
                    Err(current) => peak = current,
                }
            }
        }
    }

    #[async_trait]
    impl MarketRankingsProviderPort for BlockingProvider {
        async fn fetch_rankings(
            &self,
            _request: MarketRankingsRequest,
        ) -> Result<MarketRankingsPage, AppError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.record_peak(active);
            self.entered.add_permits(1);
            self.release.acquire().await.unwrap().forget();
            self.active.fetch_sub(1, Ordering::SeqCst);
            Ok(empty_page())
        }
    }

    fn request(skip: i32, limit: i32) -> MarketRankingsRequest {
        MarketRankingsRequest {
            skip,
            limit,
            country: None,
            sector: None,
            sort_by: None,
        }
    }

    fn empty_page() -> MarketRankingsPage {
        MarketRankingsPage {
            items: Vec::new(),
            total: 0,
        }
    }

    #[tokio::test]
    async fn a2_5_invalid_pagination_never_calls_inner_provider() {
        for invalid in [
            request(-1, 10),
            request(0, 0),
            request(0, 101),
            request(i32::MAX, 1),
        ] {
            let inner = Arc::new(SequenceProvider::new([FakeOutcome::Page]));
            let provider = BoundedMarketRankingsProvider::new(inner.clone());

            let error = provider.fetch_rankings(invalid).await.unwrap_err();

            assert_eq!(error.http_status(), 400);
            assert_eq!(error.message, INVALID_PAGINATION_MESSAGE);
            assert_eq!(inner.calls.load(Ordering::SeqCst), 0);
        }
    }

    #[tokio::test]
    async fn a2_5_transient_failures_succeed_within_three_total_attempts() {
        let inner = Arc::new(SequenceProvider::new([
            FakeOutcome::Error(ErrorKind::NetworkError, "provider secret one"),
            FakeOutcome::Error(ErrorKind::NetworkError, "provider secret two"),
            FakeOutcome::Page,
        ]));
        let provider = BoundedMarketRankingsProvider::with_policy(
            inner.clone(),
            Duration::from_secs(1),
            Duration::from_millis(1),
        );

        provider.fetch_rankings(request(0, 10)).await.unwrap();

        assert_eq!(inner.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn a2_5_transient_failures_stop_after_three_total_attempts() {
        let inner = Arc::new(SequenceProvider::new([
            FakeOutcome::Error(ErrorKind::NetworkError, "provider secret one"),
            FakeOutcome::Error(ErrorKind::NetworkError, "provider secret two"),
            FakeOutcome::Error(ErrorKind::NetworkError, "provider secret three"),
        ]));
        let provider = BoundedMarketRankingsProvider::with_policy(
            inner.clone(),
            Duration::from_secs(1),
            Duration::from_millis(1),
        );

        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();

        assert_eq!(inner.calls.load(Ordering::SeqCst), MAX_TOTAL_ATTEMPTS);
        assert_eq!(error.http_status(), 503);
        assert_eq!(error.message, PROVIDER_UNAVAILABLE_MESSAGE);
        assert!(!error.message.contains("secret"));
    }

    #[tokio::test]
    async fn a2_5_permanent_failure_is_not_retried_and_is_sanitized() {
        let inner = Arc::new(SequenceProvider::new([FakeOutcome::Error(
            ErrorKind::ExternalServiceError,
            "upstream body must not escape",
        )]));
        let provider = BoundedMarketRankingsProvider::new(inner.clone());

        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();

        assert_eq!(inner.calls.load(Ordering::SeqCst), 1);
        assert_eq!(error.http_status(), 502);
        assert_eq!(error.message, PROVIDER_FAILURE_MESSAGE);
    }

    #[tokio::test]
    async fn a2_5_invalid_provider_page_is_rejected() {
        struct InvalidPageProvider {
            page: MarketRankingsPage,
        }

        #[async_trait]
        impl MarketRankingsProviderPort for InvalidPageProvider {
            async fn fetch_rankings(
                &self,
                _request: MarketRankingsRequest,
            ) -> Result<MarketRankingsPage, AppError> {
                Ok(self.page.clone())
            }
        }

        let oversized = MarketRankingsPage {
            items: (0..11)
                .map(|index| {
                    crate::domain::shared_kernel::entities::market_data::StockScreeningResult::new(
                        format!("A2{index}"),
                        format!("A2.5 Company {index}"),
                        1.0,
                    )
                })
                .collect(),
            total: 11,
        };
        let provider =
            BoundedMarketRankingsProvider::new(Arc::new(InvalidPageProvider { page: oversized }));
        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();
        assert_eq!(error.http_status(), 502);
        assert_eq!(error.message, PROVIDER_FAILURE_MESSAGE);

        let provider = BoundedMarketRankingsProvider::new(Arc::new(InvalidPageProvider {
            page: MarketRankingsPage {
                items: Vec::new(),
                total: -1,
            },
        }));
        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();
        assert_eq!(error.http_status(), 502);
        assert_eq!(error.message, PROVIDER_FAILURE_MESSAGE);

        let provider = BoundedMarketRankingsProvider::new(Arc::new(InvalidPageProvider {
            page: MarketRankingsPage {
                items: vec![
                    crate::domain::shared_kernel::entities::market_data::StockScreeningResult::new(
                        "A25".to_string(),
                        "A2.5 Company".to_string(),
                        1.0,
                    ),
                ],
                total: 99,
            },
        }));
        let error = provider.fetch_rankings(request(99, 10)).await.unwrap_err();
        assert_eq!(error.http_status(), 502);
        assert_eq!(error.message, PROVIDER_FAILURE_MESSAGE);
    }

    #[tokio::test]
    async fn a2_5_timeout_is_sanitized_and_releases_the_permit() {
        let inner = Arc::new(SequenceProvider::new([
            FakeOutcome::Pending,
            FakeOutcome::Page,
        ]));
        let provider = BoundedMarketRankingsProvider::with_policy(
            inner.clone(),
            Duration::from_millis(10),
            Duration::from_millis(1),
        );

        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();
        assert_eq!(error.http_status(), 504);
        assert_eq!(error.message, PROVIDER_TIMEOUT_MESSAGE);

        provider.fetch_rankings(request(0, 10)).await.unwrap();
        assert_eq!(inner.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a2_5_shared_concurrency_peaks_at_five_and_saturation_fails_fast() {
        let inner = Arc::new(BlockingProvider::new());
        let provider: Arc<dyn MarketRankingsProviderPort> =
            Arc::new(BoundedMarketRankingsProvider::with_policy(
                inner.clone(),
                Duration::from_secs(2),
                Duration::from_millis(1),
            ));
        let mut tasks = JoinSet::new();

        for _ in 0..MAX_CONCURRENT_REQUESTS {
            let provider = provider.clone();
            tasks.spawn(async move { provider.fetch_rankings(request(0, 10)).await });
        }

        time::timeout(
            Duration::from_secs(1),
            inner.entered.acquire_many(MAX_CONCURRENT_REQUESTS as u32),
        )
        .await
        .unwrap()
        .unwrap()
        .forget();

        let error = provider.fetch_rankings(request(0, 10)).await.unwrap_err();
        assert_eq!(error.http_status(), 503);
        assert_eq!(error.message, PROVIDER_UNAVAILABLE_MESSAGE);
        assert_eq!(inner.calls.load(Ordering::SeqCst), MAX_CONCURRENT_REQUESTS);
        assert_eq!(inner.peak.load(Ordering::SeqCst), MAX_CONCURRENT_REQUESTS);

        inner.release.add_permits(MAX_CONCURRENT_REQUESTS);
        while let Some(result) = tasks.join_next().await {
            result.unwrap().unwrap();
        }
    }
}
