# Rust migration E2E campaign

The scenario contract preserves campaign IDs, groups 0–9, viewport matrices, route expectations, reset evidence, screenshots, DOM snapshots, accessibility/network evidence, and SHA-256 manifests from the migration campaign. Historical baseline references remain immutable evidence; active execution is Rust-native.

```bash
cargo xtask e2e doctor
cargo xtask e2e doctor --group 0
cargo xtask e2e fixture-serve --bind 127.0.0.1:48080 --token epsx-e2e-local-reset-token
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
Artifacts are separated by browser, matrix, repeat, and scenario so later
executions do not overwrite earlier evidence.

Groups 0–9 use the loopback-only Rust fixture authority. Start it before the
frontend and admin BFFs, point `API_URL`, `BACKEND_URL`,
`CONTENT_SERVICE_URL`, `NOTIFICATION_SERVICE_URL`, and `OIDC_ISSUER` at its
loopback URL, then pass `--fixture-url` and `--fixture-token` to `e2e run` (or
set `E2E_FIXTURE_URL` and `E2E_FIXTURE_TOKEN`). The fixture mints bounded RS256
sessions for the declared audience and permissions, selects only declared
failure modes, records dependency requests and mutation hashes, and resets
state before and after every execution. A per-scenario
`*.fixture-reset-proof.json` is mandatory evidence. The runner rejects any
authenticated or fixture-mode scenario when this authority is absent or does
not advertise the selected group.

## Safety and evidence

- The baseline lock is immutable and validated before a campaign.
- Scenario IDs are unique and groups must be exactly 0 through 9.
- PostgreSQL test databases use an `epsx_e2e_` prefix.
- Redis test keys use an `epsx:e2e:` prefix; destructive whole-database commands are forbidden.
- Anvil must use chain ID 31337 on loopback.
- Each scenario execution emits a screenshot and page-source artifact under its
  browser/matrix/repeat directory plus a fixture reset proof when provisioning
  is required.
- Committed evidence is verified against `evidence-manifest.json` with SHA-256.
- The runner never loads production environment files, deploys, merges, or contacts production.

Use `cargo xtask fixtures serve` for deterministic local fixture files and `cargo xtask design capture --group 0` when capturing design evidence through the same Rust WebDriver contract.
