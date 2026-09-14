# Operational scripts

The active workspace lifecycle is owned by `cargo xtask`. Shell scripts in this directory are limited to database administration, local funding, deployment helpers, and host operations that call Rust, Docker, Kubernetes, PostgreSQL, or Foundry directly.

Use these workspace commands for normal development and validation:

```bash
cargo xtask env validate
cargo xtask dev --all
cargo xtask setup-local
cargo xtask anvil-proxy
cargo xtask assets verify
cargo xtask e2e doctor
cargo xtask e2e run --group 0 --browser chromium
cargo xtask e2e verify-artifacts
cargo xtask audit no-node --strict
```

Production deployment is never implied by a migration, build, test, or audit command. It requires an explicit user instruction.
