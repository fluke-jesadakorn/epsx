use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::domain::shared_kernel::aggregate_root::{AggregateRoot, AggregateBase};
use crate::domain::shared_kernel::domain_event::{DomainEvent, EventMetadata};
use crate::domain::notification::value_objects::*;
use crate::infrastructure::adapters::repositories::diesel::types::{NotificationType, NotificationPriority};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Notification Aggregate Root
/// Manages the lifecycle of a notification from creation to delivery
#[derive(Debug, Clone)]
pub struct Notification {
    id: NotificationId,
    recipient_user_id: Option<Uuid>,
    topic: Option<NotificationTopic>,
    content: NotificationContent,
    notification_type: NotificationType,
    priority: NotificationPriority,
    channels: MultiChannelConfig,
    schedule: ScheduleInfo,
    metadata: NotificationMetadata,
    delivery_tracking: DeliveryTracking,
    status: NotificationStatus,
    base: AggregateBase,
}

impl Notification {
    /// Create new notification for specific user
    pub fn create_for_user(
        recipient_user_id: Uuid,
        content: NotificationContent,
        notification_type: NotificationType,
        priority: NotificationPriority,
        channels: MultiChannelConfig,
        schedule: ScheduleInfo,
    ) -> Result<Self, String> {
        let id = NotificationId::new();
        
        Self::validate_notification_params(&content, &channels, &schedule)?;
        
        let mut notification = Self {
            id: id.clone(),
            recipient_user_id: Some(recipient_user_id),
            topic: None,
            content,
            notification_type: notification_type.clone(),
            priority: priority.clone(),
            channels,
            schedule,
            metadata: NotificationMetadata::new(),
            delivery_tracking: DeliveryTracking::new(),
            status: NotificationStatus::Created,
            base: AggregateBase::new(),
        };

        // Publish creation event
        notification.base.add_event(Box::new(NotificationCreated::new(
            id.as_str(),
            notification.base.version,
            recipient_user_id,
            None,
            notification_type,
            priority,
            notification.status.clone(),
        )));

        Ok(notification)
    }

    /// Create new notification for topic (broadcast)
    pub fn create_for_topic(
        topic: NotificationTopic,
        content: NotificationContent,
        notification_type: NotificationType,
        priority: NotificationPriority,
        channels: MultiChannelConfig,
        schedule: ScheduleInfo,
        created_by: Option<Uuid>,
    ) -> Result<Self, String> {
        let id = NotificationId::new();
        
        Self::validate_notification_params(&content, &channels, &schedule)?;
        
        // Validate topic suitability
        if !topic.is_suitable_for_notification(&notification_type.to_string(), &priority.to_string()) {
            return Err(format!(
                "Topic '{}' is not suitable for {} notifications with {} priority",
                topic.display_name(),
                notification_type,
                priority
            ));
        }

        let mut notification = Self {
            id: id.clone(),
            recipient_user_id: None,
            topic: Some(topic.clone()),
            content,
            notification_type: notification_type.clone(),
            priority: priority.clone(),
            channels,
            schedule,
            metadata: NotificationMetadata::with_creator(created_by),
            delivery_tracking: DeliveryTracking::new(),
            status: NotificationStatus::Created,
            base: AggregateBase::new(),
        };

        // Publish creation event
        notification.base.add_event(Box::new(NotificationCreated::new(
            id.as_str(),
            notification.base.version,
            Uuid::nil(), // No specific recipient for topic notifications
            Some(topic.name().to_string()),
            notification_type,
            priority,
            notification.status.clone(),
        )));

        Ok(notification)
    }

    /// Schedule the notification for delivery
    pub fn schedule_for_delivery(&mut self) -> Result<(), String> {
        if self.status != NotificationStatus::Created {
            return Err(format!("Cannot schedule notification in status: {:?}", self.status));
        }

        if !self.schedule.is_ready_to_send() {
            if self.schedule.is_expired() {
                self.status = NotificationStatus::Expired;
                return Err("Cannot schedule expired notification".to_string());
            }
            
            self.status = NotificationStatus::Scheduled;
        } else {
            self.status = NotificationStatus::Queued;
        }

        // Publish scheduling event
        self.base.add_event(Box::new(NotificationScheduled::new(
            self.id.as_str(),
            self.base.version,
            self.schedule.scheduled_at(),
            self.status.clone(),
        )));

        self.base.touch();
        Ok(())
    }

