// Wave 11 / Track C — `EventPublisherPort` (ROADMAP §5 R7).
//
// This is the kernel-level seam that replaces the 88 `Arc<dyn DomainEventBus>`
// direct references in the application command handler layer. The in-process
// adapter (`apps/backend/src/infrastructure/adapters/events/in_process_event_publisher.rs`)
// is a no-op stub that logs at `tracing::info!` — the bus is a no-op today
// per ROADMAP §6 trap 3, so the in-process impl is intentionally minimal.
//
// A future network impl (HTTP / gRPC / message-broker) is a wave-N+2 concern.
// The trait is the seam; the impl is replaceable.
//
// Object safety: `async fn` is not object-safe on stable Rust. We use
// `#[async_trait]` to desugar to `Pin<Box<dyn Future + Send>>`, which is
// the standard escape hatch. The `Send` bound on the returned future is
// explicit (matches the multi-threaded tokio runtime requirement for
// `Arc<dyn EventPublisherPort + Send + Sync>`).

use async_trait::async_trait;

use crate::domain_event::DomainEvent;
use crate::errors::AppResult;

/// Kernel-level port for publishing domain events. The contract is
/// intentionally thin: one method, async, returns the kernel's
/// `AppResult<()>`. The port does not know about the bus, Redis, or
/// any concrete transport — those are adapter concerns.
///
/// # Object safety
///
/// `EventPublisherPort` is object-safe (`Arc<dyn EventPublisherPort>` works
/// for DI). Verified at compile time by the `_assert_object_safe` test
/// in this file's `tests` module.
///
/// # `Box<dyn DomainEvent>` over `&dyn DomainEvent`
///
/// The port takes ownership (`Box<dyn DomainEvent>`) so it can move
/// the event into a `tokio::spawn` task (the in-process adapter
/// forwards to the legacy bus asynchronously). The two migration
/// shapes for the 19 call sites:
///
/// - **Single event, owned `let event = X::new(...)`**: pass
///   `Box::new(event)` directly. This is the 16-handler shape
///   (delete/assign/remove, the 3 orphan events, and the
///   notification / market_analytics / payment command handlers).
/// - **Borrowed slice `&[Box<dyn DomainEvent>]`** (e.g. iterating
///   over `Aggregate::uncommitted_events()`): convert each event
///   to an `OwnedEvent` via
///   `OwnedEvent::from_borrowed(&**event)`, then `Box::new(...)`.
///   The `OwnedEvent` wrapper preserves the event type name,
///   aggregate id, and JSON payload. The 3-handler shape
///   (create_payment, create_stock_analysis, create_eps_ranking
///   use this pattern).
#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    /// Publish a domain event. Returns `Err` only for transport-level
    /// failures (broker unreachable, serialization broken, etc.). The
    /// in-process adapter returns `Ok(())` unconditionally because
    /// the bus is a no-op today.
    async fn publish(&self, event: Box<dyn DomainEvent>) -> AppResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    /// Compile-time guarantee that `EventPublisherPort` is object-safe.
    /// If a future change to the trait breaks object safety, this
    /// `fn` will fail to compile.
    #[allow(dead_code)]
    fn _assert_object_safe(_: &dyn EventPublisherPort) {}

    /// The port returns the kernel's `AppResult<()>`. The trait must
    /// not be coupled to any concrete error type outside the kernel.
    #[allow(dead_code)]
    fn _assert_error_is_kernel_app_result(r: AppResult<()>) -> Result<(), AppError> {
        r
    }
}
