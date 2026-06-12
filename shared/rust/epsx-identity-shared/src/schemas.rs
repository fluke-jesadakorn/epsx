//! Diesel schema table macros vendored for `epsx-identity-shared`.
//!
//! The auth code uses three tables from the backend's primary
//! schema: `wallet_users`, `web3_auth_nonces`, `openid_refresh_tokens`.
//! These are vendored here as Diesel `table!` macros so the moved
//! source files can `use crate::schemas::*;` standalone.

pub mod primary {
    diesel::table! {
        use diesel::sql_types::*;

        /// Representation of the `openid_refresh_tokens` table.
        openid_refresh_tokens (token_id) {
            #[max_length = 36]
            token_id -> Varchar,
            #[max_length = 42]
            wallet_address -> Varchar,
            expires_at -> Timestamptz,
            created_at -> Timestamptz,
            is_revoked -> Bool,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;

        /// Representation of the `wallet_users` table.
        wallet_users (wallet_address) {
            #[max_length = 42]
            wallet_address -> Varchar,
            is_active -> Bool,
            tier_level -> Varchar,
            wallet_metadata -> Nullable<Jsonb>,
            last_auth_at -> Nullable<Timestamptz>,
            updated_at -> Timestamptz,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;

        /// Representation of the `web3_auth_nonces` table.
        web3_auth_nonces (nonce) {
            #[max_length = 42]
            wallet_address -> Varchar,
            #[max_length = 64]
            nonce -> Varchar,
            message -> Text,
            expires_at -> Timestamptz,
            created_at -> Timestamptz,
        }
    }

    diesel::joinable!(openid_refresh_tokens -> wallet_users (wallet_address));

    diesel::allow_tables_to_appear_in_same_query!(
        openid_refresh_tokens,
        wallet_users,
        web3_auth_nonces,
    );
}