    /// Mark notification as being sent
    pub fn mark_sending(&mut self) -> Result<(), String> {
        match self.status {
            NotificationStatus::Queued => {
                self.status = NotificationStatus::Sending;
                self.delivery_tracking.mark_send_started();
                
                // Publish sending event
                self.base.add_event(Box::new(NotificationSending::new(
                    self.id.as_str(),
                    self.base.version,
                    self.channels.enabled_channels().len() as u32,
                )));
                
                self.base.touch();
                Ok(())
            }
            _ => Err(format!("Cannot mark notification as sending from status: {:?}", self.status)),
        }
    }

    /// Record delivery attempt for a channel
    pub fn record_delivery_attempt(
        &mut self,
        channel: &DeliveryChannelType,
        attempt_result: DeliveryResult,
    ) -> Result<(), String> {
        if self.status != NotificationStatus::Sending {
            return Err("Can only record delivery attempts for notifications being sent".to_string());
        }

        let channel_name = channel.to_string();
        self.delivery_tracking.record_attempt(&channel_name, attempt_result.clone());

        // Check if all channels have completed delivery attempts
        let enabled_channels: Vec<String> = self.channels
            .enabled_channels()
            .iter()
            .map(|c| c.channel_type().to_string())
            .collect();

        let all_attempted = enabled_channels.iter().all(|channel| {
            self.delivery_tracking.get_channel_status(channel).is_some()
        });

        if all_attempted {
            // Determine final status based on delivery results
            let any_success = self.delivery_tracking.has_successful_delivery();
            let all_failed = enabled_channels.iter().all(|channel| {
                matches!(
                    self.delivery_tracking.get_channel_status(channel),
                    Some(ChannelDeliveryStatus::Failed(_))
                )
            });

            if any_success {
                self.status = NotificationStatus::Delivered;
            } else if all_failed {
                self.status = NotificationStatus::Failed;
            } else {
                self.status = NotificationStatus::PartiallyDelivered;
            }

            // Publish completion event
            self.base.add_event(Box::new(NotificationDeliveryCompleted::new(
                self.id.as_str(),
                self.base.version,
                self.status.clone(),
                self.delivery_tracking.successful_channels(),
                self.delivery_tracking.failed_channels(),
            )));
        }

        self.base.touch();
        Ok(())
    }

    /// Mark notification as expired
    pub fn mark_expired(&mut self) -> Result<(), String> {
        if !matches!(self.status, NotificationStatus::Created | NotificationStatus::Scheduled | NotificationStatus::Queued) {
            return Err(format!("Cannot expire notification in status: {:?}", self.status));
        }

        self.status = NotificationStatus::Expired;
        
        // Publish expiry event
        self.base.add_event(Box::new(NotificationExpired::new(
            self.id.as_str(),
            self.base.version,
            self.schedule.expires_at(),
        )));

        self.base.touch();
        Ok(())
    }

    /// Update notification priority (if not yet sent)
    pub fn update_priority(&mut self, new_priority: NotificationPriority) -> Result<(), String> {
        if matches!(self.status, NotificationStatus::Sending | NotificationStatus::Delivered | NotificationStatus::Failed | NotificationStatus::PartiallyDelivered) {
            return Err("Cannot update priority of notification that has been sent".to_string());
        }

        let old_priority = self.priority.clone();
        self.priority = new_priority.clone();

        // Publish priority update event
        self.base.add_event(Box::new(NotificationPriorityUpdated::new(
            self.id.as_str(),
            self.base.version,
            old_priority,
            new_priority,
        )));

        self.base.touch();
        Ok(())
    }

