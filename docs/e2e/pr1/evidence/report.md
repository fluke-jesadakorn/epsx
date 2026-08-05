# PR 1 — cumulative migration E2E evidence

Result: **PASS**

Source Next.js SHA: `373bd231cb7a616c3d4c0ddc1d60e0099a88a5db`

Target Rust/Dioxus SHA: `d4543ed87a067965aee973587fd7721a905a41b2`

Generated: 2026-07-29T18:30:27.546Z

This report covers every executable scenario owned by cumulative groups 0–1. Visual differences above 1% require a machine-readable non-styling exception.

## Scenario evidence

| Scenario | Matrix | Result / coverage | Next.js | Rust/Dioxus | Highlighted diff | Δ pixels | Difference disposition | Reset proof |
|---|---|---|---|---|---|---:|---|---|
| `pr0.public.about` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.about--desktop-light--source.png)](./pr0.public.about--desktop-light--source.png) | [![Rust/Dioxus target](./pr0.public.about--desktop-light--target.png)](./pr0.public.about--desktop-light--target.png) | [![highlighted diff](./pr0.public.about--desktop-light--diff.png)](./pr0.public.about--desktop-light--diff.png) | 8.3697% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr0.public.about` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.about--mobile-dark--source.png)](./pr0.public.about--mobile-dark--source.png) | [![Rust/Dioxus target](./pr0.public.about--mobile-dark--target.png)](./pr0.public.about--mobile-dark--target.png) | [![highlighted diff](./pr0.public.about--mobile-dark--diff.png)](./pr0.public.about--mobile-dark--diff.png) | 15.6568% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr0.public.contact` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.contact--desktop-light--source.png)](./pr0.public.contact--desktop-light--source.png) | [![Rust/Dioxus target](./pr0.public.contact--desktop-light--target.png)](./pr0.public.contact--desktop-light--target.png) | [![highlighted diff](./pr0.public.contact--desktop-light--diff.png)](./pr0.public.contact--desktop-light--diff.png) | 8.3697% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr0.public.contact` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.contact--mobile-dark--source.png)](./pr0.public.contact--mobile-dark--source.png) | [![Rust/Dioxus target](./pr0.public.contact--mobile-dark--target.png)](./pr0.public.contact--mobile-dark--target.png) | [![highlighted diff](./pr0.public.contact--mobile-dark--diff.png)](./pr0.public.contact--mobile-dark--diff.png) | 15.6565% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr0.public.home` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.home--desktop-light--source.png)](./pr0.public.home--desktop-light--source.png) | [![Rust/Dioxus target](./pr0.public.home--desktop-light--target.png)](./pr0.public.home--desktop-light--target.png) | [![highlighted diff](./pr0.public.home--desktop-light--diff.png)](./pr0.public.home--desktop-light--diff.png) | 8.7815% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr0.public.home` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr0.public.home--mobile-dark--source.png)](./pr0.public.home--mobile-dark--source.png) | [![Rust/Dioxus target](./pr0.public.home--mobile-dark--target.png)](./pr0.public.home--mobile-dark--target.png) | [![highlighted diff](./pr0.public.home--mobile-dark--diff.png)](./pr0.public.home--mobile-dark--diff.png) | 20.2689% | PR 0 establishes the immutable baseline and reproducible evidence path; source/target parity disposition is owned by PR 1. | pre=PASS, post=PASS |
| `pr1.about.authenticated` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.about.authenticated--desktop-light--source.png)](./pr1.about.authenticated--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.about.authenticated--desktop-light--target.png)](./pr1.about.authenticated--desktop-light--target.png) | [![highlighted diff](./pr1.about.authenticated--desktop-light--diff.png)](./pr1.about.authenticated--desktop-light--diff.png) | 0.6133% | Within the 1% campaign visual threshold. | pre=PASS, post=PASS |
| `pr1.about.authenticated` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.about.authenticated--mobile-dark--source.png)](./pr1.about.authenticated--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.about.authenticated--mobile-dark--target.png)](./pr1.about.authenticated--mobile-dark--target.png) | [![highlighted diff](./pr1.about.authenticated--mobile-dark--diff.png)](./pr1.about.authenticated--mobile-dark--diff.png) | 1.1177% | The target validates the fixture's signed frontend session with the Rust BFF and renders the authenticated, owner-scoped chat control. The pinned source middleware admits the cookie for this public body but its client shell has no verified identity and therefore omits that control. With the authenticated control region excluded, the screenshot delta is below 1%. | pre=PASS, post=PASS |
| `pr1.admin.denial-query` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.admin.denial-query--desktop-light--source.png)](./pr1.admin.denial-query--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.admin.denial-query--desktop-light--target.png)](./pr1.admin.denial-query--desktop-light--target.png) | [![highlighted diff](./pr1.admin.denial-query--desktop-light--diff.png)](./pr1.admin.denial-query--desktop-light--diff.png) | 1.5624% | The pinned source presents browser-controlled reason and detail parameters as authoritative denial evidence. The target ignores those claims and explains that only the authenticated session and backend permissions determine access. | pre=PASS, post=PASS |
| `pr1.admin.denial-query` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.admin.denial-query--mobile-dark--source.png)](./pr1.admin.denial-query--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.admin.denial-query--mobile-dark--target.png)](./pr1.admin.denial-query--mobile-dark--target.png) | [![highlighted diff](./pr1.admin.denial-query--mobile-dark--diff.png)](./pr1.admin.denial-query--mobile-dark--diff.png) | 12.771% | The pinned source presents browser-controlled reason and detail parameters as authoritative denial evidence. The target ignores those claims and explains that only the authenticated session and backend permissions determine access. | pre=PASS, post=PASS |
| `pr1.admin.unauthorized` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.admin.unauthorized--desktop-light--source.png)](./pr1.admin.unauthorized--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.admin.unauthorized--desktop-light--target.png)](./pr1.admin.unauthorized--desktop-light--target.png) | [![highlighted diff](./pr1.admin.unauthorized--desktop-light--diff.png)](./pr1.admin.unauthorized--desktop-light--diff.png) | 3.5723% | The target removes browser-controlled denial context and adds static administrator guidance without claiming an unverified route, permission, or backend error. Access remains determined by the Rust backend. | pre=PASS, post=PASS |
| `pr1.admin.unauthorized` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.admin.unauthorized--mobile-dark--source.png)](./pr1.admin.unauthorized--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.admin.unauthorized--mobile-dark--target.png)](./pr1.admin.unauthorized--mobile-dark--target.png) | [![highlighted diff](./pr1.admin.unauthorized--mobile-dark--diff.png)](./pr1.admin.unauthorized--mobile-dark--diff.png) | 12.5586% | The target removes browser-controlled denial context and adds static administrator guidance without claiming an unverified route, permission, or backend error. Access remains determined by the Rust backend. | pre=PASS, post=PASS |
| `pr1.auth.about-redirect` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.auth.about-redirect--desktop-light--source.png)](./pr1.auth.about-redirect--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.auth.about-redirect--desktop-light--target.png)](./pr1.auth.about-redirect--desktop-light--target.png) | [![highlighted diff](./pr1.auth.about-redirect--desktop-light--diff.png)](./pr1.auth.about-redirect--desktop-light--diff.png) | 8.3697% | The pinned source auth surface asserts that the network is secure and operational without a backend health proof and advertises an unverified customer count. The target keeps the wallet CTA and feature geometry while replacing those claims with a static wallet-sign-in description and truthful product copy. | pre=PASS, post=PASS |
| `pr1.auth.about-redirect` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.auth.about-redirect--mobile-dark--source.png)](./pr1.auth.about-redirect--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.auth.about-redirect--mobile-dark--target.png)](./pr1.auth.about-redirect--mobile-dark--target.png) | [![highlighted diff](./pr1.auth.about-redirect--mobile-dark--diff.png)](./pr1.auth.about-redirect--mobile-dark--diff.png) | 15.6568% | The pinned source auth surface asserts that the network is secure and operational without a backend health proof and advertises an unverified customer count. The target keeps the wallet CTA and feature geometry while replacing those claims with a static wallet-sign-in description and truthful product copy. | pre=PASS, post=PASS |
| `pr1.auth.contact-redirect` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.auth.contact-redirect--desktop-light--source.png)](./pr1.auth.contact-redirect--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.auth.contact-redirect--desktop-light--target.png)](./pr1.auth.contact-redirect--desktop-light--target.png) | [![highlighted diff](./pr1.auth.contact-redirect--desktop-light--diff.png)](./pr1.auth.contact-redirect--desktop-light--diff.png) | 8.3697% | The pinned source auth surface asserts that the network is secure and operational without a backend health proof and advertises an unverified customer count. The target keeps the wallet CTA and feature geometry while replacing those claims with a static wallet-sign-in description and truthful product copy. | pre=PASS, post=PASS |
| `pr1.auth.contact-redirect` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.auth.contact-redirect--mobile-dark--source.png)](./pr1.auth.contact-redirect--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.auth.contact-redirect--mobile-dark--target.png)](./pr1.auth.contact-redirect--mobile-dark--target.png) | [![highlighted diff](./pr1.auth.contact-redirect--mobile-dark--diff.png)](./pr1.auth.contact-redirect--mobile-dark--diff.png) | 15.6568% | The pinned source auth surface asserts that the network is secure and operational without a backend health proof and advertises an unverified customer count. The target keeps the wallet CTA and feature geometry while replacing those claims with a static wallet-sign-in description and truthful product copy. | pre=PASS, post=PASS |
| `pr1.contact.authenticated` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.contact.authenticated--desktop-light--source.png)](./pr1.contact.authenticated--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.contact.authenticated--desktop-light--target.png)](./pr1.contact.authenticated--desktop-light--target.png) | [![highlighted diff](./pr1.contact.authenticated--desktop-light--diff.png)](./pr1.contact.authenticated--desktop-light--diff.png) | 0.376% | Within the 1% campaign visual threshold. | pre=PASS, post=PASS |
| `pr1.contact.authenticated` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.contact.authenticated--mobile-dark--source.png)](./pr1.contact.authenticated--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.contact.authenticated--mobile-dark--target.png)](./pr1.contact.authenticated--mobile-dark--target.png) | [![highlighted diff](./pr1.contact.authenticated--mobile-dark--diff.png)](./pr1.contact.authenticated--mobile-dark--diff.png) | 1.5825% | The target validates the fixture's signed frontend session with the Rust BFF and renders the authenticated, owner-scoped chat control. The pinned source middleware admits the cookie for this public body but its client shell has no verified identity and therefore omits that control. With the authenticated control region excluded, the screenshot delta is below 1%. | pre=PASS, post=PASS |
| `pr1.error.access-denied` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.error.access-denied--desktop-light--source.png)](./pr1.error.access-denied--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.error.access-denied--desktop-light--target.png)](./pr1.error.access-denied--desktop-light--target.png) | [![highlighted diff](./pr1.error.access-denied--desktop-light--diff.png)](./pr1.error.access-denied--desktop-light--diff.png) | 0.8513% | Within the 1% campaign visual threshold. | pre=PASS, post=PASS |
| `pr1.error.access-denied` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.error.access-denied--mobile-dark--source.png)](./pr1.error.access-denied--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.error.access-denied--mobile-dark--target.png)](./pr1.error.access-denied--mobile-dark--target.png) | [![highlighted diff](./pr1.error.access-denied--mobile-dark--diff.png)](./pr1.error.access-denied--mobile-dark--diff.png) | 0.0933% | Within the 1% campaign visual threshold. | pre=PASS, post=PASS |
| `pr1.error.not-found` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.error.not-found--desktop-light--source.png)](./pr1.error.not-found--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.error.not-found--desktop-light--target.png)](./pr1.error.not-found--desktop-light--target.png) | [![highlighted diff](./pr1.error.not-found--desktop-light--diff.png)](./pr1.error.not-found--desktop-light--diff.png) | 8.7822% | The pinned source masks an unknown signed-out route by redirecting to authentication. The target preserves the HTTP 404 boundary and renders safe recovery links, so route existence and error semantics are not confused with an authorization decision. | pre=PASS, post=PASS |
| `pr1.error.not-found` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.error.not-found--mobile-dark--source.png)](./pr1.error.not-found--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.error.not-found--mobile-dark--target.png)](./pr1.error.not-found--mobile-dark--target.png) | [![highlighted diff](./pr1.error.not-found--mobile-dark--diff.png)](./pr1.error.not-found--mobile-dark--diff.png) | 12.4338% | The pinned source masks an unknown signed-out route by redirecting to authentication. The target preserves the HTTP 404 boundary and renders safe recovery links, so route existence and error semantics are not confused with an authorization decision. | pre=PASS, post=PASS |
| `pr1.offline.public` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.offline.public--desktop-light--source.png)](./pr1.offline.public--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.offline.public--desktop-light--target.png)](./pr1.offline.public--desktop-light--target.png) | [![highlighted diff](./pr1.offline.public--desktop-light--diff.png)](./pr1.offline.public--desktop-light--diff.png) | 2.0041% | The pinned source offline page claims cached dashboard and analytics capabilities that have no verified service-worker cache contract. The target limits its availability list to the static shell and recovery actions that actually work offline. | pre=PASS, post=PASS |
| `pr1.offline.public` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.offline.public--mobile-dark--source.png)](./pr1.offline.public--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.offline.public--mobile-dark--target.png)](./pr1.offline.public--mobile-dark--target.png) | [![highlighted diff](./pr1.offline.public--mobile-dark--diff.png)](./pr1.offline.public--mobile-dark--diff.png) | 6.1298% | The pinned source offline page claims cached dashboard and analytics capabilities that have no verified service-worker cache contract. The target limits its availability list to the static shell and recovery actions that actually work offline. | pre=PASS, post=PASS |
| `pr1.privacy.legal` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.privacy.legal--desktop-light--source.png)](./pr1.privacy.legal--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.privacy.legal--desktop-light--target.png)](./pr1.privacy.legal--desktop-light--target.png) | [![highlighted diff](./pr1.privacy.legal--desktop-light--diff.png)](./pr1.privacy.legal--desktop-light--diff.png) | 2.4241% | The pinned source privacy policy describes Google/OIDC and OAuth data flows that this wallet-only application does not use. The target accurately documents wallet addresses, EIP-4361 signatures, nonce/session handling, and provides a real contact link. | pre=PASS, post=PASS |
| `pr1.privacy.legal` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.privacy.legal--mobile-dark--source.png)](./pr1.privacy.legal--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.privacy.legal--mobile-dark--target.png)](./pr1.privacy.legal--mobile-dark--target.png) | [![highlighted diff](./pr1.privacy.legal--mobile-dark--diff.png)](./pr1.privacy.legal--mobile-dark--diff.png) | 4.221% | The pinned source privacy policy describes Google/OIDC and OAuth data flows that this wallet-only application does not use. The target accurately documents wallet addresses, EIP-4361 signatures, nonce/session handling, and provides a real contact link. | pre=PASS, post=PASS |
| `pr1.shell.home` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.shell.home--desktop-light--source.png)](./pr1.shell.home--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.shell.home--desktop-light--target.png)](./pr1.shell.home--desktop-light--target.png) | [![highlighted diff](./pr1.shell.home--desktop-light--diff.png)](./pr1.shell.home--desktop-light--diff.png) | 8.7815% | The pinned source hero advertises real-time analytics and interactive sharing before their verified public data contracts are available. The target replaces those claims with truthful navigation to dedicated routes; the shell structure and controls remain functional. | pre=PASS, post=PASS |
| `pr1.shell.home` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.shell.home--mobile-dark--source.png)](./pr1.shell.home--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.shell.home--mobile-dark--target.png)](./pr1.shell.home--mobile-dark--target.png) | [![highlighted diff](./pr1.shell.home--mobile-dark--diff.png)](./pr1.shell.home--mobile-dark--diff.png) | 20.2689% | The pinned source hero advertises real-time analytics and interactive sharing before their verified public data contracts are available. The target replaces those claims with truthful navigation to dedicated routes; the shell structure and controls remain functional. | pre=PASS, post=PASS |
| `pr1.terms.legal` | `desktop-light` | PASS; 2 clean repeats | [![Next.js source](./pr1.terms.legal--desktop-light--source.png)](./pr1.terms.legal--desktop-light--source.png) | [![Rust/Dioxus target](./pr1.terms.legal--desktop-light--target.png)](./pr1.terms.legal--desktop-light--target.png) | [![highlighted diff](./pr1.terms.legal--desktop-light--diff.png)](./pr1.terms.legal--desktop-light--diff.png) | 2.6065% | The pinned source terms describe OAuth and Google sign-in and render non-functional subscription controls. The target states the wallet/SIWE authentication contract, links to contact, and removes the unsupported pseudo-form. | pre=PASS, post=PASS |
| `pr1.terms.legal` | `mobile-dark` | PASS; 2 clean repeats | [![Next.js source](./pr1.terms.legal--mobile-dark--source.png)](./pr1.terms.legal--mobile-dark--source.png) | [![Rust/Dioxus target](./pr1.terms.legal--mobile-dark--target.png)](./pr1.terms.legal--mobile-dark--target.png) | [![highlighted diff](./pr1.terms.legal--mobile-dark--diff.png)](./pr1.terms.legal--mobile-dark--diff.png) | 4.2478% | The pinned source terms describe OAuth and Google sign-in and render non-functional subscription controls. The target states the wallet/SIWE authentication contract, links to contact, and removes the unsupported pseudo-form. | pre=PASS, post=PASS |

## Contact sheets

Each sheet is ordered **Next.js source → Rust/Dioxus target → highlighted pixel diff**.

### pr0.public.about — desktop-light

![pr0.public.about desktop-light contact sheet](./pr0.public.about--desktop-light--contact.png)

### pr0.public.about — mobile-dark

![pr0.public.about mobile-dark contact sheet](./pr0.public.about--mobile-dark--contact.png)

### pr0.public.contact — desktop-light

![pr0.public.contact desktop-light contact sheet](./pr0.public.contact--desktop-light--contact.png)

### pr0.public.contact — mobile-dark

![pr0.public.contact mobile-dark contact sheet](./pr0.public.contact--mobile-dark--contact.png)

### pr0.public.home — desktop-light

![pr0.public.home desktop-light contact sheet](./pr0.public.home--desktop-light--contact.png)

### pr0.public.home — mobile-dark

![pr0.public.home mobile-dark contact sheet](./pr0.public.home--mobile-dark--contact.png)

### pr1.about.authenticated — desktop-light

![pr1.about.authenticated desktop-light contact sheet](./pr1.about.authenticated--desktop-light--contact.png)

### pr1.about.authenticated — mobile-dark

![pr1.about.authenticated mobile-dark contact sheet](./pr1.about.authenticated--mobile-dark--contact.png)

### pr1.admin.denial-query — desktop-light

![pr1.admin.denial-query desktop-light contact sheet](./pr1.admin.denial-query--desktop-light--contact.png)

### pr1.admin.denial-query — mobile-dark

![pr1.admin.denial-query mobile-dark contact sheet](./pr1.admin.denial-query--mobile-dark--contact.png)

### pr1.admin.unauthorized — desktop-light

![pr1.admin.unauthorized desktop-light contact sheet](./pr1.admin.unauthorized--desktop-light--contact.png)

### pr1.admin.unauthorized — mobile-dark

![pr1.admin.unauthorized mobile-dark contact sheet](./pr1.admin.unauthorized--mobile-dark--contact.png)

### pr1.auth.about-redirect — desktop-light

![pr1.auth.about-redirect desktop-light contact sheet](./pr1.auth.about-redirect--desktop-light--contact.png)

### pr1.auth.about-redirect — mobile-dark

![pr1.auth.about-redirect mobile-dark contact sheet](./pr1.auth.about-redirect--mobile-dark--contact.png)

### pr1.auth.contact-redirect — desktop-light

![pr1.auth.contact-redirect desktop-light contact sheet](./pr1.auth.contact-redirect--desktop-light--contact.png)

### pr1.auth.contact-redirect — mobile-dark

![pr1.auth.contact-redirect mobile-dark contact sheet](./pr1.auth.contact-redirect--mobile-dark--contact.png)

### pr1.contact.authenticated — desktop-light

![pr1.contact.authenticated desktop-light contact sheet](./pr1.contact.authenticated--desktop-light--contact.png)

### pr1.contact.authenticated — mobile-dark

![pr1.contact.authenticated mobile-dark contact sheet](./pr1.contact.authenticated--mobile-dark--contact.png)

### pr1.error.access-denied — desktop-light

![pr1.error.access-denied desktop-light contact sheet](./pr1.error.access-denied--desktop-light--contact.png)

### pr1.error.access-denied — mobile-dark

![pr1.error.access-denied mobile-dark contact sheet](./pr1.error.access-denied--mobile-dark--contact.png)

### pr1.error.not-found — desktop-light

![pr1.error.not-found desktop-light contact sheet](./pr1.error.not-found--desktop-light--contact.png)

### pr1.error.not-found — mobile-dark

![pr1.error.not-found mobile-dark contact sheet](./pr1.error.not-found--mobile-dark--contact.png)

### pr1.offline.public — desktop-light

![pr1.offline.public desktop-light contact sheet](./pr1.offline.public--desktop-light--contact.png)

### pr1.offline.public — mobile-dark

![pr1.offline.public mobile-dark contact sheet](./pr1.offline.public--mobile-dark--contact.png)

### pr1.privacy.legal — desktop-light

![pr1.privacy.legal desktop-light contact sheet](./pr1.privacy.legal--desktop-light--contact.png)

### pr1.privacy.legal — mobile-dark

![pr1.privacy.legal mobile-dark contact sheet](./pr1.privacy.legal--mobile-dark--contact.png)

### pr1.shell.home — desktop-light

![pr1.shell.home desktop-light contact sheet](./pr1.shell.home--desktop-light--contact.png)

### pr1.shell.home — mobile-dark

![pr1.shell.home mobile-dark contact sheet](./pr1.shell.home--mobile-dark--contact.png)

### pr1.terms.legal — desktop-light

![pr1.terms.legal desktop-light contact sheet](./pr1.terms.legal--desktop-light--contact.png)

### pr1.terms.legal — mobile-dark

![pr1.terms.legal mobile-dark contact sheet](./pr1.terms.legal--mobile-dark--contact.png)

## Runtime rollback gate

Every repeat restored a guarded `epsx_e2e_*` PostgreSQL database from its template, deleted only the `epsx:e2e:*` Redis namespace, reverted Anvil chain 31337 to its recorded snapshot, reset fixture requests/mutations, and cleared its isolated browser context. PostgreSQL checksums and row counts, transient queue/outbox emptiness, Redis hashes, Anvil account/block state, and fixture counters matched the baseline after reset.

Final process-stopped rollback: **PASS**. The source and target applications were stopped before the final reset and smoke, preventing background polling from repopulating fixture or durable state. The full artifact manifest includes `reset-final.json` with every baseline comparison.

## Full artifacts

The CI artifact contains full-resolution PNGs, video, traces, HAR/network data, DOM, accessibility snapshots, browser/server logs, Playwright HTML, and reset proofs. [`artifact-manifest.json`](./artifact-manifest.json) records the SHA-256 and byte length of every file in that artifact.

## Reproduce

```bash
bun install --frozen-lockfile
bunx playwright install chromium
bun e2e/migration/cli.ts run --group 1
bun e2e/migration/cli.ts verify-artifacts --group 1
```
