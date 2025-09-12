use chrono::{DateTime, Utc, Duration};
use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};

/// Schedule Information Value Object
/// Handles notification scheduling, expiry, and timing logic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleInfo {
    scheduled_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    schedule_type: ScheduleType,
    timezone_hint: Option<String>,
}

impl ScheduleInfo {
    /// Create immediate delivery schedule
    pub fn immediate() -> Self {
        Self {
            scheduled_at: None,
            expires_at: None,
            created_at: Utc::now(),
            schedule_type: ScheduleType::Immediate,
            timezone_hint: None,
        }
    }

    /// Create scheduled delivery
    pub fn scheduled(scheduled_at: DateTime<Utc>) -> Result<Self, String> {
        let now = Utc::now();
        
        if scheduled_at <= now {
            return Err("Scheduled time must be in the future".to_string());
        }

        // Don't allow scheduling too far in the future (1 year limit)
        if scheduled_at > now + Duration::days(365) {
            return Err("Cannot schedule notifications more than 1 year in advance".to_string());
        }

        Ok(Self {
            scheduled_at: Some(scheduled_at),
            expires_at: None,
            created_at: now,
            schedule_type: ScheduleType::Scheduled,
            timezone_hint: None,
        })
    }

    /// Create scheduled delivery with expiry
    pub fn scheduled_with_expiry(
        scheduled_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        let mut schedule = Self::scheduled(scheduled_at)?;
        schedule.set_expires_at(expires_at)?;
        Ok(schedule)
    }

    /// Create with expiry only (immediate delivery but expires)
    pub fn with_expiry(expires_at: DateTime<Utc>) -> Result<Self, String> {
        let now = Utc::now();
        
        if expires_at <= now {
            return Err("Expiry time must be in the future".to_string());
        }

        Ok(Self {
            scheduled_at: None,
            expires_at: Some(expires_at),
            created_at: now,
            schedule_type: ScheduleType::Immediate,
            timezone_hint: None,
        })
    }

    /// Set expiry time
    pub fn set_expires_at(&mut self, expires_at: DateTime<Utc>) -> Result<(), String> {
        let now = Utc::now();
        
        if expires_at <= now {
            return Err("Expiry time must be in the future".to_string());
        }

        // If scheduled, expiry must be after scheduled time
        if let Some(scheduled) = self.scheduled_at {
            if expires_at <= scheduled {
                return Err("Expiry time must be after scheduled time".to_string());
            }
        }

        self.expires_at = Some(expires_at);
        Ok(())
    }

    /// Set timezone hint for display purposes
    pub fn with_timezone_hint(mut self, timezone: String) -> Self {
        self.timezone_hint = Some(timezone);
        self
    }

    /// Get scheduled time
    pub fn scheduled_at(&self) -> Option<DateTime<Utc>> {
        self.scheduled_at
    }

    /// Get expiry time
    pub fn expires_at(&self) -> Option<DateTime<Utc>> {
        self.expires_at
    }

    /// Get creation time
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Get schedule type
    pub fn schedule_type(&self) -> &ScheduleType {
        &self.schedule_type
    }

    /// Get timezone hint
    pub fn timezone_hint(&self) -> Option<&str> {
        self.timezone_hint.as_deref()
    }

    /// Check if notification is ready to be sent
    pub fn is_ready_to_send(&self) -> bool {
        match self.schedule_type {
            ScheduleType::Immediate => !self.is_expired(),
            ScheduleType::Scheduled => {
                let now = Utc::now();
                if let Some(scheduled) = self.scheduled_at {
                    now >= scheduled && !self.is_expired()
                } else {
                    false
                }
            }
        }
    }

    /// Check if notification has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if notification is currently scheduled (future)
    pub fn is_scheduled(&self) -> bool {
        if let Some(scheduled) = self.scheduled_at {
            Utc::now() < scheduled
        } else {
            false
        }
    }

