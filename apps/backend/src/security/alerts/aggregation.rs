// Alert Aggregation Engine
// Intelligent event aggregation and deduplication to reduce alert noise

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration, Timelike};
use tracing::debug;

use super::models::*;
use crate::permissions::audit::SecurityEvent;

/// Event aggregation strategies
#[derive(Debug, Clone)]
pub enum AggregationStrategy {
    TimeWindow,    // Group events within time window
    EventCount,    // Aggregate when count exceeds threshold  
    RateBased,     // Aggregate based on event rate
    Pattern,       // Group similar event patterns
    Severity,      // Aggregate by severity escalation
}

/// Aggregated event data
#[derive(Debug, Clone)]
pub struct AggregatedEvent {
    pub base_event: SecurityEvent,
    pub aggregated_events: Vec<SecurityEvent>,
    pub event_count: i64,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub aggregation_strategy: AggregationStrategy,
    pub severity_escalated: bool,
    pub deduplication_key: String,
}

/// Aggregation Engine for intelligent event processing
pub struct AggregationEngine {
    max_window_minutes: i32,
    aggregation_cache: std::sync::RwLock<AggregationCache>,
}

impl AggregationEngine {
    pub fn new(max_window_minutes: i32) -> Self {
        Self {
            max_window_minutes,
            aggregation_cache: std::sync::RwLock::new(AggregationCache::new()),
        }
    }

    /// Process a security event and return aggregated result
    pub async fn process_event(&self, event: &SecurityEvent) -> AlertResult<SecurityEvent> {
        // Check if event should be aggregated
        if let Some(aggregated) = self.try_aggregate_event(event).await? {
            // Return the enhanced base event with aggregation metadata
            let mut enhanced_event = aggregated.base_event.clone();
            let enhanced_details = self.enhance_event_details(&enhanced_event, &aggregated)?;
            if let serde_json::Value::Object(map) = enhanced_details {
                enhanced_event.details = std::collections::HashMap::from_iter(map.into_iter());
            }
            return Ok(enhanced_event);
        }

        // No aggregation, return original event
        Ok(event.clone())
    }

    /// Attempt to aggregate event with existing cached events
    async fn try_aggregate_event(&self, event: &SecurityEvent) -> AlertResult<Option<AggregatedEvent>> {
        let dedup_key = self.generate_deduplication_key(event);
        
        let mut cache = self.aggregation_cache.write().unwrap();
        
        // Check for existing aggregation
        if let Some(existing) = cache.get_aggregation(&dedup_key) {
            // Update existing aggregation
            let updated = self.update_aggregation(existing.clone(), &event).await?;
            cache.update_aggregation(dedup_key, updated.clone());
            return Ok(Some(updated));
        }

        // Check if event qualifies for aggregation
        if self.should_start_aggregation(event, &cache).await? {
            let new_aggregation = self.create_new_aggregation(event, dedup_key.clone()).await?;
            cache.add_aggregation(dedup_key, new_aggregation.clone());
            return Ok(Some(new_aggregation));
        }

        Ok(None)
    }

    /// Generate deduplication key for event
    fn generate_deduplication_key(&self, event: &SecurityEvent) -> String {
        format!(
            "{}:{}:{}:{}",
            event.event_type,
            event.client_ip.as_deref().unwrap_or("unknown"),
            event.user_id.as_ref().map(|u| u.to_string()).as_deref().unwrap_or("unknown"),
            "audit" // Default source since SecurityEvent doesn't have source field
        )
    }

    /// Check if event should start a new aggregation
    async fn should_start_aggregation(
        &self,
        event: &SecurityEvent,
        cache: &AggregationCache,
    ) -> AlertResult<bool> {
        // Check for similar events in recent time window
        let time_window = Duration::minutes(5);
        let window_start = event.timestamp - time_window;
        
        let similar_count = cache.count_similar_events(
            &event.event_type,
            event.client_ip.as_deref().unwrap_or("unknown"),
            window_start,
            event.timestamp,
        );

        // Start aggregation if we have multiple similar events
        Ok(similar_count >= 2)
    }

    /// Create new aggregation for event
    async fn create_new_aggregation(
        &self,
        event: &SecurityEvent,
        dedup_key: String,
    ) -> AlertResult<AggregatedEvent> {
        let now = Utc::now();
        let window_start = event.timestamp - Duration::minutes(5);
        
        Ok(AggregatedEvent {
            base_event: event.clone(),
            aggregated_events: vec![event.clone()],
            event_count: 1,
            time_window_start: window_start,
            time_window_end: now,
            aggregation_strategy: AggregationStrategy::TimeWindow,
            severity_escalated: false,
            deduplication_key: dedup_key,
        })
    }

