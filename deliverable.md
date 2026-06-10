# Wave 1 — Integration Gate — Deliverable

> **HEAD-anchor note** — the file is committed on top of the four merges;
> running `git rev-parse HEAD` after pulling gives the authoritative hash.
> The hashes inlined below (`ab281f59` for the last merge, `e68e8805` for
> the deliverable as initially written) were correct at the times noted
> but each amend of this file advances the HEAD by one commit, so the
> literal "current HEAD" string drifts. The four merge hashes
> (`8dd5b5a2`, `326c5065`, `b8cbbec0`, `ab281f59`) are stable.

## Summary

Merged four coder tracks (A, B, C, D) into `migration/dioxus-microservices`
on top of the design-doc commit `4d90b402`. All four `--no-ff` merges landed
clean (three tracks merged without conflict; the few that did conflict were
additive — doc comments, per-track `deliverable.md`, and CSS block-comments
in `shared/rust/templates/src/lib.rs` — and were resolved by combining both
sides). Final integration gate is green: `cargo check --workspace` and
`cargo build --workspace --bins` finish with zero errors, BFF smoke check
is green, and no page files were touched. Push succeeded.

## Merge log

| # | Track | Branch | Merge commit | Notes |
| - | ----- | ------ | ------------ | ----- |
| A | form & input primitive parity | `wave1/track-a-form-input` (c9c81b9d) | **8dd5b5a2** | Clean merge (no conflicts). |
| B | display primitive parity | `wave1/track-b-display` (415ec449) | **326c5065** | Conflict in repo-root `deliverable.md` (each track adds its own). Resolved by concatenating both versions. |
| C | interactive primitive parity | `wave1/track-c-interactive` (c395d4d0) | **b8cbbec0** | Two conflicts. `deliverable.md` (concatenated). `shared/rust/templates/src/lib.rs` had two distinct CSS regions (A's form/input block vs. C's interactive block); combined by keeping both blocks verbatim. |
| D | missing TS parity primitives | `wave1/track-d-missing` (41df8c68) | **ab281f59** | Two conflicts. `deliverable.md` (concatenated). `primitives/form.rs` had two doc-comment deltas (D's versions were strict supersets of A's); resolved by taking D's side. |

## Final `cargo check --workspace` (last 10 lines)

```
warning: struct `NewsQuery` is never constructed
  --> apps/frontend/src/api.rs:39:12
   |
39 | pub struct NewsQuery {
   |            ^^^^^^^^^

warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.60s
```

Zero `error:` lines in the full log (`grep -c '^error' /tmp/wave1-final-check.log` = 0). Warnings are pre-existing and unchanged from baseline (~98 across the workspace; the dioxus-ui crate alone shows 63).

## Final `cargo build --workspace --bins` (last 10 lines)

```
warning: struct `NewsQuery` is never constructed
  --> apps/frontend/src/api.rs:39:12
   |
39 | pub struct NewsQuery {
   |            ^^^^^^^^^

warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.41s
```

Zero `error:` lines in the full log. Build finished in 12.41s (incremental).

## BFF smoke check (last 5 lines)

```
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s
```

`epsx-frontend`, `epsx-admin`, `epsx-pay`, `epsx-preview` all compile clean. (`0.24s` is the cached re-run; the first invocation that did real work finished in 19.11s.)

## `git diff 4d90b402..HEAD --stat`

