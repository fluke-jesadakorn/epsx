//! Opaque refresh credentials and their keyed database digests.
//!
//! A refresh credential is returned to the caller once. Persistence stores only
//! its key identifier and HMAC-SHA256 digest; neither the random credential bytes
//! nor the HMAC key material have `Debug`, `Display`, or serde implementations.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::{rngs::OsRng, RngCore};
use serde::de::{self, MapAccess, Visitor};
use serde::Deserialize;
use sha2::Sha256;
use thiserror::Error;

const TOKEN_VERSION: &str = "rt1";
const TOKEN_SECRET_BYTES: usize = 32;
const DIGEST_BYTES: usize = 32;
const MIN_HMAC_KEY_BYTES: usize = 32;
const MAX_KEY_ID_BYTES: usize = 32;
const MAX_KEYRING_KEYS: usize = 8;
const MAX_KEYRING_JSON_BYTES: usize = 32 * 1024;
const MAX_TOKEN_BYTES: usize = TOKEN_VERSION.len() + 1 + MAX_KEY_ID_BYTES + 1 + 43;
const HMAC_DOMAIN_SEPARATOR: &[u8] = b"epsx.refresh.v1\0";

/// Environment variable containing the identifier used for newly issued tokens.
pub const REFRESH_TOKEN_HMAC_ACTIVE_KID_ENV: &str = "REFRESH_TOKEN_HMAC_ACTIVE_KID";

/// Environment variable containing a JSON object of key-id to base64url key.
pub const REFRESH_TOKEN_HMAC_KEYS_JSON_ENV: &str = "REFRESH_TOKEN_HMAC_KEYS_JSON";

/// Errors intentionally exclude credentials, digests, and key material.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RefreshTokenDigestError {
    #[error("required refresh-token keyring environment variable is missing")]
    MissingEnvironmentVariable,
    #[error("refresh-token keyring JSON is malformed")]
    InvalidKeyringJson,
    #[error("refresh-token keyring has too many keys")]
    TooManyKeys,
    #[error("refresh-token key identifier is invalid")]
    InvalidKeyId,
    #[error("refresh-token key identifier is duplicated")]
    DuplicateKeyId,
    #[error("refresh-token key encoding is invalid")]
    InvalidKeyEncoding,
    #[error("refresh-token HMAC key is shorter than 32 bytes")]
    KeyTooShort,
    #[error("refresh-token active key is absent from the keyring")]
    ActiveKeyNotFound,
    #[error("refresh token has an invalid format")]
    InvalidTokenFormat,
    #[error("refresh token version is unsupported")]
    UnsupportedTokenVersion,
    #[error("refresh token key identifier is unknown")]
    UnknownKeyId,
}

/// Opaque bearer credential returned to the caller.
///
/// Deliberately does not implement `Debug`, `Display`, or serde traits.
pub struct RefreshTokenCredential(String);

impl RefreshTokenCredential {
    /// Explicitly expose the bearer value for a response or secure cookie.
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Explicitly transfer the bearer value to a response owner.
    pub fn into_exposed(self) -> String {
        self.0
    }
}

/// Fixed-size HMAC digest suitable for a PostgreSQL `BYTEA` column.
///
/// Deliberately does not implement `Debug`, `Display`, or serde traits.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RefreshTokenDigest([u8; DIGEST_BYTES]);

impl RefreshTokenDigest {
    pub const BYTE_LEN: usize = DIGEST_BYTES;

