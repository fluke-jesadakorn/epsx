# Native release implementation and acceptance evidence

Target: **7 October 2026**. Work performed 7–8 September 2026. This is a local
implementation and rehearsal release, **not production acceptance**. No production
job, database, DNS/Tunnel route or legacy deployed contract was changed.

## Implemented

- Nine-process native development topology: Backend, Frontend/Admin/Pay BFF,
  wallet, pay, subscription, notification and analytics. Development database
  defaults are separate; starting servers never runs migrations.
- Ten release executables including the explicit migration runner, browser Wasm,
  public assets and forward migrations in a checksummed package. Runtime assets
  load from the package, outside the source checkout. `current` and `previous`
  release links support binary rollback without restoring an older database.
- Native launchd rendering, private configuration/key-file loading, and reviewed
  four-hostname named-Tunnel example. Public HTTPS issuer/SIWE identities remain
  separate from loopback service and JWKS transport. Daemons are rendered only.
- Original `up.sql` migration discovery, pending reporting, guarded Diesel/SQLx
  history adoption and conflict detection. No baseline replay over populated
  untracked schemas, no automatic down migrations, no signing-key regeneration
  in production.
- Separate non-proxy `DealEscrow` contract with immutable deal terms, payer
  release, payee refund, party disputes, Admin resolution only while disputed,
  deposit-only pause, exact floor 30-bps release fee and full refunds. No backend
  signer, arbitrary withdrawals or automatic release. A separate deployment
  script requires an explicit chain, deployer, arbiter, treasury and token list.
- `epsx-pay` SIWE sessions, HttpOnly cookies, audience checks and refresh rotation;
  canonical plural APIs, per-checkout idempotency/reservation transactions,
  wallet transaction preparation, Pay/Admin UI and receipt-confirmed operations.
- Contract-scoped reconciliation/checkpoints, confirmations, reorg invalidation
  and replay, RPC failure gating, and idempotent signed webhook hints. Presented
  deal/operation statuses must agree with stored canonical event/block evidence;
  changing only a SQL status cannot display financial success.
- Explicit private PostgreSQL/MinIO/config/key backup and isolated restore tools,
  with matching PostgreSQL CLI/server-major preflight and checksum verification.
- Native startup fixes: core event/outbox storage uses the core database and
  wallet projection checkpoints use the migrated `read_model` schema.

## Verified locally

| Check | Result and limits |
| --- | --- |
| Native release packaging | Ten release binaries, browser Wasm and assets; detached package verified by 308 file checksums on the final release |
| Detached startup | All nine processes passed health checks; Pay/notification readiness passed; CSS, JavaScript bootstrap and browser Wasm served on all three BFFs |
| Deep links and browser | Frontend analytics/portfolio and Pay link/checkout pages loaded; unauthenticated Admin escrow route rejected access; Pay wallet button invokes runtime and reports missing MetaMask correctly |
| Migrations | Fresh core/analytics/payments/notifications/pay databases, repeated up, Diesel baseline adoption with preserved sentinel data, and deliberate checksum-conflict rejection passed |
| Foundry | 14 contract tests passed, including two fuzz tests (256 runs each), uint256 maximum, liabilities, permissions, reentrancy, repeated actions and failed transfers; separate deployment script compiles |
| Pay integration | Real isolated PostgreSQL + Anvil: concurrent idempotent redemption, owner boundaries, deposit/release/dispute/Admin refund, repeated actions, pause/resume, unrelated transaction rejection, three-confirmation delay, RPC outage/recovery, duplicate forged webhook hints, reorg invalidation/replay and SQL-only false-status rejection passed |
| Live sessions | Real SIWE signature from an Anvil test account: correct Pay domain, login/me, HttpOnly cookies without token JSON exposure, refresh rotation, replay/family revocation, logout, and backend + BFF restart persistence passed |
| Session libraries | BFF 31 and identity 32 tests passed; additional production HTTPS issuer + numeric-loopback JWKS boundary regression passed; final key-file restart rehearsal passed |
| Backup/restore | Seven PostgreSQL databases restored into new isolated databases; Pay deals/links/checkouts/operations matched exact row counts and hashes; restored MinIO object bytes and backup/key/config checksums matched |
| Rollback | A new deposit was created after restore on v7; previous v5 binary retained that new transaction, amount and canonical block proof. No down migration or old-data restore was used during rollback |
| Static checks | Workspace Clippy with all targets/features and `-D warnings`, Rust formatting, strict no-node audit, asset verification, 10 launchd plists and Tunnel ingress validation passed |
| Workspace tests | Unfiltered run reached two existing frontend navigation assertion failures. A follow-up run explicitly skipping only those two: **2,193 passed, 0 failed, 31 ignored, 2 filtered**. This is not a fully green unfiltered suite |

