// Notification Query Helper - Centralized filter configuration
// Simple filter struct for notification queries

/// NotificationFilter - Centralized filter configuration
/// Used by wallet_notification_repository to build WHERE clauses
#[derive(Debug, Clone, Default)]
pub struct NotificationQueryFilter {
    pub wallet_address: Option<String>,
    pub notification_type: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
}

impl NotificationQueryFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wallet(mut self, addr: String) -> Self {
        self.wallet_address = Some(addr);
        self
    }

    pub fn notification_type(mut self, t: String) -> Self {
        self.notification_type = Some(t);
        self
    }

    pub fn priority(mut self, p: String) -> Self {
        self.priority = Some(p);
        self
    }

    pub fn status(mut self, s: String) -> Self {
        self.status = Some(s);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filter = NotificationQueryFilter::new()
            .wallet("0x123".to_string())
            .notification_type("system".to_string())
            .priority("high".to_string());

        assert_eq!(filter.wallet_address, Some("0x123".to_string()));
        assert_eq!(filter.notification_type, Some("system".to_string()));
        assert_eq!(filter.priority, Some("high".to_string()));
    }
}
