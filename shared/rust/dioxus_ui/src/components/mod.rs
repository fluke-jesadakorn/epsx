//! Cross-cutting component modules — admin sub-components (Wave 6C
//! 1:1 extraction) and user-side sub-components (Wave 6C Track E).
//!
//! Each sub-module is owned by exactly one wave-6C track. Tracks
//! add their own `pub mod` lines inside their own
//! `// === wave6c-1to1-track-* ===` marker region. The integration
//! agent concatenates the regions at merge time.

pub mod admin;