    /// Update existing aggregation with new event
    async fn update_aggregation(
        &self,
        mut existing: AggregatedEvent,
        event: &SecurityEvent,
    ) -> AlertResult<AggregatedEvent> {
        existing.aggregated_events.push(event.clone());
        existing.event_count += 1;
        existing.time_window_end = event.timestamp;

        // Check for severity escalation
        if self.should_escalate_severity(&existing, event) {
            existing.severity_escalated = true;
            existing.base_event.severity = event.severity.clone();
        }

        // Update aggregation strategy if needed
        existing.aggregation_strategy = self.determine_aggregation_strategy(&existing);

        Ok(existing)
    }

    /// Check if severity should be escalated
    fn should_escalate_severity(&self, aggregation: &AggregatedEvent, new_event: &SecurityEvent) -> bool {
        let current_severity = self.severity_to_numeric(&aggregation.base_event.severity);
        let new_severity = self.severity_to_numeric(&new_event.severity);
        
        new_severity > current_severity
    }

    /// Convert severity string to numeric value for comparison
    fn severity_to_numeric(&self, severity: &str) -> i32 {
        match severity.to_uppercase().as_str() {
            "LOW" => 1,
            "MEDIUM" => 2,
            "HIGH" => 3,
            "CRITICAL" => 4,
            _ => 0,
        }
    }

    /// Determine optimal aggregation strategy
    fn determine_aggregation_strategy(&self, aggregation: &AggregatedEvent) -> AggregationStrategy {
        let time_span = aggregation.time_window_end - aggregation.time_window_start;
        let event_rate = aggregation.event_count as f64 / time_span.num_minutes() as f64;

        if event_rate > 10.0 {
            AggregationStrategy::RateBased
        } else if aggregation.event_count > 5 {
            AggregationStrategy::EventCount
        } else if aggregation.severity_escalated {
            AggregationStrategy::Severity
        } else {
            AggregationStrategy::TimeWindow
        }
    }

    /// Enhance event details with aggregation metadata
    fn enhance_event_details(
        &self,
        event: &SecurityEvent,
        aggregation: &AggregatedEvent,
    ) -> AlertResult<serde_json::Value> {
        let details = event.details.clone();
        
        // Convert HashMap to serde_json::Map for enhanced details
        let mut json_map = serde_json::Map::new();
        for (k, v) in details.iter() {
            json_map.insert(k.clone(), v.clone());
        }
        
        // Add aggregation metadata
        {
            json_map.insert("aggregated_count".to_string(), serde_json::json!(aggregation.event_count));
            json_map.insert("time_window_minutes".to_string(), 
                serde_json::json!((aggregation.time_window_end - aggregation.time_window_start).num_minutes()));
            json_map.insert("aggregation_strategy".to_string(), 
                serde_json::json!(format!("{:?}", aggregation.aggregation_strategy)));
            json_map.insert("severity_escalated".to_string(), 
                serde_json::json!(aggregation.severity_escalated));
            
            // Add summary of aggregated events
            let event_types: Vec<String> = aggregation.aggregated_events
                .iter()
                .map(|e| e.event_type.clone())
                .collect();
            json_map.insert("aggregated_event_types".to_string(), 
                serde_json::json!(event_types));
        }

        Ok(serde_json::Value::Object(json_map))
    }

    /// Clean up expired aggregations
    pub async fn cleanup_expired(&self) -> AlertResult<()> {
        let cutoff_time = Utc::now() - Duration::minutes(self.max_window_minutes as i64);
        
        let mut cache = self.aggregation_cache.write().unwrap();
        let removed_count = cache.cleanup_expired(cutoff_time);
        
        debug!("Cleaned up {} expired aggregations", removed_count);
        Ok(())
    }

    /// Get aggregation statistics
    pub async fn get_stats(&self) -> AggregationStats {
        let cache = self.aggregation_cache.read().unwrap();
        cache.get_stats()
    }
}

/// Cache for event aggregations
pub struct AggregationCache {
    aggregations: HashMap<String, AggregatedEvent>,
    event_timeline: Vec<(DateTime<Utc>, String, String, String)>, // (timestamp, event_type, ip, dedup_key)
}

