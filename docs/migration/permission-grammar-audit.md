# Permission grammar contract audit

This A4 artifact is a read-only migration gate. It inventories permission text
already present in Dioxus `AuthGate`/`AdminAuthGate` usages, presentation-only
`AccessDenied` usages, and the A2 service-authorization matrix. It does not
change authorization behavior and it is not a production-readiness claim.

The evidence is pinned to
`origin/development@373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.

## Authority boundary

The Rust backend remains the only business-policy authority. Its canonical
permission grammar is `platform:resource:action`, with exact matching plus only
these wildcard shapes: `*:*`, `*:*:*`, `platform:*:*`, and
`platform:resource:*`. A scoped two-segment value such as `admin:*` is not a
platform wildcard in the backend matcher. Shapes such as `admin:*:read` and
`*:users:read` are also outside the canonical wildcard grammar.

A4 owns the canonical permission decision. A7 owns any later frontend gate
remediation, and A8 owns any later admin gate remediation. Those packages must
consume backend decisions; they must not create permission or entitlement
policy in Dioxus.

## Deterministic inventory

The checked contract is
[`contracts/permission-grammar.json`](contracts/permission-grammar.json). Each
record carries source type, file, line, permission, surface, grammar
classification, and a remediation package. A candidate permission is present
only when a real backend route guard or token source literally supports it.

The current scan contains 35 records:

| Usage | Records | Readiness effect |
| --- | ---: | --- |
| Dioxus security gates | 1 | The remaining payment-intent read gate is canonical |
| Dioxus presentation literals | 1 | Reported as presentation drift, not an enforcement blocker |
| Dioxus presentation dynamic pass-throughs | 1 | Reported as presentation drift, not an enforcement blocker |
| Service-authorization permissions | 32 | All canonical three-segment values |

Across every source there are 33 canonical three-segment values, no legacy
two-segment values, 1 unknown dynamic presentation value, and 1
impossible/cross-grammar presentation value. There are currently no wildcard-aligned
inventory values.

## Backend permission decision map

This map records the source-backed backend permission or decision that future
live UI adapters must consume. Unavailable surfaces do not consume or emulate
those decisions: their frontend gates were removed with their sample data and
controls, while backend enforcement remains unchanged.

| Dioxus surface | Source-backed backend permission | Current A8 state |
| --- | --- | --- |
| Dashboard | backend dashboard read/field decision required | fail-closed UI; no operational aggregate or action is exposed |
| Analytics | backend `admin:analytics:view` plus field decision required | fail-closed UI; no metrics, records, status claims, filters, or export are exposed |
| Audit log | dedicated backend audit read permission required | fail-closed UI; the semantically wrong analytics presentation gate was removed |
| Notifications | backend read decision / `admin:notifications:manage` mutation authority | fail-closed manage/create UI; no records or actions are exposed |
| Developer portal | backend read decision / `admin:developer:manage` mutation authority | fail-closed UI; no credentials, usage data, or action is exposed |
| Payments | `admin:payments:view` | aligned for the read-only intent surface; mutations remain unavailable |
| Chat | backend read decision / `admin:chat:manage` mutation authority | fail-closed list/detail UI; no conversation data or action is exposed |
| News | backend read decision / `admin:content:manage` mutation authority | fail-closed list/create/edit UI; no records or actions are exposed |
| Media | backend read decision / `admin:media:manage` mutation authority | fail-closed UI; no object data or action is exposed |
| Settings | backend read decision / `admin:settings:manage` mutation authority | fail-closed UI; no configuration, credential, session, or action is exposed |
| Wallet credits | backend read decision / `admin:credits:manage` mutation authority | fail-closed UI; no balance, ledger data, or financial action is exposed |
| Wallet list/detail | backend `admin:users:read` plus ownership decision required | fail-closed UI; no wallet data or operation is exposed |
| Wallet disable mutation | backend `admin:users:update` plus ownership/idempotency/audit required | fail-closed UI; no confirmation or mutation control is exposed |
| Wallet access | backend split by operation: `admin:permissions:read` / `admin:permissions:manage` | fail-closed UI; no assignment data or grant/revoke controls are exposed |
| Wallet plans | backend split by operation: `admin:plans:read` / `admin:plans:manage` | fail-closed UI; no plan data, editor defaults, or mutation controls are exposed |

The dashboard, analytics, audit, chat, media, news, notification,
developer-portal, settings, wallet-credit, wallet, permission-system, and plan
read/manage decisions remain backend-owned;
removing their frontend literals does not remove or weaken any service guard.
The invented admin-auth gate was removed when `/auth` became a fixed redirect,
and the target-only policies gate was removed when `/policies` became an
explicit 404. The generic unauthorized value remains presentation-only. The
inventory deliberately does not guess or over-broaden it. Removing eleven A8
literals across unavailable or non-existent admin surfaces reduced the Dioxus
security inventory without changing backend enforcement.

Backend analytics vocabulary remains an A4 decision: token examples use
`epsx:analytics:read`, while wallet permission assignment uses
`epsx:analytics:view`. No Dioxus gate guesses between those meanings.

The owner notification page and its shared-header unread badge do not introduce
a frontend permission literal. They depend on a server-verified authenticated
session and an owner-bound backend/BFF read path; signed-out and public
`/offline` responses receive no badge runtime. This preserves the backend-only
authority boundary and keeps the removed `notifications:read` value absent from
the checked Dioxus security-gate inventory.

## Gate commands and exit contract

Run the integrity contract in ordinary CI:

```bash
./scripts/migration/verify-permission-grammar.sh --mode integrity
./scripts/migration/test-permission-grammar.sh
```

Integrity exits `0` only when the source scan, pinned development ref,
inventory, evidence anchors, candidate sources, counts, and classifier contract
all match. It exits `1` on drift and `64` on invalid CLI usage.

Readiness is intentionally stricter:

```bash
./scripts/migration/verify-permission-grammar.sh --mode readiness
```

It exits reserved status `3` while any non-canonical security-gate permission
remains. The current grammar-only readiness check exits `0`: the sole remaining
Dioxus security gate is canonical. The two presentation-only records remain
visible in integrity evidence but do not enforce access and do not block this
grammar check. This is not entitlement, ranking, route, or production
readiness; those remain governed by A4 and the other STOP contracts.
