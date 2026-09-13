use super::{table_exists, Error, Migration};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub family: String,
    pub catalog_sha256: String,
    pub legacy_versions: Vec<String>,
    /// Exact historical entries with no corresponding current migration. These
    /// remain in Diesel's ledger; they are never renamed or executed as SQL.
    pub archived_versions: Vec<String>,
    /// Pin every recognized legacy migration at the reviewed release contents.
    pub adopted_checksums: BTreeMap<String, String>,
    pub review: String,
}

pub async fn catalog(conn: &mut PgConnection) -> Result<String, Error> {
    let entries: Vec<String> = sqlx::query_scalar(include_str!("catalog.sql"))
        .fetch_all(conn)
        .await?;
    Ok(entries.join("\n"))
}

pub async fn evidence(
    conn: &mut PgConnection,
    family: &str,
    list: &[Migration],
) -> Result<Evidence, Error> {
    let mut versions: Vec<String> =
        sqlx::query_scalar("SELECT version FROM __diesel_schema_migrations")
            .fetch_all(&mut *conn)
            .await?;
    versions.sort();
    let (archived, checksums) = classify(&versions, list);
    Ok(Evidence {
        family: family.into(),
        catalog_sha256: hex::encode(Sha256::digest(catalog(conn).await?.as_bytes())),
        legacy_versions: versions,
        archived_versions: archived,
        adopted_checksums: checksums,
        review: String::new(),
    })
}

fn classify(versions: &[String], list: &[Migration]) -> (Vec<String>, BTreeMap<String, String>) {
    let mut archived = vec![];
    let mut checksums = BTreeMap::new();
    for v in versions {
        if let Some(m) = list.iter().find(|m| &m.version == v) {
            checksums.insert(v.clone(), m.checksum.clone());
        } else {
            archived.push(v.clone());
        }
    }
    (archived, checksums)
}

fn validate(
    e: &Evidence,
    family: &str,
    versions: &[String],
    list: &[Migration],
) -> Result<(), Error> {
    let (archived, checksums) = classify(versions, list);
    if e.family != family
        || e.legacy_versions != versions
        || e.archived_versions != archived
        || e.adopted_checksums != checksums
        || e.review.trim().is_empty()
        || e.catalog_sha256.len() != 64
        || !e.catalog_sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(
            "legacy adoption evidence does not match the exact history/release or lacks review"
                .into(),
        );
    }
    Ok(())
}

/// Returns a new reviewed adoption to persist, or an already recorded one.
/// Only the first adoption needs the old catalog: later forward migrations
/// intentionally change it. The unchanged legacy history and release checksums
/// continue to be checked on every invocation.
pub async fn check(
    conn: &mut PgConnection,
    family: &str,
    versions: &[String],
    list: &[Migration],
    path: Option<&Path>,
) -> Result<(Option<Evidence>, bool), Error> {
    if table_exists(conn, "epsx_schema_adoptions").await? {
        let saved: Option<serde_json::Value> =
            sqlx::query_scalar("SELECT evidence FROM epsx_schema_adoptions WHERE family=$1")
                .bind(family)
                .fetch_optional(&mut *conn)
                .await?;
        if let Some(saved) = saved {
            let e: Evidence = serde_json::from_value(saved)?;
            validate(&e, family, versions, list)?;
            if let Some(path) = path {
                let supplied: Evidence = serde_json::from_slice(&std::fs::read(path)?)?;
                if e != supplied {
                    return Err("supplied evidence differs from the recorded adoption".into());
                }
            }
            return Ok((Some(e), false));
        }
    }
    let Some(path) = path else {
        return Ok((None, false));
    };
    let e: Evidence = serde_json::from_slice(&std::fs::read(path)?)?;
    validate(&e, family, versions, list)?;
    let actual = hex::encode(Sha256::digest(catalog(conn).await?.as_bytes()));
    if e.catalog_sha256 != actual {
        return Err("catalog differs from reviewed legacy adoption snapshot".into());
    }
    Ok((Some(e), true))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_unreviewed_unknown_history_and_release_drift() {
        let list = vec![Migration {
            version: "1".into(),
            name: "1_base".into(),
            sql: String::new(),
            checksum: "pinned".into(),
        }];
        let versions = vec!["1".into(), "1_foreign_baseline".into()];
        let mut e = Evidence {
            family: "core".into(),
            catalog_sha256: "a".repeat(64),
            legacy_versions: versions.clone(),
            archived_versions: vec!["1_foreign_baseline".into()],
            adopted_checksums: BTreeMap::from([("1".into(), "pinned".into())]),
            review: "Catalog reviewed; foreign ledger entry retained.".into(),
        };
        assert!(validate(&e, "core", &versions, &list).is_ok());
        e.archived_versions.clear();
        assert!(validate(&e, "core", &versions, &list).is_err());
        e.archived_versions.push("1_foreign_baseline".into());
        e.adopted_checksums.insert("1".into(), "changed".into());
        assert!(validate(&e, "core", &versions, &list).is_err());
    }
}
