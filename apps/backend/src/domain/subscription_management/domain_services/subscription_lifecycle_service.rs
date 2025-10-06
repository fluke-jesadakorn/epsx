use crate::domain::subscription_management::{Subscription, Plan, BillingCycle};
use chrono::{Duration, Utc};

/// Domain service for subscription lifecycle management
pub struct SubscriptionLifecycleService;

impl SubscriptionLifecycleService {
    /// Calculate expiration date based on plan billing cycle
    pub fn calculate_expiration_date(
        plan: &Plan,
        start_date: chrono::DateTime<Utc>,
    ) -> Option<chrono::DateTime<Utc>> {
        match plan.billing_cycle().duration_days() {
            Some(days) => Some(start_date + Duration::days(days)),
            None => None, // Lifetime subscription
        }
    }

    /// Check if subscription should be renewed
    pub fn should_auto_renew(subscription: &Subscription) -> bool {
        if !subscription.auto_renew() {
            return false;
        }

        if let Some(expires_at) = subscription.expires_at() {
            let days_until_expiry = (expires_at - Utc::now()).num_days();
            days_until_expiry <= 7 // Renew within 7 days of expiry
        } else {
            false // Lifetime subscription
        }
    }

    /// Check if subscription has expired
    pub fn is_expired(subscription: &Subscription) -> bool {
        if let Some(expires_at) = subscription.expires_at() {
            Utc::now() > expires_at
        } else {
            false // Lifetime subscription never expires
        }
    }

    /// Calculate renewal date
    pub fn calculate_renewal_date(
        current_expires_at: chrono::DateTime<Utc>,
        billing_cycle: &BillingCycle,
    ) -> Option<chrono::DateTime<Utc>> {
        billing_cycle.duration_days()
            .map(|days| current_expires_at + Duration::days(days))
    }
}