    /// Borrow the exact bytes to bind to a `BYTEA` query parameter.
    pub fn as_db_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Copy the exact bytes into an owned value for a `BYTEA` insert.
    pub fn to_db_bytes(self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl AsRef<[u8]> for RefreshTokenDigest {
    fn as_ref(&self) -> &[u8] {
        self.as_db_bytes()
    }
}

/// Credential plus the only values that should be persisted for it.
///
/// Deliberately does not implement `Debug` or serde traits because it owns a
/// bearer credential.
pub struct IssuedRefreshToken {
    credential: RefreshTokenCredential,
    digest_key_id: String,
    digest: RefreshTokenDigest,
}

impl IssuedRefreshToken {
    pub fn credential(&self) -> &RefreshTokenCredential {
        &self.credential
    }

    pub fn into_credential(self) -> RefreshTokenCredential {
        self.credential
    }

    pub fn digest_key_id(&self) -> &str {
        &self.digest_key_id
    }

    pub fn digest(&self) -> RefreshTokenDigest {
        self.digest
    }
}

/// Database lookup material derived from a presented bearer credential.
///
/// Deliberately does not implement `Debug`, `Display`, or serde traits.
pub struct DigestedRefreshToken {
    digest_key_id: String,
    digest: RefreshTokenDigest,
}

impl DigestedRefreshToken {
    pub fn digest_key_id(&self) -> &str {
        &self.digest_key_id
    }

    pub fn digest(&self) -> RefreshTokenDigest {
        self.digest
    }
}

/// A versioned, bounded collection of dedicated refresh-token HMAC keys.
///
/// This type has no generated/default fallback and deliberately has no `Debug`
/// implementation because it owns secret key bytes.
pub struct RefreshTokenKeyring {
    active_key_id: String,
    keys: HashMap<String, Box<[u8]>>,
}

impl RefreshTokenKeyring {
    /// Construct a keyring from decoded key bytes.
    ///
    /// Exactly one entry must match `active_key_id`. At most eight total keys
    /// (one active plus retained verification keys) are accepted.
    pub fn new(
        active_key_id: impl Into<String>,
        keys: impl IntoIterator<Item = (String, Vec<u8>)>,
    ) -> Result<Self, RefreshTokenDigestError> {
        let active_key_id = active_key_id.into();
        validate_key_id(&active_key_id)?;

        let mut decoded_keys = HashMap::new();
        for (key_id, key) in keys {
            if decoded_keys.len() >= MAX_KEYRING_KEYS {
                return Err(RefreshTokenDigestError::TooManyKeys);
            }
            validate_key_id(&key_id)?;
            if key.len() < MIN_HMAC_KEY_BYTES {
                return Err(RefreshTokenDigestError::KeyTooShort);
            }
            if decoded_keys
                .insert(key_id, key.into_boxed_slice())
                .is_some()
            {
                return Err(RefreshTokenDigestError::DuplicateKeyId);
            }
        }

        if !decoded_keys.contains_key(&active_key_id) {
            return Err(RefreshTokenDigestError::ActiveKeyNotFound);
        }

        Ok(Self {
            active_key_id,
            keys: decoded_keys,
        })
    }

    /// Parse a JSON object whose values are canonical unpadded base64url keys.
    pub fn from_json(
        active_key_id: &str,
        encoded_keys_json: &str,
    ) -> Result<Self, RefreshTokenDigestError> {
        if encoded_keys_json.len() > MAX_KEYRING_JSON_BYTES {
            return Err(RefreshTokenDigestError::InvalidKeyringJson);
        }

        let EncodedKeys(encoded_keys) = serde_json::from_str(encoded_keys_json)
            .map_err(|_| RefreshTokenDigestError::InvalidKeyringJson)?;
        let decoded_keys = encoded_keys
            .into_iter()
            .map(|(key_id, encoded_key)| {
                decode_canonical_key(&encoded_key).map(|key| (key_id, key))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Self::new(active_key_id.to_owned(), decoded_keys)
    }

    /// Load the required keyring variables without any generated fallback.
    pub fn from_env() -> Result<Self, RefreshTokenDigestError> {
        let active_key_id = env::var(REFRESH_TOKEN_HMAC_ACTIVE_KID_ENV)
            .map_err(|_| RefreshTokenDigestError::MissingEnvironmentVariable)?;
        let encoded_keys_json = env::var(REFRESH_TOKEN_HMAC_KEYS_JSON_ENV)
            .map_err(|_| RefreshTokenDigestError::MissingEnvironmentVariable)?;
        Self::from_json(&active_key_id, &encoded_keys_json)
    }

    pub fn active_key_id(&self) -> &str {
        &self.active_key_id
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Generate a 256-bit opaque credential and its database digest.
    pub fn issue(&self) -> IssuedRefreshToken {
        let mut secret = [0_u8; TOKEN_SECRET_BYTES];
        OsRng.fill_bytes(&mut secret);
        self.issue_with_secret(secret)
    }

    /// Strictly parse a presented credential and derive its database lookup.
    pub fn digest_presented(
        &self,
        credential: &str,
    ) -> Result<DigestedRefreshToken, RefreshTokenDigestError> {
        let parsed = ParsedRefreshToken::parse(credential)?;
        let key = self
            .keys
            .get(&parsed.key_id)
            .ok_or(RefreshTokenDigestError::UnknownKeyId)?;
        let digest = digest_secret(key, &parsed.key_id, &parsed.secret);

        Ok(DigestedRefreshToken {
            digest_key_id: parsed.key_id,
            digest,
        })
    }

    fn issue_with_secret(&self, secret: [u8; TOKEN_SECRET_BYTES]) -> IssuedRefreshToken {
        let key = self
            .keys
            .get(&self.active_key_id)
            .expect("validated keyring must contain its active key");
        let encoded_secret = URL_SAFE_NO_PAD.encode(secret);
        let credential = RefreshTokenCredential(format!(
            "{TOKEN_VERSION}.{}.{}",
            self.active_key_id, encoded_secret
        ));
        let digest = digest_secret(key, &self.active_key_id, &secret);

        IssuedRefreshToken {
            credential,
            digest_key_id: self.active_key_id.clone(),
            digest,
        }
    }
}

struct ParsedRefreshToken {
    key_id: String,
    secret: [u8; TOKEN_SECRET_BYTES],
}

impl ParsedRefreshToken {
    fn parse(credential: &str) -> Result<Self, RefreshTokenDigestError> {
        if credential.len() > MAX_TOKEN_BYTES || !credential.is_ascii() {
            return Err(RefreshTokenDigestError::InvalidTokenFormat);
        }

        let mut parts = credential.split('.');
        let version = parts
            .next()
            .ok_or(RefreshTokenDigestError::InvalidTokenFormat)?;
        let key_id = parts
            .next()
            .ok_or(RefreshTokenDigestError::InvalidTokenFormat)?;
        let encoded_secret = parts
            .next()
            .ok_or(RefreshTokenDigestError::InvalidTokenFormat)?;
        if parts.next().is_some() {
            return Err(RefreshTokenDigestError::InvalidTokenFormat);
        }
        if version != TOKEN_VERSION {
            return Err(RefreshTokenDigestError::UnsupportedTokenVersion);
        }
        validate_key_id(key_id).map_err(|_| RefreshTokenDigestError::InvalidTokenFormat)?;

        let decoded_secret = URL_SAFE_NO_PAD
            .decode(encoded_secret)
            .map_err(|_| RefreshTokenDigestError::InvalidTokenFormat)?;
        if decoded_secret.len() != TOKEN_SECRET_BYTES
            || URL_SAFE_NO_PAD.encode(&decoded_secret) != encoded_secret
        {
            return Err(RefreshTokenDigestError::InvalidTokenFormat);
        }
        let secret = decoded_secret
            .try_into()
            .map_err(|_| RefreshTokenDigestError::InvalidTokenFormat)?;

        Ok(Self {
            key_id: key_id.to_owned(),
            secret,
        })
    }
}

fn validate_key_id(key_id: &str) -> Result<(), RefreshTokenDigestError> {
    if key_id.is_empty()
        || key_id.len() > MAX_KEY_ID_BYTES
        || !key_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(RefreshTokenDigestError::InvalidKeyId);
    }
    Ok(())
}

fn decode_canonical_key(encoded_key: &str) -> Result<Vec<u8>, RefreshTokenDigestError> {
    let key = URL_SAFE_NO_PAD
        .decode(encoded_key)
        .map_err(|_| RefreshTokenDigestError::InvalidKeyEncoding)?;
    if URL_SAFE_NO_PAD.encode(&key) != encoded_key {
        return Err(RefreshTokenDigestError::InvalidKeyEncoding);
    }
    Ok(key)
}

fn digest_secret(
    key: &[u8],
    key_id: &str,
    secret: &[u8; TOKEN_SECRET_BYTES],
) -> RefreshTokenDigest {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC-SHA256 accepts validated keys of any non-zero length");
    mac.update(HMAC_DOMAIN_SEPARATOR);
    mac.update(key_id.as_bytes());
    mac.update(&[0]);
    mac.update(secret);
    let digest: [u8; DIGEST_BYTES] = mac.finalize().into_bytes().into();
    RefreshTokenDigest(digest)
}

struct EncodedKeys(Vec<(String, String)>);

impl<'de> Deserialize<'de> for EncodedKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(EncodedKeysVisitor)
    }
}

struct EncodedKeysVisitor;

impl<'de> Visitor<'de> for EncodedKeysVisitor {
    type Value = EncodedKeys;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a refresh-token keyring JSON object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut entries = Vec::new();
        let mut seen = HashSet::new();
        while let Some((key_id, encoded_key)) = map.next_entry::<String, String>()? {
            if !seen.insert(key_id.clone()) {
                return Err(de::Error::custom("duplicate refresh-token key identifier"));
            }
            entries.push((key_id, encoded_key));
        }
        Ok(EncodedKeys(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVE_KID: &str = "key_2026_07";

    fn test_keyring() -> RefreshTokenKeyring {
        RefreshTokenKeyring::new(ACTIVE_KID, [(ACTIVE_KID.to_owned(), (0_u8..32).collect())])
            .expect("test keyring should be valid")
    }

    fn credential_for(key_id: &str, secret: &[u8]) -> String {
        format!(
            "{TOKEN_VERSION}.{key_id}.{}",
            URL_SAFE_NO_PAD.encode(secret)
        )
    }

    #[test]
    fn deterministic_hmac_vector_is_stable_and_domain_separated() {
        let keyring = test_keyring();
        let secret: Vec<u8> = (32_u8..64).collect();
        let credential = credential_for(ACTIVE_KID, &secret);

        let digested = keyring
            .digest_presented(&credential)
            .expect("vector credential should parse");

        assert_eq!(digested.digest_key_id(), ACTIVE_KID);
        assert_eq!(
            hex::encode(digested.digest().as_db_bytes()),
            "d8c970424bf98b876f9c3e3d3a05f9a27fa492cce9ad6f5c8ae241b65b3d3c40"
        );

        let other_keyring =
            RefreshTokenKeyring::new("other", [("other".to_owned(), (0_u8..32).collect())])
                .expect("second keyring should be valid");
        let other = other_keyring
            .digest_presented(&credential_for("other", &secret))
            .expect("second vector should parse");
        assert!(digested.digest() != other.digest());
    }

    #[test]
    fn issue_returns_canonical_256_bit_credential_and_matching_digest() {
        let keyring = test_keyring();
        let first = keyring.issue();
        let second = keyring.issue();

        assert!(first.credential().expose().starts_with("rt1.key_2026_07."));
        assert!(first.credential().expose() != second.credential().expose());
        assert_eq!(first.digest_key_id(), ACTIVE_KID);
        assert_eq!(
            first.digest().as_db_bytes().len(),
            RefreshTokenDigest::BYTE_LEN
        );

        let parsed = keyring
            .digest_presented(first.credential().expose())
            .expect("issued credential should parse");
        assert_eq!(parsed.digest_key_id(), first.digest_key_id());
        assert!(parsed.digest() == first.digest());
    }

    #[test]
    fn strict_parser_rejects_malformed_and_noncanonical_credentials() {
        let keyring = test_keyring();
        let zero_secret = [0_u8; TOKEN_SECRET_BYTES];
        let canonical = credential_for(ACTIVE_KID, &zero_secret);
        let noncanonical_tail = format!("{}B", &canonical[..canonical.len() - 1]);
        let padded = format!("{canonical}=");
        let too_long_kid = "a".repeat(MAX_KEY_ID_BYTES + 1);
        let malformed = [
            "".to_owned(),
            "rt1".to_owned(),
            "rt1.key".to_owned(),
            "rt1..AAAA".to_owned(),
            credential_for("bad+k", &zero_secret),
            credential_for(&too_long_kid, &zero_secret),
            credential_for(ACTIVE_KID, &[0_u8; TOKEN_SECRET_BYTES - 1]),
            credential_for(ACTIVE_KID, &[0_u8; TOKEN_SECRET_BYTES + 1]),
            format!("{canonical}.extra"),
            padded,
            noncanonical_tail,
            "rt1.key_2026_07.not/base64".to_owned(),
        ];

        for credential in malformed {
            assert!(matches!(
                keyring.digest_presented(&credential),
                Err(RefreshTokenDigestError::InvalidTokenFormat)
            ));
        }

        assert!(matches!(
            keyring.digest_presented(&canonical.replacen("rt1", "rt2", 1)),
            Err(RefreshTokenDigestError::UnsupportedTokenVersion)
        ));
        assert!(matches!(
            keyring.digest_presented(&credential_for("unknown", &zero_secret)),
            Err(RefreshTokenDigestError::UnknownKeyId)
        ));
    }

    #[test]
    fn constructor_enforces_keyring_contract() {
        assert!(matches!(
            RefreshTokenKeyring::new("bad+k", [("bad+k".to_owned(), vec![1; 32])]),
            Err(RefreshTokenDigestError::InvalidKeyId)
        ));
        assert!(matches!(
            RefreshTokenKeyring::new("active", [("active".to_owned(), vec![1; 31])]),
            Err(RefreshTokenDigestError::KeyTooShort)
        ));
        assert!(matches!(
            RefreshTokenKeyring::new("missing", [("retained".to_owned(), vec![1; 32])]),
            Err(RefreshTokenDigestError::ActiveKeyNotFound)
        ));
        assert!(matches!(
            RefreshTokenKeyring::new(
                "active",
                [
                    ("active".to_owned(), vec![1; 32]),
                    ("active".to_owned(), vec![2; 32]),
                ],
            ),
            Err(RefreshTokenDigestError::DuplicateKeyId)
        ));

        let too_many = (0..=MAX_KEYRING_KEYS)
            .map(|index| (format!("key{index}"), vec![index as u8; 32]))
            .collect::<Vec<_>>();
        assert!(matches!(
            RefreshTokenKeyring::new("key0", too_many),
            Err(RefreshTokenDigestError::TooManyKeys)
        ));
    }

    #[test]
    fn json_parser_accepts_rotation_and_rejects_unsafe_configuration() {
        let active = URL_SAFE_NO_PAD.encode([7_u8; 32]);
        let retained = URL_SAFE_NO_PAD.encode([8_u8; 64]);
        let json = format!(r#"{{"active":"{active}","old-key":"{retained}"}}"#);
        let keyring = RefreshTokenKeyring::from_json("active", &json)
            .expect("canonical rotation keyring should parse");
        assert_eq!(keyring.active_key_id(), "active");
        assert_eq!(keyring.key_count(), 2);

        let duplicate = format!(r#"{{"active":"{active}","active":"{retained}"}}"#);
        assert!(matches!(
            RefreshTokenKeyring::from_json("active", &duplicate),
            Err(RefreshTokenDigestError::InvalidKeyringJson)
        ));
        assert!(matches!(
            RefreshTokenKeyring::from_json("active", &format!(r#"{{"active":"{active}="}}"#)),
            Err(RefreshTokenDigestError::InvalidKeyEncoding)
        ));
        assert!(matches!(
            RefreshTokenKeyring::from_json("active", "[]"),
            Err(RefreshTokenDigestError::InvalidKeyringJson)
        ));
        assert!(matches!(
            RefreshTokenKeyring::from_json("active", r#"{"active":7}"#),
            Err(RefreshTokenDigestError::InvalidKeyringJson)
        ));
    }
}
