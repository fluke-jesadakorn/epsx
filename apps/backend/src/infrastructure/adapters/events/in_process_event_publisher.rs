// Wave 11 / Track C — in-process `EventPublisherPort` adapter (ROADMAP
// §5 R7).
//
// This is the in-tree impl. It is intentionally a no-op stub:
// (a) the in-process bus is a no-op today (per ROADMAP §6 trap 3 and the
//     payments audit §6, there is no queue / broker).
// (b) the audit (ROADMAP §6 trap 8) says newly-flowing events should not
//     surprise any consumer that was quietly relying on their absence.
//     The 3 previously-orphaned permission-management events
//     (`PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
//     `WalletRemovedFromPlanEvent`) are now being published. A real
//     consumer (a notification handler, an outbox projector) does not
//     exist today; the in-process adapter is a log line + an optional
//     `tokio::spawn` to the legacy `DomainEventBus` for backward
//     compatibility with the 19 handlers being migrated.
//
// Behaviour:
//   1. Always log at `tracing::info!` so observability works.
//   2. If a legacy `DomainEventBus` was provided at construction time,
//     forward the event via `tokio::spawn` so the publish call does
//     not block on the bus. The bus is best-effort: a forward
//     failure is logged at `tracing::warn!` and does not propagate.
//   3. Return `Ok(())` unconditionally — the in-process path is
//     infallible. Network impls (HTTP / gRPC) will return
//     `AppError::Infrastructure` on transport failures.
//
// The `bus: Option<Arc<dyn DomainEventBus>>` arg is a forward-compat
// seam: the wave-12 cleanup will delete the legacy bus, and this arg
// becomes `None` permanently. Until then, the bus carries the
// `published_events()` audit log for the 5 supporting tests in the
// notifications / permission_management test suites.

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use epsx_contracts::domain_event::{DomainEvent, DomainEventBus};
use epsx_contracts::errors::AppResult;
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// In-process implementation of `EventPublisherPort`. Logs every event
/// at `tracing::info!` and optionally forwards to a legacy
/// `DomainEventBus` (which is a no-op today, per ROADMAP §6 trap 3).
///
/// # Constructors
///
/// - `InProcessEventPublisher::new()` — pure log line, no bus. This
///   is the post-wave-12 shape.
/// - `InProcessEventPublisher::with_bus(bus)` — log line + bus
///   forward via `tokio::spawn`. This is the wave-11 shape that
///   preserves the pre-wave-11 `event_bus.publish(...)` behavior for
///   the 5 unit tests that assert on `SimpleEventBus::published_events()`.
pub struct InProcessEventPublisher {
    /// Legacy in-process bus. Optional: when `None`, the publisher is
    /// a pure log line.
    bus: Option<Arc<dyn DomainEventBus>>,
}

impl InProcessEventPublisher {
    /// Create a publisher with no legacy bus. Use this in the
    /// wave-12 / wave-N+2 cleanup when the bus is fully removed.
    pub fn new() -> Self {
        Self { bus: None }
    }

    /// Create a publisher that forwards to a legacy bus via
    /// `tokio::spawn`. The bus is best-effort: failures are logged
    /// and do not propagate.
    pub fn with_bus(bus: Arc<dyn DomainEventBus>) -> Self {
        Self { bus: Some(bus) }
    }
}

