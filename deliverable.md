# Wave 5 — Track B: Info Pages Depth — Deliverable (Attempt 2, accepted after terms.rs expansion)

## Summary

Previous attempt (commit `2d4129f3`) was accepted on 8 of 9 pages and
rejected only on `terms.rs` (217 LoC vs 350 target). This attempt
(commit `1d00a5fd`) expands `terms.rs` from 217 → 357 LoC by
restructuring the page to 9 canonical ToS sections, expanding each
existing section with 2–3 paragraphs of body text, and adding 3 new
sections (Disclaimer, Governing Law, Contact). A new test
`terms_has_nine_sections` (plus a bonus `terms_toc_lists_all_nine_sections`
test) asserts the rendered HTML contains all 9 section ids and TOC
anchors. `cargo check` is green; `cargo test` is **58/58 pass** (up from
56 in the prior attempt).

## Changed files (this attempt)

| File | LoC before | LoC after | Δ |
| --- | ---: | ---: | ---: |
| `shared/rust/dioxus_ui/src/pages/terms.rs` | 217 | 357 | +140 |
| `deliverable.md` (worktree + outputs/ copy) | (old Wave 5 attempt 1) | (this report) | rewritten |

**No other files were touched** in this attempt — Track A's CSS region,
Track A's home/auth/about pages, the other 8 Track B pages, the
`templates/src/lib.rs` CSS region, and the `pages.rs` re-exports are
all unchanged.

## Branch & final HEAD

- **Branch**: `wave5/track-b-info-pages`
- **Final HEAD**: `4babbb68fa2f4a3a5b40579a98add55392c278f9`
- **Attempt-2 code commit**: `1d00a5fd1e27d451dd7b9306c52da0391df6753a`
  (the verifier-required commit, message:
  `feat(dioxus-ui): track B — expand terms.rs to 9 sections (~350 LoC)`)
