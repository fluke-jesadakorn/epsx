// Error recovery strategies and retry logic implementation

use std::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;

use crate::core::errors::{
  AppError,
  ErrorKind,
  ErrorLogger,
  ErrorContextBuilder,
};

/// Error recovery configuration
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
  pub max_retries: u32,
  pub initial_delay: Duration,
  pub max_delay: Duration,
  pub backoff_multiplier: f64,
  pub jitter: bool,
}

impl Default for RecoveryConfig {
  fn default() -> Self {
    Self {
      max_retries: 3,
      initial_delay: Duration::from_millis(100),
      max_delay: Duration::from_secs(30),
      backoff_multiplier: 2.0,
      jitter: true,
    }
  }
}

impl RecoveryConfig {
  pub fn with_max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  pub fn with_initial_delay(mut self, delay: Duration) -> Self {
    self.initial_delay = delay;
    self
  }

  pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
    self.backoff_multiplier = multiplier;
    self
  }

  pub fn no_jitter(mut self) -> Self {
    self.jitter = false;
    self
  }
}

/// Recovery strategy for different error types
#[async_trait]
pub trait RecoveryStrategy<T>: Send + Sync {
  async fn can_recover(&self, error: &AppError) -> bool;
  async fn recover(
    &self,
    error: &AppError,
    attempt: u32
  ) -> Result<Option<T>, AppError>;
  fn get_config(&self) -> RecoveryConfig;
}

/// Generic retry recovery strategy
pub struct RetryRecoveryStrategy<F, Fut, T>
  where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, AppError>> + Send,
    T: Send + 'static {
  operation: F,
  config: RecoveryConfig,
  operation_name: String,
}

impl<F, Fut, T> RetryRecoveryStrategy<F, Fut, T>
  where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, AppError>> + Send,
    T: Send + 'static
{
  pub fn new(operation: F, operation_name: String) -> Self {
    Self {
      operation,
      config: RecoveryConfig::default(),
      operation_name,
    }
  }

  pub fn with_config(mut self, config: RecoveryConfig) -> Self {
    self.config = config;
    self
  }
}

#[async_trait]
impl<F, Fut, T> RecoveryStrategy<T>
  for RetryRecoveryStrategy<F, Fut, T>
  where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<T, AppError>> + Send,
    T: Send + 'static
{
  async fn can_recover(&self, error: &AppError) -> bool {
    matches!(
      error.kind,
      ErrorKind::NetworkError |
        ErrorKind::ServiceUnavailable |
        ErrorKind::TimeoutError |
        ErrorKind::ResourceExhausted |
        ErrorKind::DatabaseError // For transient database errors
    )
  }

  async fn recover(
    &self,
    error: &AppError,
    attempt: u32
  ) -> Result<Option<T>, AppError> {
    if attempt >= self.config.max_retries {
      return Ok(None);
    }

    ErrorLogger::log_recovery_attempt(
      error,
      &format!("retry_attempt_{}", attempt + 1)
    );

    // Calculate delay with exponential backoff
    let delay = calculate_backoff_delay(&self.config, attempt);

    tracing::info!(
            operation = %self.operation_name,
            attempt = attempt + 1,
            max_retries = self.config.max_retries,
            delay_ms = delay.as_millis(),
            error_kind = %error.kind,
            "Retrying operation after error"
        );

    sleep(delay).await;

    match (self.operation)().await {
      Ok(result) => {
        tracing::info!(
                    operation = %self.operation_name,
                    attempt = attempt + 1,
                    "Operation succeeded after retry"
                );
        Ok(Some(result))
      }
      Err(retry_error) => {
        tracing::warn!(
                    operation = %self.operation_name,
                    attempt = attempt + 1,
                    error = %retry_error.message,
                    "Retry attempt failed"
                );
        Err(retry_error)
      }
    }
  }

  fn get_config(&self) -> RecoveryConfig {
    self.config.clone()
  }
}

/// Circuit breaker recovery strategy
pub struct CircuitBreakerRecovery {
  failure_threshold: u32,
  recovery_timeout: Duration,
  current_failures: std::sync::atomic::AtomicU32,
  last_failure_time: std::sync::atomic::AtomicU64,
  operation_name: String,
}

impl CircuitBreakerRecovery {
  pub fn new(
    operation_name: String,
    failure_threshold: u32,
    recovery_timeout: Duration
  ) -> Self {
    Self {
      failure_threshold,
      recovery_timeout,
      current_failures: std::sync::atomic::AtomicU32::new(0),
      last_failure_time: std::sync::atomic::AtomicU64::new(0),
      operation_name,
    }
  }

  pub fn is_circuit_open(&self) -> bool {
    let failures = self.current_failures.load(
      std::sync::atomic::Ordering::Acquire
    );
    if failures < self.failure_threshold {
      return false;
    }

    let last_failure = self.last_failure_time.load(
      std::sync::atomic::Ordering::Acquire
    );
    let now = std::time::SystemTime
      ::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    now - last_failure < self.recovery_timeout.as_secs()
  }