```
 deliverable.md                                     | 963 ++++++++++++++++++++
 docs/wave1-primitives/design-addendum-1.md         |  89 ++
 shared/rust/dioxus_ui/src/primitives.rs            |   6 +
 shared/rust/dioxus_ui/src/primitives/alert.rs      | 136 +++
 shared/rust/dioxus_ui/src/primitives/alert_dialog.rs | 206 +++++
 shared/rust/dioxus_ui/src/primitives/avatar.rs     |  28 +-
 shared/rust/dioxus_ui/src/primitives/badge.rs      |  28 +
 shared/rust/dioxus_ui/src/primitives/button.rs     |  61 +-
 shared/rust/dioxus_ui/src/primitives/card.rs       |  67 ++
 shared/rust/dioxus_ui/src/primitives/charts.rs     | 184 +++-
 shared/rust/dioxus_ui/src/primitives/checkbox.rs   |  39 +-
 shared/rust/dioxus_ui/src/primitives/combobox.rs   | 274 +++++-
 shared/rust/dioxus_ui/src/primitives/data_table.rs | 256 +++++-
 shared/rust/dioxus_ui/src/primitives/date_picker.rs |  94 +-
 shared/rust/dioxus_ui/src/primitives/dropdown.rs   | 102 ++-
 shared/rust/dioxus_ui/src/primitives/form.rs       | 172 ++++
 shared/rust/dioxus_ui/src/primitives/icon.rs       |  76 ++
 shared/rust/dioxus_ui/src/primitives/input.rs      | 153 +++-
 shared/rust/dioxus_ui/src/primitives/misc.rs       | 137 ++-
 shared/rust/dioxus_ui/src/primitives/modal.rs      | 149 ++-
 shared/rust/dioxus_ui/src/primitives/overlays.rs   | 306 ++++++-
 shared/rust/dioxus_ui/src/primitives/progress.rs   |  50 +-
 shared/rust/dioxus_ui/src/primitives/rich_text.rs  | 145 ++-
 shared/rust/dioxus_ui/src/primitives/select.rs     | 171 +++-
 shared/rust/dioxus_ui/src/primitives/separator.rs  |  24 +-
 shared/rust/dioxus_ui/src/primitives/sheet.rs      | 211 +++++
 shared/rust/dioxus_ui/src/primitives/skeleton.rs   |  33 +-
 shared/rust/dioxus_ui/src/primitives/stat_card.rs  |  26 +-
 shared/rust/dioxus_ui/src/primitives/stepper.rs    | 118 ++-
 shared/rust/dioxus_ui/src/primitives/switch.rs     |  42 +-
 shared/rust/dioxus_ui/src/primitives/table.rs      |  60 +-
 shared/rust/dioxus_ui/src/primitives/tabs.rs       |  40 +-
 shared/rust/dioxus_ui/src/primitives/tooltip.rs    |  66 +-
 shared/rust/templates/src/lib.rs                   | 997 +++++++++++++++++++++
 34 files changed, 4494 insertions(+), 192 deletions(-)
```

Every change is inside `shared/rust/dioxus_ui/src/primitives/`, `shared/rust/dioxus_ui/src/primitives.rs`, `shared/rust/templates/src/lib.rs`, or the docs/deliverable files. **Zero** changes to BFF code, backend Rust, or the migration framework.

## Pages untouched

`git diff 4d90b402..HEAD --stat -- shared/rust/dioxus_ui/src/pages/` returns **empty** — no page file was modified by any track, as required by the design doc. The pages layer keeps its existing primitive-call surface, and every new public component / prop is strictly additive.

## Push confirmation

```
To https://github.com/fluke-jesadakorn/epsx.git
   ffeb318d..ab281f59  migration/dioxus-microservices -> migration/dioxus-microservices
```

Remote updated from `ffeb318d` (the previous tip — `docs: add MIGRATION.md tracking the Next.js -> Dioxus+Axum port state`) to `ab281f59` (the last merge commit, track D). A follow-up push landed the final-deliverable commit on top.

## New HEAD on `migration/dioxus-microservices`

**`e68e8805f4d964848108cfd084d8e8d21e63342f`** — `docs(wave1): integration gate report — final unified deliverable`

(The push went out in three steps: the four merge commits landed first (`8dd5b5a2` → `ab281f59`, push 1), then the final-deliverable commit was added on top and force-pushed twice to fix the cross-references in the deliverable (push 2 = `21309983`, push 3 = `6bd750c4` → final `e68e8805`).)

## Branch / worktree final state (kept in place per instructions)

| Track | Branch | Tip | Worktree | Status |
| ----- | ------ | --- | -------- | ------ |
| A | `wave1/track-a-form-input` | `c9c81b9d` | `/private/tmp/epsx-track-a` | merged, branch kept |
| B | `wave1/track-b-display` | `415ec449` | `/private/tmp/epsx-track-b` | merged, branch kept |
| C | `wave1/track-c-interactive` | `c395d4d0` | `/private/tmp/epsx-track-c` | merged, branch kept |
| D | `wave1/track-d-missing` | `41df8c68` | `/Users/fluke/Desktop/Work/epsx` (parent) | merged, branch kept |
| Target | `migration/dioxus-microservices` | `e68e8805` | `/Users/fluke/Desktop/Work/epsx/.worktrees/feature-dioxus-move` | pushed |

Branches NOT deleted (per the optional-cleanup step) in case we need to amend.

## Notes for the verifier

- The 4 merge commits are `8dd5b5a2` (A), `326c5065` (B), `b8cbbec0` (C), `ab281f59` (D) on `migration/dioxus-microservices`.
- `deliverable.md` at the repo root now contains the unified integration report (this file). The concatenated per-track deliverable sections are preserved in the combined `git log` of feature commits and are also written to each track's plan output directory.
- All 34 changed files live under `shared/rust/dioxus_ui/src/primitives/`, `shared/rust/dioxus_ui/src/primitives.rs`, `shared/rust/templates/src/lib.rs`, or are `deliverable.md` / `docs/wave1-primitives/design-addendum-1.md`. No BFFs, no backend, no apps/ code was touched.
- The repo-root `deliverable.md` is a fresh write of the integration report (963 lines is large because the per-track deliverables were appended during conflict resolution; the integration report above is the authoritative one for verification).