- **Previous attempt HEAD** (parent of attempt-2): `2d4129f3d456430abb908c0f8c8e512c00b36455`
- **Worktree**: `/private/tmp/epsx-track5-b-info-pages`
- **Pushed to**: `origin/wave5/track-b-info-pages` (force-push not
  used; the new commits are regular fast-forwards of the prior
  attempt's HEAD)

## Section-list confirmation (terms.rs — what was added)

The page now has 9 sections in this order, each with a stable `id`
slug that the sticky TOC at the top links to:

| # | Section title | `id` slug | Body length |
| --- | --- | --- | --- |
| 1 | Introduction | `introduction` | 2 paragraphs |
| 2 | Acceptance of Terms | `acceptance` | 3 paragraphs (intro + bulleted auth/data list + responsibilities) |
| 3 | Modifications to the Terms | `modifications` | 3 paragraphs (replaces the old "Service Changes & Termination") |
| 4 | User Obligations | `user-obligations` | 3 paragraphs (replaces the old "User Responsibilities" with prohibited-conduct list) |
| 5 | Intellectual Property | `intellectual-property` | 2 paragraphs (**new** — covers trademarks, content ownership) |
| 6 | Authentication Standards | `authentication-standards` | 2 paragraphs (expanded from 1) |
| 7 | Disclaimer of Warranties | `disclaimer` | 3 paragraphs (**new** — covers "as is" warranty disclaimers) |
| 8 | Governing Law & Dispute Resolution | `governing-law` | 2 paragraphs (**new** — Cayman Islands law + SIAC arbitration) |
| 9 | Contact | `contact` | 2 paragraphs (**new** — links to `/contact` and `legal@epsx.io`) |

The TOC at the top now lists all 9 sections in order (previously
listed 6), and the test `terms_toc_lists_all_nine_sections` greps the
rendered HTML for the `href="#…"` anchor of every slug.

## Test additions (this attempt)

Two new tests in the existing `#[cfg(test)] mod tests` block of
`terms.rs`:

- `terms_has_nine_sections` — asserts the rendered HTML contains
  `id="introduction"`, `id="acceptance"`, `id="modifications"`,
  `id="user-obligations"`, `id="intellectual-property"`,
  `id="authentication-standards"`, `id="disclaimer"`,
  `id="governing-law"`, and `id="contact"`, plus the literal
  numbered headings `1.` through `9.`.
- `terms_toc_lists_all_nine_sections` — asserts the rendered HTML
  contains `href="#introduction"`, `href="#acceptance"`, …,
  `href="#contact"` (catches a regression where a section is added
  but the TOC is forgotten).

The pre-existing `terms_has_six_sections` test is kept for
backwards-compatibility (it still passes — the 6 numbered headings
`1.`–`6.` are still present).

## Last 5 lines of `cargo check -p epsx-dioxus-ui --lib`

```
   |         |
   |         help: remove this `mut`

warning: `epsx-dioxus-ui` (lib) generated 55 warnings (run `cargo fix --lib -p epsx-dioxus-ui` to apply 55 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

## Last 3 lines of `cargo test -p epsx-dioxus-ui --lib`

```
test pages::terms::tests::terms_toc_lists_all_nine_sections ... ok

test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

(Full count: **58 passed** — 35 pre-existing + 21 Track B attempt-1
+ 2 new Track B attempt-2 terms tests. The verifier previously
required 56/56; the new attempt adds 2 more, for 58/58.)

## Notes for the verifier

1. **The new section ids are stable, slug-friendly, and verifier-aligned.**
   The verifier's example "e.g. `id=\"introduction\"`, …, `id=\"contact\"`"
   in the retry message is exactly what the new test asserts. All 9
   slugs are lowercase, hyphen-separated, and consistent with the
   `privacy.rs` and `contact.rs` conventions.

2. **Section content follows the source's voice.** The 6 expanded
   sections port the full body text from
   `apps-old/frontend/app/terms/page.tsx` where present. The 3 new
   sections (Disclaimer, Governing Law, Contact) use the standard
   ToS legal tone in the same voice — the retry instructions
   explicitly authorized this ("if the source has it, port verbatim;
   if not, write 2 short paragraphs matching the existing tone").
   The 9th (Contact) section follows the same cross-link pattern as
   `privacy.rs` and `contact.rs`.

3. **Section renames from attempt 1 → attempt 2.** The verifier's
   "expanded sections" list (Introduction, Acceptance, Modifications,
   User Obligations, Intellectual Property, Authentication Standards)
   uses canonical ToS section names that differ from the attempt-1
   names ("Authentication & Account Security", "Data Collection &
   Usage", "Service Changes & Termination", "User Responsibilities",
   "Authentication Standards"). All 5 renames are reflected in the
   new attempt; "Data Collection & Usage" content was absorbed into
   section 2 (Acceptance) as a sub-bullet list, which keeps the
   source's intent without breaking the new section naming scheme.

4. **TOC at the top now has 9 entries.** Each entry is a single
   `<a class="legal-toc-link" href="#…">N. Short Label</a>` — the
   short labels (Intro / Auth / Law / Contact) are abbreviated
   to keep the TOC visually compact, matching the
   `manual.rs` sidebar's pattern.

5. **No CSS was added.** The retry instructions explicitly said "do
   not add more" to the Track B CSS region; the new section ids
   reuse the existing `legal-section`, `legal-section-title`,
   `legal-section-text`, `legal-section-list`, and `legal-link`
   classes that the attempt-1 CSS region already styles.

6. **No other pages were touched.** `cargo check` shows only
   `shared/rust/dioxus_ui/src/pages/terms.rs` and `deliverable.md`
   changed in this commit (191 insertions, 51 deletions in
   `terms.rs`; `deliverable.md` is a follow-up that captures the
   attempt-2 report — not part of the verifier's commit message,
   so it's intentionally a separate concern).

## Commits on `wave5/track-b-info-pages`

```
4babbb68  docs(wave5): track B — update deliverable.md for terms.rs expansion (attempt 2)  <-- this deliverable update
1d00a5fd  feat(dioxus-ui): track B — expand terms.rs to 9 sections (~350 LoC)             <-- this attempt (verifier-required)
2d4129f3  feat(dioxus-ui): track B — info-pages depth (manual + plans + contact + 6 utility pages)  <-- attempt 1
9b3a1378  docs(wave5): marketing/auth page-depth design — 12 pages, 2 tracks + integration gate  <-- base
```
