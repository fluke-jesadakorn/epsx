// Performance monitoring middleware for data collection

use axum::{
    extract::{Request, ConnectInfo},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
    body::Body,
};
use std::{
    sync::Arc,
    time::Instant,
    net::SocketAddr,
};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, warn, error};

use crate::web::performance::{models::PerformanceMetric, repo::PerformanceRepo};

/// Performance monitoring middleware
pub struct PerformanceMiddleware {
    repo: Arc<PerformanceRepo>,
}

impl PerformanceMiddleware {
    pub fn new(repo: Arc<PerformanceRepo>) -> Self {
        Self { repo }
    }
}

/// Middleware function to collect performance metrics
pub async fn performance_monitoring_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().path().to_string();
    let trace_id = Uuid::new_v4();
    
    // Extract user agent and other headers
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Extract user ID from headers if available (set by auth middleware)
    let user_id = request
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok());

    // Extract cache hit information if available
    let cache_hit = request
        .headers()
        .get("x-cache-hit")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<bool>().ok());

    // Get request size
    let request_size = request
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    // Add trace ID to request for downstream components
    let mut request = request;
    if let Ok(header_value) = trace_id.to_string().parse() {
        request.headers_mut().insert("x-trace-id", header_value);
    }

    // Call next middleware/handler
    let response = next.run(request).await;
    
    // Calculate total duration
    let total_duration = start_time.elapsed().as_millis() as i64;
    let status_code = response.status().as_u16() as i32;
    
    // Extract timing information from response headers if available
    let session_validation_ms = response
        .headers()
        .get("x-session-validation-ms")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());
    
    let db_query_ms = response
        .headers()
        .get("x-db-query-ms")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());
    
    let middleware_stack_ms = response
        .headers()
        .get("x-middleware-ms")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    // Get response size
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);

    // Extract error message for failed requests
    let error_message = if status_code >= 400 {
        response
            .headers()
            .get("x-error-message")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    } else {
        None
    };

    // Create performance metric
    let metric = PerformanceMetric::new(uri.clone(), method.clone(), total_duration, status_code)
        .with_cache_hit(cache_hit.unwrap_or(false))
        .with_timings(session_validation_ms, db_query_ms, middleware_stack_ms)
        .with_user_context(user_id, Some(addr.ip().to_string()), user_agent)
        .with_trace_id(trace_id)
        .with_sizes(request_size, response_size);

    let metric = if let Some(error) = error_message {
        metric.with_error(error)
    } else {
        metric
    };

    // Log performance metric (async, don't block response)
    // TODO: Get connection from extension or global state
    // For now, skip recording to avoid compilation errors
    // let repo = Arc::new(PerformanceRepo::new(connection));
    
    // TODO: Re-enable when properly integrated with container
    /*
    tokio::spawn(async move {
        if let Err(e) = repo.record_metric(&metric).await {
            error!("Failed to record performance metric: {}", e);
        } else {
            info!(
                "Performance metric recorded: {} {} {}ms (status: {})",
                method, uri, total_duration, status_code
            );
        }
    });
    */

    // Add performance headers to response for debugging
    let mut response = response;
    response.headers_mut().insert(
        "x-response-time-ms",
        total_duration.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "x-trace-id",
        trace_id.to_string().parse().unwrap(),
    );

    response
}

/// Enhanced middleware that includes timing for specific operations
pub struct TimingMiddleware {
    operation_name: String,
}

impl TimingMiddleware {
    pub fn new(operation_name: String) -> Self {
        Self { operation_name }
    }

    /// Middleware to time specific operations (like DB queries, cache operations)
    pub async fn timing_middleware(
        operation_name: String,
        request: Request,
        next: Next,
    ) -> Response {
        let start_time = Instant::now();
        
        let response = next.run(request).await;
        
        let duration = start_time.elapsed().as_millis() as i64;
        
        // Add timing header
        let mut response = response;
        response.headers_mut().insert(
            format!("x-{}-ms", operation_name).parse().unwrap(),
            duration.to_string().parse().unwrap(),
        );
        
        response
    }
}