The two outstanding navigation assertions are
`ssr::tests::shared_navigation_inlines_icons_and_preserves_progressive_actions`
and `ssr::tests::shared_navigation_uses_lg_desktop_and_mobile_contract` in
`apps/frontend/src/ssr.rs`; their expected Developer menu differs from the current
navigation. No UI requirement was invented to make those assertions pass.

## Evidence and local review environment

- Release: `target/native-releases/20260907-validation-v7` (final transport and
  evidence-validation revision; build log `/tmp/epsx-native-package-final.log`).
- Detached review root: `/tmp/epsx-native-rehearsal-20260907`; `current` points at
  the final release and `previous` retains the prior reviewed package.
- Rehearsal URLs: Frontend `http://localhost:39700`, Admin
  `http://localhost:39701`, Pay `http://localhost:39752`, Backend
  `http://127.0.0.1:39180`. These are isolated test ports, not production routes.
- Internal service ports: wallet 39102, Pay 39103, subscription 39104,
  notification 39106 and analytics 39107. Redis 39379, MinIO 39100/39101,
  restored MinIO 39400/39401, Anvil 38545. Do not expose these through Tunnel.
- Backup: `/tmp/epsx-native-backup-20260907-v3`; restore evidence:
  `/tmp/epsx-native-restored-20260907-v3/result.json` and `rollback-evidence.json`.
  These contain test data and private test keys; they are not release contents.
- Migration evidence: `target/native-validation`; Foundry log:
  `/tmp/epsx-deal-escrow-tests.log`; Pay integration:
  `/tmp/epsx-pay-integration-final.log`; workspace checks:
  `/tmp/epsx-native-workspace-tests-v2.log`,
  `/tmp/epsx-native-workspace-tests-remaining.log`,
  `/tmp/epsx-native-clippy-final.log`. Final transport boundary tests: Pay library 10, Pay binary 6 and service-auth 9 passed; the opt-in Anvil integration passed separately. Wallet/subscription/notification/analytics tests also passed after the transport change.
- The analytics service needs both the backend analytics and analytics-service
  migration families in its database.

## Gates still required before real-money launch

1. Resolve the two navigation tests and run the unfiltered suite green; review
   the 31 explicitly ignored integration tests rather than treating them as passes.
2. Complete real browser + wallet approval/deposit/release/refund/dispute/Admin
   resolution journeys, including USDT/USDC, refresh during a long checkout,
   repeat checkouts and browser/process interruption. The in-app browser used
   for inspection has no MetaMask; HTTP/Anvil tests do not replace this gate.
3. Verify free/paid rankings, cross-owner watchlist denial and legacy package
   purchase-to-entitlement end to end against representative development data.
   The new escrow never grants a subscription on its own.
4. After an explicit deployment instruction, deploy the reviewed contract to
   BSC testnet and verify actual RPC receipts, token behavior and confirmations.
   Complete contract/backend security review and recovery/reorg monitoring drills.
5. Rehearse backup/restore and binary rollback with representative production
   data under approved access, validate production signing-key continuity,
   domain/SIWE/cookie behavior, and boot/process-restart supervision.
6. Only after the gates above and a separate explicit production instruction:
   deploy the mainnet contract, install reviewed native daemons and switch the
   existing EPSX Tunnel routes; retire only the verified EPSX bridges/NodePorts.

Local smoke tests and Anvil events are not proof that the live production system
or the real-money payment flow is ready to launch.
