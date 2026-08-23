//! Global, SSR-injected session truth for every admin page.
//!
//! The BFF writes exactly one value into [`SESSION_STATE_PARAM`] on every
//! render: `verified` (cryptographically verified bearer session), `fixture`
//! (UI-only dev/design bypass identity), or `anonymous`. Layout chrome reads
//! this instead of guessing from cookies.

pub const SESSION_STATE_PARAM: &str = "data_session_state";

pub const SESSION_STATE_VERIFIED: &str = "verified";
pub const SESSION_STATE_FIXTURE: &str = "fixture";
pub const SESSION_STATE_ANONYMOUS: &str = "anonymous";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    Verified,
    Fixture,
    Anonymous,
}

impl SessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => SESSION_STATE_VERIFIED,
            Self::Fixture => SESSION_STATE_FIXTURE,
            Self::Anonymous => SESSION_STATE_ANONYMOUS,
        }
    }

    pub fn from_params(params: &std::collections::HashMap<String, String>) -> Option<Self> {
        match params.get(SESSION_STATE_PARAM).map(String::as_str) {
            Some(SESSION_STATE_VERIFIED) => Some(Self::Verified),
            Some(SESSION_STATE_FIXTURE) => Some(Self::Fixture),
            Some(SESSION_STATE_ANONYMOUS) => Some(Self::Anonymous),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_state() {
        for state in [
            SessionState::Verified,
            SessionState::Fixture,
            SessionState::Anonymous,
        ] {
            let mut params = std::collections::HashMap::new();
            params.insert(SESSION_STATE_PARAM.to_string(), state.as_str().to_string());
            assert_eq!(SessionState::from_params(&params), Some(state));
        }
        assert_eq!(SessionState::from_params(&Default::default()), None);
    }
}