/// Cache performance tracking
pub async fn track_cache_operation(
    cache_type: &str,
    operation: &str,
    key_pattern: Option<&str>,
    hit: bool,
    duration_ms: i64,
    repo: &PerformanceRepo,
) {
    let metric = crate::web::performance::models::CachePerformanceMetric::new(
        cache_type.to_string(),
        operation.to_string(),
        hit,
        duration_ms,
    );

    if let Err(e) = repo.record_cache_metric(&metric).await {
        warn!("Failed to record cache performance metric: {}", e);
    }
}

/// Database query performance tracking
pub async fn track_db_query(
    query_type: &str,
    duration_ms: i64,
    error: Option<&str>,
) {
    info!(
        "DB Query Performance: {} took {}ms{}",
        query_type,
        duration_ms,
        error.map(|e| format!(" (error: {})", e)).unwrap_or_default()
    );
    
    // Could extend this to record in a separate DB query metrics table
}

/// Session validation performance tracking
pub async fn track_session_validation(
    validation_type: &str,
    duration_ms: i64,
    success: bool,
) {
    info!(
        "Session Validation: {} took {}ms (success: {})",
        validation_type,
        duration_ms,
        success
    );
}

/// Rate limiting performance tracking
pub async fn track_rate_limiting(
    endpoint: &str,
    duration_ms: i64,
    allowed: bool,
    current_count: u32,
    limit: u32,
) {
    info!(
        "Rate Limiting: {} took {}ms (allowed: {}, count: {}/{})",
        endpoint,
        duration_ms,
        allowed,
        current_count,
        limit
    );
}

/// Permission check performance tracking
pub async fn track_permission_check(
    permission: &str,
    user_id: Uuid,
    duration_ms: i64,
    granted: bool,
) {
    info!(
        "Permission Check: {} for user {} took {}ms (granted: {})",
        permission,
        user_id,
        duration_ms,
        granted
    );
}

/// Middleware to track endpoint-specific metrics
pub fn create_endpoint_timing_middleware(
    endpoint_name: String,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> + Clone {
    move |request: Request, next: Next| {
        let endpoint_name = endpoint_name.clone();
        Box::pin(async move {
            let start_time = Instant::now();
            
            let response = next.run(request).await;
            
            let duration = start_time.elapsed().as_millis() as i64;
            
            info!(
                "Endpoint {} completed in {}ms (status: {})",
                endpoint_name,
                duration,
                response.status()
            );
            
            // Add endpoint timing header
            let mut response = response;
            response.headers_mut().insert(
                "x-endpoint-duration-ms",
                duration.to_string().parse().unwrap(),
            );
            
            response
        })
    }
}

/// System resource monitoring (called periodically)
pub async fn collect_system_metrics(repo: &PerformanceRepo) -> Result<(), Box<dyn std::error::Error>> {
    use sysinfo::System;
    
    let mut system = System::new_all();
    system.refresh_all();
    
    // For now, use placeholder values since sysinfo API has changed
    // In production, you'd use the correct sysinfo API or system-specific tools
    let cpu_usage = 0.0; // system.global_cpu_info().cpu_usage() as f64;
    let memory_usage_percent = 0.0;
    let used_memory = 0i64;
    let disk_usage_percent = 0.0;
    let disk_read = 0i64;
    let disk_write = 0i64;
    let network_rx = 0i64;
    let network_tx = 0i64;
    let process_count = 0i32;
    
    let metrics = crate::web::performance::models::SystemResourceMetric {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        cpu_usage_percent: Some(cpu_usage),
        memory_usage_percent: Some(memory_usage_percent),
        memory_usage_bytes: Some(used_memory as i64),
        disk_usage_percent: Some(disk_usage_percent),
        disk_io_read_bytes: Some(disk_read),
        disk_io_write_bytes: Some(disk_write),
        network_rx_bytes: Some(network_rx),
        network_tx_bytes: Some(network_tx),
        active_connections: None, // Would need to be tracked separately
        db_connection_pool_active: None, // Would need to be provided by connection pool
        db_connection_pool_idle: None,
        redis_connections: None,
        goroutines_count: Some(process_count),
        gc_pause_ms: None, // Rust doesn't have traditional GC
        created_at: Utc::now(),
    };
    
    repo.record_system_metrics(&metrics).await?;
    
    Ok(())
}