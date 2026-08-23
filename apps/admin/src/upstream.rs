//! Shared upstream failure taxonomy for admin data adapters.
//!
//! Every admin adapter classifies unsuccessful upstream responses through
//! [`classify`] so page state stays truthful: authentication failures are
//! never rendered as infrastructure outages.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpstreamFailure {
    /// The forwarded bearer was rejected upstream (expired/revoked token).
    Unauthorized,
    /// The verified session lacks the permission the route requires.
    Forbidden,
    /// The response violated the strict adapter contract.
    Malformed,
    /// Network failure or any other unsuccessful status (5xx/404/timeouts).
    Unavailable,
}

impl UpstreamFailure {
    pub(crate) fn classify(status: reqwest::StatusCode) -> Self {
        if status == reqwest::StatusCode::UNAUTHORIZED {
            Self::Unauthorized
        } else if status == reqwest::StatusCode::FORBIDDEN {
            Self::Forbidden
        } else if status == reqwest::StatusCode::BAD_REQUEST
            || status == reqwest::StatusCode::UNPROCESSABLE_ENTITY
        {
            Self::Malformed
        } else {
            Self::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UpstreamFailure;

    #[test]
    fn classification_is_exact_per_status_family() {
        use reqwest::StatusCode;
        assert_eq!(
            UpstreamFailure::classify(StatusCode::UNAUTHORIZED),
            UpstreamFailure::Unauthorized
        );
        assert_eq!(
            UpstreamFailure::classify(StatusCode::FORBIDDEN),
            UpstreamFailure::Forbidden
        );
        assert_eq!(
            UpstreamFailure::classify(StatusCode::BAD_REQUEST),
            UpstreamFailure::Malformed
        );
        assert_eq!(
            UpstreamFailure::classify(StatusCode::UNPROCESSABLE_ENTITY),
            UpstreamFailure::Malformed
        );
        for status in [
            StatusCode::NOT_FOUND,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::SERVICE_UNAVAILABLE,
            StatusCode::BAD_GATEWAY,
            StatusCode::GATEWAY_TIMEOUT,
        ] {
            assert_eq!(
                UpstreamFailure::classify(status),
                UpstreamFailure::Unavailable,
                "status {status}"
            );
        }
    }
}