    /// Get time until scheduled delivery
    pub fn time_until_scheduled(&self) -> Option<Duration> {
        if let Some(scheduled) = self.scheduled_at {
            let now = Utc::now();
            if scheduled > now {
                Some(scheduled - now)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get time until expiry
    pub fn time_until_expiry(&self) -> Option<Duration> {
        if let Some(expires_at) = self.expires_at {
            let now = Utc::now();
            if expires_at > now {
                Some(expires_at - now)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get current status
    pub fn status(&self) -> ScheduleStatus {
        let now = Utc::now();
        
        if self.is_expired() {
            return ScheduleStatus::Expired;
        }

        match self.schedule_type {
            ScheduleType::Immediate => ScheduleStatus::ReadyToSend,
            ScheduleType::Scheduled => {
                if let Some(scheduled) = self.scheduled_at {
                    if now >= scheduled {
                        ScheduleStatus::ReadyToSend
                    } else {
                        ScheduleStatus::Scheduled
                    }
                } else {
                    ScheduleStatus::ReadyToSend
                }
            }
        }
    }

    /// Get priority adjustment based on timing
    pub fn timing_priority_adjustment(&self) -> i8 {
        match self.schedule_type {
            ScheduleType::Immediate => {
                // Immediate notifications get slight priority boost
                1
            }
            ScheduleType::Scheduled => {
                if let Some(time_until) = self.time_until_scheduled() {
                    if time_until <= Duration::minutes(5) {
                        // High priority for notifications due very soon
                        2
                    } else if time_until <= Duration::hours(1) {
                        // Medium priority for notifications due within an hour
                        1
                    } else {
                        // Normal priority for future notifications
                        0
                    }
                } else {
                    // Already due scheduled notifications get high priority
                    2
                }
            }
        }
    }

    /// Check if notification should be delivered during quiet hours
    pub fn should_respect_quiet_hours(&self) -> bool {
        // Scheduled notifications always respect quiet hours
        // Immediate notifications only respect quiet hours for non-urgent content
        match self.schedule_type {
            ScheduleType::Immediate => false, // Let the caller decide based on priority
            ScheduleType::Scheduled => true,
        }
    }

    /// Get suggested delivery window
    pub fn suggested_delivery_window(&self) -> DeliveryWindow {
        let now = Utc::now();
        
        match self.schedule_type {
            ScheduleType::Immediate => DeliveryWindow {
                start: now,
                end: now + Duration::minutes(5), // 5 minute window for immediate
            },
            ScheduleType::Scheduled => {
                if let Some(scheduled) = self.scheduled_at {
                    DeliveryWindow {
                        start: scheduled,
                        end: scheduled + Duration::minutes(15), // 15 minute window for scheduled
                    }
                } else {
                    DeliveryWindow {
                        start: now,
                        end: now + Duration::minutes(5),
                    }
                }
            }
        }
    }

    /// Create default expiry time based on schedule type and creation time
    pub fn with_default_expiry(mut self) -> Self {
        if self.expires_at.is_none() {
            let default_expiry = match self.schedule_type {
                ScheduleType::Immediate => self.created_at + Duration::days(7), // 1 week for immediate
                ScheduleType::Scheduled => {
                    if let Some(scheduled) = self.scheduled_at {
                        scheduled + Duration::days(3) // 3 days after scheduled time
                    } else {
                        self.created_at + Duration::days(7)
                    }
                }
            };
            self.expires_at = Some(default_expiry);
        }
        self
    }
}

impl Default for ScheduleInfo {
    fn default() -> Self {
        Self::immediate()
    }
}

/// Schedule types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleType {
    Immediate,
    Scheduled,
}

impl ScheduleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleType::Immediate => "immediate",
            ScheduleType::Scheduled => "scheduled",
        }
    }
}

/// Current schedule status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleStatus {
    Scheduled,   // Waiting for scheduled time
    ReadyToSend, // Ready to be sent now
    Expired,     // Past expiry time
}

impl ScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScheduleStatus::Scheduled => "scheduled",
            ScheduleStatus::ReadyToSend => "ready_to_send",
            ScheduleStatus::Expired => "expired",
        }
    }
}

/// Delivery time window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DeliveryWindow {
    /// Check if current time is within delivery window
    pub fn is_current(&self) -> bool {
        let now = Utc::now();
        now >= self.start && now <= self.end
    }

    /// Get duration of the window
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

