# Native production cutover — execution record

User authorized implementation, merging the tested result into `development`,
and production deployment after all acceptance checks pass (2026-09-13).
The user subsequently changed rehearsal from BSC testnet to local Anvil (31337),
with MetaMask used through the local browser console. Production remains BSC 56.
No second general deployment permission is required. Local acceptance and
production verification gates remain mandatory.

## Fixed decisions

- Host: this MacBook Pro M3 Max, macOS system LaunchDaemons, no login dependency.
- Native Rust/Dioxus: frontend 4700, admin 4701, pay 4752, backend 9180;
  wallet/pay/subscription/notification/analytics 38102/38103/38104/38106/38107.
- Separate PostgreSQL 17.11 at 55433, Redis at 6380, MinIO at 9100/9101.
  Preserve all development and unrelated services. Paths: /opt/epsx, /etc/epsx,
  /var/db/epsx. Named Cloudflare tunnel: epsx-prod.
- Import the four actual Neon production databases; preserve user identities,
  plans, assignments, direct permissions, credits and payment history.
- EPSX package users/plans/access remain owned by the core Rust backend.
- One login after cutover is accepted. Preserve the RSA pair recovered from the
  encrypted backup; initialize and persist the new refresh-token HMAC keyring.
- Mainnet chain 56. Admin/deployer/arbiter/treasury:
  0xea64439c9cb1b9Aa588a8D1cE61292DB4036E3dF. Company merchant/recipient:
  0x7877e415a13532d9E43Df7Fd2CC256f93a39ced7. Production previously paid Admin;
  the user explicitly selected changing package receipts to the company wallet.
- All flows: packages, DealEscrow, merchant Direct/Escrow, QR checkout. Preserve
  legacy PaymentEscrow 0x56e44c9b61Aa24D47C22414e799DA8D76B345Db0.
- Preserve production prices/promotions; USD amounts equal USDT/USDC amounts.
  BSC mainnet USDT/USDC use 18 decimals; BNB merchant amounts are explicit.
- Free RPC only, no automatic billing. Gate payments if verified indexing is
  unavailable. Alchemy Free historical queries must respect its 10-block limit.
- Local encrypted backups only: /var/backups/epsx, daily 03:00 and before deploy,
  retain 7 daily / 4 weekly. Rehearse isolated restore; backup shares this host.
- Remove the three EPSX Vercel projects and their Blob dependency after successful
  cutover and 24-hour verification. Preserve Neon/resources shared by other apps.

## Source evidence

Read-only production audit: PostgreSQL 17.11; 8 wallet_users, 12 plans,
9 wallet_plan_assignments, 34 wallet_direct_permissions, 58 plan_permissions,
29 permissions, one payment in the payments database. Ten news articles refer
to five Blob objects. Recount at final snapshot; local epsx_prod is stale.

The source ledger has mixed legacy versions, including payments entries in core
and 20260418000000. Do not delete/forge a ledger or replay a consolidated baseline.
Reconcile schema and archive evidence before adoption on a restored copy.

RSA variables in Vercel are sensitive and cannot be read back. Source configuration
and database credentials must never be placed in this repository or logs.

Wallet audit correction: the existing service-auth middleware already returns
404 for raw-key custody and placeholder balance/transaction endpoints. Preserve
that protection; remove unreachable placeholder handlers rather than enabling them.

## Gates and progress