impl AggregationCache {
    pub fn new() -> Self {
        Self {
            aggregations: HashMap::new(),
            event_timeline: Vec::new(),
        }
    }

    pub fn get_aggregation(&self, dedup_key: &str) -> Option<&AggregatedEvent> {
        self.aggregations.get(dedup_key)
    }

    pub fn add_aggregation(&mut self, dedup_key: String, aggregation: AggregatedEvent) {
        self.event_timeline.push((
            aggregation.base_event.timestamp,
            aggregation.base_event.event_type.clone(),
            aggregation.base_event.client_ip.clone().unwrap_or_default(),
            dedup_key.clone(),
        ));
        
        self.aggregations.insert(dedup_key, aggregation);
        
        // Keep timeline sorted
        self.event_timeline.sort_by_key(|&(timestamp, _, _, _)| timestamp);
    }

    pub fn update_aggregation(&mut self, dedup_key: String, aggregation: AggregatedEvent) {
        self.aggregations.insert(dedup_key, aggregation);
    }

    pub fn count_similar_events(
        &self,
        event_type: &str,
        ip_address: &str,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> i64 {
        self.event_timeline
            .iter()
            .filter(|(timestamp, etype, ip, _)| {
                *timestamp >= window_start
                    && *timestamp <= window_end
                    && etype == event_type
                    && ip == ip_address
            })
            .count() as i64
    }

    pub fn cleanup_expired(&mut self, cutoff_time: DateTime<Utc>) -> usize {
        let initial_count = self.aggregations.len();

        // Remove expired aggregations
        self.aggregations.retain(|_, aggregation| {
            aggregation.time_window_end > cutoff_time
        });

        // Clean up timeline
        self.event_timeline.retain(|(timestamp, _, _, _)| {
            *timestamp > cutoff_time
        });

        initial_count - self.aggregations.len()
    }

    pub fn get_stats(&self) -> AggregationStats {
        let mut stats = AggregationStats::default();
        
        stats.total_aggregations = self.aggregations.len();
        stats.total_events_in_timeline = self.event_timeline.len();

        for aggregation in self.aggregations.values() {
            stats.total_aggregated_events += aggregation.event_count as usize;
            
            match aggregation.aggregation_strategy {
                AggregationStrategy::TimeWindow => stats.time_window_aggregations += 1,
                AggregationStrategy::EventCount => stats.event_count_aggregations += 1,
                AggregationStrategy::RateBased => stats.rate_based_aggregations += 1,
                AggregationStrategy::Pattern => stats.pattern_aggregations += 1,
                AggregationStrategy::Severity => stats.severity_aggregations += 1,
            }

            if aggregation.severity_escalated {
                stats.escalated_aggregations += 1;
            }

            let events_per_aggregation = aggregation.event_count as f64;
            if events_per_aggregation > stats.max_events_per_aggregation {
                stats.max_events_per_aggregation = events_per_aggregation;
            }
            
            stats.avg_events_per_aggregation += events_per_aggregation;
        }

        if stats.total_aggregations > 0 {
            stats.avg_events_per_aggregation /= stats.total_aggregations as f64;
        }

        stats
    }
}

/// Statistics for aggregation performance monitoring
#[derive(Debug, Default)]
pub struct AggregationStats {
    pub total_aggregations: usize,
    pub total_aggregated_events: usize,
    pub total_events_in_timeline: usize,
    pub time_window_aggregations: usize,
    pub event_count_aggregations: usize,
    pub rate_based_aggregations: usize,
    pub pattern_aggregations: usize,
    pub severity_aggregations: usize,
    pub escalated_aggregations: usize,
    pub max_events_per_aggregation: f64,
    pub avg_events_per_aggregation: f64,
}

/// Deduplication utilities
pub struct DeduplicationUtils;

impl DeduplicationUtils {
    /// Generate sophisticated deduplication key with multiple factors
    pub fn generate_advanced_key(event: &SecurityEvent, time_window_minutes: i32) -> String {
        let time_bucket = Self::get_time_bucket(event.timestamp, time_window_minutes);
        let content_hash = Self::hash_event_content(event);
        
        format!(
            "{}:{}:{}:{}:{}:{}",
            time_bucket,
            event.event_type,
            event.severity,
            event.client_ip.as_deref().unwrap_or("unknown"),
            event.user_id.as_ref().map(|u| u.to_string()).as_deref().unwrap_or(""),
            content_hash
        )
    }

