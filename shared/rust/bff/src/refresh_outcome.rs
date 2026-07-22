//! Closed refresh-rotation outcome contract shared by the Rust BFFs.
//!
//! HTTP status alone cannot distinguish a pre-rotation dependency failure from
//! a response lost after a refresh transaction committed. The canonical
//! backend therefore attests the mutation boundary with a small, token-free
//! header. Any missing, invalid, or status-inconsistent value fails closed.

use axum::response::Response;
use http::{HeaderMap, StatusCode};

pub const REFRESH_OUTCOME_HEADER: &str = "x-epsx-refresh-outcome";
pub const REFRESH_OUTCOME_ROTATED: &str = "rotated";
pub const REFRESH_OUTCOME_NOT_ROTATED: &str = "not_rotated";
pub const REFRESH_OUTCOME_REJECTED: &str = "rejected";
pub const REFRESH_OUTCOME_UNKNOWN: &str = "outcome_unknown";
pub const SESSION_STATE_HEADER: &str = "x-epsx-session-state";
pub const SESSION_STATE_ROTATED: &str = "rotated";
pub const SESSION_STATE_PRESERVED: &str = "preserved";
pub const SESSION_STATE_CLEARED: &str = "cleared";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshDisposition {
    Replace,
    Preserve,
    Clear,
}

fn exact_refresh_outcome(headers: &HeaderMap) -> Option<&str> {
    let mut values = headers.get_all(REFRESH_OUTCOME_HEADER).iter();
    let outcome = values.next()?.to_str().ok()?;
    values.next().is_none().then_some(outcome)
}

/// Mark the BFF-to-browser result without exposing credentials or identity.
pub fn mark_session_state(response: &mut Response, disposition: RefreshDisposition) {
    let value = match disposition {
        RefreshDisposition::Replace => SESSION_STATE_ROTATED,
        RefreshDisposition::Preserve => SESSION_STATE_PRESERVED,
        RefreshDisposition::Clear => SESSION_STATE_CLEARED,
    };
    response
        .headers_mut()
        .insert(SESSION_STATE_HEADER, http::HeaderValue::from_static(value));
}

/// Classify the backend response without guessing from status alone.
///
/// Only exact status/header pairs are accepted. In particular, a raw 500 or
/// 503 is never considered proof that rotation did not commit.
pub fn classify_refresh_outcome(status: StatusCode, headers: &HeaderMap) -> RefreshDisposition {
    let outcome = exact_refresh_outcome(headers);

    match (status, outcome) {
        (StatusCode::OK, Some(REFRESH_OUTCOME_ROTATED)) => RefreshDisposition::Replace,
        (
            StatusCode::BAD_REQUEST
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::SERVICE_UNAVAILABLE,
            Some(REFRESH_OUTCOME_NOT_ROTATED),
        ) => RefreshDisposition::Preserve,
        (StatusCode::UNAUTHORIZED, Some(REFRESH_OUTCOME_REJECTED))
        | (
            StatusCode::INTERNAL_SERVER_ERROR | StatusCode::SERVICE_UNAVAILABLE,
            Some(REFRESH_OUTCOME_UNKNOWN),
        ) => RefreshDisposition::Clear,
        _ => RefreshDisposition::Clear,
    }
}

