# Native release operations

Build: `cargo xtask native package --output target/native-releases/<release-id>`.
This builds ten native executables with the existing release profile, the three
Dioxus Fullstack server/client bundles, and the recovery service worker. It copies
complete hydration assets and forward migrations. It does not deploy or migrate.
Output must be a new directory; `manifest.json` records file checksums. Never put
secrets or production database dumps in a release.

CI uses this same native package command with pinned Dioxus/wasm-bindgen CLIs.
`audit no-node --strict` checks the reviewed browser adapters by exact source/hash;
the older marker inventory and archived Kubernetes overlays are not native
release gates. Formatting, strict workspace Clippy, workspace tests, frozen
assets/E2E evidence, migration/authority audits and Foundry tests remain gates.

`ci-migrations.py` requires eight explicit family URLs pointing to empty local
`epsx_*_shadow` databases (analytics may share its database with analytics-service).
It applies every forward migration, adds preservation sentinels, then applies
again and verifies the unchanged checksum ledger and sentinels. It refuses a
populated starting database. The CI workflow additionally checks split-database
plan reconciliation and preservation through the same-database plan trigger.

Copy the release to `/opt/epsx/releases/<release-id>`. Keep configuration and
persistent signing keys outside releases, for example `/etc/epsx`. Copy the
`*.env.example` templates to that private directory, remove `.example`, fill in
existing credentials/key paths and `chmod 600 *.env`. Do not regenerate signing
or refresh-token HMAC keys. Populate `migrate.env` with the named family URLs
shown by `migrate --help` / the runner source; service URLs may share a physical
PostgreSQL database but each family keeps its own ledger.

All public issuer/SIWE origins use HTTPS. Internal service calls use numeric
loopback HTTP. Per-process `PORT` settings are in the templates. No extracted
service is exposed through Tunnel.

Run from any directory with `/bin/bash <release>/ops/run-service.sh <binary>
<absolute-config-directory>`. This loads only explicit configuration and reads
assets/migrations from the release. Migrations are an explicit operation:

```
/bin/bash <release>/ops/run-service.sh migrate /etc/epsx pending
/bin/bash <release>/ops/run-service.sh migrate /etc/epsx up
```

The runner refuses untracked populated schemas, checksum mismatches, unknown
legacy versions and gaps in baseline history. A Diesel entry is adopted with
its original version and a recorded adoption source, without executing its SQL.
Historical Diesel never stored a checksum; adoption pins the reviewed release
content for all subsequent runs. Reconcile archived or manually applied history
against a backup before adoption; do not fabricate a baseline ledger.

Render (only) launch daemon plists:

```
cargo xtask native launchd --release /opt/epsx/current --config /etc/epsx \
  --output target/native-launchd --user <service-user> \
  --cloudflared /opt/homebrew/bin/cloudflared --tunnel-config /etc/epsx/tunnel/config.yml
```

After cutover is explicitly authorized, create `/etc/epsx/logs` writable by the
service user, install the reviewed plists under `/Library/LaunchDaemons` and
bootstrap them in the system launchd domain. System daemons start after boot;
LaunchAgents start at user login. Run `cloudflared` as a separate system service
with the reviewed EPSX named-tunnel configuration. Preserve unrelated tunnels.
Omit both Tunnel flags if retaining an existing remotely managed Tunnel daemon.
Validate named-tunnel ingress and DNS in the active management mode (remote
configuration versus local YAML) before switching traffic.

Maintain `current` and `previous` symlinks. To switch/rollback, stop only EPSX
native jobs, atomically replace `current` with the selected complete release and
start them again. Never run down migrations or restore an old database during a
binary rollback: new transactions must remain. Inspect `/ready`, assets, deep
links, sessions and Pay checkpoints before accepting traffic. The old release
must tolerate additive schema changes; rehearse this against a restored database
containing transactions written by the new release. Remove the EPSX socat/NodePort
bridges only at the authorized cutover, after checking their exact labels/ports.

