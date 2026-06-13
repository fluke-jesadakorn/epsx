// Wave 11 / Track C — `EventPublisherPort` adapters.
//
// The in-process impl (`in_process_event_publisher`) is the
// production impl today. A future network impl (HTTP / gRPC) is
// a wave-N+2 concern — see `ROADMAP.md` §5 R7.

pub mod in_process_event_publisher;

// Migration table (file:line before/after for every migrated call
// site). Comments-only — no runtime effect. See the file body for
// the full table.
#[allow(dead_code)]
mod event_publisher_migration;

pub use in_process_event_publisher::InProcessEventPublisher;

// Test helper module. NOT included in the production binary —
// gated by `#[cfg(test)]` in the file body. Re-exported at the
// `events` module root so that tests in other crates / modules
// can `use infrastructure::adapters::events::CapturingEventPublisher`.
#[cfg(test)]
mod test_helpers;

#[cfg(test)]
pub use test_helpers::CapturingEventPublisher;
