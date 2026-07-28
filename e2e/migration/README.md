# Migration E2E campaign

This harness compares the immutable Next.js baseline at
`373bd231cb7a616c3d4c0ddc1d60e0099a88a5db` with the checked-out
Rust/Dioxus migration branch. It is separate from the historical
`tools/e2e*` production-comparison scripts: migration evidence never uses
production as its source of truth.

## PR 0 quick start

```bash
bun install --frozen-lockfile
bunx playwright install chromium
bun run e2e:migration:doctor
bun run test:e2e:migration:pr0
```

The runner creates a detached source worktree at the pinned SHA when one is
not already present. Its default location is an OS-temporary directory outside
the target repository, preventing Next.js/Turbopack from inferring a workspace
root from unrelated local checkout state. Override it with
`E2E_SOURCE_ROOT=/absolute/path` only when that checkout resolves to the exact
locked commit.

The historical source lock is checked against its committed SHA-256 and
installed with the pinned Bun 1.3.4 binary in `--no-save` mode. The runner
then rechecks the lock hash and refuses any tracked source change. This is
necessary because the historical lock requests normalization from current
Bun even though its pinned package graph is usable; the immutable source
checkout is never rewritten.

## What a run does

1. Validates the 28 frontend and 27 admin routes against the canonical route
   contract.
2. Starts isolated PostgreSQL, Redis, and Anvil containers bound only to
   loopback.
3. Starts a deterministic loopback fixture service.
4. Creates guarded `epsx_e2e_*` PostgreSQL template/runtime databases and
   records the Redis, Anvil, fixture, and database baseline.
5. Starts Next.js from the pinned source worktree and the Rust/Dioxus BFF from
   the target checkout.
6. Runs every PR 0 smoke scenario twice from a clean baseline in fresh
   Playwright contexts.
7. Captures screenshots, highlighted diffs, DOM, accessibility snapshots,
   HAR/network data, browser logs, redirects, video, traces, server logs, and
   runtime reset proofs.
8. Requires same-side screenshot, normalized DOM, and accessibility hashes to
   match across repeats.
9. Generates review-sized evidence under `docs/e2e/pr0/evidence/` and a
   SHA-256 manifest for the full CI artifact.
10. Runs a post-reset smoke and removes only the isolated Compose services.

## Reset safety

The reset manager is fail-closed:

- `E2E_ALLOW_RUNTIME_MUTATION=1` is required.
- PostgreSQL, Redis, fixture, and Anvil endpoints must use loopback hosts.
- PostgreSQL database names must start with `epsx_e2e_`.
- Redis deletion is restricted to an `epsx:e2e:` prefix; `FLUSHDB` and
  `FLUSHALL` are never used.
- Anvil must report chain ID `31337`.
- The runner does not load `.env.prod`, contact production, deploy, or merge.

PostgreSQL reset restores the runtime database from its template, so
outboxes, notification jobs, provider callbacks, SSE cursors, and worker
leases return to their baseline. Redis values are compared using hashes.
Anvil account balance, nonce, bytecode hash, block, and chain state are
checked after `evm_revert`. Fixture request counters and mutations must be
empty after reset. Each browser context clears cookies, local/session
storage, IndexedDB, CacheStorage, and service workers before it closes.
Raw network evidence retains every browser event. A Chromium
`net::ERR_ABORTED` emitted after a successful 2xx/3xx `HEAD` response is
classified as a non-blocking transport cancellation because `HEAD` has no
response body; every other failed request fails the scenario.
The raw HTML remains untouched. Reproducibility uses a separately saved
semantic DOM with runtime scripts/styles removed and attributes sorted, so
Next.js development-flight timing metadata cannot masquerade as UI drift.
The Next.js development-tools portal is hidden during screenshots because its
compile-status badge is tooling chrome rather than application UI.
Repeat screenshots retain exact PNG hashes and also pass through a
pixel-equivalence gate (`threshold=0.1`, maximum `0.001%` differing pixels)
to tolerate only subpixel antialias/compositor noise. The repeat diff PNG and
metrics are part of the full artifact manifest.

## Evidence contract

Committed evidence contains source, target, diff, and contact-sheet PNGs plus:

- `report.md` — the PR-facing scenario table and reset summary;
- `artifact-manifest.json` — SHA-256 and byte length for every full artifact;
- `evidence-manifest.json` — hashes for committed evidence files.

CI uploads the full artifact directory. Verify a downloaded/local artifact
against the committed manifest with:

```bash
bun run e2e:migration:verify-artifacts
```

## Feature groups and bypasses

`scenarios.json` owns feature-group route assignment and matrices.
`bypasses.json` is the only bypass registry. A future bypass entry must name
the scenario, exact blocker, dependency group, captured evidence, and return
milestone. A bypass never counts as a pass. PR 0 and PR 9 require an empty
registry.