    /// Cancel notification (if not yet sent)
    pub fn cancel(&mut self, reason: String) -> Result<(), String> {
        if matches!(self.status, NotificationStatus::Sending | NotificationStatus::Delivered | NotificationStatus::Failed | NotificationStatus::PartiallyDelivered) {
            return Err("Cannot cancel notification that has been sent".to_string());
        }

        self.status = NotificationStatus::Cancelled;
        self.metadata.add_note(format!("Cancelled: {}", reason));

        // Publish cancellation event
        self.base.add_event(Box::new(NotificationCancelled::new(
            self.id.as_str(),
            self.base.version,
            reason,
        )));

        self.base.touch();
        Ok(())
    }

    /// Get effective priority (considering schedule timing)
    pub fn effective_priority(&self) -> i32 {
        let base_priority = match self.priority {
            NotificationPriority::Urgent => 100,
            NotificationPriority::Critical => 90,
            NotificationPriority::High => 75,
            NotificationPriority::Normal => 50,
            NotificationPriority::Low => 25,
        };

        let timing_adjustment = self.schedule.timing_priority_adjustment() as i32;
        base_priority + timing_adjustment
    }

    /// Check if notification should be processed now
    pub fn should_process_now(&self) -> bool {
        match self.status {
            NotificationStatus::Scheduled => self.schedule.is_ready_to_send(),
            NotificationStatus::Queued => true,
            _ => false,
        }
    }

    /// Validate notification parameters
    fn validate_notification_params(
        content: &NotificationContent,
        channels: &MultiChannelConfig,
        schedule: &ScheduleInfo,
    ) -> Result<(), String> {
        // Validate content length for enabled channels
        for channel in channels.enabled_channels() {
            if let Some(limits) = channel.max_content_length() {
                if content.title().len() > limits.title_max {
                    return Err(format!(
                        "Title too long for {} channel: {} > {}",
                        channel.channel_type(),
                        content.title().len(),
                        limits.title_max
                    ));
                }
                if content.body().len() > limits.body_max {
                    return Err(format!(
                        "Body too long for {} channel: {} > {}",
                        channel.channel_type(),
                        content.body().len(),
                        limits.body_max
                    ));
                }
            }
        }

        // Validate schedule
        if schedule.is_expired() {
            return Err("Cannot create notification with expired schedule".to_string());
        }

        Ok(())
    }

    // Getters
    pub fn id(&self) -> &NotificationId { &self.id }
    pub fn recipient_user_id(&self) -> Option<Uuid> { self.recipient_user_id }
    pub fn topic(&self) -> Option<&NotificationTopic> { self.topic.as_ref() }
    pub fn content(&self) -> &NotificationContent { &self.content }
    pub fn notification_type(&self) -> &NotificationType { &self.notification_type }
    pub fn priority(&self) -> &NotificationPriority { &self.priority }
    pub fn channels(&self) -> &MultiChannelConfig { &self.channels }
    pub fn schedule(&self) -> &ScheduleInfo { &self.schedule }
    pub fn metadata(&self) -> &NotificationMetadata { &self.metadata }
    pub fn delivery_tracking(&self) -> &DeliveryTracking { &self.delivery_tracking }
    pub fn status(&self) -> &NotificationStatus { &self.status }
}

impl AggregateRoot for Notification {
    type Id = NotificationId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn version(&self) -> u64 {
        self.base.version
    }

    fn increment_version(&mut self) {
        self.base.increment_version();
    }

    fn uncommitted_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.base.events
    }

    fn mark_events_as_committed(&mut self) {
        self.base.clear_events();
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.base.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.base.updated_at
    }

    fn touch(&mut self) {
        self.base.touch();
    }
}

/// Notification metadata and tracking information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationMetadata {
    created_by: Option<Uuid>,
    image_url: Option<String>,
    action_url: Option<String>,
    data_payload: Option<serde_json::Value>,
    tags: Vec<String>,
    notes: Vec<String>,
}