Backup: supply existing database URLs, `EPSX_KEYS_DIR`, `EPSX_CONFIG_DIR` and `EPSX_MINIO_DATA_DIR`
(the existing native MinIO data directory) to `backup.py --output <new-directory>
--writers-quiesced --minio-stopped`. Quiesce EPSX writers and stop the EPSX MinIO instance for the cross-store recovery point.
Backups are private, checksummed and must also be copied to a separate failure
domain. `restore-rehearsal.py` refuses existing output directories and restores
only to newly created `epsx_restore_*` databases. It copies full MinIO data (including versions/metadata), secrets and keys to an
isolated directory; run a separate MinIO instance on it for application tests.
No backup or restore script is run implicitly by build/start/deploy.
Set `EPSX_PG_BIN` to the PostgreSQL server's matching CLI directory (for example
`/opt/homebrew/opt/postgresql@14/bin`). Backup checks each server major before
writing dumps; restore checks the saved dump major before creating databases.

For this cutover, `system-runtime.py import-candidate --snapshot <verified-backup>`
creates seven new `epsx_candidate_*` databases in the isolated system PostgreSQL
instance on 55433. It verifies every snapshot checksum, restores under separate
database-owner roles without superuser/role-management/database-creation rights,
and stores private connection settings under `/etc/epsx/candidates`. Analytics
and the analytics service share one database. Use `--check-only` before the root
operation. This does not install app jobs, replace an existing database, or
change traffic; the source remains live until the final frozen snapshot.

`encrypted_backup.py encrypt --input <snapshot> --output <new.age> --identity
<persistent-age-identity>` encrypts the complete verified snapshot and verifies
an isolated decryption before reporting success. `decrypt` requires a new output
directory and verifies both age authentication and every manifest checksum;
links, traversal paths and unlisted files are rejected. Keep the age identity
outside the archive and preserve it across deployments. A lost identity makes
encrypted backups unrecoverable.

`backup-cycle.py` is the root system-job entry point, installed together with
`backup.py`, `pg_tools.py` and `encrypted_backup.py` in `/opt/epsx/ops`. It uses
`/etc/epsx/backup/config.json` (an `environment` object containing all eight
database-family URLs) and `/etc/epsx/backup/identity.txt`, both root-owned mode
600. It stops only the loaded EPSX native app/Tunnel/MinIO jobs, creates and
verifies the encrypted snapshot, then resumes those exact jobs. Tunnel restart
requires backend readiness. An interrupted run leaves a recovery journal;
the next invocation restores jobs and reports the interruption. Retention
keeps seven daily and four ISO-week recovery points. Schedule at 03:00 in the
system launchd domain after the final runtime configuration is installed.
These operations cause a brief maintenance interval and are not run at build
or app startup. For this host the user chose local encrypted backups; they do
not protect against loss of the entire machine.

Pay v1 requires `PAY_ESCROW_V1_CONTRACT`, `PAY_ESCROW_ADMIN`,
`PAY_ESCROW_DEPLOYMENT_BLOCK`, `PAY_ESCROW_RPC_URL`, `CHAIN_ID`, and
`PAY_ESCROW_TOKENS` (JSON mapping BNB/USDT/USDC to address and decimals). Set these
only after deploying and reviewing the new non-proxy contract on the intended
chain. Backend processes never need a fund-moving private key. Default minimum
confirmations are mainnet 15, testnet 3, Anvil 1. A reorg invalidates projections
and replays from the deployment checkpoint; requests remain gated while the
scanner is unavailable or catching up. The legacy package-payment monitor keeps
its original contract configuration.

Event scans use inclusive batches of at most ten blocks. Readiness remains
unavailable until the checkpoint catches up after startup or a reorg. Contract
validation checks version, fee, authority, treasury and configured token
decimals, including QR tokens. `PAY_ESCROW_TREASURY` defaults to the configured
Admin when omitted. Production RPC throughput/quota must be verified before
accepting payments; a bounded log range alone is not proof of sufficient quota.

### Frontend development through dev.epsx.io

