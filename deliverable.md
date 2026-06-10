<<<<<<< HEAD
# Track A — Form & Input Primitive Parity — Deliverable (retry)

## Summary

Brought the form & input Dioxus primitives in `shared/rust/dioxus_ui/src/primitives/`
to 1:1 UX/UI parity with the Next.js shadcn/Radix components. Added 7 new
components and enhanced 4 existing ones, while preserving the existing public
API (no signature changes, no removed parameters, no required booleans). On
the verifier's retry note, also added the missing design-system CSS for the
~50 new class names used by these primitives into
`shared/rust/templates/src/lib.rs`, and documented them in a single block
comment.

## Branch

- Branch name: `wave1/track-a-form-input`
- Forked from: `migration/dioxus-microservices` @ `4d90b402`
- Commits on this branch:
  - `e90468a8` — `feat(dioxus-ui): track A — form & input primitive parity` (9 files, +1079/−61)
  - `64300cce` — `docs(wave1): track A deliverable`
  - `7eea2dea` — `docs(wave1): track A — include both commit hashes`
  - `c13e8a54` — `docs(wave1): design addendum — Wave 1 coders may add CSS to epsx-templates lib.rs` (added by plan owner, before this retry)
  - `226a62d4` — `feat(epsx-templates): track A — CSS for new form & input primitive classes` (this retry; +500 lines in `shared/rust/templates/src/lib.rs`)

## Retry-specific changes (this submission)

- **Added 50 new CSS rules to `shared/rust/templates/src/lib.rs`** in a
  single block at the end of the `<style>` block emitted by
  `design_system_head()`. The CSS reuses the existing `--epsx-*` CSS
  custom properties (e.g. `--epsx-red`, `--epsx-blue-start`, `--border`,
  `--bg-secondary`, `--text-muted`, `--primary`, `--shadow`, `--gradient-brand`)
  and the global Tailwind v2 utility set.
- **Added a block comment** in `lib.rs` listing every new class with its
  purpose and the consumer primitive file. See "Class catalog" below for
  the full list.
- **Replaced the false "no new class names" claim** in the previous
  deliverable with the real class list and the location of the CSS.

## Files modified in this retry

| File | Δ | Purpose |
| --- | --- | --- |
| `shared/rust/templates/src/lib.rs` | +500 / −0 | CSS rules + block comment for the new classes |

## Files modified in the original Track A submission (unchanged in this retry)

| File | Δ | Highlights |
| --- | --- | --- |
| `shared/rust/dioxus_ui/src/primitives/input.rs` | 153 / −30 | New `InputKind::File`, `Color`, `Hidden`; icon now rendered **inside** the wrap; controlled + uncontrolled mode; `aria-invalid` always emitted; `aria_invalid` opt exposed. |
| `shared/rust/dioxus_ui/src/primitives/form.rs` | 170 / −0 | Added `Label`, `InputGroup`, `FormSection`, `FormRow`, `RadioGroup`. All existing components preserved. |
| `shared/rust/dioxus_ui/src/primitives/select.rs` | 171 / −1 | Added `MultiSelect` with chip rendering, dropdown menu, optional `max` cap, hidden-input mirror. |
| `shared/rust/dioxus_ui/src/primitives/combobox.rs` | 274 / −1 | Added `ComboboxAsync` and `ComboboxMulti`. |
| `shared/rust/dioxus_ui/src/primitives/date_picker.rs` | 94 / −2 | Added `DateTimePicker`. |
| `shared/rust/dioxus_ui/src/primitives/stepper.rs` | 118 / −3 | Added `progress: bool` flag, `Step` struct, `StepperSteps`. |
| `shared/rust/dioxus_ui/src/primitives/checkbox.rs` | 39 / −1 | Added `indeterminate: Option<bool>`. |
| `shared/rust/dioxus_ui/src/primitives/switch.rs` | 42 / −1 | Added `SwitchSize` enum + `size` prop. |
| `shared/rust/dioxus_ui/src/primitives/misc.rs` | 79 / −42 | `Slider` a11y, interactive `Rating`, `KbdCombo`. |

## Class catalog (added to `shared/rust/templates/src/lib.rs` in this retry)

The following classes are emitted by the Track A Dioxus primitives and now
have CSS rules in `shared/rust/templates/src/lib.rs` (inside
`design_system_head`):

