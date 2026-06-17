// WEB3-FIRST AUTHENTICATION MODULE — WAVE 9 SHIM (TRACK B)
// kernel extraction wave9 — re-export shim
//
// The canonical source for this module lives in
// `shared/rust/epsx-identity-shared/`. This file is a re-export
// shim so the existing importers
// (`infrastructure/container/*`, `infrastructure/cache/*`,
// `web/...`, etc.) can keep their `use crate::auth::*` paths
// unchanged.
//
// Crate name note: the shared library is `epsx-identity-shared`,
// not `epsx-identity`, because the latter is already taken by
// `services/identity` (the identity service binary). The
// integration gate resolves this collision; the Track C
// `epsx-identity-stub` will be deleted on integration.

pub use epsx_identity_shared::*;

// Re-export submodule paths explicitly so `crate::auth::foo::Bar`
// and `crate::auth::Bar` (the latter from the `pub use` above) both
// resolve.
pub mod auth_service {
    pub use epsx_identity_shared::auth_service::*;
}
pub mod challenge_service {
    pub use epsx_identity_shared::challenge_service::*;
}
pub mod verification_service {
    pub use epsx_identity_shared::verification_service::*;
}
pub mod token_service {
    pub use epsx_identity_shared::token_service::*;
}
pub mod key_manager {
    pub use epsx_identity_shared::key_manager::*;
}
pub mod granular_permissions {
    pub use epsx_identity_shared::granular_permissions::*;
}
pub mod unified_permission_service {
    pub use epsx_identity_shared::unified_permission_service::*;
}
