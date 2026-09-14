# Backend helper scripts

These local wrappers delegate environment validation and application execution to the Rust workspace:

```bash
apps/backend/scripts/check-env.sh
apps/backend/scripts/migrate_all.sh
apps/backend/scripts/run.sh
```

The direct equivalents are:

```bash
cargo xtask env validate
cargo run -p epsx --features cli-tools --bin migrate -- up
cargo xtask dev --backend
```

Database migrations are additive and guarded. Never deploy or run production migrations without an explicit user instruction.