| Class | Purpose | Consumer file |
| --- | --- | --- |
| `input-error` | invalid input border + ring | `primitives/input.rs` |
| `input-with-icon` | left padding when an icon is present | `primitives/input.rs` |
| `input-icon` | absolutely-positioned icon inside the wrap | `primitives/input.rs` |
| `label-required` | red "*" indicator on required labels | `primitives/form.rs` :: `Label` |
| `form-section` | boxed subsection of a long form | `primitives/form.rs` :: `FormSection` |
| `form-section-header` | header row holding title + description | `primitives/form.rs` :: `FormSection` |
| `form-section-title` | h3 title inside the section header | `primitives/form.rs` :: `FormSection` |
| `form-section-description` | muted description under the section title | `primitives/form.rs` :: `FormSection` |
| `form-section-body` | content area below the section header | `primitives/form.rs` :: `FormSection` |
| `form-row` | responsive 1/2-column grid for fields | `primitives/form.rs` :: `FormRow` |
| `input-group` | label + control + trailing-button row | `primitives/form.rs` :: `InputGroup` |
| `input-group-label` | label rendered above the control row | `primitives/form.rs` :: `InputGroup` |
| `input-group-control` | flex row holding the control(s) | `primitives/form.rs` :: `InputGroup` |
| `input-group-help` | inline help text below the control row | `primitives/form.rs` :: `InputGroup` |
| `input-group-error` | red error text below the control row | `primitives/form.rs` :: `InputGroup` |
| `radio-group` | vertical stack of radio rows | `primitives/form.rs` :: `RadioGroup` |
| `radio-group-label` | group label rendered above the rows | `primitives/form.rs` :: `RadioGroup` |
| `radio-group-help` | help text below the radio stack | `primitives/form.rs` :: `RadioGroup` |
| `radio-group-error` | error text below the radio stack | `primitives/form.rs` :: `RadioGroup` |
| `radio-row` | single radio row (label + input) | `primitives/form.rs` :: `RadioGroup` |
| `radio-row.selected` | visual cue for the currently-selected row | `primitives/form.rs` :: `RadioGroup` |
| `radio-row-label` | label text inside a radio row | `primitives/form.rs` :: `RadioGroup` |
| `multiselect` | top-level wrapper around a multi-select | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-control` | flex row holding chips + trigger | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-chip` | single chip for a selected value | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-chip-remove` | × button inside a chip | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-trigger` | "Add…" button that opens the dropdown | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-menu` | dropdown panel listing the options | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-option` | single option inside the dropdown | `primitives/select.rs` :: `MultiSelect` |
| `multiselect-option.selected` | visual cue for selected options | `primitives/select.rs` :: `MultiSelect` |
| `combobox-async` | modifier on a combobox with async load | `primitives/combobox.rs` :: `ComboboxAsync` |
| `combobox-loading` | "Loading…" item inside the menu | `primitives/combobox.rs` :: `ComboboxAsync` |
| `combobox-empty` | "No matches" item inside the menu | `primitives/combobox.rs` :: `ComboboxAsync` |
| `combobox-multi` | modifier on a multi-select combobox | `primitives/combobox.rs` :: `ComboboxMulti` |
| `combobox-multi-control` | flex row holding chips + search input | `primitives/combobox.rs` :: `ComboboxMulti` |
| `combobox-multi-chip` | single chip in a multi-select combobox | `primitives/combobox.rs` :: `ComboboxMulti` |
| `combobox-multi-chip-remove` | × button inside a multi chip | `primitives/combobox.rs` :: `ComboboxMulti` |
| `combobox-multi-input` | trailing search input after the chips | `primitives/combobox.rs` :: `ComboboxMulti` |
| `datetime-picker` | flex row holding the date + time inputs | `primitives/date_picker.rs` :: `DateTimePicker` |
| `stepper-wrap` | outer wrapper around the progress bar + stepper row | `primitives/stepper.rs` |
| `stepper-progress` | linear progress bar above the stepper | `primitives/stepper.rs` |
| `rating-interactive` | hover-able, clickable rating | `primitives/misc.rs` :: `Rating` |
| `rating-disabled` | non-interactive, dimmed rating | `primitives/misc.rs` :: `Rating` |
| `switch-sm`, `switch-md`, `switch-lg` | size variants of the switch | `primitives/switch.rs` |
| `state-checked`, `state-unchecked` | checked/unchecked visual state for `SwitchRoot` | `primitives/switch.rs` |
| `SwitchRoot`, `SwitchInput`, `SwitchThumb`, `SwitchLabel` | switch parts (pre-existing classes, now defined) | `primitives/switch.rs` |
| `kbd-combo` | wrapper for multi-key keyboard shortcut | `primitives/misc.rs` :: `KbdCombo` |
| `kbd-combo-sep` | "+" separator between combo keys | `primitives/misc.rs` :: `KbdCombo` |
| `kbd` | the actual `<kbd>` chip | `primitives/misc.rs` :: `Kbd` + `KbdCombo` |
| `slider-field` | vertical layout wrapper around a slider | `primitives/misc.rs` :: `Slider` |
| `slider` | the range input itself | `primitives/misc.rs` :: `Slider` |
| `checkbox-indeterminate` | partial-fill visual state | `primitives/checkbox.rs` |

## New public symbols (unchanged from the previous submission)

- `primitives::input::InputKind` — gained `File`, `Color`, `Hidden` variants.
- `primitives::form::Label`, `InputGroup`, `FormSection`, `FormRow`, `RadioGroup`.
- `primitives::select::MultiSelect`.
- `primitives::combobox::ComboboxAsync`, `ComboboxMulti`.
- `primitives::date_picker::DateTimePicker`.
- `primitives::stepper::Step`, `StepperSteps`.
- `primitives::switch::SwitchSize`.
- `primitives::misc::KbdCombo`.

All are auto-re-exported from `epsx_dioxus_ui::*` via the existing
`pub use primitives::*;` chain — no new re-exports needed in
`primitives.rs` / `lib.rs`.

## Signatures changed

None. Every change is additive. New params on existing components are
`#[props(default = …)]` and `Option<T>`. No call site in
`shared/rust/dioxus_ui/src/pages/` was touched.

## A11y

| Component | a11y attributes |
| --- | --- |
| `Input` | `aria-invalid`, `html-for` label, `required` |
| `Label` | `html-for`, `*` indicator |
| `RadioGroup` | `role="radiogroup"`, `aria-labelledby`, `aria-required`, `aria-invalid` |
| `MultiSelect` | `role="listbox"`, `aria-multiselectable`, `aria-invalid`, `aria-label` on chip-remove buttons |
| `ComboboxAsync` | `aria-expanded`, `aria-busy`, `aria-autocomplete`, `aria-invalid`, `role="listbox"` / `role="option"` |
| `ComboboxMulti` | `role="listbox"`, `aria-multiselectable`, `aria-label` on chip-remove |
| `DatePicker` / `DateTimePicker` | `aria-invalid` |
| `Stepper` / `StepperSteps` | `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax` on the progress bar |
| `Checkbox` | `data-state="indeterminate"`, `aria-checked="mixed"` |
| `Switch` | `role="switch"`, `aria-checked` |
| `Slider` | `role="slider"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-label`, `disabled` |
| `Rating` | `role="radiogroup"`, `role="radio"`, `aria-checked`, `aria-label="N star(s)"`, `tabindex` |

## A11y gaps intentionally left for Wave 2

- **Combobox keyboard selection** — the existing `Combobox` doesn't fire
  `onchange` on Enter / click into the value field; the user must commit by
  clicking an option. A future Wave 2 track can wire Enter to commit the
  focused option.
- **Slider keyboard support** — `<input type="range">` natively supports
  arrow keys, so the a11y contract is satisfied at the HTML level. A Wave 2
  track may want to wire `Home` / `End` / `PageUp` / `PageDown` shortcuts.
- **Modal focus trap on `MultiSelect` open** — clicking the trigger button
  does not move focus to the dropdown menu. Wave 2 can add a focus-trap
  when the popover is open.
- **ComboboxMulti "create new"** — when `allow_free` is true, hitting
  Enter on a non-matching query should be able to create a new option.
  Currently the `Combobox` family has this prop but doesn't fire a creation
  callback.

## Rules respected

- ✅ No new icons added to the lucide registry.
- ✅ No `dangerous_inner_html` outside the existing `QrCode` and the
  icon helper.
- ✅ No page files were edited.
- ✅ No BFF files were edited.
- ✅ The CSS additions live in `shared/rust/templates/src/lib.rs` (the
  design-system CSS file) — the only file allowed by the retry
  instructions.
- ✅ All CSS reuses the existing `--epsx-*` custom properties and the
  global Tailwind v2 utility set — no new utility frameworks, no new
  Tailwind plugins, no PostCSS changes.
- ✅ The block comment at the top of the new CSS section lists every
  new class with its purpose and the consumer primitive file.

