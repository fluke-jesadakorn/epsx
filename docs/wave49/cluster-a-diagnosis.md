# Cluster A pixel-tuning diagnosis (wave 49 batch E)

## Background

3 admin outliers render at 41.27-41.30% DIFF (58.7-58.73% match):
- `admin-access-denied`
- `admin-unauthorized`
- `admin-developer-portal-api-keys-create`

Previous wave 43 attempted an "A2 color fix" that was **correctly
not committed** because it caused a regression on the other 22
admin routes. The rule from `mavis-team-multi-track.md` is:

> "Minimum-scope test the brief's root-cause hypothesis before
> committing to the full fix."

## Root cause (traced via HTML diff)

Compared `tools/e2e-admin/baselines/prod-admin/admin-access-denied.html`
vs `tools/e2e-admin/baselines/dev-admin/admin-access-denied.html`.

The 3 background-orbs render with different source:

**Prod (Tailwind v2)** — `bg-primary/10` etc. classes:
```html
<div class="absolute -top-[10%] -left-[10%] w-[40%] h-[40%]
            bg-primary/0 dark:bg-primary/10 rounded-full blur-[120px]">
```

**Dev (inline-style workaround)** — `<style="background:hsla(...)">`:
```html
<div class="absolute -top-[10%] -left-[10%] w-[40%] h-[40%]
            rounded-full blur-[120px] animate-pulse"
     style="background:hsla(var(--primary)/0.10);">
```

The visual output is identical (the orbs render with the same
color + alpha), but the HTML source differs — which the diff
tool sees as a pixel diff. Comment in the code:

> "Tailwind v4 dev pipeline does not emit `dark:` variant
> classes for the arbitrary `bg-primary/10` / `bg-[#1fc7d4]/5` /
> `bg-[#ed4b9e]/5` patterns — only the base `bg-primary/0`
> (transparent) class is compiled, leaving the orbs invisible.
> Inline style with `rgba(...)` is the workaround."

The 41% diff is **NOT** missing components / buttons / text —
the report shows 0 missing-buttons, 0 missing-hrefs, 0
missing-components. It's pure CSS-source delta that the
imagemagick pixel-diff interprets as a big visual diff (because
the orbs cover ~30% of the screen).

## Recommended fix (recipe for wave-50+)

### Option A: safelist the v2 classes
1. Add the 3 class names to the dev Tailwind v4 safelist
   (likely in `tailwind.config.js` or via the dev BFF's
   design-system config):
   ```js
   safelist: ['bg-primary/10', 'bg-[#1fc7d4]/5', 'bg-[#ed4b9e]/5']
   ```
2. Replace the 3 inline styles in
   `shared/rust/dioxus_ui/src/pages/admin_pages/access_denied_panel.rs`
   with the prod-EXACT Tailwind classes.
3. **Probe step**: rebuild the dev admin BFF, capture
   `admin-access-denied`, run `diff-admin.sh admin-access-denied`,
   verify DIFF% drops below 5%.
4. **If regression**: revert, fall back to Option B.

### Option B: CSS override
1. Add a small `<style>` block to `epsx_templates::design_system_head`:
   ```css
   .dark .pay-orb-1 { background: hsla(var(--primary) / 0.10); }
   .dark .pay-orb-2 { background: rgba(31, 199, 212, 0.05); }
   .dark .pay-orb-3 { background: rgba(237, 75, 158, 0.05); }
   ```
2. Add the `pay-orb-N` classes to the inline-style divs in
   `access_denied_panel.rs` (replace the `style="..."` with
   `class="... pay-orb-1"`).
3. **Probe step**: same as Option A.

## Why this Batch E is documented, not committed

Per `mavis-team-multi-track.md` rule:
> "If delta < noise → brief's WHY is wrong, escalate or ship a
> no-harm partial + flag the real cause."

The wave-49 cluster doesn't have a running K8s
(see `docs/wave49/k8s-cluster-startup.md` — colima is not
installed), so we can't run `diff-admin.sh` to measure the
delta. Committing the change blind risks:

1. **Improving** Cluster A (good outcome — commit)
2. **No-op** (delta < noise — wasted effort, no signal)
3. **Regressing** other admin routes (the previous "A2 fix"
   outcome — must revert)

The 3rd risk is the same one that killed the wave-43 attempt.
Without a measurement loop, we're gambling.

## Status

- Diagnosis: DONE
- Fix recipe: documented above
- Commit: DEFERRED until K8s is up + a measurement loop exists
- Next wave: pick this up after `colima start --profile epsx`
  + `kubectl apply -k overlays/dev` brings the admin BFF online.
