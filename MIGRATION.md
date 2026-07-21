# Migration status: development to Dioxus and Rust services

This branch is an active migration, not a completed production cutover.

- Source baseline audited: `origin/development` at `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`.
- Target baseline audited: `migration/dioxus-microservices` at
  `975c09567fe14ce278370720bd7a0e5aa571e116` before the readiness-document update.
- Production system of record: the Rust monolith in `apps/backend` until each
  extracted domain passes its contract, security, data, and rollback gates.
- Deployment status: no production deployment is authorized by this document.

## What route parity means today

The Dioxus dispatchers have path-level counterparts for all 28 audited frontend
pages and all 27 audited admin pages. This is path presence only:

- `28/28 frontend` means every audited source page has a target dispatcher path.
- `27/27 admin` means every audited source page has a target dispatcher path;
  two paths intentionally redirect to canonical sub-pages.
- The E2E inventories contain route samples, dynamic examples, redirects, and
  target-only routes. Their sample count is not a functional-completion score.

Path presence does not prove matching interactions, authentication, live data,
backend authorization, checkout behavior, visual fidelity, or production
operability. Those gates remain incomplete.

## Verified narrow baseline

The `epsx-dioxus-ui` unit and documentation tests are the narrow test baseline
for shared UI components. A green result for this package does not imply that
the workspace, BFFs, microservices, migrations, Kubernetes manifests, or E2E
flows are production-ready.

Use the detailed execution plan and current evidence in
[`docs/migration/PRODUCTION_READINESS_PLAN.md`](docs/migration/PRODUCTION_READINESS_PLAN.md).

## Migration strategy

Use a controlled hybrid:

1. Keep monolith authentication, permissions, plans, and durable domain data as
   the source of truth.
2. Put contract and security tests around the BFF/monolith boundary.
3. Extract one vertical domain at a time behind internal routing.
4. Shadow and canary only after migration/backfill and rollback rehearsals pass.
5. Remove a monolith fallback only after its replacement meets the full
   definition of done.

## Production guard

Never deploy, change DNS/Cloudflare routing, apply Kubernetes resources, run a
production migration, or remove a monolith fallback without explicit user
approval for that specific production action. A passing test suite, merged
branch, or completed plan is not deployment approval.