## `cargo check` results

### `cargo check -p epsx-dioxus-ui --all-targets` (last 10 lines)

```
warning: variable does not need to be mutable
  --> shared/rust/dioxus_ui/src/pages/admin_pages/news.rs:62:9
   |
62 |     let mut body = use_signal(|| "## Introduction\n\nWrite your news article here in markdown.\n\n- Point 1\n- Point 2\n\n[Read more](htt...
   |         ----^^^^
   |         |
   |         help: remove this `mut`

warning: `epsx-dioxus-ui` (lib test) generated 60 warnings (60 duplicates)
warning: `epsx-dioxus-ui` (lib) generated 60 warnings (run `cargo fix --lib -p epsx-dioxus-ui` to apply 59 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 36.22s
```

All warnings are pre-existing; no new warnings introduced by this track.

### `cargo check --workspace` (last 5 lines)
=======
# Track D — Missing TS parity primitives (Dioxus) — Attempt 2 (a11y fixes)

## Summary

Addressed the 3 a11y gaps the verifier flagged in attempt 2:
(1) added `aria-labelledby` + `aria-describedby` wiring to
`AlertDialog` and `Sheet` with auto-generated stable IDs, (2) added
`role="region"` and `tabindex="0"` to the `ScrollArea` viewport,
(3) swapped the 3 unregistered lucide icons (`check-circle`,
`alert-triangle`, `alert-circle`) for registered equivalents
(`check`, `bell`, `x`) and added a TODO comment listing the icons
to add to the lucide registry. `cargo check --workspace` exits 0.

## Branch / commits

- **Branch**: `wave1/track-d-missing`
- **Latest commit**: `9496c824` (a11y fixes; on top of `ce539b77`
  deliverable + `48ad0bd1` initial code).
- **Files changed in this commit** (4):
  ```
  shared/rust/dioxus_ui/src/primitives/alert.rs           |  19 ++++++++--
  shared/rust/dioxus_ui/src/primitives/alert_dialog.rs   |  40 ++++++++++++++++++----
  shared/rust/dioxus_ui/src/primitives/sheet.rs          |  38 +++++++++++++++++---
  shared/rust/dioxus_ui/src/primitives/misc.rs           |  10 +++++-
  ```

## What changed (verifier's 3 issues)

### Issue (1) — `aria-labelledby` / `aria-describedby` wiring

**`alert_dialog.rs`**:
- Added a `static ALERT_DIALOG_NEXT_ID: AtomicU64` that gets
  incremented per component instance to generate a stable
  per-instance nonce.
- The `AlertDialog` root now sets:
  - `role="alertdialog"` (already there)
  - `aria-modal="true"` (already there)
  - `aria-labelledby="alertdialog-title-{n}"` (new)
  - `aria-describedby="alertdialog-desc-{n}"` (new)
- The title `<h2>` and description `<p>` now carry matching
  `id="alertdialog-title-{n}"` / `id="alertdialog-desc-{n}"`.
- `AlertDialogTitle` and `AlertDialogDescription` slot components
  also generate their own IDs so a future caller-managed
  composition can wire them up.

**`sheet.rs`**:
- Added a `static SHEET_NEXT_ID: AtomicU64` with the same pattern.
- The `Sheet` root now sets:
  - `role="dialog"` (already there)
  - `aria-modal="true"` (already there)
  - `aria-labelledby="sheet-title-{n}"` (new)
  - `aria-describedby="sheet-desc-{n}"` (new)
- The root sheet renders a visually-hidden `<h2 id="...">Sheet
  panel</h2>` and `<p id="...">Slide-in side panel</p>` pair
  (using `class="sr-only"`) so the dialog always has a valid
  accessible name + description, matching the TS shadcn source's
  `VisuallyHidden.Root` pattern.
- `SheetTitle` and `SheetDescription` slot components generate
  their own IDs for richer visible titles/descriptions.

### Issue (2) — `ScrollArea` viewport a11y

**`misc.rs`** — added to the inner `scroll-area-viewport` div:
- `role: "region"` (the viewport is a content region)
- `tabindex: "0"` (the region is focusable for keyboard scrolling)

This mirrors the behaviour of Radix's `ScrollArea.Viewport` (which
sets `tabindex` and `data-radix-scroll-area-viewport`).

### Issue (3) — Lucide icon registry gaps

`epsx_templates::lucide` only registers these names: `chart-column`,
`code`, `building`, `chevron-down`, `chevron-right`, `trending-up`,
`chart-line`, `zap`, `users`, `calendar`, `newspaper`, `pin`,
`arrow-right`, `info`, `mail`, `help-circle`, `circle-help`, `menu`,
`x`, `sun`, `moon`, `wallet`, `log-out`, `user`, `settings`,
`check`, `plus`, `search`, `share`, `bell`, `book`, `key`,
`layout-dashboard`, `message-circle`, `file-text`, `history`,
`credit-card`, `link`, `external-link`, `briefcase`.

My previous default-icon map used `check-circle`, `alert-triangle`,
and `alert-circle`, all of which are NOT in the registry (only
`info` and `check` are). Swapped to:

| `AlertKind` | Old (not registered) | New (registered) |
| --- | --- | --- |
| `Default`  | `info`           | `info` (no change) |
| `Success`  | `check-circle`   | `check` |
| `Warning`  | `alert-triangle` | `bell` |
| `Danger`   | `alert-circle`   | `x` |
| `Info`     | `info`           | `info` (no change) |

Added a `TODO: add lucide registry entries for the proper shadcn
names (check-circle, alert-triangle, alert-circle)` comment in
`alert.rs` documenting the gap.

## `cargo check --workspace` (last 10 lines)
>>>>>>> wave1/track-d-missing

```
warning: struct `NewsQuery` is never constructed
  --> apps/frontend/src/api.rs:39:12
   |
39 | pub struct NewsQuery {
   |            ^^^^^^^^^

<<<<<<< HEAD
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.10s
```

### `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`

```
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.06s
```

### `cargo test -p epsx-dioxus-ui --lib`

```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

(0 tests because the project has no `#[cfg(test)]` modules in the
primitives crate — this is the baseline state.)

## Verifier gate

| Check | Result |
| --- | --- |
| `cargo check --workspace` | green (warnings only, no errors) |
| `cargo check -p epsx-dioxus-ui --all-targets` | green |
| `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview` | green |
| `cargo test -p epsx-dioxus-ui --lib` | green (0 tests) |
| `git diff --stat -- shared/rust/dioxus_ui/src/pages/` | empty (no pages modified) |
| `git diff --stat -- apps/` | empty (no BFF modified) |
| New classes have CSS in `shared/rust/templates/src/lib.rs` | ✅ (this retry) |
| Branch | `wave1/track-a-form-input` |
| Commit hashes | `e90468a8`, `226a62d4` (+ docs) |

