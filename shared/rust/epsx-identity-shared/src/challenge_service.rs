// SIWE Challenge Generation and Validation
// Handles nonce lifecycle: generate, store, validate, cleanup
//
// MIGRATED TO SQLX (real): no stubs.

use chrono::{Duration, Utc};
use ethers::types::Address;
use tracing::info;

use super::auth_service::{UnifiedWeb3AuthService, Web3AuthError, Web3Challenge};

fn siwe_origin_for_domain(domain: &str) -> String {
    let host = domain
        .strip_prefix('[')
        .and_then(|value| value.split_once(']').map(|(host, _)| host))
        .unwrap_or_else(|| domain.split(':').next().unwrap_or(domain));
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host.to_ascii_lowercase().ends_with(".localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    let scheme = if loopback { "http" } else { "https" };
    format!("{scheme}://{domain}")
}

impl UnifiedWeb3AuthService {
    /// Generate Web3 authentication challenge (SIWE)
    pub async fn generate_challenge(
        &self,
        wallet_address: &str,
    ) -> Result<Web3Challenge, Web3AuthError> {
        let wallet_address = wallet_address.trim().to_lowercase();

        let address = Address::from_str(&wallet_address)
            .map_err(|e| Web3AuthError::InvalidWalletAddress(format!("Invalid format: {}", e)))?;

        let nonce = self.generate_secure_nonce();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(self.nonce_expiry_minutes);

        let message = self.create_siwe_message(&address, &nonce)?;

        // Delete expired nonces for this wallet (best-effort cleanup).
        sqlx::query("DELETE FROM web3_auth_nonces WHERE expires_at < $1")
            .bind(now)
            .execute(self.db_pool)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        // Insert the new nonce.
        sqlx::query(
            "INSERT INTO web3_auth_nonces (wallet_address, nonce, message, expires_at, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&wallet_address)
        .bind(&nonce)
        .bind(&message)
        .bind(expires_at)
        .bind(now)
        .execute(self.db_pool)
        .await
        .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        info!("Generated Web3 challenge for wallet: {}", wallet_address);

        Ok(Web3Challenge {
            wallet_address,
            nonce,
            message,
            expires_at,
            created_at: now,
        })
    }

    /// Generate secure random nonce
    pub(super) fn generate_secure_nonce(&self) -> String {
        use rand::Rng;
        use std::fmt::Write;
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| rng.gen_range(0..16))
            .fold(String::new(), |mut acc, n| {
                let _ = write!(acc, "{:x}", n);
                acc
            })
    }

    /// Create SIWE message
    pub(super) fn create_siwe_message(
        &self,
        address: &Address,
        nonce: &str,
    ) -> Result<String, Web3AuthError> {
        use siwe::{Message, Version};

        let domain = self.domain.parse().map_err(|e| {
            Web3AuthError::InvalidDomain(format!("Invalid domain {}: {}", self.domain, e))
        })?;

        let origin = siwe_origin_for_domain(&self.domain);
        let uri = origin.parse().map_err(|e| {
            Web3AuthError::InvalidDomain(format!("Invalid URI {}: {}", self.domain, e))
        })?;

        let issued_at = Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
            .parse()
            .map_err(|e| {
                Web3AuthError::InvalidTimestamp(format!("Failed to parse issued_at: {}", e))
            })?;

        let expiration_time = Some(
            (Utc::now() + Duration::minutes(self.nonce_expiry_minutes))
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string()
                .parse()
                .map_err(|e| {
                    Web3AuthError::InvalidTimestamp(format!(
                        "Failed to parse expiration_time: {}",
                        e
                    ))
                })?,
        );

        let message = Message {
            domain,
            address: (*address).into(),
            statement: Some("Sign in to EPSX Data Analytics Platform".to_string()),
            uri,
            version: Version::V1,
            chain_id: std::env::var("CHAIN_ID")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(56),
            nonce: nonce.to_string(),
            issued_at,
            expiration_time,
            not_before: None,
            request_id: None,
            resources: vec![],
        };

        Ok(message.to_string())
    }

    /// Cleanup used nonce
    pub(super) async fn cleanup_nonce(
        &self,
        wallet_address: &str,
        nonce: &str,
    ) -> Result<(), Web3AuthError> {
        let wallet_address = wallet_address.trim().to_lowercase();

        sqlx::query("DELETE FROM web3_auth_nonces WHERE wallet_address = $1 AND nonce = $2")
            .bind(&wallet_address)
            .bind(nonce)
            .execute(self.db_pool)
            .await
            .map_err(|e| Web3AuthError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::siwe_origin_for_domain;

    #[test]
    fn siwe_origin_uses_http_only_for_loopback_authorities() {
        assert_eq!(
            siwe_origin_for_domain("localhost:3000"),
            "http://localhost:3000"
        );
        assert_eq!(
            siwe_origin_for_domain("127.0.0.1:3000"),
            "http://127.0.0.1:3000"
        );
        assert_eq!(siwe_origin_for_domain("dev.epsx.io"), "https://dev.epsx.io");
    }
}
