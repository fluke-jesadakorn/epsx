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

The current scan contains 64 records:

| Usage | Records | Readiness effect |
| --- | ---: | --- |
| Dioxus security gates | 32 | 31 legacy two-segment and 1 impossible scoped wildcard; all 32 block |
| Dioxus presentation literals | 1 | Reported as presentation drift, not an enforcement blocker |
| Dioxus presentation dynamic pass-throughs | 1 | Reported as presentation drift, not an enforcement blocker |
| Service-authorization permissions | 30 | All canonical three-segment values |

Across every source there are 30 canonical three-segment values, 31 legacy
two-segment values, 1 unknown dynamic presentation value, and 2
impossible/cross-grammar values. There are currently no wildcard-aligned
inventory values.

## Source-backed admin remediation map

These are migration candidates, not policy changes:

| Dioxus surface | Source-backed backend candidate |
| --- | --- |
| Dashboard | `admin:dashboard:view` |
| Analytics and audit log | `admin:analytics:view` |
| Notifications | `admin:notifications:manage` |
| Developer portal | `admin:developer:manage` |
| Payments | `admin:payments:manage` |
| Chat | `admin:chat:manage` |
| News | `admin:content:manage` |
| Media | `admin:media:manage` |
| Settings | `admin:settings:manage` |
| Wallet list/detail | `admin:users:read` |
| Wallet disable mutation | `admin:users:update` |
| Wallet access | split by operation: `admin:permissions:read` / `admin:permissions:manage` |
| Wallet plans | split by operation: `admin:plans:read` / `admin:plans:manage` |

The admin auth, policies, and generic unauthorized presentation surfaces have
no single source-backed backend guard candidate in this audit. They remain A4
decisions; the inventory deliberately does not guess.

The frontend analytics gate is also unresolved. Token examples use
`epsx:analytics:read`, while wallet permission assignment uses
`epsx:analytics:view`. The other frontend gate strings have no real backend
guard tied to those UI surfaces in the audited sources, so their candidate
lists remain empty pending A4 decisions.

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
