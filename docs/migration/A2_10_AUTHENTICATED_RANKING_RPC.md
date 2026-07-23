# A2.10 authenticated ranking RPC composition

Evidence date: 2026-07-22 (Asia/Bangkok)

## Outcome

A2.10 adds one exported but deliberately unwired identity-library composition
for `GetWalletRankingOffset`. The composition accepts authentication only from
the exact gRPC `authorization` metadata value, parses one strict
`Bearer <credential>` value, invokes a workload-specific authorizer once,
requires subject `epsx-analytics-service` and audience
`epsx-identity-service`, validates and lowercases one canonical EVM wallet,
then invokes the existing ranking query port once.

The boundary fails closed before wallet or ranking work. Missing, duplicate,
non-ASCII, malformed or rejected credentials are indistinguishable
`UNAUTHENTICATED` responses. Authorizer outage is `UNAVAILABLE`; a verified
but wrong workload is `PERMISSION_DENIED`; an invalid wallet is
`INVALID_ARGUMENT`; ranking-store unavailability is `UNAVAILABLE`; corrupt or
unexpected authority data is a sanitized `INTERNAL` response. Credential and
wallet values enter the request boundary, but they and raw database details,
correlation identifiers, and `AppError` messages never enter outward gRPC
`Status` messages.

## Frozen boundary

The immutable target base is
`60ababc75a79d173b3b217df8e9b9155795a1117`, the completed post-A2.9
snapshot. A2.10 adds only
`shared/rust/epsx-identity-service/src/authenticated_ranking_rpc.rs` and one
library export in `shared/rust/epsx-identity-service/src/lib.rs`.

The production `main.rs`, existing always-Free identity service, A2.7 resolver,
A2.8 core adapter and shared snapshot contract, identity proto, analytics
client, identity Cargo manifest, workspace lockfile, historical event modules,
database/migration/schema paths, infrastructure, UI, payment and indexer paths
remain unchanged from that target base.

## Exact fail-closed order

The in-process RPC composition has one permitted order:

1. Require exactly one ASCII `authorization` metadata value.
2. Require exact case-sensitive `Bearer ` plus a non-empty credential with no
   ASCII whitespace.
3. Invoke the workload authorizer exactly once.
4. Require the exact workload subject and audience.
5. Validate exactly 42 ASCII bytes, lowercase `0x`, and 40 ASCII hex digits;
   accept mixed-case hex and lowercase it exactly once. Do not trim, resolve
   ENS, accept alternate prefixes, or reject the all-zero address.
6. Invoke the ranking query exactly once with only the normalized wallet.
7. Return the offset or the fixed sanitized status/message pair.

Authentication and workload authorization therefore precede wallet parsing,
query work, and any future store adapter. This order is required before the
A2.8 adapter can be considered for runtime activation.

## Proto and runtime non-activation

Authentication uses tonic metadata and adds no protobuf field. The identity
proto remains byte-identical to the target base, including request field
`wallet = 1` and response field `offset = 1`. The analytics client still sends
no authorization metadata. No concrete credential verifier, issuer trust,
key rotation, credential source, tonic interceptor, TLS policy, listener,
channel, database adapter or deployment wiring is added.

`main.rs` remains byte-identical and still constructs
`FreePlanRankingOffsetService`; successful runtime queries are therefore still
unauthenticated always-Free legacy-shim results. Passing A2.10 never represents
runtime activation.

## Hermetic evidence

The verifier reads only local Git objects and local files, freezes SHA-256
digests, rejects production-looking and live dependency variables, removes
proxy variables and forces Cargo offline. It proves the exported/unwired
module shape, exact ordering and status matrix, unchanged proto and runtime
surfaces, and the absence of listener/network/store code in the new module.

Eleven exact library tests use fake authorizer/query counters and in-process
tonic requests. The verifier enumerates the library test list, requires every
name to be unique, and runs each fully qualified test with `--exact`. No
listener, socket, channel, TLS session, database or service is started.

The deterministic self-test copies only declared evidence. Its source tamper
cases update both contract and copied-verifier digest pins before moving wallet
normalization ahead of authorization or changing the forbidden-caller status.
Those mutations must reach and fail the semantic ordering/status assertions,
not merely a hash check. Readiness continues to exit `3`.

## Residual STOP conditions

- No concrete workload credential verifier or trusted issuer/signature/expiry
  adapter exists.
- Credential issuance, storage and rotation are undefined.
- The analytics client injects no identity-RPC authorization metadata.
- Identity RPC transport has no reviewed workload TLS/mTLS policy.
- The A2.8 store adapter, schema adoption and query-plan proof remain absent
  from runtime composition.
- Production `main.rs` still serves the unauthenticated always-Free legacy
  shim; A2.10 is exported but unwired.
- No owner-delegation or wallet-on-behalf-of binding exists between the
  workload credential and the queried wallet.
- No runtime, listener, network, image, manifest rendering, deployment,
  staging, live parity, cutover or rollback proof exists.
- No production action is authorized.

## Gate usage

```sh
./scripts/migration/verify-a2-10-authenticated-ranking-rpc.sh --mode integrity
./scripts/migration/verify-a2-10-authenticated-ranking-rpc.sh --mode report
./scripts/migration/verify-a2-10-authenticated-ranking-rpc.sh --mode readiness  # expected exit 3
./scripts/migration/test-a2-10-authenticated-ranking-rpc.sh
```

Passing integrity proves only a hermetic, fail-closed, unwired composition
seam. It is not evidence of credential trust, owner delegation, authoritative
paid entitlement, database readiness, transport security or production
readiness.
