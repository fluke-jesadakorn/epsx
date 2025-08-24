// Performance metrics repository implementation

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::infra::db::diesel::AsyncPgConnection;
use crate::web::performance::models::*;
use crate::schema::{
    performance_metrics, 
    cache_performance_metrics, 
    performance_alert_config,
    performance_alerts,
    performance_recommendations,
    system_resource_metrics,
    performance_aggregates
};

/// Repository for performance metrics data access
pub struct PerformanceRepo {
    conn: AsyncPgConnection,
}

impl PerformanceRepo {
    pub fn new(conn: AsyncPgConnection) -> Self {
        Self { conn }
    }

    /// Record a performance metric
    pub async fn record_metric(&mut self, metric: &PerformanceMetric) -> Result<(), diesel::result::Error> {
        diesel::insert_into(performance_metrics::table)
            .values((
                performance_metrics::id.eq(metric.id),
                performance_metrics::timestamp.eq(metric.timestamp),
                performance_metrics::endpoint.eq(&metric.endpoint),
                performance_metrics::method.eq(&metric.method),
                performance_metrics::duration_ms.eq(metric.duration_ms),
                performance_metrics::status_code.eq(metric.status_code),
                performance_metrics::cache_hit.eq(metric.cache_hit),
                performance_metrics::session_validation_ms.eq(metric.session_validation_ms),
                performance_metrics::db_query_ms.eq(metric.db_query_ms),
                performance_metrics::middleware_stack_ms.eq(metric.middleware_stack_ms),
                performance_metrics::request_size_bytes.eq(metric.request_size_bytes),
                performance_metrics::response_size_bytes.eq(metric.response_size_bytes),
                performance_metrics::user_id.eq(metric.user_id),
                performance_metrics::client_ip.eq(metric.client_ip.as_deref()),
                performance_metrics::user_agent.eq(metric.user_agent.as_deref()),
                performance_metrics::error_message.eq(metric.error_message.as_deref()),
                performance_metrics::trace_id.eq(metric.trace_id),
                performance_metrics::created_at.eq(metric.created_at),
            ))
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }

    /// Record cache performance metric
    pub async fn record_cache_metric(&mut self, metric: &CachePerformanceMetric) -> Result<(), diesel::result::Error> {
        diesel::insert_into(cache_performance_metrics::table)
            .values((
                cache_performance_metrics::id.eq(metric.id),
                cache_performance_metrics::timestamp.eq(metric.timestamp),
                cache_performance_metrics::cache_type.eq(&metric.cache_type),
                cache_performance_metrics::operation.eq(&metric.operation),
                cache_performance_metrics::key_pattern.eq(metric.key_pattern.as_deref()),
                cache_performance_metrics::hit.eq(metric.hit),
                cache_performance_metrics::duration_ms.eq(metric.duration_ms),
                cache_performance_metrics::key_size_bytes.eq(metric.key_size_bytes),
                cache_performance_metrics::value_size_bytes.eq(metric.value_size_bytes),
                cache_performance_metrics::ttl_seconds.eq(metric.ttl_seconds),
                cache_performance_metrics::evicted.eq(metric.evicted),
                cache_performance_metrics::error_message.eq(metric.error_message.as_deref()),
                cache_performance_metrics::created_at.eq(metric.created_at),
            ))
            .execute(&mut self.conn)
            .await?;

        Ok(())
    }

