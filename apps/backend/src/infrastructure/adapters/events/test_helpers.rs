// Test helper module for wave 11 / Track C.
//
// `CapturingEventPublisher` is a thread-safe `EventPublisherPort`
// impl that records the event types it receives. Tests in the
// permission_management handlers use it to assert that the
// `PlanDeletedEvent` / `WalletAssignedToPlanEvent` /
// `WalletRemovedFromPlanEvent` (the 3 previously-orphaned events
// from R8) actually flow through the port when the handler runs.
//
// The helper is `pub` so the tests in other modules can use it.
// It is NOT included in the production binary (the file is in
// `#[cfg(test)] mod tests`).

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use epsx_contracts::domain_event::DomainEvent;
use epsx_contracts::errors::AppResult;
use epsx_contracts::event_publisher_port::EventPublisherPort;

#[derive(Debug, Clone)]
pub struct CapturedEvent {
    pub event_type: String,
    pub aggregate_id: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Default)]
pub struct CapturingEventPublisher {
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl CapturingEventPublisher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn captured(&self) -> Vec<CapturedEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.events.lock().unwrap().len()
    }

    pub fn captured_of_type(&self, event_type: &str) -> Vec<CapturedEvent> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }
}

#[async_trait]
impl EventPublisherPort for CapturingEventPublisher {
    async fn publish(&self, event: Box<dyn DomainEvent>) -> AppResult<()> {
        self.events.lock().unwrap().push(CapturedEvent {
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            occurred_at: event.occurred_at(),
        });
        Ok(())
    }
}
