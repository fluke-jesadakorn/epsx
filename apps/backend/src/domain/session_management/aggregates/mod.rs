// Session Management Aggregates
// Aggregate roots for session lifecycle and collection management

pub mod user_session_manager;
// TODO: Implement session cleanup orchestrator as needed
// pub mod session_cleanup_orchestrator;

pub use user_session_manager::{UserSessionManager, SessionManagerError};
// TODO: Re-enable export once module is implemented
// pub use session_cleanup_orchestrator::{SessionCleanupOrchestrator, CleanupResult};