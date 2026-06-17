//! Minimal `infrastructure::cache::unified_permission_cache` shim
//! for `epsx-identity-shared`.
//!
//! The real backend has a full Redis-backed `UnifiedPermissionCache`
//! in `apps/backend/src/infrastructure/cache/unified_permission_cache.rs`
//! (380 lines). For the purposes of compiling
//! `unified_permission_service.rs` standalone, we expose the same
//! method surface but with no-op bodies. The backend's shim bridges
//! between this and the real cache at runtime.

use std::sync::Arc;

#[derive(Clone, Default)]
pub struct UnifiedPermissionCache {
    _placeholder: Arc<()>,
}

impl UnifiedPermissionCache {
    pub fn new(_redis: Arc<()>) -> Self {
        Self::default()
    }

    pub async fn get_permission_check(
        &self,
        _wallet_address: &str,
        _permission: &str,
    ) -> Option<bool> {
        None
    }

    pub async fn set_permission_check(
        &self,
        _wallet_address: &str,
        _permission: &str,
        _granted: bool,
    ) {
    }

    pub async fn get_wallet_permissions(
        &self,
        _wallet_address: &str,
    ) -> Option<Vec<super::unified_permission_service::PermissionDetail>> {
        None
    }

    pub async fn set_wallet_permissions(
        &self,
        _wallet_address: &str,
        _perms: &[super::unified_permission_service::PermissionDetail],
    ) {
    }

    pub async fn invalidate_wallet(&self, _wallet_address: &str) {
    }
}