---

# Track B — Display Primitive Parity

## Summary
Brought the 12 display Dioxus primitives (`button`, `card`, `badge`, `avatar`,
`table`, `data_table`, `charts`, `stat_card`, `progress`, `skeleton`,
`separator`, `icon`) to 1:1 UX/UI parity with the Next.js shadcn/Radix sources.
Added 14 new public components and 13 new optional props/slots while keeping
every existing public API fully backwards-compatible — no page edits required.

## Branch & commit
- **Branch:** `wave1/track-b-display`
- **Feature commit:** `9c99486cb2ba4b2743d106127e6320aba381cc09` — `feat(dioxus-ui): track B — display primitive parity`
- **Deliverable commit:** `3630d42afc7a8db013b58b3083b70a507e7c07a4` — `docs(wave1): track B deliverable`
- **Worktree:** `/private/tmp/epsx-track-b` (shared workspace, see notes)

## Components added

| File | New public items |
| --- | --- |
| `primitives/button.rs` | `as` (root tag override), `left_icon`, `right_icon` props on `Button` |
| `primitives/card.rs` | `CardLink` (anchor-wrapped card), `CardDivider` (horizontal rule with `role="separator"`) |
| `primitives/badge.rs` | `dot: Option<bool>` (color-coded dot prefix), `truncate: Option<bool>` (ellipsis) on `Badge` |
| `primitives/avatar.rs` | `status: Option<String>` ("online"/"offline"/"away"/"busy") on `Avatar` |
| `primitives/table.rs` | `TableEmpty { colspan, children }`, `TableLoading { colspan }`, `TableFooter { variant }`, `caption: Option<String>` on `Table` |
| `primitives/data_table.rs` | `cell_renders: Option<Vec<(String, Callback<Row, Element>)>>`, `selectable: bool`, `selected: Option<Vec<String>>`, `on_select_change: Option<EventHandler<Vec<String>>>`, `bulk_actions: Option<Element>`, `caption: Option<String>` on `DataTable` |
| `primitives/charts.rs` | `ChartArea` (filled line chart), `ChartStackedBar` (multi-series stacked) |
| `primitives/stat_card.rs` | `trend_direction: Option<String>` ("up"/"down"/"flat" auto-color), `sparkline: Option<Element>` slot |
| `primitives/progress.rs` | `indeterminate: Option<bool>` (animated bar), `size: Option<String>` ("sm"/"md"/"lg"), `label: Option<String>` (above bar with percent) |
| `primitives/skeleton.rs` | `SkeletonCircle { size }`, `SkeletonBlock { width, height }` |
| `primitives/separator.rs` | `label: Option<String>` (centered "or" divider with two flanking lines) |
| `primitives/icon.rs` | `IconButton { name, aria_label, title, ...Button props }` (square button with `btn-icon`) |

## Signature changes

All changes are **strictly additive**. No existing parameter was renamed or
removed, no required parameter was added, and the `Column` struct was kept
as-is to preserve the `Column { key, label, sortable, align, width,
class_name }` struct-literal usage across all `pages/admin_pages/*.rs` files.

The `render: Option<Callback<Row, Element>>` cell-rendering capability is
exposed via a new `DataTable` prop `cell_renders: Option<Vec<(String,
Callback<Row, Element>)>>` — a list of `(column_key, callback)` pairs.
Unspecified columns fall back to the raw string cell. This avoids touching
the `Column` struct and the dozen pages that use `Column { ... }` struct
literals.

## A11y gaps intentionally left for Wave 2

- **`StatCard.trend_direction` color-only signaling.** The green/red/gray
  color is the only indicator. A future iteration should add an
  `aria-label` like "up by 5%" or include a screen-reader-only `▲`/`▼`
  symbol. Wave 1 keeps the visual parity minimal.
- **`DataTable` sortable header keyboard navigation.** Sort currently fires
  on click only; `Enter`/`Space` on the `<th>` does not toggle. Tracked
  for Wave 2 alongside the `Tabs` keyboard-roving work.
- **`Avatar.status` dot is decorative.** It uses `role="img"` with
  `aria-label` of the status string, but most screen readers will
  announce the avatar's `name` first, then the status. A more thorough
  solution would integrate the status into the avatar's accessible name
  (e.g. "Online user John"). Noted as Wave 2.
- **`IconButton` keyboard focus ring.** Inherits whatever `:focus-visible`
  the design system provides; no extra focus styles were added. Wave 2
  will add the standard focus ring if missing from the design tokens.

## `cargo check --workspace` result (last 10 lines)

```
warning: `epsx-identity` (bin "identity") generated 2 warnings (run `cargo fix --bin "identity"` to apply 2 suggestions)
warning: `epsx-content` (bin "content") generated 2 warnings (run `cargo fix --bin "content"` to apply 2 suggestions)
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
warning: `epsx-notification` (bin "notification") generated 4 warnings (run `cargo fix --bin "notification"` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.47s
```

## `cargo check -p epsx-dioxus-ui --all-targets` result

Compiles cleanly with 63 warnings (all pre-existing `mut` / unused-import
warnings in pages and pre-existing `MouseData::default` issue in
`primitives/modal.rs` which is out of scope for Track B). No new errors
or warnings introduced by Track B.

## Notes for the verifier

1. **Worktree path.** This work was developed in
   `/private/tmp/epsx-track-b` (a `git worktree` of the `epsx` repo on
   branch `wave1/track-b-display`). The main workspace at
   `/Users/fluke/Desktop/Work/epsx` is shared with Track A / C / D —
   multiple agents were checking out their own branches, so the worktree
   was required to keep the working tree stable. The branch itself lives
   in the main repo (visible from any clone of the repo), and the commit
   is reachable from `refs/heads/wave1/track-b-display`.

2. **Pages not edited.** `git diff --stat HEAD~1..HEAD -- shared/rust/dioxus_ui/src/pages/`
   returns empty — no page file was touched.

3. **`Column` struct unchanged.** Existing `pages/admin_pages/*.rs` files
   use `Column { key, label, sortable, align, width, class_name }` with
   no `..Default::default()`. The new cell-render capability is exposed
   via the `cell_renders` prop on `DataTable` instead of as a `Column`
   field, to avoid breaking every page.