impl Default for InProcessEventPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventPublisherPort for InProcessEventPublisher {
    async fn publish(&self, event: Box<dyn DomainEvent>) -> AppResult<()> {
        // 1. Always log at info!. Use the event's `to_json` for the
        //    payload (already serializes per `DomainEvent::to_json`).
        let event_type = event.event_type();
        let aggregate_type = event.aggregate_type();
        let aggregate_id = event.aggregate_id();
        let occurred_at = event.occurred_at();
        let payload = event.to_json().unwrap_or_else(|e| {
            // The `DomainEvent::to_json` contract is a `Result<String,
            // Box<dyn Error>>`. The vast majority of events serialize
            // cleanly; for the rare case where a custom event type
            // fails, we log the event type and continue with a
            // placeholder so observability is never blocked.
            warn!(
                event_type = event_type,
                error = %e,
                "EventPublisherPort: failed to serialize event payload"
            );
            format!("\"failed-to-serialize: {}\"", e)
        });
        info!(
            target: "epsx::event_publisher",
            event_type = event_type,
            aggregate_type = aggregate_type,
            aggregate_id = %aggregate_id,
            occurred_at = %occurred_at,
            payload = %payload,
            "EventPublisherPort.publish (in-process, no-op)"
        );

        // 2. Forward to the legacy bus via tokio::spawn so the
        //    publish call does not block. The event is owned
        //    (`Box<dyn DomainEvent>`) so the `Send + 'static`
        //    bounds on `tokio::spawn` are satisfied.
        if let Some(bus) = self.bus.clone() {
            // We need the event as `&dyn DomainEvent` to call
            // `DomainEventBus::publish(&self, event: &dyn DomainEvent)`.
            // The event is a `Box<dyn DomainEvent>` (Send bound from
            // the trait method signature); the bus's `publish` is
            // sync and takes `&dyn DomainEvent`.
            let event_type_for_log = event_type;
            tokio::spawn(async move {
                let event_ref: &dyn DomainEvent = &*event;
                bus.publish(event_ref);
                info!(
                    target: "epsx::event_publisher",
                    event_type = event_type_for_log,
                    "InProcessEventPublisher: forwarded to legacy DomainEventBus"
                );
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use epsx_contracts::domain_event::{EventMetadata, InMemoryEventBus};
    use uuid::Uuid;

    /// Minimal event used by the round-trip tests. Wraps the
    /// `EventMetadata` + a static event type string.
    struct TestEvent {
        metadata: EventMetadata,
        event_type_str: &'static str,
    }

    impl std::fmt::Debug for TestEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TestEvent")
                .field("metadata", &self.metadata)
                .field("event_type", &self.event_type_str)
                .finish()
        }
    }

    impl TestEvent {
        fn new(event_type: &'static str, aggregate_id: String) -> Self {
            Self {
                metadata: EventMetadata {
                    event_id: Uuid::new_v4(),
                    occurred_at: Utc::now(),
                    aggregate_version: 1,
                    aggregate_id,
                },
                event_type_str: event_type,
            }
        }
    }

    impl DomainEvent for TestEvent {
        fn event_id(&self) -> Uuid {
            self.metadata.event_id
        }
        fn event_type(&self) -> &'static str {
            self.event_type_str
        }
        fn aggregate_type(&self) -> &'static str {
            "TestAggregate"
        }
        fn occurred_at(&self) -> chrono::DateTime<Utc> {
            self.metadata.occurred_at
        }
        fn aggregate_version(&self) -> u64 {
            self.metadata.aggregate_version
        }
        fn aggregate_id(&self) -> String {
            self.metadata.aggregate_id.clone()
        }
        fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
            Ok(serde_json::json!({
                "event_id": self.metadata.event_id.to_string(),
                "event_type": self.event_type_str,
                "aggregate_id": self.metadata.aggregate_id,
            })
            .to_string())
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[tokio::test]
    async fn port_trait_publish_round_trip_returns_ok() {
        // The in-process impl is infallible. The port trait's
        // contract says `AppResult<()>`, and the impl returns `Ok(())`
        // regardless of bus presence.
        let publisher = InProcessEventPublisher::new();
        let event: Box<dyn DomainEvent> =
            Box::new(TestEvent::new("test.event", "agg-1".to_string()));
        let result = publisher.publish(event).await;
        assert!(result.is_ok(), "InProcessEventPublisher::publish should return Ok(())");
    }

    #[tokio::test]
    async fn publisher_with_bus_forwards_via_tokio_spawn() {
        // The publisher must forward to the legacy bus. The bus
        // is synchronous; the publisher wraps it in `tokio::spawn`
        // so the publish call itself is non-blocking.
        let bus = Arc::new(InMemoryEventBus::new());
        let publisher = InProcessEventPublisher::with_bus(bus.clone());

        let event: Box<dyn DomainEvent> =
            Box::new(TestEvent::new("test.forwarded", "agg-2".to_string()));
        let result = publisher.publish(event).await;
        assert!(result.is_ok());

        // Give the spawned task a moment to complete. In production
        // this is a `tokio::yield_now().await`; in tests we use
        // `tokio::time::sleep` with a tiny budget.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let published = bus.published_events();
        assert!(
            published.contains(&"test.forwarded".to_string()),
            "Legacy bus should have received the forwarded event; got: {:?}",
            published
        );
    }

    #[tokio::test]
    async fn publisher_without_bus_does_not_panic() {
        // The pure-log-line shape (post-wave-12) must work with
        // no bus at all. No `tokio::spawn` call should be made.
        let publisher = InProcessEventPublisher::new();
        let event: Box<dyn DomainEvent> =
            Box::new(TestEvent::new("test.no_bus", "agg-3".to_string()));
        let result = publisher.publish(event).await;
        assert!(result.is_ok());
    }

    /// Object-safety smoke test: the publisher impl must be
    /// `Arc<dyn EventPublisherPort>`-compatible. This is the
    /// same shape that the container and the 19 migrated
    /// application command handlers use.
    #[tokio::test]
    async fn publisher_is_object_safe_via_dyn() {
        let publisher: Arc<dyn EventPublisherPort> = Arc::new(InProcessEventPublisher::new());
        let event: Box<dyn DomainEvent> =
            Box::new(TestEvent::new("test.dyn", "agg-4".to_string()));
        let result = publisher.publish(event).await;
        assert!(result.is_ok());
    }

    /// Compile-time guard: the test event type satisfies the
    /// `'static` bound implied by `Box<dyn DomainEvent + Send>`.
    #[allow(dead_code)]
    fn _assert_event_is_send_static(event: Box<dyn DomainEvent>) {
        fn _requires_send<T: Send + 'static>(_: T) {}
        _requires_send(event);
    }

    // =================================================================
    // wave11(track-c): 3 orphan event capture tests.
    //
    // The previously-orphaned permission-management events
    // (`PlanDeletedEvent`, `WalletAssignedToPlanEvent`,
    // `WalletRemovedFromPlanEvent`) are now published via the
    // `EventPublisherPort`. The in-process adapter is a no-op
    // stub today, so we test the publish path end-to-end with
    // a `CapturingEventPublisher` mock and the actual orphan
    // event types imported from the `permission_management`
    // domain module.
    //
    // These tests live in the `events` adapter module (not the
    // handler modules) because they only need the publisher +
    // the event types, not the repository ports. The handler
    // integration tests would need mock repositories for
    // `PermissionPlanRepositoryPort` and
    // `PlanAssignmentRepositoryPort` — out of scope for this
    // track (the 3 handler modules are part of the wave-12
    // cleanup; today they are not used by the web layer per
    // the wave-8 audit).
    // =================================================================
    mod orphan_event_tests {
        use super::*;
        use crate::infrastructure::adapters::events::CapturingEventPublisher;
        use crate::domain::permission_management::events::{
            PlanDeletedEvent, WalletAssignedToPlanEvent, WalletRemovedFromPlanEvent,
        };
        use std::sync::Arc;

        /// The in-process publisher routes a `PlanDeletedEvent`
        /// through its `tracing::info!` log line and (optionally)
        /// the legacy bus. When the publisher is constructed
        /// without a bus, the event reaches the `tracing::info!`
        /// log and `Ok(())` is returned.
        #[tokio::test]
        async fn plan_deleted_event_publishes_via_in_process_publisher() {
            let publisher = InProcessEventPublisher::new();
            let event: Box<dyn DomainEvent> = Box::new(PlanDeletedEvent::new(
                "plan-1".to_string(),
                0,
                "plan-1".to_string(),
                chrono::Utc::now(),
            ));
            let result = publisher.publish(event).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn wallet_assigned_event_publishes_via_in_process_publisher() {
            let publisher = InProcessEventPublisher::new();
            let event: Box<dyn DomainEvent> = Box::new(WalletAssignedToPlanEvent::new(
                "plan-1".to_string(),
                0,
                "plan-1".to_string(),
                "0xtest".to_string(),
                chrono::Utc::now(),
            ));
            let result = publisher.publish(event).await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn wallet_removed_event_publishes_via_in_process_publisher() {
            let publisher = InProcessEventPublisher::new();
            let event: Box<dyn DomainEvent> = Box::new(WalletRemovedFromPlanEvent::new(
                "plan-1".to_string(),
                0,
                "plan-1".to_string(),
                "0xtest".to_string(),
                chrono::Utc::now(),
            ));
            let result = publisher.publish(event).await;
            assert!(result.is_ok());
        }

        /// The `CapturingEventPublisher` mock records the event
        /// type. The 3 orphan events flow through the port and
        /// arrive at the mock with the correct `event_type()`
        /// header.
        #[tokio::test]
        async fn all_three_orphan_events_captured_by_mock_publisher() {
            let mock = Arc::new(CapturingEventPublisher::new());
            let publisher: Arc<dyn EventPublisherPort> = mock.clone();

            let plan_event: Box<dyn DomainEvent> = Box::new(PlanDeletedEvent::new(
                "plan-2".to_string(),
                0,
                "plan-2".to_string(),
                chrono::Utc::now(),
            ));
            publisher.publish(plan_event).await.unwrap();

            let assign_event: Box<dyn DomainEvent> = Box::new(WalletAssignedToPlanEvent::new(
                "plan-2".to_string(),
                0,
                "plan-2".to_string(),
                "0xassign".to_string(),
                chrono::Utc::now(),
            ));
            publisher.publish(assign_event).await.unwrap();

            let remove_event: Box<dyn DomainEvent> = Box::new(WalletRemovedFromPlanEvent::new(
                "plan-2".to_string(),
                0,
                "plan-2".to_string(),
                "0xremove".to_string(),
                chrono::Utc::now(),
            ));
            publisher.publish(remove_event).await.unwrap();

            let captured = mock.captured();
            assert_eq!(captured.len(), 3, "all 3 orphan events should be captured");
            // The event_type() returns the short name (no "Event" suffix):
            //   PlanDeletedEvent -> "PlanDeleted"
            //   WalletAssignedToPlanEvent -> "WalletAssignedToPlan"
            //   WalletRemovedFromPlanEvent -> "WalletRemovedFromPlan"
            assert_eq!(captured[0].event_type, "PlanDeleted");
            assert_eq!(captured[1].event_type, "WalletAssignedToPlan");
            assert_eq!(captured[2].event_type, "WalletRemovedFromPlan");
        }
    }
}
