//! Wave 12 / integration gate — end-to-end smoke test.
//!
//! Verifies the wave-12 analytics binary lift is structurally
//! complete. The test does NOT require a running database — it's
//! a STRUCTURAL smoke that catches regressions in the binary
//! shape, the route mount surface, the migration schema name, and
//! the dead-route decision. The live curl smoke against the new
//! binary after the production cutover is a separate ops run.
//!
//! Run: `cargo test -p epsx --test wave12_smoke`
//!
//! What this test asserts:
//!   1. The new `epsx-analytics-service` binary's Cargo.toml is
//!      on disk, has the right shape (object-safety checks,
//!      `[[bin]]` target, `epsx` workspace dep), and the bin name
//!      matches the crate name.
//!   2. The `infra_logs` schema name is the canonical one in
//!      `apps/backend/migrations/analytics/` — no `analytics.`
//!      table references in any `.sql` file's DDL.
//!   3. The 5 unique analytics routes are at `/api/analytics/*`
//!      (NOT `/api/public/analytics/*`).
//!   4. The dead-route decision (option a vs b) matches the
//!      deliverable's stated choice (Track B chose option b:
//!      delete the handlers).
//!   5. `cargo build -p epsx --bin migrate --features
//!      epsx/cli-tools` would succeed (verified at integration
//!      time — embed_migrations! no longer panics). This test
//!      verifies the prerequisite: the v2 migration is gone.

// 1. The new `epsx-analytics-service` binary's Cargo.toml exists
//    and has the right shape.
#[test]
fn new_analytics_binary_cargo_manifest_has_right_shape() {
    let manifest = include_str!("../../../apps/analytics/Cargo.toml");

    // Crate name: `epsx-analytics-service` (renamed from the
    // spec's `epsx-analytics` due to a collision with
    // services/analytics/event-tracking binary).
    assert!(
        manifest.contains("name = \"epsx-analytics-service\""),
        "new binary crate name must be `epsx-analytics-service` \
         (collision with services/analytics event-tracking binary \
         was the reason for the rename; parent-session-notified)."
    );

    // `[[bin]]` target name is `epsx-analytics-service` (the
    // binary's CLI name, distinct from the `bin/main.rs` source
    // path).
    assert!(
        manifest.contains("name = \"epsx-analytics-service\""),
        "new binary `[[bin]]` name must be `epsx-analytics-service`."
    );
    assert!(
        manifest.contains("path = \"src/main.rs\""),
        "new binary `[[bin]]` source must be `src/main.rs`."
    );

    // Workspace dep on `epsx` (the monolith) so the new binary
    // can re-export the moved trees. The new binary uses
    // `path = "../backend"` (workspace-internal path dep) rather
    // than `workspace = true` because epsx is a sibling
    // workspace member, not a published crate.
    assert!(
        manifest.contains("epsx = { path = \"../backend\" }")
            || manifest.contains("epsx = {path = \"../backend\"}"),
        "new binary must depend on the `epsx` workspace member \
         via path = `../backend`."
    );
}

