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

The runner uses W3C WebDriver at a loopback URL. Chromium and Firefox run on
Linux; Safari uses the macOS WebDriver. Target frontend and admin URLs must also
be loopback-only. Browser-generated wasm-bindgen glue is created under
`target/` and is not repository source.

Every selected group executes every declared viewport/color matrix and every
declared repeat. `wait-for`, `click`, `fill`, and `set-input-files` issue real
WebDriver element commands. Path, query, text, selector, attribute, HTTP status,
and horizontal-overflow outcomes are asserted rather than accepted as metadata.
Artifacts are separated by browser, matrix, and repeat so later executions do
not overwrite earlier evidence.

Only signed-out scenarios without target fixture modes can run until a
scenario-state provisioner exists. The runner deliberately rejects groups that
need authenticated sessions or target fixture modes; it will not treat an
unprovisioned page load as E2E evidence. This currently makes group 0 the only
runnable group. Groups 1–9 remain a staging blocker until the provisioner can
mint audience/permission-specific test sessions, select bounded fixture modes,
reset mutations between repeats, and emit reset proof.

## Safety and evidence

- The baseline lock is immutable and validated before a campaign.
- Scenario IDs are unique and groups must be exactly 0 through 9.
- PostgreSQL test databases use an `epsx_e2e_` prefix.
- Redis test keys use an `epsx:e2e:` prefix; destructive whole-database commands are forbidden.
- Anvil must use chain ID 31337 on loopback.
- Each scenario execution emits a screenshot and page-source artifact under its
  browser/matrix/repeat directory.
- Committed evidence is verified against `evidence-manifest.json` with SHA-256.
- The runner never loads production environment files, deploys, merges, or contacts production.

Use `cargo xtask fixtures serve` for deterministic local fixture files and `cargo xtask design capture --group 0` when capturing design evidence through the same Rust WebDriver contract.