pub fn is_rejected_refresh_outcome(status: StatusCode, headers: &HeaderMap) -> bool {
    status == StatusCode::UNAUTHORIZED
        && exact_refresh_outcome(headers) == Some(REFRESH_OUTCOME_REJECTED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    fn headers(outcome: Option<&'static str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(outcome) = outcome {
            headers.insert(REFRESH_OUTCOME_HEADER, HeaderValue::from_static(outcome));
        }
        headers
    }

    #[test]
    fn accepts_only_exact_closed_status_and_outcome_pairs() {
        let cases = [
            (
                StatusCode::OK,
                REFRESH_OUTCOME_ROTATED,
                RefreshDisposition::Replace,
            ),
            (
                StatusCode::BAD_REQUEST,
                REFRESH_OUTCOME_NOT_ROTATED,
                RefreshDisposition::Preserve,
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                REFRESH_OUTCOME_NOT_ROTATED,
                RefreshDisposition::Preserve,
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                REFRESH_OUTCOME_NOT_ROTATED,
                RefreshDisposition::Preserve,
            ),
            (
                StatusCode::UNAUTHORIZED,
                REFRESH_OUTCOME_REJECTED,
                RefreshDisposition::Clear,
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                REFRESH_OUTCOME_UNKNOWN,
                RefreshDisposition::Clear,
            ),
        ];

        for (status, outcome, expected) in cases {
            assert_eq!(
                classify_refresh_outcome(status, &headers(Some(outcome))),
                expected
            );
        }
    }

    #[test]
    fn missing_invalid_and_intermediary_outcomes_fail_closed() {
        for status in [
            StatusCode::OK,
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::REQUEST_TIMEOUT,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::BAD_GATEWAY,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::GATEWAY_TIMEOUT,
            StatusCode::TEMPORARY_REDIRECT,
        ] {
            assert_eq!(
                classify_refresh_outcome(status, &headers(None)),
                RefreshDisposition::Clear
            );
            assert_eq!(
                classify_refresh_outcome(status, &headers(Some("invented"))),
                RefreshDisposition::Clear
            );
        }

        assert_eq!(
            classify_refresh_outcome(
                StatusCode::SERVICE_UNAVAILABLE,
                &headers(Some(REFRESH_OUTCOME_ROTATED)),
            ),
            RefreshDisposition::Clear
        );
        assert_eq!(
            classify_refresh_outcome(StatusCode::OK, &headers(Some(REFRESH_OUTCOME_NOT_ROTATED)),),
            RefreshDisposition::Clear
        );
    }

    #[test]
    fn raw_server_errors_never_imply_preservation() {
        for status in [
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
        ] {
            assert_eq!(
                classify_refresh_outcome(status, &headers(None)),
                RefreshDisposition::Clear
            );
            assert_eq!(
                classify_refresh_outcome(status, &headers(Some(REFRESH_OUTCOME_UNKNOWN))),
                RefreshDisposition::Clear
            );
        }
    }

    #[test]
    fn rejected_reason_requires_both_exact_status_and_marker() {
        assert!(is_rejected_refresh_outcome(
            StatusCode::UNAUTHORIZED,
            &headers(Some(REFRESH_OUTCOME_REJECTED)),
        ));
        assert!(!is_rejected_refresh_outcome(
            StatusCode::UNAUTHORIZED,
            &headers(None),
        ));
        assert!(!is_rejected_refresh_outcome(
            StatusCode::BAD_GATEWAY,
            &headers(Some(REFRESH_OUTCOME_REJECTED)),
        ));
    }

    #[test]
    fn duplicate_or_conflicting_outcome_headers_fail_closed() {
        let mut duplicate = headers(Some(REFRESH_OUTCOME_NOT_ROTATED));
        duplicate.append(
            REFRESH_OUTCOME_HEADER,
            HeaderValue::from_static(REFRESH_OUTCOME_NOT_ROTATED),
        );
        assert_eq!(
            classify_refresh_outcome(StatusCode::SERVICE_UNAVAILABLE, &duplicate),
            RefreshDisposition::Clear
        );

        let mut conflicting = headers(Some(REFRESH_OUTCOME_NOT_ROTATED));
        conflicting.append(
            REFRESH_OUTCOME_HEADER,
            HeaderValue::from_static(REFRESH_OUTCOME_UNKNOWN),
        );
        assert_eq!(
            classify_refresh_outcome(StatusCode::SERVICE_UNAVAILABLE, &conflicting),
            RefreshDisposition::Clear
        );
        assert!(!is_rejected_refresh_outcome(
            StatusCode::UNAUTHORIZED,
            &conflicting
        ));
    }

    #[test]
    fn browser_state_marker_is_closed_and_token_free() {
        for (disposition, expected) in [
            (RefreshDisposition::Replace, SESSION_STATE_ROTATED),
            (RefreshDisposition::Preserve, SESSION_STATE_PRESERVED),
            (RefreshDisposition::Clear, SESSION_STATE_CLEARED),
        ] {
            let mut response = Response::new(axum::body::Body::empty());
            mark_session_state(&mut response, disposition);
            assert_eq!(
                response
                    .headers()
                    .get(SESSION_STATE_HEADER)
                    .and_then(|value| value.to_str().ok()),
                Some(expected)
            );
            assert!(!expected.contains("token"));
            assert!(!expected.contains("bearer"));
        }
    }
}
