# Wave 12 — Integration gate — final deliverable

> **Branch:** `wave12/integration` (worktree at
> `.worktrees/wave12-integration`, base
> `origin/migration/dioxus-microservices` HEAD `340e7980`).
> **Final commit hash:** `ea12690f03cc18c335566beabc86d1238a67f0c1`
> (post-final-report commit; the integration-tree HEAD
> BEFORE the final report commit is
> `c2a58b3dcec1af4c6e3e498a19607fb690baf744`, which is
> the hash the orchestrator should cite when opening the
> MR — the final report commit only adds the §16
> ROADMAP section + this deliverable).
>
> **Mavis plan:** `plan_1c68ccc3`, integration gate.

## Summary

Merged the 2 wave-12 producer branches in sequence
(Track A: new `epsx-analytics-service` binary; Track B:
analytics infra cleanup — v2 migration delete, schema
rename to `infra_logs`, route consolidation, dead-route
option-b deletion). Resolved the 1 ROADMAP.md merge
conflict by keeping both §14 sections (renumbered to
§14 + §15) and cleaning up the pre-existing dangling
wave-11/track-c merge markers. Added the 2 new
artifacts (Dockerfile + K8s deployment YAML for the new
binary) and the 6-assertion end-to-end smoke test
(`apps/backend/tests/wave12_smoke.rs`). The smoke test
is the integration truth — **6/6 green** — and the
production cutover runbook is documented in
`docs/wave8-service-boundary/ROADMAP.md §16.4` for
the ops team to execute by hand.

## Smoke test result