    /// Get time bucket for grouping events
    fn get_time_bucket(timestamp: DateTime<Utc>, window_minutes: i32) -> String {
        let bucket_start = timestamp
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        
        let minutes_since_hour = bucket_start.minute() as i32;
        let bucket_minute = (minutes_since_hour / window_minutes) * window_minutes;
        
        bucket_start
            .with_minute(bucket_minute as u32)
            .unwrap()
            .format("%Y%m%d_%H%M")
            .to_string()
    }

    /// Generate hash of event content for deduplication
    fn hash_event_content(event: &SecurityEvent) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        event.event_type.hash(&mut hasher);
        event.severity.hash(&mut hasher);
        "audit".hash(&mut hasher); // Default source
        
        // Hash relevant details from JSON
        if let Ok(details_str) = serde_json::to_string(&event.details) {
            details_str.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())[..8].to_string()
    }

    /// Check if two events are similar enough to deduplicate
    pub fn are_events_similar(event1: &SecurityEvent, event2: &SecurityEvent) -> bool {
        event1.event_type == event2.event_type
            && event1.client_ip == event2.client_ip
            && event1.user_id == event2.user_id
            // source field not available in permissions::audit::SecurityEvent
            && Self::are_details_similar(&serde_json::to_value(&event1.details).unwrap_or_default(), &serde_json::to_value(&event2.details).unwrap_or_default())
    }

    /// Compare event details for similarity
    fn are_details_similar(details1: &serde_json::Value, details2: &serde_json::Value) -> bool {
        // Simplified similarity check - in production, use more sophisticated comparison
        match (details1, details2) {
            (serde_json::Value::Object(map1), serde_json::Value::Object(map2)) => {
                // Check key fields for similarity
                let key_fields = ["path", "method", "status_code", "user_agent"];
                
                for field in &key_fields {
                    if map1.get(*field) != map2.get(*field) {
                        return false;
                    }
                }
                
                true
            }
            _ => details1 == details2,
        }
    }
}

/// Smart aggregation rules engine
pub struct SmartAggregationEngine {
    rules: Vec<AggregationRule>,
}

#[derive(Debug, Clone)]
pub struct AggregationRule {
    pub name: String,
    pub conditions: serde_json::Value,
    pub strategy: AggregationStrategy,
    pub window_minutes: i32,
    pub threshold: i32,
    pub priority: i32,
}

impl SmartAggregationEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
        }
    }

    /// Default aggregation rules for common patterns
    fn default_rules() -> Vec<AggregationRule> {
        vec![
            AggregationRule {
                name: "Brute Force Aggregation".to_string(),
                conditions: serde_json::json!({
                    "event_type": "MULTIPLE_FAILED_LOGINS",
                    "threshold": 5
                }),
                strategy: AggregationStrategy::EventCount,
                window_minutes: 10,
                threshold: 5,
                priority: 1,
            },
            AggregationRule {
                name: "Rate Limit Aggregation".to_string(),
                conditions: serde_json::json!({
                    "event_type": "RATE_LIMIT_EXCEEDED"
                }),
                strategy: AggregationStrategy::RateBased,
                window_minutes: 5,
                threshold: 10,
                priority: 2,
            },
            AggregationRule {
                name: "Suspicious Activity Aggregation".to_string(),
                conditions: serde_json::json!({
                    "event_type": "SUSPICIOUS_ACTIVITY"
                }),
                strategy: AggregationStrategy::Pattern,
                window_minutes: 15,
                threshold: 3,
                priority: 3,
            },
        ]
    }

    /// Find matching aggregation rule for event
    pub fn find_matching_rule(&self, event: &SecurityEvent) -> Option<&AggregationRule> {
        self.rules
            .iter()
            .filter(|rule| self.matches_conditions(event, &rule.conditions))
            .min_by_key(|rule| rule.priority)
    }

    /// Check if event matches rule conditions
    fn matches_conditions(&self, event: &SecurityEvent, conditions: &serde_json::Value) -> bool {
        if let serde_json::Value::Object(cond_map) = conditions {
            for (key, value) in cond_map {
                match key.as_str() {
                    "event_type" => {
                        if let Some(expected) = value.as_str() {
                            if event.event_type != expected {
                                return false;
                            }
                        }
                    }
                    "severity" => {
                        if let Some(expected) = value.as_str() {
                            if event.severity != expected {
                                return false;
                            }
                        }
                    }
                    _ => {} // Skip unknown conditions
                }
            }
        }
        
        true
    }
}