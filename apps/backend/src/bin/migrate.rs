//! Explicit, transactional migrations for the original Diesel directories and service SQL files.
//! Does not load checkout dotenv files, create databases, or replay untracked baselines.
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256, Sha384};
use sqlx::{Connection, PgConnection, Row};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};
type Error = Box<dyn std::error::Error + Send + Sync>;
#[path = "migrate/adoption.rs"]
mod adoption;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Release migrations directory (contains core, analytics, payments, notifications).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    /// Run one family only. Service families: wallet, pay, subscription, analytics-service.
    #[arg(long, global = true)]
    family: Option<String>,
    /// Override the environment variable containing the connection URL, never the URL itself.
    #[arg(long, global = true)]
    database_env: Option<String>,
    /// Reviewed exact legacy catalog/history evidence; requires one family.
    #[arg(long, global = true)]
    legacy_adoption: Option<PathBuf>,
}
#[derive(Subcommand)]
enum Command {
    Pending,
    Up,
    /// Export a read-only candidate. Review schema/history and fill review before adoption.
    LegacySnapshot {
        #[arg(long)]
        output: PathBuf,
    },
}
#[derive(Clone, Debug)]
struct Migration {
    version: String,
    name: String,
    sql: String,
    checksum: String,
}

fn migrations(directory: &Path) -> Result<Vec<Migration>, Error> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let sql_path = if path.is_dir() {
            path.join("up.sql")
        } else {
            path.clone()
        };
        if !sql_path.is_file() || sql_path.extension().and_then(|v| v.to_str()) != Some("sql") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("invalid migration filename")?
            .to_string();
        let version = name
            .split('_')
            .next()
            .ok_or("missing version")?
            .replace('-', "");
        if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("invalid migration version: {name}").into());
        }
        let sql = fs::read_to_string(sql_path)?;
        let checksum = hex::encode(Sha256::digest(sql.as_bytes()));
        if result
            .insert(
                version.clone(),
                Migration {
                    version,
                    name,
                    sql,
                    checksum,
                },
            )
            .is_some()
        {
            return Err("duplicate migration version after Diesel normalization".into());
        }
    }
    if result.is_empty() {
        return Err(format!("no up migrations in {}", directory.display()).into());
    }
    Ok(result.into_values().collect())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if cli.database_env.is_some() && cli.family.is_none() {
        return Err("--database-env requires --family".into());
    }
    if (cli.legacy_adoption.is_some() || matches!(cli.command, Command::LegacySnapshot { .. }))
        && cli.family.is_none()
    {
        return Err("legacy adoption/snapshot requires --family".into());
    }
    let root = cli
        .root
        .or_else(|| env::var_os("EPSX_MIGRATIONS_DIR").map(PathBuf::from))
        .unwrap_or_else(|| {
            let packaged = env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("migrations");
            if packaged.is_dir() {
                packaged
            } else {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
            }
        });
    let families = [
        ("core", "DATABASE_URL", "core"),
        ("analytics", "ANALYTICS_DATABASE_URL", "analytics"),
        ("payments", "PAYMENTS_DATABASE_URL", "payments"),
        (
            "notifications",
            "NOTIFICATIONS_DATABASE_URL",
            "notifications",
        ),
        ("wallet", "WALLET_DATABASE_URL", "services/wallet"),
        ("pay", "PAY_SERVICE_DATABASE_URL", "services/pay"),
        (
            "subscription",
            "SUBSCRIPTION_DATABASE_URL",
            "services/subscription",
        ),
        (
            "analytics-service",
            "ANALYTICS_SERVICE_DATABASE_URL",
            "services/analytics",
        ),
    ];
    if cli
        .family
        .as_ref()
        .is_some_and(|f| !families.iter().any(|(name, _, _)| name == f))
    {
        return Err("unknown family".into());
    }
    let mut processed = 0;
    for (family, key, relative) in families {
        if cli.family.as_ref().is_some_and(|f| f != family) {
            continue;
        }
        let key = cli.database_env.as_deref().unwrap_or(key);
        let Ok(url) = env::var(key) else {
            if cli.family.is_some() {
                return Err(format!("{key} is required").into());
            }
            println!("{family}: skipped ({key} unset)");
            continue;
        };
        let directory = if relative.starts_with("services/") && !root.join(relative).exists() {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .join(relative)
                .join("migrations")
        } else {
            root.join(relative)
        };
        let list = migrations(&directory)?;
        if let Command::LegacySnapshot { output } = &cli.command {
            let mut conn = PgConnection::connect(&url).await?;
            sqlx::query("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
                .execute(&mut conn)
                .await?;
            let evidence = adoption::evidence(&mut conn, family, &list).await?;
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            file.write_all(serde_json::to_string_pretty(&evidence)?.as_bytes())?;
            println!("{family}: read-only candidate written; review required before adoption");
        } else {
            run(
                &url,
                family,
                &list,
                matches!(cli.command, Command::Up),
                cli.legacy_adoption.as_deref(),
            )
            .await?;
        }
        processed += 1;
    }
    if processed == 0 {
        return Err("no database configured; nothing checked".into());
    }
    Ok(())
}

