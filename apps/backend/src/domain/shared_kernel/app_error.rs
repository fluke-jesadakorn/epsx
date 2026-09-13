// kernel extraction wave9 — re-export shim (R4)
//
// `AppError` / `ErrorKind` / `AppResult` are now defined in
// `epsx_contracts::errors`. The original 821-LOC `AppError` enum-based
// type (with the Web3-specific variants and the `From<reqwest::Error>` /
// `From<diesel::result::Error>` / `From<serde_json::Error>` impls) has been
// collapsed: the canonical `AppError` is the struct-based one in
// `epsx_contracts::errors`, and the `From<diesel::result::Error>` /
// `From<serde_json::Error>` impls are already provided there.
//
// The `From<ValueObjectError>` impl that previously lived here now lives
// in `epsx_contracts::errors` (re-pointed at the new `crate::value_object`
// path inside that crate).
//
// Callers of `AppError::infrastructure_error(msg)` (a 1-arg helper that
// only existed on the old enum) were migrated to
// `AppError::internal_server_error(msg)` on the struct, which produces
// the equivalent `ErrorKind::InternalServerError` value.
//
// The `ErrorContext` struct from the old enum-based `AppError` (which
// had different fields: `error_id`, `component`, `operation`,
// `user_context`, `chain_id`, `environment`, `metadata`) is **not**
// re-exported. It has no external importers in the current tree; if a
// future migration needs it, that should be a separate task.

pub use epsx_contracts::errors::{AppError, AppResult, ErrorKind};