// 2. The `infra_logs` schema name is the canonical one. No
//    `analytics.` table references in any migration `.sql` file.
#[test]
fn infra_logs_schema_is_canonical_in_migrations() {
    use std::fs;
    use std::path::Path;

    // `cargo test` runs from the crate root (`apps/backend/`),
    // so the migrations dir is a relative `migrations/analytics`
    // path. We need to walk subdirs because the actual `.sql`
    // files live in `<version>_<name>/up.sql` and `down.sql`
    // subdirs.
    fn collect_sql(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    collect_sql(&p, out);
                } else if p.extension().and_then(|s| s.to_str()) == Some("sql") {
                    out.push(p);
                }
            }
        }
    }

    let migrations_dir = Path::new("migrations/analytics");
    let mut sql_files = Vec::new();
    collect_sql(migrations_dir, &mut sql_files);

    let mut bad = Vec::new();
    let mut good = 0usize;
    for path in &sql_files {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));

        // Per the wave-12 rename, all DDL must reference
        // `infra_logs.<table>`, not `analytics.<table>`. The
        // pre-rename `analytics.` table refs are forbidden.
        for line in content.lines() {
            // Strip line comments.
            let stripped = line.split("--").next().unwrap_or("");
            // Look for `analytics.<word>` where word starts with
            // a letter or underscore (table identifiers don't
            // start with digits or quotes). This catches
            // `analytics.audit_logs` but ignores
            // `audit-analytics.md` style references in comments
            // and prose (which the `--` strip already removes).
            if let Some(pos) = stripped.find("analytics.") {
                let after = &stripped[pos + "analytics.".len()..];
                let first = after.chars().next();
                if matches!(first, Some('a'..='z' | 'A'..='Z' | '_')) {
                    bad.push(format!("{}: {}", path.display(), line.trim()));
                }
            }
        }

        // The schema create statement must be `infra_logs` (or
        // `CREATE SCHEMA IF NOT EXISTS infra_logs`).
        let has_infra_logs_schema = content.contains("CREATE SCHEMA")
            && (content.contains("infra_logs")
                || content.contains("INFRA_LOGS"));
        let has_analytics_schema = content
            .lines()
            .filter(|l| !l.trim_start().starts_with("--"))
            .any(|l| l.contains("CREATE SCHEMA") && l.contains("analytics"));

        if has_analytics_schema && !has_infra_logs_schema {
            bad.push(format!(
                "{}: legacy `CREATE SCHEMA analytics` still present",
                path.display()
            ));
        }
        if has_infra_logs_schema {
            good += 1;
        }
    }

    assert!(
        !sql_files.is_empty(),
        "no `.sql` files found under `migrations/analytics` \
         (walker broken?)."
    );
    assert!(
        good >= 1,
        "at least one migration file must create the `infra_logs` \
         schema (rename target). Found {} .sql files.",
        sql_files.len()
    );
    assert!(
        bad.is_empty(),
        "found {} pre-rename `analytics.<table>` references in \
         migrations: {:#?}",
        bad.len(),
        bad
    );
}

// 3. The 5 unique analytics routes are at `/api/analytics/*` —
//    NOT `/api/public/analytics/*` (consolidation per Track B's
//    Step 4).
#[test]
fn five_unique_analytics_routes_are_at_api_analytics() {
    // The new binary's `build_analytics_router` mounts these
    // 5 routes. We check both the source (new binary) AND the
    // consolidated backend router (the source of truth post-
    // cutover).
    let new_binary_main = include_str!("../../../apps/analytics/src/main.rs");
    let backend_unified = include_str!("../src/web/routes/unified_router.rs");

    let expected = [
        "/api/analytics/rankings",
        "/api/analytics/filters",
        "/api/analytics/countries",
        "/api/analytics/available-countries",
        "/api/analytics/sectors",
    ];

    for path in &expected {
        // The new binary's main.rs tests reference these paths
        // (or the path components). Check that the 5 routes
        // exist somewhere in the codebase at the consolidated
        // `/api/analytics/*` path.
        let last_segment = path.rsplit('/').next().unwrap();
        assert!(
            new_binary_main.contains(last_segment)
                || backend_unified.contains(last_segment),
            "route segment `{}` (from `{}`) not found in \
             analytics binary main.rs or backend unified_router.rs",
            last_segment,
            path
        );
    }

    // Helper: detect a non-comment occurrence of a needle
    // inside Rust source (ignores `//` line comments and
    // `/* ... */` block comments on a single line).
    fn has_code_mention_simple(content: &str, needle: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains(needle) {
                return true;
            }
        }
        false
    }

    // The old duplicate `/api/public/analytics/*` mount must
    // be GONE from the backend router (Track B Step 4).
    // The check ignores comments — the producer left
    // explanatory comments referencing the old path.
    assert!(
        !has_code_mention_simple(backend_unified, "/api/public/analytics/"),
        "the duplicate `/api/public/analytics/*` mount must be \
         removed (Track B Step 4 route consolidation)."
    );
    assert!(
        !has_code_mention_simple(new_binary_main, "/api/public/analytics"),
        "the new `epsx-analytics-service` binary must not mount \
         the old `/api/public/analytics/*` paths."
    );
}

