//! Notification port adapters.
//!
//! Wave 10 service-boundary refactor. The current default remains the
//! in-process adapter (`in_process_adapter`); the `http_adapter` is
//! available as an explicit remote mode while cutover evidence is gathered.

pub mod http_adapter;
pub mod in_process_adapter;

pub use http_adapter::HttpNotificationAdapter;
pub use in_process_adapter::InProcessNotificationAdapter;

use epsx_contracts::errors::{AppError, AppResult};
use epsx_contracts::notification_port::NotificationPort;
use epsx_contracts::pubsub_port::PubsubPort;
use std::sync::Arc;

/// Production and explicitly remote-configured processes must not boot
/// without a notification adapter. A missing port would otherwise let
/// publisher call sites silently drop events while the backend is healthy.
pub fn notification_adapter_required() -> bool {
    let adapter = std::env::var("NOTIFICATION_ADAPTER").ok();
    let environments = ["EPSX_ENV", "RUST_ENV", "NODE_ENV"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .collect::<Vec<_>>();
    notification_adapter_required_for_values(
        adapter.as_deref(),
        environments.iter().map(String::as_str),
    )
}

fn notification_adapter_is_remote(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("remote"))
}

fn notification_adapter_value_is_supported(value: Option<&str>) -> bool {
    value.is_none()
        || value.is_some_and(|value| {
            value.eq_ignore_ascii_case("remote") || value.eq_ignore_ascii_case("in_process")
        })
}

fn notification_adapter_required_for_values<'a>(
    adapter: Option<&str>,
    environments: impl IntoIterator<Item = &'a str>,
) -> bool {
    !notification_adapter_value_is_supported(adapter)
        || notification_adapter_is_remote(adapter)
        || environments
            .into_iter()
            .any(|value| value.eq_ignore_ascii_case("production"))
}

/// Build the publisher port without changing any publisher call site. Remote
/// mode is explicit; the default remains the legacy in-process adapter until
/// N3 cutover and reconciliation are approved.
pub async fn build_notification_port(
    pubsub: Option<Arc<dyn PubsubPort>>,
) -> AppResult<Arc<dyn NotificationPort>> {
    let configured = std::env::var("NOTIFICATION_ADAPTER").ok();
    if !notification_adapter_value_is_supported(configured.as_deref()) {
        return Err(AppError::configuration_error(
            "NOTIFICATION_ADAPTER must be remote or in_process",
        ));
    }
    if notification_adapter_is_remote(configured.as_deref()) {
        Ok(Arc::new(HttpNotificationAdapter::from_env()?))
    } else {
        Ok(Arc::new(
            InProcessNotificationAdapter::try_new(pubsub).await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        notification_adapter_is_remote, notification_adapter_required_for_values,
        notification_adapter_value_is_supported,
    };

    #[test]
    fn adapter_requirement_is_case_insensitive_for_remote_mode_and_production() {
        assert!(notification_adapter_required_for_values(Some("REMOTE"), []));
        assert!(notification_adapter_required_for_values(
            None,
            ["Production"]
        ));
        assert!(notification_adapter_required_for_values(
            None,
            ["development", "PRODUCTION"]
        ));
    }

    #[test]
    fn non_production_default_can_omit_adapter_for_harnesses() {
        assert!(!notification_adapter_required_for_values(
            None,
            ["development", "test"]
        ));
        assert!(!notification_adapter_required_for_values(
            Some("in_process"),
            []
        ));
    }

    #[test]
    fn remote_adapter_selection_matches_the_startup_requirement() {
        assert!(notification_adapter_is_remote(Some("remote")));
        assert!(notification_adapter_is_remote(Some("REMOTE")));
        assert!(!notification_adapter_is_remote(Some("in_process")));
        assert!(!notification_adapter_is_remote(None));
    }

    #[test]
    fn unknown_adapter_configuration_is_not_silently_downgraded() {
        assert!(!notification_adapter_value_is_supported(Some("typo")));
        assert!(notification_adapter_required_for_values(Some("typo"), []));
        assert!(notification_adapter_value_is_supported(Some("IN_PROCESS")));
        assert!(notification_adapter_value_is_supported(None));
    }
}