async fn table_exists(conn: &mut PgConnection, name: &str) -> Result<bool, Error> {
    Ok(
        sqlx::query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
            .bind(format!("public.{name}"))
            .fetch_one(conn)
            .await?,
    )
}

async fn run(
    url: &str,
    family: &str,
    list: &[Migration],
    apply: bool,
    adoption_path: Option<&Path>,
) -> Result<(), Error> {
    let mut conn = PgConnection::connect(url).await?;
    // One database lock also serializes families sharing the same physical database.
    sqlx::query("SELECT pg_advisory_lock(1162892112)")
        .execute(&mut conn)
        .await?;
    let mut applied = BTreeMap::<String, String>::new();
    let tracked = table_exists(&mut conn, "epsx_schema_migrations").await?;
    if tracked {
        for row in
            sqlx::query("SELECT version, checksum FROM epsx_schema_migrations WHERE family=$1")
                .bind(family)
                .fetch_all(&mut conn)
                .await?
        {
            applied.insert(row.get("version"), row.get("checksum"));
        }
    }
    let mut legacy = BTreeMap::<String, String>::new();
    let mut adoption_to_record = None;
    let is_backend = ["core", "analytics", "payments", "notifications"].contains(&family);
    if is_backend && table_exists(&mut conn, "__diesel_schema_migrations").await? {
        let mut versions =
            sqlx::query_scalar::<_, String>("SELECT version FROM __diesel_schema_migrations")
                .fetch_all(&mut conn)
                .await?;
        versions.sort();
        let (evidence, fresh) =
            adoption::check(&mut conn, family, &versions, list, adoption_path).await?;
        for version in versions {
            if !list.iter().any(|m| m.version == version) {
                if evidence
                    .as_ref()
                    .is_some_and(|e| e.archived_versions.contains(&version))
                {
                    println!("{family}: preserved archived Diesel entry {version}");
                    continue;
                }
                return Err(format!("{family}: unknown Diesel version {version}; reconcile archived history before migration").into());
            }
            legacy.insert(version, "diesel".into());
        }
        if fresh {
            adoption_to_record = evidence;
        }
    } else if adoption_path.is_some() {
        return Err("legacy adoption requires an existing backend Diesel ledger".into());
    }
    if table_exists(&mut conn, "_sqlx_migrations").await? {
        for row in sqlx::query("SELECT version, checksum, success FROM _sqlx_migrations")
            .fetch_all(&mut conn)
            .await?
        {
            let version: i64 = row.get("version");
            if let Some(m) = list
                .iter()
                .find(|m| m.version.parse::<i64>().ok() == Some(version))
            {
                let checksum: Vec<u8> = row.get("checksum");
                if !row.get::<bool, _>("success")
                    || checksum != Sha384::digest(m.sql.as_bytes()).to_vec()
                {
                    return Err(
                        format!("{family}: conflicting SQLx history for {}", m.version).into(),
                    );
                }
                legacy.insert(m.version.clone(), "sqlx".into());
            } else if is_backend {
                return Err(format!("{family}: unknown SQLx version {version}").into());
            }
        }
    }
    for (version, checksum) in &applied {
        let m = list
            .iter()
            .find(|m| &m.version == version)
            .ok_or_else(|| format!("{family}: tracked migration {version} missing from release"))?;
        if &m.checksum != checksum {
            return Err(format!("{family}: checksum conflict at {version}").into());
        }
    }
    // A populated schema without a matching baseline is not a fresh database.
    // Service tables can coexist with backend families: their CREATE statements are
    // allowed only when none of this family's migrations claim an existing table.
    let tables: Vec<String> = sqlx::query_scalar("SELECT tablename::text FROM pg_tables WHERE schemaname='public' AND tablename NOT IN ('epsx_schema_migrations','__diesel_schema_migrations','_sqlx_migrations')").fetch_all(&mut conn).await?;
    if !tables.is_empty()
        && applied.is_empty()
        && legacy.is_empty()
        && (is_backend
            || list
                .iter()
                .any(|m| tables.iter().any(|table| creates_table(&m.sql, table))))
    {
        return Err(format!("{family}: populated schema has no matching history; refusing baseline replay (restore/reconcile the original ledger first)").into());
    }
    let mut gap = false;
    for m in list {
        let done = applied.contains_key(&m.version) || legacy.contains_key(&m.version);
        if done && gap {
            return Err(format!(
                "{family}: non-prefix history at {}; refusing out-of-order baseline replay",
                m.version
            )
            .into());
        }
        if !done {
            gap = true;
        }
    }
    if apply {
        sqlx::query("CREATE TABLE IF NOT EXISTS epsx_schema_migrations (family TEXT NOT NULL, version TEXT NOT NULL, name TEXT NOT NULL, checksum TEXT NOT NULL, source TEXT NOT NULL, applied_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY(family,version))").execute(&mut conn).await?;
        if let Some(evidence) = adoption_to_record {
            let mut tx = conn.begin().await?;
            sqlx::query("CREATE TABLE IF NOT EXISTS epsx_schema_adoptions (family TEXT PRIMARY KEY, evidence JSONB NOT NULL, adopted_at TIMESTAMPTZ NOT NULL DEFAULT now())").execute(&mut *tx).await?;
            sqlx::query("INSERT INTO epsx_schema_adoptions(family,evidence) VALUES($1,$2)")
                .bind(family)
                .bind(serde_json::to_value(evidence)?)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
    }
    for m in list {
        if applied.contains_key(&m.version) {
            println!("{family} applied {}", m.name);
            continue;
        }
        let source = legacy
            .get(&m.version)
            .map(String::as_str)
            .unwrap_or("native");
        println!(
            "{family} {} {}",
            if source == "native" {
                "pending"
            } else {
                "adopt"
            },
            m.name
        );
        if !apply {
            continue;
        }
        let mut tx = conn.begin().await?;
        if source == "native" {
            sqlx::raw_sql(&m.sql).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO epsx_schema_migrations(family,version,name,checksum,source) VALUES($1,$2,$3,$4,$5)").bind(family).bind(&m.version).bind(&m.name).bind(&m.checksum).bind(source).execute(&mut *tx).await?;
        tx.commit().await?;
    }
    // Dropping the dedicated connection releases the lock even on an error.
    Ok(())
}
fn creates_table(sql: &str, table: &str) -> bool {
    let normalized = sql
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    [
        format!("create table {table}"),
        format!("create table if not exists {table}"),
        format!("create table public.{table}"),
        format!("create table if not exists public.{table}"),
    ]
    .iter()
    .any(|pattern| normalized.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_real_directory_layout_and_diesel_version_order() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations");
        for family in ["core", "analytics", "payments", "notifications"] {
            let list = migrations(&root.join(family)).unwrap();
            assert!(!list.is_empty());
            assert!(list.windows(2).all(|w| w[0].version < w[1].version));
            assert!(list.iter().all(|m| !m.version.contains('-')));
        }
    }
}
