# Wave 1 — Dioxus UI primitive parity (Next.js shadcn port) — DECISION ADDENDUM

Date: 2026-06-11
Author: plan owner (mvs_9052b3e1b5cb428c82321e8da661f194)

## Context

This addendum amends the original design doc
(`/Users/fluke/Desktop/Work/epsx/docs/wave1-primitives/design.md`) to fix an
over-strict rule that blocked three of four tracks from passing verification.

## The original rule (was over-strict)

> **§4 — Styling**
> All visual styling is provided by the **global CSS emitted by
> `epsx_templates::design_system_head`** ...
> Reuse the **existing class conventions** ...
> **Don't invent new class names** — if a needed utility doesn't exist, add a
> doc comment noting the gap and use the closest existing class. CSS additions
> go in a separate track (out of scope for Wave 1).

## Why this was wrong

Several Wave 1 components are net-new shadcn primitives that **genuinely
require new CSS** to render correctly: `MultiSelect` (chip layout + dropdown
panel), `RadioGroup` (vertical row layout), `ComboboxAsync` (loading indicator),
`ComboboxMulti` (selected chips), `InputGroup` (label + control + button row),
`FormSection` (boxed grouping), `FormRow` (horizontal layout), `DateTimePicker`
(combined date + time), `KbdCombo` (key chips), plus enhancements to `Switch`
size variants and `Checkbox` indeterminate state.

The original rule forced coders into a binary: ship the component with
unstyled markup, or skip the component. They shipped the markup; the verifier
correctly flagged it; the result is a stuck cycle.

## The amended rule (effective for the retry and all future Wave 1 tracks)

**Wave 1 coders MAY add new CSS rules to
`shared/rust/templates/src/lib.rs` (the design-system CSS file) when a
primitive requires a class name that does not yet exist.** The constraints are:

1. **Stay inside the existing class-naming conventions.** Reuse the patterns
   already used in `shared/rust/templates/src/lib.rs`:
   - Prefix with the component family: `multiselect-*`, `combobox-async-*`,
     `radio-group-*`, `form-section-*`, `input-group-*`, `datetime-picker-*`,
     `kbd-combo-*`, `rating-*`, `stepper-progress`, `switch-{sm,md,lg}`.
   - Reuse the global Tailwind v2 utility set (e.g. `flex`, `gap-2`,
     `rounded-md`, `border`, `text-sm`, `text-muted-foreground`,
     `bg-card`, `hover:bg-accent`).
   - Reuse the existing design tokens (CSS custom properties like
     `--epsx-card-bg`, `--epsx-border`, etc. — see
     `shared/rust/templates/src/lib.rs` for the full list).
2. **No new dependencies.** All styles must be achievable with vanilla CSS
   rules in the design-system file. No new utility frameworks, no new
   Tailwind plugins, no PostCSS changes.
3. **No new global state.** No `:root` custom properties outside what
   already exists. If you need a new token, justify it in a code comment
   and reuse the closest existing token.
4. **Keep the public API of `epsx_templates` unchanged.** The new CSS
   rules are added as additional strings emitted by existing functions
   (e.g. add a new class to the body of `design_system_head` or to a new
   helper called from it), not as new public API.
5. **Document each new class** in a single block comment in
   `lib.rs` with: the component it belongs to, a one-line purpose, and
   the consumer (which Dioxus primitive file emits it).

## Verifier update

The verifier's Check 8 ("class name hygiene") is amended to:
- **PASS** if the component is used in any page OR its new classes are
  defined in `shared/rust/templates/src/lib.rs`.
- **FAIL** if the component is unused (dead code) OR its new classes are
  undefined (the old strict rule, but only for cases where CSS wasn't added).

The verifier should also confirm the producer added the CSS in the same
commit that added the component (no orphan markup).

## What this means for the rest of Wave 1

- **Track A retry:** add the missing CSS rules to
  `shared/rust/templates/src/lib.rs` (47 new class strings). Use the naming
  conventions above. Keep the change small and focused.
- **Tracks C / D retries:** the verifier's other findings (a11y gaps) still
  apply; CSS additions are not the issue there. Fix only the a11y items
  the verifier called out.
- **Future Wave 1 work** (any new component in this wave): same rule.
  Add the CSS at the same time, in the same commit, in `lib.rs`.
- **Wave 2+:** a separate "design system polish" wave can refactor any
  class names that grow messy. Not in scope for Wave 1.
