# wave13b(integration) — final report

**Branch:** `wave13b/integration` HEAD `13cf1fa2bdc3be267490a1d3f1a8333ac607c1c3`
**Base:** `origin/migration/dioxus-microservices` HEAD `60305b6c` (wave 13a integration head)
**Worktree:** `/Users/fluke/Desktop/Work/epsx/.worktrees/wave13b-integration`
**Date:** 2026-06-14 (Asia/Bangkok, UTC+7)

---

## Summary

Merged Track A (`wave13b/track-a-sse-server`, commit `d467c416`) + Track B
(`wave13b/track-b-sse-consumer`, commit `d3fa5a4a`) into a single integration
branch `wave13b/integration`, deployed to the dev cluster, and ran the full
end-to-end SSE round-trip smoke test. **All 8 checks pass** (5 wave-12 HTTP +
1 wave-13a gRPC + 1 wave-13b SSE + 1 wave-13a fallback). The critical wave-13b
check (event lands in the analytics stream) is solid: 2 events emitted in
sequence both arrive in the same SSE stream within ~2s.

---

## Changed files

### Branch `wave13b/integration` — files added/modified by the integration

**From Track A merge (commit `9201e12c`, with the integration report on top in `13cf1fa2`):**
- `shared/rust/epsx-identity-service/src/sse_handler.rs` (new, 124 lines)
- `shared/rust/epsx-identity-service/src/emit_handler.rs` (new, 114 lines)
- `shared/rust/epsx-identity-service/src/event_bus.rs` (new, 109 lines)
- `shared/rust/epsx-identity-service/src/lib.rs` (modified, +382 lines)
- `shared/rust/epsx-identity-service/src/main.rs` (modified, +185 lines)
- `shared/rust/epsx-identity-service/Cargo.toml` (modified, +28 lines)
- `shared/proto/identity.proto` (modified, +56 lines)
- `infrastructure/kubernetes/base/identity/deployment.yaml` (modified, +52 lines)
- `infrastructure/kubernetes/base/identity/service.yaml` (modified, +11 lines)
- `infrastructure/kubernetes/overlays/dev/kustomization.yaml` (modified, +11 lines)
- `infrastructure/kubernetes/overlays/dev/patches/services-identity.yaml` (modified, +22 lines)
- `infrastructure/kubernetes/overlays/dev/patches/services-nodeport.yaml` (modified, +21 lines)
- `Cargo.lock` (modified, +7 lines)
- `deliverable-wave13b-track-a.md` (new, 241 lines)
- `docs/wave8-service-boundary/ROADMAP.md` (modified, +372 lines, §17.2 Track A)

**From Track B merge (already in integration branch via Track A ort merge):**
- `apps/analytics/src/sse_consumer.rs` (new, ~770 lines)
- `apps/analytics/src/main.rs` (modified, +gRPC client + SSE consumer wiring)
- `apps/analytics/Cargo.toml` (modified, +reqwest, +futures, +rand)
- `infrastructure/kubernetes/base/analytics/deployment.yaml` (modified,
  `IDENTITY_SSE_URL=http://epsx-identity:50052/v1/stream/ranking-offsets`)
