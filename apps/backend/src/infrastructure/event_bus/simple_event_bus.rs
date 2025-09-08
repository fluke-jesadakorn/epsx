use std::sync::{Arc, RwLock};
use tracing::info;

use crate::domain::shared_kernel::{DomainEvent, DomainEventBus};

/// Simple implementation of domain event bus that stores events in memory
pub struct SimpleEventBus {
    events: Arc<RwLock<Vec<String>>>, // Store event types for now
}

impl SimpleEventBus {
    pub fn new() -> Self {
        Self { 
            events: Arc::new(RwLock::new(Vec::new()))
        }
    }
    
    /// Get published event types (for testing/debugging)
    pub fn published_events(&self) -> Vec<String> {
        let events = self.events.read().unwrap();
        events.clone()
    }
    
    /// Clear all stored events
    pub fn clear(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }
}

impl Default for SimpleEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainEventBus for SimpleEventBus {
    fn publish(&self, event: &Box<dyn DomainEvent>) {
        let event_type = event.event_type();
        info!("Publishing domain event: {}", event_type);
        
        // Store event type for tracking
        if let Ok(mut events) = self.events.write() {
            events.push(event_type.to_string());
        }
        
        // In a real implementation, this would publish to message queue, etc.
        info!("Domain event '{}' published successfully", event_type);
    }
    
    fn publish_batch(&self, events: &[Box<dyn DomainEvent>]) {
        for event in events {
            self.publish(event);
        }
    }
}