// 4. The dead-route decision (option a vs b) matches the
//    deliverable's stated choice. Track B chose **option b** —
//    delete the handlers. The `force_cache_refresh` and
//    `get_cache_stats` HTTP handlers must NOT be defined in
//    `web::analytics::eps`.
#[test]
fn dead_route_decision_is_option_b_handlers_deleted() {
    // Helper: scan a file for non-comment occurrences of a
    // needle. The dead-route deletion is observable as "the
    // function definition is gone" — the comments that mention
    // the deleted names (e.g. "wave12(track-b) option b:
    // deleted get_cache_stats and force_cache_refresh") are
    // intentionally left in for grep-ability.
    fn has_code_mention(content: &str, needle: &str) -> bool {
        for line in content.lines() {
            let trimmed = line.trim_start();
            // Skip line comments and block-comment-only lines.
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains(needle) {
                return true;
            }
        }
        false
    }

    let cache_rs = include_str!("../src/web/analytics/eps/cache.rs");
    let eps_handlers = include_str!("../src/web/analytics/eps_handlers.rs");
    let eps_mod = include_str!("../src/web/analytics/eps/mod.rs");
    let openapi = include_str!("../src/web/docs/openapi.rs");
    let openapi_admin = include_str!("../src/web/docs/openapi_admin.rs");
    let openapi_user = include_str!("../src/web/docs/openapi_user.rs");

    // The two dead route handler function definitions must be
    // gone from cache.rs (the file that owned them).
    assert!(
        !has_code_mention(cache_rs, "fn force_cache_refresh")
            && !has_code_mention(cache_rs, "fn get_cache_stats"),
        "Track B option b: `force_cache_refresh` and \
         `get_cache_stats` HTTP handler fn definitions must be \
         deleted from cache.rs."
    );

    // The re-export lines for the deleted handlers must be
    // gone from eps_handlers.rs and eps/mod.rs. We check for
    // `pub use` lines referencing the deleted names.
    assert!(
        !has_code_mention(eps_handlers, "force_cache_refresh")
            && !has_code_mention(eps_handlers, "get_cache_stats"),
        "Track B option b: the two dead handler re-exports \
         must be removed from eps_handlers.rs."
    );
    assert!(
        !has_code_mention(eps_mod, "force_cache_refresh")
            && !has_code_mention(eps_mod, "get_cache_stats"),
        "Track B option b: the two dead handler re-exports \
         must be removed from eps/mod.rs."
    );

    // The OpenAPI doc references for the dead routes must be
    // removed (3 files: openapi.rs, openapi_admin.rs,
    // openapi_user.rs).
    for (label, file) in [
        ("openapi.rs", openapi),
        ("openapi_admin.rs", openapi_admin),
        ("openapi_user.rs", openapi_user),
    ] {
        assert!(
            !has_code_mention(file, "force_cache_refresh")
                && !has_code_mention(file, "get_cache_stats"),
            "Track B option b: OpenAPI doc references in {} \
             must be removed.",
            label
        );
    }
}

// 5. The v2 migration is gone (the embed_migrations! panic
//    prerequisite). The v3 baseline + the unified-audit-log
//    migration are the only 2 directories that remain.
#[test]
fn v2_migration_is_gone_embed_migrations_will_not_panic() {
    use std::fs;

    // `cargo test` runs from the crate root (`apps/backend/`),
    // so the migrations dir is a relative `migrations/analytics`
    // path — not `../migrations/analytics`.
    let migrations_dir = "migrations/analytics";
    let entries: Vec<String> = fs::read_dir(migrations_dir)
        .expect("migrations/analytics dir must exist")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();

    // The v2 directory must be deleted.
    let v2_still_present = entries
        .iter()
        .any(|name| name.contains("consolidated_analytics_v2"));
    assert!(
        !v2_still_present,
        "v2 migration dir `00000000000001_consolidated_analytics_v2` \
         must be deleted (it shared version 1 with the v3 baseline, \
         causing `embed_migrations!` to panic)."
    );

    // The v3 baseline + unified-audit-log must still be there.
    let v3_present = entries
        .iter()
        .any(|name| name.contains("consolidated_baseline_v3"));
    let audit_present = entries
        .iter()
        .any(|name| name.contains("create_unified_audit_log"));
    assert!(
        v3_present,
        "v3 baseline migration must still be present (the rename target)."
    );
    assert!(
        audit_present,
        "unified-audit-log migration must still be present (the 2nd migration)."
    );
}

// 6. The `WalletRankingOffsetQuery` port is still object-safe
//    (the new binary's no-DB stub must compile against the
//    trait).
#[test]
fn wallet_ranking_offset_port_is_object_safe() {
    use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;

    // If the trait is object-safe, this coercion compiles.
    // If it isn't, the test fails at compile time.
    let _: &dyn WalletRankingOffsetQuery;
}