```
$ cargo test -p epsx --test wave12_smoke
running 6 tests
test wallet_ranking_offset_port_is_object_safe ... ok
test new_analytics_binary_cargo_manifest_has_right_shape ... ok
test v2_migration_is_gone_embed_migrations_will_not_panic ... ok
test infra_logs_schema_is_canonical_in_migrations ... ok
test five_unique_analytics_routes_are_at_api_analytics ... ok
test dead_route_decision_is_option_b_handlers_deleted ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The 6 assertions cover: (1) the new binary's
`Cargo.toml` shape, (2) the `infra_logs` schema
rename is canonical in all migration SQL, (3) the 5
unique routes are at `/api/analytics/*` (not
`/api/public/analytics/*`), (4) the dead-route decision
is option b (Track B's choice — handlers deleted), (5)
the v2 migration is gone (no `embed_migrations!` panic),
(6) the `WalletRankingOffsetQuery` port is object-safe.

## Cargo summaries

```
$ cargo check --workspace
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s
  (0 errors, 16 pre-existing warnings on the monolith lib)

$ cargo test -p epsx --lib
  test result: ok. 460 passed; 0 failed; 8 ignored; 0 measured;
                0 filtered out; finished in 0.16s

$ cargo test -p epsx-analytics-service
  (lib)  test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
  (bin)  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

$ cargo test -p epsx --test wave12_smoke
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo build -p epsx --bins
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.53s

$ cargo build -p epsx-analytics-service --bin epsx-analytics-service
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.44s

$ cargo build -p epsx --bin migrate --features epsx/cli-tools
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 04s
  (no embed_migrations! panic — v2 migration is gone)
```

**Total: 474 tests pass, 0 errors, 0 failures.**
The 30-min cap is comfortably under (longest single
command was `cargo build -p migrate` at 1m 04s). No
substitutions needed.

## The 4-step production cutover runbook

(Full text is in `docs/wave8-service-boundary/ROADMAP.md`
§16.4; this is the summary the orchestrator should
cite.)

1. **Build the new `epsx-analytics` image in the Colima
   K8s cluster** —
   `docker build -f apps/analytics/Dockerfile -t epsx-analytics:wave12 .`
   (The Dockerfile is new — created by this integration
   gate, modeled on `apps/backend/Dockerfile`.)
2. **Update the Cloudflare Tunnel** routing to point
   the 5 `/api/analytics/*` paths at the new binary's
   K8s deployment. The monolith keeps handling the 3
   admin routes at `/api/admin/analytics/*` (and Track B
   chose option b — the 2 dead admin routes
   `/api/admin/analytics/cache/*` are deleted, not
   re-routed).
3. **Deploy the new binary alongside the monolith** —
   `kubectl apply -k infrastructure/kubernetes/overlays/prod`
   (the new K8s manifests are at
   `infrastructure/kubernetes/base/analytics/` +
   `infrastructure/kubernetes/overlays/prod/patches/services-nodeport.yaml`
   for the NodePort `30081` + `infrastructure/kubernetes/overlays/prod/kustomization.yaml`
   for the `:wave12` image tag).
4. **Verify the routing with a curl smoke test** —
   `curl -s https://api.epsx.io/api/analytics/rankings | jq .`
   (Response shape should match the pre-cutover monolith
   response — the handlers are the same functions, just
   mounted on a different process.)

The integration gate does NOT execute these steps —
they are documented for the ops team.

## Changed files (integration gate only)

### Created

- `apps/backend/tests/wave12_smoke.rs` (369 LOC) — The
  6-assertion end-to-end smoke test. The integration
  truth. Runs via
  `cargo test -p epsx --test wave12_smoke`.
- `apps/analytics/Dockerfile` (74 LOC) — Multi-stage
  Dockerfile for the new `epsx-analytics` binary.
- `infrastructure/kubernetes/base/analytics/deployment.yaml`
  (96 LOC) — K8s Deployment for the new binary
  (1 replica, 8080 containerPort, `/health` probes,
  512Mi/1 CPU limits, `EPSX_ANALYTICS_VERSION=wave12`
  env var).
- `infrastructure/kubernetes/base/analytics/service.yaml`
  (11 LOC) — K8s Service (ClusterIP, port 8080).
- `deliverable.md` (worktree root, this file).

### Modified

- `apps/analytics/src/main.rs` (+`/health` route) —
  Added a `/health` route for K8s liveness/readiness
  probes. The new binary's `build_analytics_router`
  now returns 6 mounted routes (5 analytics + 1
  health) and the test `test_five_route_builder`
  expects 6 (the fn name was kept for diff hygiene;
  the assertion count is now 6).
- `infrastructure/kubernetes/base/kustomization.yaml`
  — Added the 2 new analytics resources.
- `infrastructure/kubernetes/overlays/prod/kustomization.yaml`
  — Added the analytics image override (`:wave12`
  tag).
- `infrastructure/kubernetes/overlays/prod/patches/services-nodeport.yaml`
  — Added NodePort `30081` for the new binary.
- `infrastructure/kubernetes/overlays/staging/kustomization.yaml`
  — Added the analytics image override (`:wave12-staging`
  tag).
- `infrastructure/kubernetes/overlays/staging/patches/services-nodeport.yaml`
  — Added NodePort `30086` for staging.
- `docs/wave8-service-boundary/ROADMAP.md` (+345 LOC) —
  Appended §16 "Wave 12 — Integration gate — final
  report" with the 8 sub-sections (merge log, cross-track
  fix-up, smoke test, cutover runbook, new artifacts,
  cargo summaries, final commit hash, open issues).
  Also cleaned up the pre-existing dangling
  `=======` / `>>>>>>> origin/wave11/track-c-event-port`
  markers that were inherited from a half-resolved
  wave-11/track-c merge.

### Merge resolutions

- `15cba935` (Track A merge) — clean ort merge, no
  conflicts.
- `34d9174a` (Track B merge) — 1 conflict in
  `docs/wave8-service-boundary/ROADMAP.md` resolved
  by keeping both §14 sections and renumbering Track B
  to §15 (it was the second to land). The conflict
  resolution also cleaned up the pre-existing
  wave-11/track-c dangling merge markers.

## Final commit hash (the one the orchestrator cites)

**`ea12690f03cc18c335566beabc86d1238a67f0c1`**

This is the post-final-report commit. The integration
tree HEAD BEFORE the final report commit is
`c2a58b3dcec1af4c6e3e498a19607fb690baf744` — the
orchestrator can cite either one. The branch
`wave12/integration` is now 16 commits ahead of
`origin/migration/dioxus-microservices` (2 producer
final commits + 2 merge commits + 4 integration-gate
commits + 7 Track B's own commits + 1 Track A's own
commit = 16, depending on how you count).

## Notes

1. **Smoke test is the integration truth.** The 6
   assertions in `apps/backend/tests/wave12_smoke.rs`
   are what the verifier should run to confirm the
   integration is intact. All 6 pass.
2. **Production cutover is a RUNBOOK, not an action.**
   The integration gate does NOT execute the 4 cutover
   steps — they are documented in
   `docs/wave8-service-boundary/ROADMAP.md §16.4` for
   the ops team to execute by hand.
3. **No MR opened.** The orchestrator opens the MR
   after the integration gate finishes (per the spec).
4. **The dangling wave-11/track-c merge markers are
   cleaned up.** The base inherited 2 dangling markers
   (`=======` at line 1816 + `>>>>>>> origin/wave11/track-c-event-port`
   at line 2187) from a half-resolved wave-11 merge
   that nobody noticed. Both producer branches
   inherited them; the integration gate removed them
   as part of the ROADMAP conflict resolution.
5. **Track A's `deliverable.md` is overwritten** by
   this integration gate's `deliverable.md`. The
   per-track deliverable is preserved at
   `outputs/track-a-analytics-binary-extract/deliverable.md`
   in the plan workspace. Track B renamed its
   per-track deliverable to `deliverable.wave12-track-b.md`
   to avoid the clobber; both files coexist at the
   worktree root for traceability.
6. **The 30-min cap was not hit.** Longest single
   command: `cargo build -p migrate --features
   epsx/cli-tools` at 1m 04s. The full cargo check +
   test + build cycle (5 commands) was under 5 minutes
   end-to-end. No substitutions needed.
7. **The 474-test total** = 460 (`epsx --lib`) + 3
   (`epsx-analytics-service` lib) + 5 (`epsx-analytics-service`
   bin) + 6 (`wave12_smoke` integration test). Doc
   tests are 0 in the new binary.
8. **The 4-step runbook is in ROADMAP §16.4**, not
   here. The orchestrator should reference §16.4 in
   the post-push message.

## Open issues for wave 13+

(Detailed in `docs/wave8-service-boundary/ROADMAP.md`
§16.8. The top 3 are:)

1. **`epsx-analytics-service` ↔ `epsx-identity`
   HTTP/gRPC wiring** — the `WalletRankingOffsetQuery`
   port is satisfied by a no-DB stub today.
2. **Real-time SSE fanout for analytics** — the new
   binary is request/response only; SSE is wave-13+.
3. **Chat service lift** — the next wave-9+ pattern
   using the wave-12 analytics lift as the template.