    /// Get performance summary for dashboard
    pub async fn get_performance_summary(
        &mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<PerformanceSummary, diesel::result::Error> {
        // Note: Complex aggregations with percentiles will need raw SQL or multiple queries
        // This is a simplified implementation
        let stats = performance_metrics::table
            .filter(performance_metrics::timestamp.between(start_time, end_time))
            .select((
                diesel::dsl::count_star(),
                diesel::dsl::avg(performance_metrics::duration_ms),
                diesel::dsl::count(performance_metrics::id.nullable().filter(performance_metrics::status_code.ge(400))),
                diesel::dsl::count(performance_metrics::id.nullable().filter(performance_metrics::cache_hit.eq(Some(true)))),
            ))
            .first::<(i64, Option<f64>, i64, i64)>(&mut self.conn)
            .await?;

        let (total_requests, avg_response_time, error_count, cache_hits) = stats;
        let error_rate = if total_requests > 0 {
            (error_count as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        let cache_hit_rate = if total_requests > 0 {
            (cache_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let duration_seconds = (end_time - start_time).num_seconds();
        let throughput_rps = if duration_seconds > 0 {
            total_requests as f64 / duration_seconds as f64 * 60.0
        } else {
            0.0
        };

        let unique_endpoints = performance_metrics::table
            .filter(performance_metrics::timestamp.between(start_time, end_time))
            .select(performance_metrics::endpoint)
            .distinct()
            .load::<String>(&mut self.conn)
            .await?
            .len() as i64;

        let active_alerts = self.count_active_alerts().await?;

        Ok(PerformanceSummary {
            total_requests,
            avg_response_time: avg_response_time.unwrap_or(0.0),
            p95_response_time: 0.0, // TODO: Implement with raw SQL
            p99_response_time: 0.0, // TODO: Implement with raw SQL  
            error_rate,
            cache_hit_rate,
            throughput_rps,
            unique_endpoints,
            active_alerts,
        })
    }

    /// Get slowest endpoints (simplified implementation)
    pub async fn get_slowest_endpoints(
        &mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: i32,
    ) -> Result<Vec<EndpointPerformance>, diesel::result::Error> {
        use diesel::dsl::*;

        let results = performance_metrics::table
            .filter(performance_metrics::timestamp.between(start_time, end_time))
            .group_by((performance_metrics::endpoint, performance_metrics::method))
            .having(count_star().ge(5i64))
            .select((
                performance_metrics::endpoint,
                performance_metrics::method,
                count_star(),
                avg(performance_metrics::duration_ms),
                max(performance_metrics::duration_ms),
                count(performance_metrics::id.nullable().filter(performance_metrics::status_code.ge(400))),
                count(performance_metrics::id.nullable().filter(performance_metrics::cache_hit.eq(Some(true)))),
            ))
            .order(avg(performance_metrics::duration_ms).desc())
            .limit(limit as i64)
            .load::<(String, String, i64, Option<f64>, Option<i64>, i64, i64)>(&mut self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(endpoint, method, count, avg_dur, max_dur, errors, cache_hits)| {
                let error_rate = if count > 0 { (errors as f64 / count as f64) * 100.0 } else { 0.0 };
                let cache_hit_rate = if count > 0 { (cache_hits as f64 / count as f64) * 100.0 } else { 0.0 };
                
                EndpointPerformance {
                    endpoint,
                    method,
                    request_count: count,
                    avg_duration: avg_dur.unwrap_or(0.0),
                    p95_duration: 0.0, // TODO: Implement with raw SQL
                    p99_duration: 0.0, // TODO: Implement with raw SQL  
                    max_duration: max_dur.unwrap_or(0) as f64,
                    error_count: errors,
                    error_rate,
                    cache_hit_rate,
                }
            })
            .collect())
    }

    /// Get performance trends (simplified)
    pub async fn get_performance_trends(
        &mut self,
        endpoint: Option<&str>,
        metric_type: MetricType,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        _time_bucket: TimeBucket,
    ) -> Result<Vec<PerformanceTrend>, diesel::result::Error> {
        // This is a simplified implementation - full time bucketing requires raw SQL
        let mut query = performance_metrics::table
            .filter(performance_metrics::timestamp.between(start_time, end_time))
            .into_boxed();

        if let Some(endpoint) = endpoint {
            query = query.filter(performance_metrics::endpoint.eq(endpoint));
        }

        let results = match metric_type {
            MetricType::Latency => {
                query
                    .select((performance_metrics::timestamp, performance_metrics::duration_ms))
                    .order(performance_metrics::timestamp.asc())
                    .load::<(DateTime<Utc>, i64)>(&mut self.conn)
                    .await?
            }
            _ => {
                // Simplified - just return latency for other types
                query
                    .select((performance_metrics::timestamp, performance_metrics::duration_ms))
                    .order(performance_metrics::timestamp.asc())
                    .load::<(DateTime<Utc>, i64)>(&mut self.conn)
                    .await?
            }
        };

        let mut trends = Vec::new();
        let mut previous_value: Option<f64> = None;

        for (timestamp, value) in results {
            let metric_value = value as f64;
            let (trend_direction, percentage_change) = if let Some(prev) = previous_value {
                let change = ((metric_value - prev) / prev) * 100.0;
                let direction = if change.abs() < 5.0 {
                    TrendDirection::Stable
                } else if change > 0.0 {
                    TrendDirection::Up
                } else {
                    TrendDirection::Down
                };
                (direction, Some(change))
            } else {
                (TrendDirection::Stable, None)
            };

            trends.push(PerformanceTrend {
                timestamp,
                metric_value,
                baseline_value: previous_value,
                trend_direction,
                percentage_change,
            });

            previous_value = Some(metric_value);
        }

        Ok(trends)
    }

    /// Get cache analytics (simplified)
    pub async fn get_cache_analytics(
        &mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<CacheAnalytics, diesel::result::Error> {
        use diesel::dsl::*;

        let overall_stats = cache_performance_metrics::table
            .filter(cache_performance_metrics::timestamp.between(start_time, end_time))
            .select((
                count(cache_performance_metrics::id.nullable().filter(cache_performance_metrics::hit.eq(true))),
                count_star(),
                avg(cache_performance_metrics::duration_ms),
                count(cache_performance_metrics::id.nullable().filter(cache_performance_metrics::evicted.eq(Some(true)))),
            ))
            .first::<(i64, i64, Option<f64>, i64)>(&mut self.conn)
            .await?;

        let (hits, total, avg_time, evictions) = overall_stats;
        let hit_rate = if total > 0 { (hits as f64 / total as f64) * 100.0 } else { 0.0 };
        let eviction_rate = if total > 0 { (evictions as f64 / total as f64) * 100.0 } else { 0.0 };

        // Get hit rate by type
        let hit_rate_by_type = cache_performance_metrics::table
            .filter(cache_performance_metrics::timestamp.between(start_time, end_time))
            .group_by(cache_performance_metrics::cache_type)
            .select((
                cache_performance_metrics::cache_type,
                count(cache_performance_metrics::id.nullable().filter(cache_performance_metrics::hit.eq(true))),
                count_star(),
            ))
            .load::<(String, i64, i64)>(&mut self.conn)
            .await?
            .into_iter()
            .map(|(cache_type, hits, total)| {
                let rate = if total > 0 { (hits as f64 / total as f64) * 100.0 } else { 0.0 };
                (cache_type, rate)
            })
            .collect();

        // Get top missed keys
        let top_missed_keys = cache_performance_metrics::table
            .filter(cache_performance_metrics::timestamp.between(start_time, end_time))
            .filter(cache_performance_metrics::hit.eq(false))
            .filter(cache_performance_metrics::key_pattern.is_not_null())
            .group_by(cache_performance_metrics::key_pattern)
            .select((cache_performance_metrics::key_pattern, count_star()))
            .order(count_star().desc())
            .limit(10)
            .load::<(Option<String>, i64)>(&mut self.conn)
            .await?
            .into_iter()
            .filter_map(|(key, _)| key)
            .collect();

        Ok(CacheAnalytics {
            overall_hit_rate: hit_rate,
            hit_rate_by_type,
            avg_response_time: avg_time.unwrap_or(0.0),
            cache_size_mb: 0.0, // TODO: Implement cache size tracking
            eviction_rate,
            top_missed_keys,
        })
    }

    /// Create alert configuration
    pub async fn create_alert_config(&mut self, config: &CreateAlertConfigRequest) -> Result<Uuid, diesel::result::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        diesel::insert_into(performance_alert_config::table)
            .values((
                performance_alert_config::id.eq(id),
                performance_alert_config::name.eq(&config.name),
                performance_alert_config::description.eq(config.description.as_deref()),
                performance_alert_config::metric_type.eq(config.metric_type as i32),
                performance_alert_config::threshold_value.eq(config.threshold_value),
                performance_alert_config::threshold_operator.eq(config.threshold_operator as i32),
                performance_alert_config::time_window_minutes.eq(config.time_window_minutes),
                performance_alert_config::endpoint_pattern.eq(config.endpoint_pattern.as_deref()),
                performance_alert_config::severity.eq(config.severity as i32),
                performance_alert_config::cooldown_minutes.eq(config.cooldown_minutes),
                performance_alert_config::notification_channels.eq(&config.notification_channels),
                performance_alert_config::created_at.eq(now),
                performance_alert_config::updated_at.eq(now),
            ))
            .execute(&mut self.conn)
            .await?;

        Ok(id)
    }

    /// Get alert configurations (simplified)
    pub async fn get_alert_configs(&mut self) -> Result<Vec<PerformanceAlertConfig>, diesel::result::Error> {
        performance_alert_config::table
            .order(performance_alert_config::created_at.desc())
            .select((
                performance_alert_config::id,
                performance_alert_config::name,
                performance_alert_config::description,
                performance_alert_config::metric_type,
                performance_alert_config::threshold_value,
                performance_alert_config::threshold_operator,
                performance_alert_config::time_window_minutes,
                performance_alert_config::endpoint_pattern,
                performance_alert_config::severity,
                performance_alert_config::enabled,
                performance_alert_config::cooldown_minutes,
                performance_alert_config::notification_channels,
                performance_alert_config::created_at,
                performance_alert_config::updated_at,
            ))
            .load::<(Uuid, String, Option<String>, i32, f64, i32, i32, Option<String>, i32, bool, i32, Option<serde_json::Value>, DateTime<Utc>, DateTime<Utc>)>(&mut self.conn)
            .await?
            .into_iter()
            .map(|(id, name, description, metric_type, threshold_value, threshold_operator, time_window_minutes, endpoint_pattern, severity, enabled, cooldown_minutes, notification_channels, created_at, updated_at)| {
                PerformanceAlertConfig {
                    id,
                    name,
                    description,
                    metric_type: MetricType::from_i32(metric_type),
                    threshold_value,
                    threshold_operator: ThresholdOperator::from_i32(threshold_operator),
                    time_window_minutes,
                    endpoint_pattern,
                    severity: AlertSeverity::from_i32(severity),
                    enabled,
                    cooldown_minutes,
                    notification_channels,
                    created_at,
                    updated_at,
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| diesel::result::Error::DeserializationError("Invalid enum value".into()))
    }

    /// Count active alerts
    pub async fn count_active_alerts(&mut self) -> Result<i64, diesel::result::Error> {
        performance_alerts::table
            .filter(performance_alerts::resolved_at.is_null())
            .select(diesel::dsl::count_star())
            .first(&mut self.conn)
            .await
    }

    // TODO: Implement remaining methods with proper Diesel queries
    // For now, providing minimal implementations to reduce compilation errors
    
    pub async fn create_alert(&mut self, _alert: &PerformanceAlert) -> Result<(), diesel::result::Error> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get_active_alerts(&mut self) -> Result<Vec<PerformanceAlert>, diesel::result::Error> {
        // TODO: Implement
        Ok(Vec::new())
    }

    pub async fn acknowledge_alert(&mut self, _alert_id: Uuid, _acknowledged_by: Uuid, _notes: Option<String>) -> Result<(), diesel::result::Error> {
        // TODO: Implement
        Ok(())
    }

    pub async fn resolve_alert(&mut self, _alert_id: Uuid) -> Result<(), diesel::result::Error> {
        // TODO: Implement
        Ok(())
    }

    pub async fn create_recommendation(&mut self, _rec: &PerformanceRecommendation) -> Result<(), diesel::result::Error> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get_recommendations(&mut self, _status: Option<RecommendationStatus>, _limit: Option<i32>) -> Result<Vec<PerformanceRecommendation>, diesel::result::Error> {
        // TODO: Implement
        Ok(Vec::new())
    }

    pub async fn record_system_metrics(&mut self, _metrics: &SystemResourceMetric) -> Result<(), diesel::result::Error> {
        // TODO: Implement
        Ok(())
    }

    pub async fn get_system_health(&mut self) -> Result<SystemHealthMetrics, diesel::result::Error> {
        // TODO: Implement with default values
        Ok(SystemHealthMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            connection_pool_utilization: 0.0,
            goroutines_count: 0,
            gc_pause_ms: 0.0,
            health_score: 100.0,
        })
    }

    pub async fn get_aggregates(&mut self, _start_time: DateTime<Utc>, _end_time: DateTime<Utc>, _time_bucket: TimeBucket, _endpoint: Option<String>) -> Result<Vec<PerformanceAggregate>, diesel::result::Error> {
        // TODO: Implement
        Ok(Vec::new())
    }

    pub async fn detect_anomalies(&mut self, _lookback_hours: i32) -> Result<Vec<PerformanceAnomalyDetection>, diesel::result::Error> {
        // TODO: Implement
        Ok(Vec::new())
    }
}