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

The current scan contains 46 records:

| Usage | Records | Readiness effect |
| --- | ---: | --- |
| Dioxus security gates | 12 | 10 canonical; 2 legacy two-segment values still block |
| Dioxus presentation literals | 1 | Reported as presentation drift, not an enforcement blocker |
| Dioxus presentation dynamic pass-throughs | 1 | Reported as presentation drift, not an enforcement blocker |
| Service-authorization permissions | 32 | All canonical three-segment values |

Across every source there are 42 canonical three-segment values, 2 legacy
two-segment values, 1 unknown dynamic presentation value, and 1
impossible/cross-grammar presentation value. There are currently no wildcard-aligned
inventory values.

## Source-backed admin consumption map

The unambiguous A8.1 rows are now consumed by the UI. This changes only gate
literals; it does not move business policy into Dioxus:

| Dioxus surface | Source-backed backend permission | A8.1 state |
| --- | --- | --- |
| Dashboard | backend dashboard read/field decision required | fail-closed UI; no operational aggregate or action is exposed |
| Analytics | `admin:analytics:view` | aligned |
| Audit log | dedicated backend audit read permission required | fail-closed UI; the semantically wrong analytics presentation gate was removed |
| Notifications | backend read decision / `admin:notifications:manage` mutation authority | fail-closed manage/create UI; no records or actions are exposed |
| Developer portal | backend read decision / `admin:developer:manage` mutation authority | fail-closed UI; no credentials, usage data, or action is exposed |
| Payments | `admin:payments:view` | aligned for the read-only intent surface; mutations remain unavailable |
| Chat | backend read decision / `admin:chat:manage` mutation authority | fail-closed list/detail UI; no conversation data or action is exposed |
| News | backend read decision / `admin:content:manage` mutation authority | fail-closed list/create/edit UI; no records or actions are exposed |
| Media | backend read decision / `admin:media:manage` mutation authority | fail-closed UI; no object data or action is exposed |
| Settings | backend read decision / `admin:settings:manage` mutation authority | fail-closed UI; no configuration, credential, session, or action is exposed |
| Wallet credits | backend read decision / `admin:credits:manage` mutation authority | fail-closed UI; no balance, ledger data, or financial action is exposed |
| Wallet list/detail | `admin:users:read` | aligned |
| Wallet disable mutation | `admin:users:update` | aligned |
| Wallet access | split by operation: `admin:permissions:read` / `admin:permissions:manage` | aligned in A8.2; data remains visible to readers and mutation controls are nested |
| Wallet plans | split by operation: `admin:plans:read` / `admin:plans:manage` | aligned in A8.2; list data remains visible to readers and mutation controls are nested |

The dashboard, audit, chat, media, news, notification, developer-portal,
settings, and wallet-credit read decisions remain backend-owned;
removing their frontend literals does not remove or weaken any service guard.
The admin auth and policies gates have no single source-backed backend guard
candidate in this audit. The generic unauthorized value is presentation-only.
These remain explicit residuals; the inventory deliberately does not guess or
over-broaden them. The two new A8.2 nested manage-gate source records are
canonical evidence and do not increase the readiness blocker count.

The frontend analytics gate is also unresolved. Token examples use
`epsx:analytics:read`, while wallet permission assignment uses
`epsx:analytics:view`. The other frontend gate strings have no real backend
guard tied to those UI surfaces in the audited sources, so their candidate
lists remain empty pending A4 decisions.

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
remains. Presentation-only drift is reported separately and does not inflate
the security-gate blocker count. Readiness may exit `0` only after source code
and this checked inventory are updated by their owning packages and integrity
still passes.
