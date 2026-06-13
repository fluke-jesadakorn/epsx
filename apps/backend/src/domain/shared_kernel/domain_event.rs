// kernel extraction wave9 — re-export shim.
// wave11(track-c): source-of-truth moved from
// `epsx_contracts::traits::domain_event` to the top-level
// `epsx_contracts::domain_event` (ROADMAP §5 R7). The old path is
// preserved as a `pub use` re-export in `epsx_contracts::traits` for
// the migration window; this shim re-exports from the new canonical
// path so the in-tree callers keep working.
pub use epsx_contracts::domain_event::*;