4. **`primitives.rs` re-exports.** The new public items
   (`CardLink`, `CardDivider`, `TableEmpty`, `TableLoading`, `TableFooter`,
   `ChartArea`, `ChartStackedBar`, `SkeletonCircle`, `SkeletonBlock`,
   `IconButton`) are auto-exported via the existing `pub use ...::*;`
   glob re-exports in `primitives.rs`. No change to `primitives.rs` was
   required.

5. **Pre-existing build issues in Track A / C scope (NOT my work):**
   - `primitives/modal.rs:101-102` — `MouseData::default()` no longer
     exists in dioxus-html 0.7.9; this is a Track C issue.
   - `primitives/form.rs` — duplicate `Label` definition (Track A was
     adding `Label` simultaneously).
   These are pre-existing or owned by other tracks; my Track B changes do
   not touch `modal.rs` or `form.rs` and do not introduce any new errors.

6. **CSS classes introduced.** The new variants use existing utility
   classes (`btn-icon`, `truncate`, `max-w-[12rem]`, `bg-emerald-400`,
   `bg-red-400`, `bg-amber-400`, `bg-blue-400`, `bg-purple-400`,
   `bg-cyan-400`, `bg-orange-400`, `bg-gray-400`, `text-emerald-400`,
   `text-red-400`, `text-muted-foreground`, `w-1.5 h-1.5 rounded-full`,
   `inline-block`, `h-1`, `h-2`, `h-2.5`, `mr-1`, `chart-area`,
   `chart-bar-stacked`, `data-table-bulk-actions`, `flex-1 h-px bg-border`).
   Two new identifiers used in `ChartArea` and `DataTable` markup
   (`chart-area`, `chart-bar-stacked`, `data-table-bulk-actions`) need
   CSS rules — the design-system track should add them. Until then, they
   degrade gracefully (no layout) but won't have distinct styling.

7. **Icons.** No new icons were added to the lucide registry. The `Icon`
   component still uses `epsx_templates::lucide`. Wave 1 only consumes
   existing icons.

---

# Track C — Interactive primitive parity (Wave 1)

## Summary

Brought the interactive Dioxus primitives in
`shared/rust/dioxus_ui/src/primitives/` to 1:1 UX/UI parity with the
Next.js shadcn/Radix sources listed in the design doc. The work is
additive — all existing public APIs are preserved (no renamed or removed
parameters), every component gained controlled-state, `side`/`align`
positioning, slot components, and a11y attributes (`role`, `aria-*`).

A second pass (amended on the same commit) closed two a11y gaps
flagged by the verifier:
1. `Dropdown` trigger wrapper now carries `role="button"`,
   `aria-haspopup="menu"`, `aria-expanded={open}`, and `tabindex="0"`
   so screen readers announce the open state.
2. `Tooltip` bubble gets a stable `id` and the trigger wrapper carries
   `aria-describedby={bubble_id}` so assistive tech can announce the
   tooltip text when the trigger is focused or hovered.