  pub fn record_success(&self) {
    self.current_failures.store(0, std::sync::atomic::Ordering::Release);
    tracing::info!(
            operation = %self.operation_name,
            "Circuit breaker reset after successful operation"
        );
  }

  pub fn record_failure(&self) {
    let failures =
      self.current_failures.fetch_add(1, std::sync::atomic::Ordering::AcqRel) +
      1;
    let now = std::time::SystemTime
      ::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    self.last_failure_time.store(now, std::sync::atomic::Ordering::Release);

    if failures >= self.failure_threshold {
      tracing::warn!(
                operation = %self.operation_name,
                failures = failures,
                threshold = self.failure_threshold,
                "Circuit breaker opened due to repeated failures"
            );
    }
  }
}

/// Fallback recovery strategy
pub struct FallbackRecovery<T> {
  fallback_value: T,
  operation_name: String,
}

impl<T> FallbackRecovery<T> where T: Clone + Send + Sync + 'static {
  pub fn new(fallback_value: T, operation_name: String) -> Self {
    Self {
      fallback_value,
      operation_name,
    }
  }
}

#[async_trait]
impl<T> RecoveryStrategy<T>
  for FallbackRecovery<T>
  where T: Clone + Send + Sync + 'static
{
  async fn can_recover(&self, _error: &AppError) -> bool {
    true // Fallback can recover from any error
  }

  async fn recover(
    &self,
    error: &AppError,
    _attempt: u32
  ) -> Result<Option<T>, AppError> {
    tracing::warn!(
            operation = %self.operation_name,
            error_kind = %error.kind,
            "Using fallback value due to error"
        );

    Ok(Some(self.fallback_value.clone()))
  }

  fn get_config(&self) -> RecoveryConfig {
    RecoveryConfig {
      max_retries: 1, // Fallback only needs one attempt
      ..Default::default()
    }
  }
}

/// Recovery orchestrator that combines multiple strategies
pub struct RecoveryOrchestrator<T> {
  strategies: Vec<Box<dyn RecoveryStrategy<T>>>,
}

impl<T> RecoveryOrchestrator<T> where T: Send + 'static {
  pub fn new() -> Self {
    Self {
      strategies: Vec::new(),
    }
  }

  pub fn add_strategy(
    mut self,
    strategy: Box<dyn RecoveryStrategy<T>>
  ) -> Self {
    self.strategies.push(strategy);
    self
  }

  pub async fn recover_from_error(
    &self,
    error: AppError
  ) -> Result<T, AppError> {
    let mut current_error = error;

    for strategy in &self.strategies {
      if !strategy.can_recover(&current_error).await {
        continue;
      }

      let config = strategy.get_config();
      let mut attempt = 0;

      while attempt < config.max_retries {
        match strategy.recover(&current_error, attempt).await {
          Ok(Some(result)) => {
            return Ok(result);
          }
          Ok(None) => {
            break;
          } // Strategy gave up
          Err(retry_error) => {
            current_error = retry_error;
            attempt += 1;
          }
        }
      }
    }

    // All strategies failed
    let final_error = AppError::new(
      ErrorKind::InternalError,
      format!(
        "All recovery strategies failed for error: {}",
        current_error.message
      )
    ).with_context(
      ErrorContextBuilder::new("error_recovery", "recovery_orchestrator")
        .metadata("original_error_kind", format!("{}", current_error.kind))
        .metadata("strategies_attempted", self.strategies.len().to_string())
        .build()
    );

    ErrorLogger::log_error(&final_error);
    Err(final_error)
  }
}

/// Calculate backoff delay with exponential backoff and optional jitter
fn calculate_backoff_delay(config: &RecoveryConfig, attempt: u32) -> Duration {
  let base_delay = config.initial_delay.as_millis() as f64;
  let exponential_delay =
    base_delay * config.backoff_multiplier.powi(attempt as i32);

  let mut delay = Duration::from_millis(exponential_delay as u64);

  // Cap at max delay
  if delay > config.max_delay {
    delay = config.max_delay;
  }

  // Add jitter if enabled
  if config.jitter {
    let jitter_range = (delay.as_millis() as f64) * 0.1; // 10% jitter
    let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
    let jittered_delay = ((delay.as_millis() as f64) + jitter) as u64;
    delay = Duration::from_millis(jittered_delay.max(0));
  }

  delay
}

/// Convenient macro for creating retry operations
#[macro_export]
macro_rules! with_retry {
  ($operation:expr, $operation_name:expr) => {
        with_retry!($operation, $operation_name, RecoveryConfig::default())
  };
  ($operation:expr, $operation_name:expr, $config:expr) => {
    {
        let strategy = RetryRecoveryStrategy::new($operation, $operation_name.to_string())
            .with_config($config);
        
        let orchestrator = RecoveryOrchestrator::new()
            .add_strategy(Box::new(strategy));
        
        // Execute the operation first, then try recovery if it fails
        match ($operation)().await {
            Ok(result) => Ok(result),
            Err(error) => orchestrator.recover_from_error(error).await,
        }
    }
  };
}
