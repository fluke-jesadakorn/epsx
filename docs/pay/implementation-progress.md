# EPSX Pay implementation and qualification

Status: **implemented locally; not approved for production cutover**. Recorded 8 September 2026. The target remains 7 October 2026, subject to the outstanding gates below.

No BSC contract was broadcast, production database changed, signing key replaced, or production Tunnel/DNS route switched. All transaction evidence in this report uses isolated PostgreSQL databases and Anvil chain 31337 with test accounts. The previous local review runtime on ports 39700/39701/39752 remains separate.

## Implemented

- Public merchant registration through signed wallet authentication; merchant-scoped hashed, revocable test/live keys; tenant isolation; hosted links and guest checkout capabilities. Buyers connect and sign transactions without a Pay account or SIWE login.
- Separate non-upgradeable direct-payment and public merchant escrow contracts. Direct payments charge 50 bps; public escrow charges 100 bps on release. Full direct refunds retain the original fee and require the merchant's signature and full funding. Escrow refunds return the deposit without a fee. The existing 30 bps escrow contract and legacy package-payment interpretation remain separate.
- Immutable checkout terms, integer token amounts, idempotency conflicts, transactional limited-link reservations, duplicate protection, wallet-signed transaction preparation and confirmation polling.
- Confirmed-chain reconciliation with exact receipt/event/calldata verification, contract/token validation, persistent checkpoints, canonical block evidence, corrective revisions and reorg recovery. Each new payment has one financial status writer.
- Transactional webhook outbox, timestamped raw-payload HMAC signatures, endpoint rotation, leases, 72-hour retry windows, 30-day replay/log access, and public HTTPS destination validation with pinned DNS and redirects disabled.
- Merchant dashboard, payment details and refund actions, hosted checkout, integration documentation, and Admin escrow resolution and pause/resume screens.
- EPSX backend order pricing, authenticated Pay checkout creation, signed webhook verification and authenticated payment retrieval, idempotent purchase grants, and purchase-specific refund/reorg reversals. Manual plan assignments and unrelated purchases are preserved. Paid permissions are reloaded from PostgreSQL even after Redis cache loss.
- Native release packaging of all nine servers, browser Wasm, CSS/public assets, migrations and Pay documentation. Configuration and existing keys remain outside releases.

Public integration details are in [merchant-api.md](merchant-api.md), configuration and deployment gates in [operations.md](operations.md), and the launch pricing and revenue scenarios in [revenue-model.md](revenue-model.md).

## Verification completed

| Check | Result |
|---|---|
| `cargo test --workspace --locked`, unfiltered final source | 2,189 passed; 34 existing opt-in tests ignored; 96 suites; no failures |
| Explicit merchant PostgreSQL/Anvil integration and webhook unit tests | 4 passed, including the otherwise ignored integration test |
| Explicit EPSX core fulfillment integration and unit tests | 4 passed, including the otherwise ignored database tests |
| All Foundry contract tests | 28 passed across legacy escrow and merchant contracts; fuzz tests included |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed |
| Workspace format, no-Node audit and asset verification | Passed |
| Browser runtime Wasm build/check | Passed; included in native package |
| Fresh core/Pay migrations, populated-history upgrade and repeated `up` | Passed; repeated migrations preserved all 19 merchant/fulfillment/plan table hashes |
| Historical checksum conflict | Runner rejected a modified migration in an isolated copied fixture; original migrations unchanged |
| Detached release startup | Nine services healthy outside source checkout; Pay chain readiness passed |
| HTTP application and assets | Frontend `/` and `/analytics`, authenticated Admin merchant escrows, Pay dashboard/docs/guest deep link and seven referenced assets passed |
| Signed sessions | Real Anvil SIWE signatures through Pay, Frontend and Admin BFFs; HttpOnly cookies, refresh rotation, restart persistence and Pay logout checked |

The explicit merchant integration covers BNB, allowlisted test USDT and USDC in both payment modes; full refunds; disputes and Admin resolution; tenant/capability boundaries; repeated requests and limited-link contention; malformed transaction evidence; RPC outages; reorg/checkpoint recovery; duplicate/out-of-order webhook handling; failed delivery, rotation, replay, lease recovery, retry exhaustion and private-network endpoint rejection.

Packaged HTTP journeys additionally verified:

1. A guest paid through a reusable link without SIWE. The backend rejected checkout access without its capability and confirmed the exact 0.5% fee from chain evidence.
2. EPSX created a 100 USDT package checkout through the public API. A verified payment event granted the purchase once; duplicate delivery did not grant twice. A full refund revoked that purchase and preserved unrelated plan assignments.
3. After another purchase/refund, the test restarted Redis and reused a JWT still containing the paid scope. The backend's effective permissions correctly excluded the refunded entitlement.
4. After restore, a new merchant escrow was funded, disputed and released by a signed Admin transaction while new deposits were paused. It earned exactly 1%, emitted one settlement event, and pause was subsequently removed.

The packaged EPSX webhook receiver journey used a **local transport fixture** that signed actual persisted Pay events. EPSX then retrieved the payment through the authenticated public Pay API. Actual delivery-worker HTTP behavior was tested separately against a local test-only transport. No external merchant webhook destination was contacted, and production SSRF protections were not relaxed.

Qualification exposed and fixed two integration defects: Admin's route allowlist initially rejected its registered merchant endpoints, and operation polling locked rows in the reverse order from reconciliation. Regression tests now cover both, including a real PostgreSQL contention test.

## Recovery evidence and limits

The rehearsal stopped only the nine isolated application writers and their isolated MinIO process, then backed up PostgreSQL, configuration, persistent keys and stopped object storage. It restored into new `epsx_restore_*` databases/directories. All 19 merchant, fulfillment and plan tables matched their pre-backup counts and hashes. The restored MinIO served `native-rehearsal/probe.txt` with the original checksum. Existing API credentials, signed sessions, paid/refunded proofs and entitlements remained usable after application restart. Analytics database aliases retain their original shared-database relationship; the backup also includes the duplicate logical-family dump.

New payment records created after restore survived a temporary Pay-service binary switch from candidate v7 to v6 and back to v7, with no database rollback. Confirmed records and checkpoints remained readable/reconcilable. **This is a compatibility rehearsal, not approval to serve traffic with v6**: v6 contains the polling lock-order defect fixed in v7. Gateway, authorization and Admin remained on v7. A normal whole-system rollback to an independently qualified prior merchant release remains outstanding. Pre-merchant binaries cannot own these new payments; keep compatible reconciliation running during any partial rollback.

This is a local test-data restore, not a rehearsal against production data or an off-machine disaster recovery test. The legacy package monitor remains configured separately; its production-equivalent purchase flow is still a release gate.

## Review runtime and artifacts

| Application | Local review URL |
|---|---|
| Frontend | http://localhost:39800 |
| Admin | http://localhost:39801 |
| Pay | http://localhost:39852 |
| Public API gateway | http://127.0.0.1:39280/api/v1/pay |

Only the local test environment is configured. These are HTTP development origins; production must retain its configured HTTPS domains, issuer, audiences and secure cookies.

- Packaged candidate: `target/native-releases/20260908-merchant-validation-v7` (manifest checksums verified).
- Detached runtime: `/tmp/epsx-merchant-rehearsal-20260908/current`, using restored isolated data.
- Private backup: `/tmp/epsx-merchant-backup-20260908-v1`; restored files: `/tmp/epsx-merchant-restored-20260908-v1`. These contain secrets and must not be published.
- Sanitized machine-readable evidence: [qualification-2026-09-08.json](qualification-2026-09-08.json).
- Test logs: `/tmp/epsx-merchant-workspace-tests-release.log`, `/tmp/epsx-merchant-integration-v4.log`, `/tmp/epsx-merchant-fulfillment-v7.log`, `/tmp/epsx-all-contract-tests.log`, and `/tmp/epsx-merchant-final-clippy.log`.

## Outstanding launch gates

1. Real browser/wallet journeys and visual/accessibility review on all three apps. The Mac remained locked and CUA could not unlock it, so HTTP/Anvil tests do not substitute for browser acceptance.
2. Explicitly authorized BSC testnet deployment, reviewed real token addresses/decimals, sufficient-confirmation journeys for both payment modes and EPSX purchases, and the legacy package purchase regression journey.
3. Contract/backend security review and real public HTTPS webhook integration, including operating procedures for disputes and endpoint changes.
4. Production-representative backup/restore, off-machine recovery, and a qualified prior merchant release for whole-system rollback.
5. Explicit deployment instructions for any new BSC contract broadcast and production cutover after the gates pass.

The API currently lists the newest 100 records; arbitrary historical pagination is not implemented. Stored history and direct resource lookup remain available. Pricing has no fiat conversion, recurring debit, partial refunds or additional chain support.
