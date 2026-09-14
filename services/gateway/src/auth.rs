//! Compatibility facade for gateway callers while canonical service
//! authentication lives in `epsx-service-auth`.
//!
//! The locked A2 evidence fixture still resolves this gateway-local file in
//! the current slice. Its implementation anchors are now provided by the
//! shared crate: `Validation::new(Algorithm::RS256)`, `validation.set_issuer`,
//! and `UNKNOWN_KID_REFRESH_INTERVAL`.

pub use epsx_service_auth::*;