- [x] Read actual production data/configuration without production writes.
- [x] Preserve working/index diffs and source hashes in private recovery storage.
- [x] Create codex/native-production-cutover from the existing migration checkout.
- [ ] Complete readiness, canonical wallet/plan integration and migration adoption.
- [x] Export rehearsal snapshot and five Blob objects; restore four databases on PostgreSQL 17.
- [x] Reconcile legacy schema/history and apply all four backend migration families on restored copies.
- [x] Verify original populated table rows and legacy ledger entries are unchanged.
- [x] Reconcile all 12 core plans into the payments projection, including display order; repeat check passes.
- [x] Install PostgreSQL, Redis and MinIO as isolated system LaunchDaemons.
- [x] Repair PostgreSQL launchd locale (LC_ALL=C, LANG=C); port 55433 accepts connections.
- [x] Run current development backend/wallet builds; backend /ready verifies four databases and Redis (HTTP 200).
- [ ] Add/install isolated system runtime, backup and health operations.
- [x] Compile contracts with solc 0.8.30, Paris EVM and optimizer; 33 Foundry tests pass.
- [x] Backend library: 581 passed, 15 opt-in tests ignored; wallet: 13 passed; migration runner: 2 passed.
- [x] Admin library: 196 passed, including production-shaped wallet/plan transport and session boundaries.
- [x] Restored wallet integration: all 8 identities, 9 assignments and 34 direct permissions returned from core.
- [x] Live JWT/API-key integration: revoked/expired grants and disabled wallets take effect without a Redis invalidation flag; database failure returns retryable 503.
- [x] API-key SQLx compatibility and owner/admin CRUD rehearsal: actual api_keys table/text-array columns, secret-free reads, owner-bound revocation, create/revoke replay protection and transactional local audit.
- [x] Continuous core-to-payments plan projection: exact precision/metadata, idempotent retries, schema-drift refusal and complete rollback on verification failure.
- [x] Native/merchant Anvil integration: confirmation boundaries, Direct/Escrow/QR, refunds/disputes, ownership, webhooks, RPC failure/reorg recovery; ten-block scans and treasury/decimals checks pass on a fresh isolated chain/database pair.
- [x] Encrypted backup rehearsal: authenticated age round-trip, isolated PostgreSQL restore and exact row-count/content hashes across 172 table checks in eight family restores; signing/object files in this test are explicit fixtures.
- [x] Candidate import commands restore all seven databases under restricted database-owner roles in rehearsal; system import awaits the user's root command.
- [ ] Complete local deployment via browser + MetaMask; verify all receipts and runtime bytecode.
- [ ] Test all payment flows, permissions, sessions, RPC outage/reorg and restore.
- [ ] Build and validate exact release, merge into development and test merged head.
- [ ] Deploy/verify mainnet contracts and configuration, then maintenance cutover.
- [ ] Verify services after reboot before user login.
- [ ] Complete 24-hour verification and remove EPSX Vercel dependencies.

First-cutover rollback before new writes can return to the old deployment.
After new transactions, retain current data and use a compatible binary rollback
or maintenance/fix-forward. Never restore an old database over new transactions.

Rehearsal evidence is private under `/var/tmp/epsx-cutover-20260913`; it must not
be committed. The source remains live: take a new snapshot with writers stopped
before final cutover. No production routes or application services have switched.
Only MockUSDT has been deployed and bytecode-verified on local Anvil so far.

The line above refers specifically to the MetaMask browser rehearsal. Separate
automated tests deployed and exercised DealEscrow, merchant Direct/Escrow, QR
and test tokens on fresh private Anvil instances using public test accounts.
Do not treat those as a completed browser-wallet gate or any mainnet deployment.

Startup no longer seeds/overwrites imported admin plans or production news.
Current wallet pages and plan-assignment options use core, and legacy commerce
wallet-status/access/plan forms refuse writes and require a page reload.
Admin credits now use the canonical payments ledger. Legacy minor-unit forms
refuse writes instead of changing the separate wallet-service projection.

The backup scripts are implemented and their archive/restore/retention tests
pass, but the system backup job and final key/configuration have not yet been
installed. Remaining gates include private app rehearsal on system storage,
production release tests, actual free-RPC quota/catch-up
verification, the browser wallet gate, mainnet deployment, final frozen import,
development merge, system app/Tunnel installation and before-login reboot proof.


### Latest local verification (13 September)

PostgreSQL system LaunchDaemon accepts connections on 55433 after the
LC_ALL=C/LANG=C repair. PostgreSQL, Redis and MinIO are running in the system
domain. Candidate import still awaits the root command result; application
jobs, Tunnel and production routes have not switched.