- `Cargo.lock` (modified)
- `deliverable-wave13b-track-b.md` (new)
- `dev-cluster-round-trip.txt` (new, Track B attempt-#5 transcript)
- `docs/wave8-service-boundary/ROADMAP.md` (modified, §17.2.1 Track B)

**From integration gate (this task):**
- `docs/wave8-service-boundary/ROADMAP.md` (modified, +386 lines, §17.2.2
  integration report — appended at end of file, see line 5026+)
- `smoke-test-transcript.md` (new, 200 lines, full smoke test output)
- `deliverable.md` (this file, worktree-root)
- `deliverable-wave13b-integration-gate.md` (engine deliverable, plan output dir)

### Integration conflict resolution summary

**One conflict** during the Track A merge: `docs/wave8-service-boundary/ROADMAP.md`
(both the wave-13a integration head and Track A appended §17.2 sections to the
same anchor). Resolved by:
- Keeping BOTH §17.2 blocks.
- Renaming the wave-13a integration head's section to §17.2 (Track A).
- Renaming Track B's section to §17.2.1.
- Adding a new §17.2.2 section for the integration gate's final report.
- Dropping only the `<<<<<<<` / `=======` / `>>>>>>>` markers; no content
  was removed.

**Zero K8s conflicts** — Track A and Track B touched different files
(Track A: `base/identity/`, Track B: `base/analytics/`).

**Zero code conflicts** — Track A and Track B added new files in disjoint
crates (Track A: `epsx-identity-service`, Track B: `epsx-analytics-service`).

---

## Final commit hash

```
$ git rev-parse HEAD
13cf1fa2bdc3be267490a1d3f1a8333ac607c1c3
```

**Commit message:**
```
wave13b(integration): merge Track A — SSE server in identity

Conflict resolution: docs/wave8-service-boundary/ROADMAP.md had a
content conflict (Track A and the wave-13a integration head both
appended §17.2 sections). Resolved by keeping BOTH blocks and
renumbering the wave-13b/track-a section to §17.2 (Track A) —
Track B's merge will renumber its block to §17.2.1 in the next
merge commit, per the wave-13a convention from line 3759.

K8s files: no actual conflicts — Track A and Track B touched
different files (Track A modified base/identity/, Track B modified
base/analytics/). The user's pre-merge steering was conservative
(K8s conflicts 'possible' but the actual conflict was only in
ROADMAP).
```

**Track B was a no-op merge** (Track B's commits were already ancestors
of the integration branch via the ort strategy pulling them in through
the common wave-13a base). The integration branch has 1 merge commit
(`13cf1fa2` is the final commit, with `9201e12c` as its merge-commit ancestor) representing the combined wave-13b work.

---

## Full smoke test output

The 8 smoke test checks (all pass):

| # | Test | Status | Notes |
|---|------|--------|-------|
| 1 | `GET /health` (analytics) | ✅ PASS | HTTP 200 |
| 2 | `GET /rankings` (analytics) | ✅ PASS | HTTP 200 |
| 3 | `GET /filters` (analytics) | ✅ PASS | HTTP 200 |
| 4 | `GET /countries` (analytics) | ✅ PASS | HTTP 200 |
| 5 | `GET /sectors?country=america` (analytics) | ✅ PASS | HTTP 200 |
| 6 | gRPC `GetWalletRankingOffset` (identity) | ✅ PASS | `{"offset":100}` |
| 7 | **SSE round-trip: emit `0xA,10` + `0xB,20`** | ✅ PASS | Both events land in ~2s |
| 8 | Fallback: kill identity, analytics still serves `/rankings` | ✅ PASS | HTTP 200 from stub |

**Critical wave-13b check (7) transcript:**
```
$ kubectl port-forward -n epsx-dev svc/epsx-analytics 18084:8080 &
$ curl -sN -m 18 http://localhost:18084/v1/rankings/stream > /tmp/stream6.log &
$ curl -sX POST -d '{"wallet":"0xA","offset":10}' http://127.0.0.1:30105/v1/emit
{"delivered_to":1}
$ curl -sX POST -d '{"wallet":"0xB","offset":20}' http://127.0.0.1:30105/v1/emit
{"delivered_to":1}
$ sleep 3
$ cat /tmp/stream6.log
data: {"wallet":"0xA","offset":10,"changed_at_ms":1781429936276}

data: {"wallet":"0xB","offset":20,"changed_at_ms":1781429938318}

$ grep -E 'data:.*0xA.*10' /tmp/stream6.log  # FOUND-A
$ grep -E 'data:.*0xB.*20' /tmp/stream6.log  # FOUND-B
```

Full transcript: see `smoke-test-transcript.md` in the worktree root.

---

## Dev cluster final state

```
$ kubectl get pods -n epsx-dev
NAME                              READY   STATUS                  RESTARTS        AGE
epsx-admin-b54bcdfc6-sz5l6        1/1     Running                 0               12h
epsx-analytics-85f87cb58b-rnql9   1/1     Running                 0               97s
epsx-backend-59f79649fd-xxdn9     0/1     Init:CrashLoopBackOff   152             12h
epsx-backend-84c6c5dbff-xctwh     0/1     CrashLoopBackOff        152             12h
epsx-frontend-b49594598-jz4q6     1/1     Running                 0               12h
epsx-identity-9f7989ffd-ldztp     1/1     Running                 0               97s

$ kubectl get svc -n epsx-dev
NAME             TYPE       PORT(S)
epsx-admin       NodePort   3001:30102/TCP
epsx-analytics   NodePort   8080:30103/TCP
epsx-backend     NodePort   8080:30100/TCP
epsx-frontend    NodePort   3000:30101/TCP
epsx-identity    NodePort   50051:30104/TCP,50052:30105/TCP

$ kubectl get pod -n epsx-dev -l app=epsx-identity -o jsonpath='...'
ports: 50051/grpc, 50052/sse
image: epsx-identity:wave13b-dev
        (sha d87fd6ac4ede83bafcd36a3ebafbd28521f61ce2bdf7014c8b29a76e3ce17650)

$ kubectl get pod -n epsx-dev -l app=epsx-analytics -o jsonpath='...' (env)
IDENTITY_SSE_URL=http://epsx-identity:50052/v1/stream/ranking-offsets
IDENTITY_GRPC_URL=http://epsx-identity:50051
```

**Image IDs (rebuilt `--no-cache` on the integration branch HEAD):**
- `epsx-identity:wave13b-dev` = `d87fd6ac4ede83bafcd36a3ebafbd28521f61ce2bdf7014c8b29a76e3ce17650`
- `epsx-analytics:wave13b-dev` = `2ba684e9ff5c0f73c80603afc93ed2934d579e315e2550216f3b071d8e88c7f7`

**`epsx-backend` in CrashLoopBackOff is expected** — no PostgreSQL in
the dev cluster (per the wave-12 dev cluster runbook). **Not a regression.**

---

## ROADMAP §17.2.2 final report

A 386-line section was appended to `docs/wave8-service-boundary/ROADMAP.md`
covering:

- §17.2.2.1 — Producer commits merged (Track A `d467c416` + Track B `d3fa5a4a`)
- §17.2.2.2 — Merge log (1 conflict in ROADMAP, 0 K8s conflicts, 0 code
  conflicts; Track B was a no-op merge via the common base)
- §17.2.2.3 — Cross-track fix-up list (just the ROADMAP §17.2 renumber)
- §17.2.2.4 — Final 5-pod dev cluster state (with the 2 ports on the
  identity pod and the `IDENTITY_SSE_URL` env var on the analytics pod)
- §17.2.2.5 — Full smoke test output (8 checks, all pass)
- §17.2.2.6 — `delivered_to:1` vs Track B's `delivered_to:2` (subscriber
  topology explanation, **not a regression**)