A third pass (also amended) added the design-system CSS for every new
class name the six interactive-primitive files introduce, per the
design addendum (`docs/wave1-primitives/design-addendum-1.md`). The
CSS lives in `shared/rust/templates/src/lib.rs` and is documented in
a single block comment (Track C section, after Track A's block).

The `cargo check --workspace` and `cargo check -p epsx-dioxus-ui --all-targets`
runs at the end of this track are green (warnings only — no errors).

## Branch & commit

- **Branch:** `wave1/track-c-interactive`
- **Worktree:** `/private/tmp/epsx-track-c` (isolated from the parallel
  Track A/B/D agents that were stomping on the main checkout)
- **Commit:** `feat(dioxus-ui): track C — interactive primitive parity`
  (amended after the verifier a11y pass)
- **Base:** `4d90b402 docs(wave1): design doc for Dioxus UI primitive parity (3 coder tracks + verifier)`

The branch was created and committed inside an isolated worktree at
`/private/tmp/epsx-track-c` after the main checkout got hit by parallel
agent activity (a Track A `git stash pop` and Track D untracked files
overwrote the first attempt). The committed files here are the second,
clean attempt.

## Changed files

| File | Change |
| --- | --- |
| `shared/rust/dioxus_ui/src/primitives/dropdown.rs` | New `align` / `side` / `menu_class` props, `on_open_change`, `DropdownLabel`, `DropdownCheckboxItem`, `DropdownItem.inset`; trigger wrapper now has `role="button"`, `aria-haspopup="menu"`, `aria-expanded`, `tabindex="0"` |
| `shared/rust/dioxus_ui/src/primitives/modal.rs` | New `on_open_change`, focus trap on open, Escape handler, `close_on_overlay` / `close_on_escape` / `initial_focus` toggles, `ModalHeader` / `ModalBody` / `ModalFooter` slot components |
| `shared/rust/dioxus_ui/src/primitives/tabs.rs` | New `vertical` orientation, `on_change` alias for `on_select`, `class_name` override, `aria-orientation` |
| `shared/rust/dioxus_ui/src/primitives/tooltip.rs` | New `side` / `align` / `delay` / controlled `open`; bubble only renders when trigger is hovered / focused or `open` is set; stable `id` on the bubble + `aria-describedby` on the trigger wrapper |
| `shared/rust/dioxus_ui/src/primitives/overlays.rs` | `Popover` controlled `open` + `on_open_change` + `side` / `align`; `HoverCard` `open_delay` / `close_delay`; `Accordion` controlled `open_keys` + `on_change`; `Collapsible` controlled `open` + `on_open_change`; `CommandPalette` keyboard nav (Arrow / Enter / Escape) + `on_select` / `on_close` |
| `shared/rust/dioxus_ui/src/primitives/rich_text.rs` | Toolbar buttons now wrap the live textarea selection via a JS helper that runs through `document::eval`. The broken `format!("{}**bold**", ...)` path is gone |
| `shared/rust/templates/src/lib.rs` | **Design-system CSS pass**: added CSS rules for every new class introduced by the six interactive-primitive files (per the design addendum). Includes a single block comment cataloguing each new class. No public API change. |

Re-exports in `primitives.rs` are unchanged — the new components ride
the existing `pub use overlays::*; pub use dropdown::*;` glob imports.

## Component-level notes

### Dropdown (`dropdown.rs`)

- New optional params: `on_open_change: Option<EventHandler<bool>>`,
  `align: Option<String>` ("start" | "center" | "end"),
  `side: Option<String>` ("top" | "bottom"),
  `menu_class: Option<String>`.
- `DropdownItem` gained `inset: Option<bool>` for Radix-style indented
  items. Behaviour and class names are unchanged.
- New slot components: `DropdownLabel { children }` (role="presentation"
  wrapper) and `DropdownCheckboxItem { checked, onchange, icon, children }`
  (role="menuitemcheckbox", `aria-checked`, with a leading check icon).
- The trigger wrapper carries `role="button"`, `tabindex="0"`,
  `aria-haspopup="menu"`, and `aria-expanded={open}` so screen readers
  announce the open state and keyboard users can focus it.
- The menu container has `role="menu"`; menu items have `role="menuitem"`.
- The legacy `on_toggle: EventHandler<MouseEvent>` is preserved.

### Modal (`modal.rs`)

- New optional params: `on_open_change: Option<EventHandler<bool>>`,
  `close_on_overlay: bool` (default `true`),
  `close_on_escape: bool` (default `true`),
  `initial_focus: bool` (default `true`),
  `class_name: Option<String>`.
- Focus trap: on open, a `document::eval` script runs to focus the
  first focusable descendant inside the dialog panel. The script is a
  no-op during SSR.
- Escape key: the dialog has an `onkeydown` handler that fires
  `on_open_change(false)` when the key is `Escape` and
  `close_on_escape` is true. The legacy `on_close` is also called
  where the keystroke is converted to a click-equivalent event.
- New slot components: `ModalHeader`, `ModalBody`, `ModalFooter`.
- The legacy `on_close: EventHandler<MouseEvent>` is preserved.

### Tabs (`tabs.rs`)

- New optional params: `on_change: Option<EventHandler<String>>` (fires
  in addition to `on_select` so callers can pick either name),
  `vertical: bool` (default `false`; adds `tabs-vertical` class and
  `aria-orientation="vertical"`),
  `class_name: Option<String>`.
- The `items` shape, `active` / `on_select` signature, and `TabItem`
  struct are unchanged.

### Tooltip (`tooltip.rs`)

- New optional params: `side`, `align`, `delay` (ms; emitted as a CSS
  custom property `--tooltip-delay`), controlled `open`.
- The bubble is rendered with `role="tooltip"` and gets
  `tooltip-side-{top,left,right,bottom}` and `tooltip-align-{start,end,center}`
  modifier classes. With no `open` prop and no CSS hover/focus
  handling, the bubble stays hidden — i.e. it is no longer always-on,
  fixing the previous "tooltip overlay is always visible" bug.
- A11y: the bubble gets a stable `id` (counter-based, SSR-friendly) and
  the trigger wrapper (the `span` that contains the caller's children)
  carries `aria-describedby={bubble_id}` so screen readers announce
  the tooltip text when the trigger is focused or hovered.
- Text prop remains required for backward compatibility.

### Overlays (`overlays.rs`)

- **Popover** — `trigger` and `children` are still the two required
  params. New: `open: Option<bool>` (controlled), `on_open_change:
  Option<EventHandler<bool>>`, `side`, `align`. The trigger has
  `aria-expanded` reflecting the current state.
- **HoverCard** — `trigger` and `children` are still the two required
  params. New: `open_delay: Option<u32>` (default 200ms),
  `close_delay: Option<u32>` (default 300ms). The CSS custom
  properties `--hover-card-open-delay` and `--hover-card-close-delay`
  are emitted for the stylesheet to consume; a `data-visible`
  attribute mirrors the JS state for `hover-card-content`.
- **Accordion** — `items` is still required. New: `open_keys:
  Option<Vec<String>>` (controlled), `on_change: Option<EventHandler<Vec<String>>>`.
  The existing `initial_open` and `allow_multiple` params are kept.
  Triggers get `aria-expanded`.
- **Collapsible** — `trigger` and `children` are still the two
  required params. New: `open: Option<bool>` (controlled),
  `on_open_change: Option<EventHandler<bool>>`. The existing
  `initial_open` is kept. Triggers get `aria-expanded`.
- **CommandPalette** — `commands` is still required. The render is
  gated on `open: bool` (default `false`). New: `on_select:
  Option<EventHandler<String>>` (fires on click or Enter on the
  focused row), `on_close: Option<EventHandler<MouseEvent>>`
  (declared in the signature, but the Escape keypress is a no-op —
  see gap below). The input has `onkeydown` handling for
  ArrowUp / ArrowDown / Enter / Escape.

### Rich text editor (`rich_text.rs`)

The original `format!("{}**bold**", content.read().clone())` is gone.
The four toolbar buttons (B / C / 🔗 / H) now run a JS helper via
`document::eval` that:
1. Reads `selectionStart` / `selectionEnd` from the live textarea.
2. Splices `prefix` + (selected text or placeholder) + `suffix` into
   the buffer at the selection.
3. Restores the cursor inside the inserted text.
4. Pushes the new value back into the `content` signal so the
   controlled state stays in sync.

The selection range is also tracked on `onmouseup` and `onkeyup` so
keystrokes that change the cursor are picked up before the next wrap.
The script is a no-op on the server (no `document`), so SSR still
renders the editor with the previous append-style behaviour as a
graceful degradation.

## Known gaps (intentional, documented)

### `CommandPalette.on_close` is unreachable from keyboard Escape

The task asks for
`on_close: Option<EventHandler<MouseEvent>>` to fire when the user
presses Escape. Dioxus 0.7's `MouseEvent` is `Event<MouseData>` and
`MouseData::new` requires a `HasMouseData + 'static` impl. Synthesising
one without a real DOM target is non-trivial and would add a stub
trait impl that the rest of the codebase doesn't need.

**Mitigation:** the param is still on the public signature (so callers
that opt into it aren't broken), and we added `let _ = on_close;` to
suppress unused warnings. The parent component can attach their own
`onkeydown` listener and toggle their `open` state to dismiss the
palette. The same caveat is repeated in the `CommandPalette` doc
comment.

This is the only place where the task spec was not implementable
without a new dependency or a lot of unsafe code. The on_select
handler (the more useful callback) works fully.

### `Tooltip.delay` only emits a CSS custom property

`delay: Option<u32>` is forwarded as `--tooltip-delay: {n}ms;` on the
wrapper. The actual JS-driven show / hide after the delay is left to
the design-system stylesheet. The previous version was always-on, so
this is still an improvement; the design-system track will pick up
the CSS hook.

### `RichTextEditor` requires JS for cursor-aware editing

SSR-only renderers (no hydration) will fall back to "append at start"
because the `document::eval` call is a no-op. The news / policy
editor is rendered through Dioxus's fullstack pipeline with
hydration, so the wrap-on-cursor path activates after the first user
interaction.

## Verification

```
$ cargo check --workspace 2>&1 | tail -10
warning: function `require_admin` is never used
  --> apps/admin/src/auth.rs:55:8
   |
55 | pub fn require_admin(headers: &HeaderMap, jwt: &JwtAuth) -> Result<AuthUser, AuthError> {
   |        ^^^^^^^^^^^^^

warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-notification` (bin "notification") generated 4 warnings (run `cargo fix --bin "notification"` to apply 1 suggestion)
warning: `epsx-content` (bin "content") generated 2 warnings (run `cargo fix --bin "content"` to apply 2 suggestions)
warning: `epsx-identity` (bin "identity") generated 2 warnings (run `cargo fix --bin "identity"` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.71s
```

`cargo check -p epsx-dioxus-ui --all-targets` is also green (60
warnings, no errors — same baseline as before the changes).
`cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`
is green (all BFFs still compile, no pages needed editing).
`cargo check -p epsx-templates` is green.

## CSS classes added (per the design addendum)

The third pass added design-system CSS for every new class name
introduced by the six interactive-primitive files. The full list with
purpose and consumer is documented in a single block comment at the
bottom of the `<style>` body in `shared/rust/templates/src/lib.rs`
(`design_system_head`). The classes are:

| Family | Classes | Consumer |
| --- | --- | --- |
| Dropdown | `dropdown-label`, `dropdown-item-inset`, `dropdown-item-check`, `dropdown-item-checked`, `dropdown-checkbox-item`, `dropdown-menu-side-top/bottom`, `dropdown-menu-align-start/end/center` | `primitives/dropdown.rs` |
| Modal | `modal-overlay`, `modal-header`, `modal-title`, `modal-close`, `modal-description`, `modal-body`, `modal-footer`, `modal-sm/lg/xl/full` | `primitives/modal.rs` |
| Tabs | `tabs`, `tab`, `tab-active`, `tab-icon`, `tabs-vertical` | `primitives/tabs.rs` |
| Tooltip | `tooltip-wrapper`, `tooltip-content`, `tooltip-open`, `tooltip-side-top/bottom/left/right`, `tooltip-align-start/end/center` | `primitives/tooltip.rs` |
| Popover | `popover`, `popover-trigger`, `popover-content`, `popover-content-side-top/bottom/left/right`, `popover-content-align-start/end/center` | `primitives/overlays.rs` |
| HoverCard | `hover-card`, `hover-card-content` | `primitives/overlays.rs` |
| Accordion | `accordion`, `accordion-item`, `accordion-trigger`, `accordion-content`, `accordion-icon`, `accordion-item.open` | `primitives/overlays.rs` |
| Collapsible | `collapsible`, `collapsible-trigger`, `collapsible-content`, `collapsible.open` | `primitives/overlays.rs` |
| Command palette | `command-palette-overlay`, `command-palette`, `command-input`, `command-list`, `command-item`, `command-item.active`, `command-empty`, `command-hint` | `primitives/overlays.rs` |
| Rich text | `rich-text-editor`, `rte-toolbar`, `rte-preview` (plus preview typography) | `primitives/rich_text.rs` |

All rules reuse the existing `--epsx-*` CSS custom properties and the
global Tailwind v2 utility set. No new global tokens, no new
dependencies, no public-API change to `epsx_templates`.

## Files owned (per task scope)

All edits are inside the seven files the task grants ownership of, plus
`shared/rust/templates/src/lib.rs` (the design-system CSS file, per
the addendum's expanded rule):

- `shared/rust/dioxus_ui/src/primitives/dropdown.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/modal.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/tabs.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/tooltip.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/overlays.rs` — modified
- `shared/rust/dioxus_ui/src/primitives/rich_text.rs` — modified
- `shared/rust/dioxus_ui/src/primitives.rs` — unchanged (the new
  public items ride the existing glob re-exports)
- `shared/rust/templates/src/lib.rs` — design-system CSS pass

No `pages/*.rs` files were edited. The single consumer
`admin_pages/news.rs` continues to work unchanged because the
`RichTextEditor` signature is preserved.
=======
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.13s
```

Exit code: 0. No errors. All 60 warnings in `epsx-dioxus-ui` are
pre-existing from other files; none originate in my 4 changed
files (verified with `cargo check -p epsx-dioxus-ui 2>&1 |
grep -E "alert|alert_dialog|sheet|misc.rs"`, returns nothing).

## a11y recap (post-fix)

| Component | Required a11y | Status |
| --- | --- | --- |
| `Alert` | `role="alert"` | ✅ (already had) |
| `AlertDialog` | `role="alertdialog"` + `aria-modal` | ✅ (already had) |
| `AlertDialog` | `aria-labelledby` + `aria-describedby` | ✅ (NEW — this commit) |
| `Sheet` | `role="dialog"` + `aria-modal` | ✅ (already had) |
| `Sheet` | `aria-labelledby` + `aria-describedby` | ✅ (NEW — this commit) |
| `ScrollArea` viewport | `role="region"` | ✅ (NEW — this commit) |
| `ScrollArea` viewport | `tabindex="0"` | ✅ (NEW — this commit) |
| `Label` | standard `<label for>` | ✅ (already had) |

## Notes for the verifier

- The previous attempt's commit (`48ad0bd1`) and deliverable commit
  (`ce539b77`) are still in the branch — the fix is on top as
  `9496c824`. `git log wave1/track-d-missing` shows the full
  history.
- The new `AtomicU64` ID-generation counters are
  module-private statics. The TS shadcn source uses Radix's
  `useId()` hook for the same purpose; the Dioxus equivalent
  pattern is a `static AtomicU64` (same as `epsx_dioxus_ui`'s
  existing `feedback::toast::NEXT_ID`).
- No pages were edited, no new dependencies were added, no
  primitive signatures were changed.
- All four edits are purely additive (new attribute strings, new
  static counter, new imports of `std::sync::atomic`). Existing
  consumers of these components continue to work unchanged.
- The lucide registry gap is a real gap in `epsx-templates`, not
  in my code. The TODO comment in `alert.rs` lists the exact
  icons to add; until then, the substitutes render fine (each
  `epsx_templates::lucide(...)` call falls back to an empty body
  if the name isn't registered, which would have shown blank
  icons; now they render the substitute instead).
>>>>>>> wave1/track-d-missing

---

# Track D — Missing TS parity primitives (Dioxus) — Attempt 2 (a11y fixes)

## Summary

Addressed the 3 a11y gaps the verifier flagged in attempt 2:
(1) added `aria-labelledby` + `aria-describedby` wiring to
`AlertDialog` and `Sheet` with auto-generated stable IDs, (2) added
`role="region"` and `tabindex="0"` to the `ScrollArea` viewport,
(3) swapped the 3 unregistered lucide icons (`check-circle`,
`alert-triangle`, `alert-circle`) for registered equivalents
(`check`, `bell`, `x`) and added a TODO comment listing the icons
to add to the lucide registry. `cargo check --workspace` exits 0.

## Branch / commits

- **Branch**: `wave1/track-d-missing`
- **Latest commit**: `9496c824` (a11y fixes; on top of `ce539b77`
  deliverable + `48ad0bd1` initial code).
- **Files changed in this commit** (4):
  ```
  shared/rust/dioxus_ui/src/primitives/alert.rs           |  19 ++++++++--
  shared/rust/dioxus_ui/src/primitives/alert_dialog.rs   |  40 ++++++++++++++++++----
  shared/rust/dioxus_ui/src/primitives/sheet.rs          |  38 +++++++++++++++++---
  shared/rust/dioxus_ui/src/primitives/misc.rs           |  10 +++++-
  ```

## What changed (verifier's 3 issues)

### Issue (1) — `aria-labelledby` / `aria-describedby` wiring

**`alert_dialog.rs`**:
- Added a `static ALERT_DIALOG_NEXT_ID: AtomicU64` that gets
  incremented per component instance to generate a stable
  per-instance nonce.
- The `AlertDialog` root now sets:
  - `role="alertdialog"` (already there)
  - `aria-modal="true"` (already there)
  - `aria-labelledby="alertdialog-title-{n}"` (new)
  - `aria-describedby="alertdialog-desc-{n}"` (new)
- The title `<h2>` and description `<p>` now carry matching
  `id="alertdialog-title-{n}"` / `id="alertdialog-desc-{n}"`.
- `AlertDialogTitle` and `AlertDialogDescription` slot components
  also generate their own IDs so a future caller-managed
  composition can wire them up.

**`sheet.rs`**:
- Added a `static SHEET_NEXT_ID: AtomicU64` with the same pattern.
- The `Sheet` root now sets:
  - `role="dialog"` (already there)
  - `aria-modal="true"` (already there)
  - `aria-labelledby="sheet-title-{n}"` (new)
  - `aria-describedby="sheet-desc-{n}"` (new)
- The root sheet renders a visually-hidden `<h2 id="...">Sheet
  panel</h2>` and `<p id="...">Slide-in side panel</p>` pair
  (using `class="sr-only"`) so the dialog always has a valid
  accessible name + description, matching the TS shadcn source's
  `VisuallyHidden.Root` pattern.
- `SheetTitle` and `SheetDescription` slot components generate
  their own IDs for richer visible titles/descriptions.

### Issue (2) — `ScrollArea` viewport a11y

**`misc.rs`** — added to the inner `scroll-area-viewport` div:
- `role: "region"` (the viewport is a content region)
- `tabindex: "0"` (the region is focusable for keyboard scrolling)

This mirrors the behaviour of Radix's `ScrollArea.Viewport` (which
sets `tabindex` and `data-radix-scroll-area-viewport`).

### Issue (3) — Lucide icon registry gaps

`epsx_templates::lucide` only registers these names: `chart-column`,
`code`, `building`, `chevron-down`, `chevron-right`, `trending-up`,
`chart-line`, `zap`, `users`, `calendar`, `newspaper`, `pin`,
`arrow-right`, `info`, `mail`, `help-circle`, `circle-help`, `menu`,
`x`, `sun`, `moon`, `wallet`, `log-out`, `user`, `settings`,
`check`, `plus`, `search`, `share`, `bell`, `book`, `key`,
`layout-dashboard`, `message-circle`, `file-text`, `history`,
`credit-card`, `link`, `external-link`, `briefcase`.

My previous default-icon map used `check-circle`, `alert-triangle`,
and `alert-circle`, all of which are NOT in the registry (only
`info` and `check` are). Swapped to:

| `AlertKind` | Old (not registered) | New (registered) |
| --- | --- | --- |
| `Default`  | `info`           | `info` (no change) |
| `Success`  | `check-circle`   | `check` |
| `Warning`  | `alert-triangle` | `bell` |
| `Danger`   | `alert-circle`   | `x` |
| `Info`     | `info`           | `info` (no change) |

Added a `TODO: add lucide registry entries for the proper shadcn
names (check-circle, alert-triangle, alert-circle)` comment in
`alert.rs` documenting the gap.

## `cargo check --workspace` (last 10 lines)

```
warning: struct `NewsQuery` is never constructed
  --> apps/frontend/src/api.rs:39:12
   |
39 | pub struct NewsQuery {
   |            ^^^^^^^^^

warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.13s
```

Exit code: 0. No errors. All 60 warnings in `epsx-dioxus-ui` are
pre-existing from other files; none originate in my 4 changed
files (verified with `cargo check -p epsx-dioxus-ui 2>&1 |
grep -E "alert|alert_dialog|sheet|misc.rs"`, returns nothing).

## a11y recap (post-fix)

| Component | Required a11y | Status |
| --- | --- | --- |
| `Alert` | `role="alert"` | ✅ (already had) |
| `AlertDialog` | `role="alertdialog"` + `aria-modal` | ✅ (already had) |
| `AlertDialog` | `aria-labelledby` + `aria-describedby` | ✅ (NEW — this commit) |
| `Sheet` | `role="dialog"` + `aria-modal` | ✅ (already had) |
| `Sheet` | `aria-labelledby` + `aria-describedby` | ✅ (NEW — this commit) |
| `ScrollArea` viewport | `role="region"` | ✅ (NEW — this commit) |
| `ScrollArea` viewport | `tabindex="0"` | ✅ (NEW — this commit) |
| `Label` | standard `<label for>` | ✅ (already had) |

## Notes for the verifier

- The previous attempt's commit (`48ad0bd1`) and deliverable commit
  (`ce539b77`) are still in the branch — the fix is on top as
  `9496c824`. `git log wave1/track-d-missing` shows the full
  history.
- The new `AtomicU64` ID-generation counters are
  module-private statics. The TS shadcn source uses Radix's
  `useId()` hook for the same purpose; the Dioxus equivalent
  pattern is a `static AtomicU64` (same as `epsx_dioxus_ui`'s
  existing `feedback::toast::NEXT_ID`).
- No pages were edited, no new dependencies were added, no
  primitive signatures were changed.
- All four edits are purely additive (new attribute strings, new
  static counter, new imports of `std::sync::atomic`). Existing
  consumers of these components continue to work unchanged.
- The lucide registry gap is a real gap in `epsx-templates`, not
  in my code. The TODO comment in `alert.rs` lists the exact
  icons to add; until then, the substitutes render fine (each
  `epsx_templates::lucide(...)` call falls back to an empty body
  if the name isn't registered, which would have shown blank
  icons; now they render the substitute instead).
