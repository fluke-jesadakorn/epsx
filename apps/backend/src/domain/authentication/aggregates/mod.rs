// Authentication Aggregates
// Core business entities that maintain consistency and enforce invariants

pub mod authentication_session;
// TODO: Implement these authentication aggregates as needed
// pub mod oidc_client;
// pub mod token_pair;
// pub mod authorization_grant;

pub use authentication_session::{AuthenticationSession, AuthenticationError, TerminationReason};
// TODO: Re-enable these exports once modules are implemented
// pub use oidc_client::OIDCClient;
// pub use token_pair::TokenPair;
// pub use authorization_grant::AuthorizationGrant;