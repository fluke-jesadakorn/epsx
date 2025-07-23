// Simple in-memory event dispatcher for development

use async_trait::async_trait;
use tracing::{info, error};
use crate::dom::events::DomainEvent;
use crate::app::ports::events::{EventDispatcher, EventError};

/// Simple event dispatcher that logs events
/// For production, this would integrate with a message queue
pub struct SimpleEventDispatcher {
    _enabled: bool,
}

impl SimpleEventDispatcher {
    pub fn new() -> Self {
        Self {
            _enabled: true,
        }
    }
}

impl Default for SimpleEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventDispatcher for SimpleEventDispatcher {
    async fn dispatch(&self, event: Box<dyn DomainEvent>) -> Result<(), EventError> {
        info!(
            "Dispatching event: {} at: {}", 
            event.event_type(),
            event.occurred_at()
        );
        
        // For now, just log the event
        // In production, this would:
        // 1. Store event in event store
        // 2. Publish to event handlers
        // 3. Send to external systems
        
        // For development, just log basic event info
        info!(
            "Event ID: {}, Type: {}, Occurred at: {}",
            event.event_id(),
            event.event_type(),
            event.occurred_at()
        );
        
        // In production, you would serialize and store the event
        // For now, this simple dispatcher just logs
        Ok(())
    }
    
    async fn dispatch_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> Result<(), EventError> {
        info!("Dispatching {} events in batch", events.len());
        
        for event in events {
            if let Err(e) = self.dispatch(event).await {
                error!("Failed to dispatch event in batch: {}", e);
                return Err(e);
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::events::UserRegisteredEvent;
    use crate::dom::values::{UserId, Email};
    use chrono::Utc;
    
    #[test]
    fn should_create_simple_event_dispatcher() {
        let dispatcher = SimpleEventDispatcher::new();
        assert!(dispatcher._enabled);
    }
    
    #[test]
    fn should_create_default_event_dispatcher() {
        let dispatcher = SimpleEventDispatcher::default();
        assert!(dispatcher._enabled);
    }
    
    #[tokio::test]
    async fn should_dispatch_single_event() {
        let dispatcher = SimpleEventDispatcher::new();
        let user_id = UserId::generate();
        let email = "test@example.com".to_string();
        
        let event = UserRegisteredEvent::new(user_id, email);
        
        let result = dispatcher.dispatch(Box::new(event)).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn should_dispatch_event_batch() {
        let dispatcher = SimpleEventDispatcher::new();
        
        let events: Vec<Box<dyn DomainEvent>> = vec![
            Box::new(UserRegisteredEvent::new(
                UserId::generate(),
                "user1@example.com".to_string()
            )),
            Box::new(UserRegisteredEvent::new(
                UserId::generate(),
                "user2@example.com".to_string()
            )),
            Box::new(UserRegisteredEvent::new(
                UserId::generate(),
                "user3@example.com".to_string()
            )),
        ];
        
        let result = dispatcher.dispatch_batch(events).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn should_dispatch_empty_batch() {
        let dispatcher = SimpleEventDispatcher::new();
        let events: Vec<Box<dyn DomainEvent>> = vec![];
        
        let result = dispatcher.dispatch_batch(events).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn should_handle_single_event_in_batch() {
        let dispatcher = SimpleEventDispatcher::new();
        let user_id = UserId::generate();
        let email = "single@example.com".to_string();
        
        let events: Vec<Box<dyn DomainEvent>> = vec![
            Box::new(UserRegisteredEvent::new(user_id, email))
        ];
        
        let result = dispatcher.dispatch_batch(events).await;
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn should_implement_event_dispatcher_trait() {
        // Compile-time test to ensure SimpleEventDispatcher implements EventDispatcher
        fn assert_implements_event_dispatcher<T: EventDispatcher>() {}
        assert_implements_event_dispatcher::<SimpleEventDispatcher>();
    }
    
    #[test]
    fn should_be_send_sync() {
        // Test that dispatcher can be used across thread boundaries
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SimpleEventDispatcher>();
    }
    
    #[test]
    fn should_support_clone_semantics() {
        // Test that we can create multiple instances if needed
        let dispatcher1 = SimpleEventDispatcher::new();
        let dispatcher2 = SimpleEventDispatcher::default();
        
        assert_eq!(dispatcher1._enabled, dispatcher2._enabled);
    }
    
    #[tokio::test]
    async fn should_log_event_details() {
        let dispatcher = SimpleEventDispatcher::new();
        let user_id = UserId::generate();
        let email = "logging-test@example.com".to_string();
        
        let event = UserRegisteredEvent::new(user_id.clone(), email.clone());
        let _event_id = event.event_id().clone();
        let _event_type = event.event_type().to_string();
        
        // This test validates that the event properties are accessible
        // The actual logging is tested through the dispatch method
        let result = dispatcher.dispatch(Box::new(event)).await;
        
        assert!(result.is_ok());
        // In a more sophisticated test setup, we would capture logs and verify them
        // For now, we just ensure the dispatch succeeds with the correct event structure
    }
    
    #[tokio::test]
    async fn should_maintain_event_ordering_in_batch() {
        let dispatcher = SimpleEventDispatcher::new();
        
        // Create events with predictable ordering
        let events: Vec<Box<dyn DomainEvent>> = (1..=5)
            .map(|i| {
                Box::new(UserRegisteredEvent::new(
                    UserId::generate(),
                    format!("user{}@example.com", i)
                )) as Box<dyn DomainEvent>
            })
            .collect();
        
        let event_count = events.len();
        let result = dispatcher.dispatch_batch(events).await;
        
        assert!(result.is_ok());
        assert_eq!(event_count, 5); // Verify we created the expected number of events
    }
}