impl NotificationMetadata {
    pub fn new() -> Self {
        Self {
            created_by: None,
            image_url: None,
            action_url: None,
            data_payload: None,
            tags: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn with_creator(creator: Option<Uuid>) -> Self {
        Self {
            created_by: creator,
            image_url: None,
            action_url: None,
            data_payload: None,
            tags: Vec::new(),
            notes: Vec::new(),
        }
    }

    pub fn created_by(&self) -> Option<Uuid> { self.created_by }
    pub fn image_url(&self) -> Option<&str> { self.image_url.as_deref() }
    pub fn action_url(&self) -> Option<&str> { self.action_url.as_deref() }
    pub fn data_payload(&self) -> Option<&serde_json::Value> { self.data_payload.as_ref() }
    pub fn tags(&self) -> &[String] { &self.tags }
    pub fn notes(&self) -> &[String] { &self.notes }

    pub fn set_image_url(&mut self, url: String) {
        self.image_url = Some(url);
    }

    pub fn set_action_url(&mut self, url: String) {
        self.action_url = Some(url);
    }

    pub fn set_data_payload(&mut self, payload: serde_json::Value) {
        self.data_payload = Some(payload);
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn add_note(&mut self, note: String) {
        self.notes.push(note);
    }
}

/// Delivery tracking for all channels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryTracking {
    send_started_at: Option<DateTime<Utc>>,
    channel_status: HashMap<String, ChannelDeliveryStatus>,
    total_attempts: u32,
}

impl DeliveryTracking {
    pub fn new() -> Self {
        Self {
            send_started_at: None,
            channel_status: HashMap::new(),
            total_attempts: 0,
        }
    }

    pub fn mark_send_started(&mut self) {
        self.send_started_at = Some(Utc::now());
    }

    pub fn record_attempt(&mut self, channel: &str, result: DeliveryResult) {
        self.total_attempts += 1;
        let status = match result {
            DeliveryResult::Success { delivered_at, message_id } => {
                ChannelDeliveryStatus::Delivered { delivered_at, message_id }
            }
            DeliveryResult::Failed { error_message, retry_after } => {
                ChannelDeliveryStatus::Failed(DeliveryError {
                    error_message,
                    retry_after,
                    attempted_at: Utc::now(),
                })
            }
        };
        self.channel_status.insert(channel.to_string(), status);
    }

    pub fn get_channel_status(&self, channel: &str) -> Option<&ChannelDeliveryStatus> {
        self.channel_status.get(channel)
    }

    pub fn has_successful_delivery(&self) -> bool {
        self.channel_status.values().any(|status| {
            matches!(status, ChannelDeliveryStatus::Delivered { .. })
        })
    }

    pub fn successful_channels(&self) -> Vec<String> {
        self.channel_status
            .iter()
            .filter_map(|(channel, status)| {
                if matches!(status, ChannelDeliveryStatus::Delivered { .. }) {
                    Some(channel.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn failed_channels(&self) -> Vec<String> {
        self.channel_status
            .iter()
            .filter_map(|(channel, status)| {
                if matches!(status, ChannelDeliveryStatus::Failed(_)) {
                    Some(channel.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn send_started_at(&self) -> Option<DateTime<Utc>> { self.send_started_at }
    pub fn total_attempts(&self) -> u32 { self.total_attempts }
}

/// Delivery status for individual channels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelDeliveryStatus {
    Delivered {
        delivered_at: DateTime<Utc>,
        message_id: Option<String>,
    },
    Failed(DeliveryError),
}

/// Delivery error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryError {
    pub error_message: String,
    pub retry_after: Option<DateTime<Utc>>,
    pub attempted_at: DateTime<Utc>,
}

/// Result of a delivery attempt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeliveryResult {
    Success {
        delivered_at: DateTime<Utc>,
        message_id: Option<String>,
    },
    Failed {
        error_message: String,
        retry_after: Option<DateTime<Utc>>,
    },
}

/// Notification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Created,              // Just created
    Scheduled,            // Scheduled for future delivery
    Queued,              // Ready to be sent
    Sending,             // Currently being sent
    Delivered,           // Successfully delivered to all channels
    PartiallyDelivered,  // Delivered to some channels
    Failed,              // Failed to deliver to all channels
    Expired,             // Expired before delivery
    Cancelled,           // Manually cancelled
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationStatus::Created => "created",
            NotificationStatus::Scheduled => "scheduled",
            NotificationStatus::Queued => "queued",
            NotificationStatus::Sending => "sending",
            NotificationStatus::Delivered => "delivered",
            NotificationStatus::PartiallyDelivered => "partially_delivered",
            NotificationStatus::Failed => "failed",
            NotificationStatus::Expired => "expired",
            NotificationStatus::Cancelled => "cancelled",
        }
    }
}

// Domain Events are defined inline for now

// Placeholder for the events module - will be implemented next
mod placeholder {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationCreated {
        pub metadata: EventMetadata,
        pub recipient_user_id: Uuid,
        pub topic_name: Option<String>,
        pub notification_type: NotificationType,
        pub priority: NotificationPriority,
        pub status: NotificationStatus,
    }
    
    impl NotificationCreated {
        pub fn new(
            aggregate_id: String,
            aggregate_version: u64,
            recipient_user_id: Uuid,
            topic_name: Option<String>,
            notification_type: NotificationType,
            priority: NotificationPriority,
            status: NotificationStatus,
        ) -> Self {
            Self {
                metadata: EventMetadata::new(aggregate_id, aggregate_version),
                recipient_user_id,
                topic_name,
                notification_type,
                priority,
                status,
            }
        }
    }

    impl DomainEvent for NotificationCreated {
        fn event_id(&self) -> Uuid { self.metadata.event_id }
        fn event_type(&self) -> &'static str { "NotificationCreated" }
        fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
        fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
        fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
        fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
            Ok(serde_json::to_string(self)?)
        }
    }
}

// Temporarily use placeholder events until events module is created
pub use placeholder::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationScheduled {
    pub metadata: EventMetadata,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub status: NotificationStatus,
}

impl NotificationScheduled {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        scheduled_at: Option<DateTime<Utc>>,
        status: NotificationStatus,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            scheduled_at,
            status,
        }
    }
}

impl DomainEvent for NotificationScheduled {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationScheduled" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSending {
    pub metadata: EventMetadata,
    pub channel_count: u32,
}

impl NotificationSending {
    pub fn new(aggregate_id: String, aggregate_version: u64, channel_count: u32) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            channel_count,
        }
    }
}

impl DomainEvent for NotificationSending {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationSending" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDeliveryCompleted {
    pub metadata: EventMetadata,
    pub status: NotificationStatus,
    pub successful_channels: Vec<String>,
    pub failed_channels: Vec<String>,
}

impl NotificationDeliveryCompleted {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        status: NotificationStatus,
        successful_channels: Vec<String>,
        failed_channels: Vec<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            status,
            successful_channels,
            failed_channels,
        }
    }
}