impl Display for ScheduleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.schedule_type {
            ScheduleType::Immediate => {
                if let Some(expires_at) = self.expires_at {
                    write!(f, "Immediate (expires {})", expires_at.format("%Y-%m-%d %H:%M UTC"))
                } else {
                    write!(f, "Immediate")
                }
            }
            ScheduleType::Scheduled => {
                if let Some(scheduled_at) = self.scheduled_at {
                    if let Some(expires_at) = self.expires_at {
                        write!(f, "Scheduled {} (expires {})", 
                            scheduled_at.format("%Y-%m-%d %H:%M UTC"),
                            expires_at.format("%Y-%m-%d %H:%M UTC"))
                    } else {
                        write!(f, "Scheduled {}", scheduled_at.format("%Y-%m-%d %H:%M UTC"))
                    }
                } else {
                    write!(f, "Scheduled (invalid)")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_immediate_schedule() {
        let schedule = ScheduleInfo::immediate();
        assert_eq!(schedule.schedule_type(), &ScheduleType::Immediate);
        assert!(schedule.is_ready_to_send());
        assert!(!schedule.is_scheduled());
        assert!(!schedule.is_expired());
    }

    #[test]
    fn test_scheduled_future() {
        let future_time = Utc::now() + Duration::hours(1);
        let schedule = ScheduleInfo::scheduled(future_time).unwrap();
        
        assert_eq!(schedule.schedule_type(), &ScheduleType::Scheduled);
        assert!(!schedule.is_ready_to_send());
        assert!(schedule.is_scheduled());
        assert!(!schedule.is_expired());
    }

    #[test]
    fn test_scheduled_past() {
        let past_time = Utc::now() - Duration::hours(1);
        let result = ScheduleInfo::scheduled(past_time);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be in the future"));
    }

    #[test]
    fn test_scheduled_too_far_future() {
        let far_future = Utc::now() + Duration::days(400);
        let result = ScheduleInfo::scheduled(far_future);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("more than 1 year"));
    }

    #[test]
    fn test_with_expiry() {
        let expiry_time = Utc::now() + Duration::hours(2);
        let schedule = ScheduleInfo::with_expiry(expiry_time).unwrap();
        
        assert_eq!(schedule.expires_at(), Some(expiry_time));
        assert!(schedule.is_ready_to_send());
        assert!(!schedule.is_expired());
    }

    #[test]
    fn test_scheduled_with_expiry() {
        let scheduled_time = Utc::now() + Duration::hours(1);
        let expiry_time = Utc::now() + Duration::hours(2);
        
        let schedule = ScheduleInfo::scheduled_with_expiry(scheduled_time, expiry_time).unwrap();
        
        assert_eq!(schedule.scheduled_at(), Some(scheduled_time));
        assert_eq!(schedule.expires_at(), Some(expiry_time));
        assert!(!schedule.is_ready_to_send()); // Not yet time to send
        assert!(!schedule.is_expired());
    }

    #[test]
    fn test_invalid_expiry() {
        let scheduled_time = Utc::now() + Duration::hours(2);
        let expiry_time = Utc::now() + Duration::hours(1); // Before scheduled
        
        let result = ScheduleInfo::scheduled_with_expiry(scheduled_time, expiry_time);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("after scheduled time"));
    }

    #[test]
    fn test_expired_notification() {
        let past_expiry = Utc::now() - Duration::minutes(1);
        let schedule = ScheduleInfo::immediate();
        
        // Set expiry in the past (bypassing validation for testing)
        schedule.expires_at = Some(past_expiry);
        
        assert!(schedule.is_expired());
        assert!(!schedule.is_ready_to_send());
        assert_eq!(schedule.status(), ScheduleStatus::Expired);
    }

    #[test]
    fn test_time_until_calculations() {
        let scheduled_time = Utc::now() + Duration::hours(1);
        let expiry_time = Utc::now() + Duration::hours(2);
        
        let schedule = ScheduleInfo::scheduled_with_expiry(scheduled_time, expiry_time).unwrap();
        
        let time_until_scheduled = schedule.time_until_scheduled().unwrap();
        let time_until_expiry = schedule.time_until_expiry().unwrap();
        
        assert!(time_until_scheduled <= Duration::hours(1));
        assert!(time_until_expiry <= Duration::hours(2));
        assert!(time_until_expiry > time_until_scheduled);
    }

    #[test]
    fn test_priority_adjustment() {
        let immediate = ScheduleInfo::immediate();
        assert!(immediate.timing_priority_adjustment() > 0);
        
        let far_future = Utc::now() + Duration::hours(5);
        let scheduled = ScheduleInfo::scheduled(far_future).unwrap();
        assert_eq!(scheduled.timing_priority_adjustment(), 0);
        
        let soon = Utc::now() + Duration::minutes(2);
        let scheduled_soon = ScheduleInfo::scheduled(soon).unwrap();
        assert!(scheduled_soon.timing_priority_adjustment() > 0);
    }

    #[test]
    fn test_delivery_window() {
        let schedule = ScheduleInfo::immediate();
        let window = schedule.suggested_delivery_window();
        
        assert!(window.duration() <= Duration::minutes(5));
        assert!(window.is_current() || window.start <= Utc::now());
    }

    #[test]
    fn test_status() {
        let immediate = ScheduleInfo::immediate();
        assert_eq!(immediate.status(), ScheduleStatus::ReadyToSend);
        
        let future_time = Utc::now() + Duration::hours(1);
        let scheduled = ScheduleInfo::scheduled(future_time).unwrap();
        assert_eq!(scheduled.status(), ScheduleStatus::Scheduled);
    }

    #[test]
    fn test_default_expiry() {
        let schedule = ScheduleInfo::immediate().with_default_expiry();
        assert!(schedule.expires_at().is_some());
        assert!(schedule.expires_at().unwrap() > Utc::now());
    }

    #[test]
    fn test_timezone_hint() {
        let schedule = ScheduleInfo::immediate().with_timezone_hint("America/New_York".to_string());
        assert_eq!(schedule.timezone_hint(), Some("America/New_York"));
    }
}