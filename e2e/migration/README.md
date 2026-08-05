# Rust migration E2E campaign

The scenario contract preserves campaign IDs, groups 0–9, viewport matrices, route expectations, reset evidence, screenshots, DOM snapshots, accessibility/network evidence, and SHA-256 manifests from the migration campaign. Historical baseline references remain immutable evidence; active execution is Rust-native.

```bash
cargo xtask e2e doctor
cargo xtask e2e doctor --group 0
cargo xtask e2e run --group 0 --browser chromium
cargo xtask e2e run --group 9 --browser firefox
cargo xtask e2e report
cargo xtask e2e verify-artifacts
```

The runner uses W3C WebDriver at a loopback URL. Chromium and Firefox run on Linux; Safari uses the macOS WebDriver. Target frontend and admin URLs must also be loopback-only. Browser-generated wasm-bindgen glue is created under `target/` and is not repository source.

## Safety and evidence

- The baseline lock is immutable and validated before a campaign.
- Scenario IDs are unique and groups must be exactly 0 through 9.
- PostgreSQL test databases use an `epsx_e2e_` prefix.
- Redis test keys use an `epsx:e2e:` prefix; destructive whole-database commands are forbidden.
- Anvil must use chain ID 31337 on loopback.
- Each scenario emits a screenshot and page-source artifact.
- Committed evidence is verified against `evidence-manifest.json` with SHA-256.
- The runner never loads production environment files, deploys, merges, or contacts production.

Use `cargo xtask fixtures serve` for deterministic local fixture files and `cargo xtask design capture --group 0` when capturing design evidence through the same Rust WebDriver contract.