- §17.2.2.7 — Track B bytes_stream decoder deviation (informational only,
  did NOT block the integration)
- §17.2.2.8 — Open issues for wave-13+ (5 items, including the
  bytes_stream decoder fix as wave-14 Track A)
- §17.2.2.9 — Production cutover runbook (internal-only, dev cluster; **do
  NOT deploy to production without explicit user confirmation**)

---

## Notes for the verifier

1. **The integration branch stays separate on origin** — `wave13b/integration`
   will NOT be fast-forwarded to `migration/dioxus-microservices` by the
   integration gate. The user reviews and fast-forwards when ready (per the
   wave-13a convention).

2. **The Track B `bytes_stream` decoder bug** is a real issue for the
   long-lived SSE consumer in production but did NOT block the integration
   smoke test. Tracked as §17.2.2.8 open issue #1 (wave-14 Track A candidate).

3. **The `delivered_to:1` count in the integration smoke test is correct
   for this test's topology** (1 host curl subscriber; the analytics-pod
   consumer is internal to the local bus). Track B's attempt-#5 had 2
   because it opened 2 direct host-curl subscribers. See §17.2.2.6 for the
   full explanation.

4. **The dev kustomization image tag for analytics is still
   `:wave13a-dev`** (Track A bumped identity to `:wave13b-dev` but didn't
   bump analytics). The image at that tag is now the wave-13b build (we
   overwrote the image ID at the same tag). This is correct and avoids
   needing a separate kustomization bump.

5. **The dev kustomization has a "no-op apply" footgun** when
   `kubectl delete deployment` is followed by `kubectl apply -k` — the
   service patch may not re-apply. Tracked as §17.2.2.8 open issue #3.
   Manual recovery is one apply away.

6. **The user-steering pre-merge warnings** about 2 K8s conflicts
   (analytics/deployment.yaml, identity/service.yaml) and 1 kustomization
   conflict turned out to be conservative — Track A and Track B had clean
   file-level separation. The only real conflict was in ROADMAP.md (both
   branches appended §17.2 sections to the same anchor).

7. **Per the user's standing rules (CLAUDE.md):** "Never deploy to
   production unless explicitly instructed by the user. Making code changes
   locally is always safe; deploying to prod requires explicit user
   confirmation each time." The integration gate is dev-cluster-only. The
   production cutover runbook in §17.2.2.9 mirrors wave-13a's §17.7 shape
   and explicitly says "do not deploy to production without explicit user
   confirmation".

8. **Both producer commits (d467c416 + d3fa5a4a) are in the integration
   branch** (verified with `git merge-base --is-ancestor`), satisfying the
   user-steering's requirement that the verifier's image rebuild would
   find both producer lineages.

---

## Final integration commit hash (pinned)

```
13cf1fa2bdc3be267490a1d3f1a8333ac607c1c3
```