#### Default: workspace UI hot reload

Run `python3 infrastructure/native/dev-control.py hmr ui` to start Frontend,
Admin and Pay with `dx serve --hot-reload true` through the existing dev domains
on ports 3000, 3001 and 3002. Use `hmr bff-frontend`, `hmr bff-admin` or
`hmr bff-pay` for one UI. Backend, internal services, Anvil and Tunnel are not
restarted. Existing external dev configuration and keys are preserved.

The command reuses already configured LaunchAgents and waits for an actual
rendered page before starting the next UI (up to 30 minutes per initial build).
One shared recovery-worker watcher runs as `com.epsx.dev.ui-worker`. All builds
use the root Cargo workspace/lockfile and `target/`, with two Cargo jobs and
sequential client/server builds. The shared asset worker strips optional DWARF
sections from completed dev WASM bundles atomically (DX 0.7.9 retains them even
with `--debug-symbols false`). Code, exports and RSX HMR data stay intact. This
reduces asset downloads without another Rust build. DX responses use
`Cache-Control: no-store` to prevent stale hydration assets through the tunnel.
Rust incremental graphs are disabled for these DX jobs: concurrent DX 0.7.9
builders can invalidate the same shared crate's graph. Compiled Cargo dependency
artifacts are still shared; RSX hot reload does not depend on Rust incremental.
DX bundles remain separate under
`target/dx/{dx-frontend,dx-admin,epsx-pay}`. Cargo reuses dependencies whose
target, features and profile match; native and WASM artifacts are distinct.

RSX edits can hot reload across shared UI crates. Rust logic/signature changes
rebuild automatically; experimental Rust hot-patching is disabled. Tailwind
is managed by DX for Frontend/Admin. Public files and CSS embedded with
`include_str!` may require a rebuild in DX 0.7.9. Generated build directories
are not watched. Do not run the older `--all-hmr` command alongside these jobs:
it starts backend services as well and would conflict with occupied ports.

Inspect `python3 infrastructure/native/dev-control.py status ui` and logs in
`~/.config/epsx/dev/logs/{bff-frontend,bff-admin,bff-pay,ui-worker}.log`.
Verify HTTP and the `/_dioxus` WebSocket through each dev domain after changes;
the DX loading page alone is not proof that the app is ready.
The asset worker is installed under `~/.config/epsx/dev/tools`, like the Anvil
helper, with the checkout passed explicitly. Test orchestration and bundle
processing with `python3 infrastructure/native/test_dev_control.py`.

Restore the exact previous UI LaunchAgent configurations with
`python3 infrastructure/native/dev-control.py restore-hmr ui` (or one UI name).
Backups live beside the LaunchAgent plists with `.plist.pre-hmr` suffixes.
The worker watcher is stopped when the last HMR UI is restored. Production
release packaging and production jobs are unaffected.

#### Legacy full-build watcher

Run `python3 infrastructure/native/dev-control.py watch bff-frontend` to serve
this checkout through the existing dev tunnel on port 3000. The dev LaunchAgent
runs `dev-frontend-watch.sh`, watches frontend/shared Rust sources, rebuilds the
recovery worker plus matching Dioxus SSR/WASM assets and frontend, and restarts the frontend on edits. Refresh the
browser after compilation completes; rebuilding can briefly interrupt requests.
It preserves the external dev environment configuration and uses workspace
assets. No release packaging, migrations, DNS updates, or production changes are
needed. Admin and backend keep their existing dev processes.

Logs: `~/.config/epsx/dev/logs/bff-frontend.log`. To restore the packaged dev
frontend, run `python3 infrastructure/native/dev-control.py install bff-frontend`
and then `python3 infrastructure/native/dev-control.py restart bff-frontend`.

The frontend dev watcher serves a private snapshot of the completed DX native
server, WASM/public assets, styles and recovery worker. Other DX builds can then
replace their build outputs without mixing new client code with an old SSR tree.
The snapshot is removed when its dev process exits; production binary names and
launchd configuration are unchanged.
