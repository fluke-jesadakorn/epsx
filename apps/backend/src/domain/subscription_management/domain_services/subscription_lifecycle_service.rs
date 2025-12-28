use crate::domain::subscription_management::{Subscription, Plan, BillingCycle};
use chrono::{Duration, Utc};

/// Domain service for subscription lifecycle management
/// Simplified for pay-to-extend model (no auto-renewal)
pub struct SubscriptionLifecycleService;

impl SubscriptionLifecycleService {
    /// Calculate expiration date based on plan billing cycle
    pub fn calculate_expiration_date(
        plan: &Plan,
        start_date: chrono::DateTime<Utc>,
    ) -> Option<chrono::DateTime<Utc>> {
        plan.billing_cycle().duration_days().map(|days| start_date + Duration::days(days))
    }

    /// Get default plan duration (30 days for most plans)
    pub fn get_default_duration() -> Duration {
        Duration::days(30)
    }

    /// Calculate extension duration based on plan billing cycle
    pub fn calculate_extension_duration(billing_cycle: &BillingCycle) -> Duration {
        billing_cycle.duration_days()
            .map(|days| Duration::days(days))
            .unwrap_or_else(Self::get_default_duration)
    }

    /// Check if subscription has expired
    pub fn is_expired(subscription: &Subscription) -> bool {
        if let Some(expires_at) = subscription.expires_at() {
            Utc::now() > expires_at
        } else {
            false // Lifetime subscription never expires
        }
    }

    /// Check if subscription is expiring soon (within threshold days)
    pub fn is_expiring_soon(subscription: &Subscription, threshold_days: i64) -> bool {
        if let Some(days_remaining) = subscription.days_until_expiry() {
            days_remaining >= 0 && days_remaining <= threshold_days
        } else {
            false
        }
    }

    /// Get expiry status for display
    pub fn get_expiry_status(subscription: &Subscription) -> ExpiryStatus {
        match subscription.days_until_expiry() {
            Some(days) if days < 0 => ExpiryStatus::Expired { days_ago: -days },
            Some(days) if days <= 7 => ExpiryStatus::ExpiringSoon { days_remaining: days },
            Some(days) => ExpiryStatus::Active { days_remaining: days },
            None => ExpiryStatus::Lifetime,
        }
    }
}

/// Expiry status for UI display
#[derive(Debug, Clone)]
pub enum ExpiryStatus {
    Active { days_remaining: i64 },
    ExpiringSoon { days_remaining: i64 },
    Expired { days_ago: i64 },
    Lifetime,
}