impl DomainEvent for NotificationDeliveryCompleted {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationDeliveryCompleted" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationExpired {
    pub metadata: EventMetadata,
    pub expired_at: Option<DateTime<Utc>>,
}

impl NotificationExpired {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        expired_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            expired_at,
        }
    }
}

impl DomainEvent for NotificationExpired {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationExpired" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPriorityUpdated {
    pub metadata: EventMetadata,
    pub old_priority: NotificationPriority,
    pub new_priority: NotificationPriority,
}

impl NotificationPriorityUpdated {
    pub fn new(
        aggregate_id: String,
        aggregate_version: u64,
        old_priority: NotificationPriority,
        new_priority: NotificationPriority,
    ) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            old_priority,
            new_priority,
        }
    }
}

impl DomainEvent for NotificationPriorityUpdated {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationPriorityUpdated" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationCancelled {
    pub metadata: EventMetadata,
    pub reason: String,
}

impl NotificationCancelled {
    pub fn new(aggregate_id: String, aggregate_version: u64, reason: String) -> Self {
        Self {
            metadata: EventMetadata::new(aggregate_id, aggregate_version),
            reason,
        }
    }
}

impl DomainEvent for NotificationCancelled {
    fn event_id(&self) -> Uuid { self.metadata.event_id }
    fn event_type(&self) -> &'static str { "NotificationCancelled" }
    fn occurred_at(&self) -> DateTime<Utc> { self.metadata.occurred_at }
    fn aggregate_version(&self) -> u64 { self.metadata.aggregate_version }
    fn aggregate_id(&self) -> String { self.metadata.aggregate_id.clone() }
    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(self)?)
    }
}