The final premerge workspace run passed 2,300 tests with zero failures and 42
explicitly ignored external-fixture tests across 103 result groups. This includes
the canonical credit commands and lint cleanup. Selected restored-data and Anvil
tests were exercised separately; ignored tests do not automatically satisfy
those production gates. The development merge and package provenance are
recorded by Git and the release manifest.

Admin Credits reads original payments balances and history through core,
retaining decimal strings. Adjustments derive the actor from verified auth,
require an idempotency key, and commit the key/result, balance and ledger entry
in one transaction. Isolated restored-database tests cover concurrent replay,
payload conflicts, pending-credit protection, precision/overflow limits and
forced completion-write rollback. The Admin provider test checks decimal
transport, canonical routing and same-origin enforcement. Development payments
were backed up before the additive 20260913000000_admin_credit_commands migration;
the new table belongs to the normal application role.

Strict all-target/all-feature workspace Clippy, formatting, frozen assets,
strict no-node, migration audit and authority audit pass. The E2E doctor passes
and verifies 4,682 evidence files. Dioxus signal reads retain explicit closures;
provider callback aliases and boxed detail payloads preserve serialized data.
Retired SSR routes remain test fixtures and are excluded from native apps.

CI now packages the actual native/Fullstack release using pinned CLI versions.
The exact browser-adapter audit replaces the obsolete embedded-marker inventory;
archived Kubernetes overlays are not release prerequisites. All eight migration
families were tested on empty isolated PostgreSQL 17 databases: core 29,
analytics 5, payments 10, notifications 11, wallet 3, pay 9, subscription 4 and
analytics-service 2. A second run preserves the checksum ledger and sentinels.
The workflow's split-database reconciliation, destination-only preservation,
analytics partition recovery and same-database plan trigger checks also pass.
Existing migration bytes were preserved. The reviewed watchlist forward SQL
replaces a named CHECK constraint without deleting rows; historical down SQL
is checksum-bound and is never executed by native tooling.

The first package at `target/native-releases/20260913-cutover-candidate` contains
ten executables and three Fullstack bundles; all 478 manifest hashes match.
It predates the final lint cleanup and records a dirty tree. It is a rehearsal
artifact: deploy only a new package from the tested committed development source.
Development backend and Pay readiness respond 200. UI watchers have rebuilt the affected sources; rendered frontend, admin and
Pay pages respond 200.

Backup archive/restore tests preserve 172 table counts/content hashes. Mocked
orchestration verifies resume after snapshot failure and journal preservation
when resume fails. These are separate from the pending real system/MinIO/key
restore rehearsal and scheduled backup-job installation.

RPC preflight: PublicNode reports BSC 56, legacy contract bytecode and recent
ten-block logs, but rejects the historical production receipt with an archive
token requirement. Binance supplies that receipt and its canonical block but
rejects eth_getLogs. A free archive provider and sustained catch-up/throughput
validation remain production gates. Additional dRPC and OnFinality public
endpoint probes returned HTTP 429; they have not qualified as a fallback.
No billing was enabled.

The browser-wallet rehearsal still requires the user's permitted MetaMask
interaction. Automated local Anvil contract tests do not represent a browser
signature or a mainnet deployment. The persistent development chain is preserved.
After the remaining gates, take a fresh snapshot with old writers stopped,
install the system apps/Tunnel, verify boot before login, and observe 24 hours
of healthy production before removing the three EPSX Vercel projects and Blob.
Preserve the shared Neon resource.

The local development merge at `94378ba88929fc102c6517e31d384eacdfa7565c`
passed the same 2,300 tests (42 explicitly ignored) across 103 result groups.
GitHub development is protected and requires a PR; PR #42 carries this release
through its required checks. Vercel preview deployments are disabled for the
EPSX frontend/admin projects; both existing production targets were verified
unchanged. Project deletion remains after the agreed observation window.
