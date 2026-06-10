# Track A — Form & Input Primitive Parity — Deliverable

## Summary

Brought the form & input Dioxus primitives in `shared/rust/dioxus_ui/src/primitives/`
to 1:1 UX/UI parity with the Next.js shadcn/Radix components. Added 7 new
components and enhanced 4 existing ones, all while preserving the existing
public API (no signature changes, no removed parameters, no required booleans).

## Branch

- Branch name: `wave1/track-a-form-input`
- Forked from: `migration/dioxus-microservices` @ `4d90b402`
- Commits:
  - `e90468a8` — `feat(dioxus-ui): track A — form & input primitive parity` (9 files, +1079/−61)
  - `64300cce` — `docs(wave1): track A deliverable` (deliverable.md)

## Changed files (9)

| File | Δ | Highlights |
| --- | --- | --- |
| `shared/rust/dioxus_ui/src/primitives/input.rs` | 153 / −30 | New `InputKind::File`, `Color`, `Hidden`; icon now rendered **inside** the wrap (fixes the absolute-positioning context); added controlled + uncontrolled mode (`value` vs `default_value`); `aria-invalid` always emitted; `aria_invalid` opt exposed. |
| `shared/rust/dioxus_ui/src/primitives/form.rs` | 170 / −0 | Added `Label`, `InputGroup`, `FormSection`, `FormRow`, `RadioGroup`. All existing components (`Form`, `Field`, `FormField`, `Textarea`, `SelectField`, `CheckboxField`, `FormActions`) preserved bit-for-bit. |
| `shared/rust/dioxus_ui/src/primitives/select.rs` | 171 / −1 | Added `MultiSelect` with chip rendering, dropdown menu, optional `max` cap, hidden-input mirror for form submission. Single `Select` now also accepts `help` / `error` / `id` / `required` (additive — all default to `None` / `false`). |
| `shared/rust/dioxus_ui/src/primitives/combobox.rs` | 274 / −1 | Added `ComboboxAsync` (externally-loaded options + `loading` flag + `empty_text` + `on_select` callback) and `ComboboxMulti` (multi-select with chips + `on_change: EventHandler<Vec<String>>` + optional `max`). Original `Combobox` unchanged. |
| `shared/rust/dioxus_ui/src/primitives/date_picker.rs` | 94 / −2 | Added `DateTimePicker` (date + time inputs side-by-side, optional `combined_name` for a single hidden `YYYY-MM-DDTHH:MM` field). Existing `DatePicker` + `DateRangePicker` preserved. |
| `shared/rust/dioxus_ui/src/primitives/stepper.rs` | 118 / −3 | Added `progress: bool` flag on `Stepper` (renders a `progressbar` above the row). Added `Step` struct with `label` / `complete` / `icon` and new `StepperSteps` component for the icon-rich variant. Original tuple API still works (used by `pages/payment.rs`). |
| `shared/rust/dioxus_ui/src/primitives/checkbox.rs` | 39 / −1 | Added `indeterminate: Option<bool>` and `class_name: Option<String>`. Sets `data-state="indeterminate"`, `aria-checked="mixed"`, and adds `checkbox-indeterminate` class. |
| `shared/rust/dioxus_ui/src/primitives/switch.rs` | 42 / −1 | Added `SwitchSize` enum (`Sm` / `Md` / `Lg`) and `size` prop. Root element gets `switch-{sm|md|lg}` and `state-checked/unchecked` classes. Default size is `Md`. |
| `shared/rust/dioxus_ui/src/primitives/misc.rs` | 79 / −42 | `Slider` now emits `role="slider"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax`, `aria-label`, plus `disabled` prop. `Rating` is now clickable: each star fires `onchange: EventHandler<u8>`, the hidden `<input>` reflects the latest value, and a `disabled` prop disables the interaction. New `KbdCombo { keys: Vec<String> }` component for keyboard shortcuts (e.g. `["Cmd", "K"]`). Original `Kbd { text: String }` kept for single-key use. |

## New public symbols

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

None — every change is additive. New params on existing components are
`#[props(default = …)]` and `Option<T>`. No call site in
`shared/rust/dioxus_ui/src/pages/` was touched.

## A11y

| Component | a11y attributes |
| --- | --- |
| `Input` | `aria-invalid`, `html-for` label, `required` |
| `Label` | `html-for`, `*` indicator |
| `RadioGroup` | `role="radiogroup"`, `aria-labelledby`, `aria-required`, `aria-invalid` |
| `MultiSelect` | `role="listbox"`, `aria-multiselectable`, `aria-invalid`, `aria-label` on chip-remove buttons |
| `ComboboxAsync` | `aria-expanded`, `aria-busy`, `aria-autocomplete`, `aria-invalid`, `aria-label` on close, `role="listbox"` / `role="option"` |
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
  focused option, matching the Radix combobox behavior.
- **Slider keyboard support** — `<input type="range">` natively supports
  arrow keys, so the a11y contract is satisfied at the HTML level. A Wave 2
  track may want to wire `Home` / `End` / `PageUp` / `PageDown` shortcuts.
- **Modal focus trap on `MultiSelect` open** — clicking the trigger button
  does not move focus to the dropdown menu. Wave 2 can add a focus-trap
  when the popover is open.
- **ComboboxMulti "create new"** — when `allow_free` is true, hitting
  Enter on a non-matching query should be able to create a new option.
  Currently the `Combobox` family has this prop but doesn't fire a creation
  callback; the parent can add it via `on_change` if needed.

## Rules respected

- No new class names added to global CSS — only existing design-system
  classes (`input`, `input-with-icon`, `input-icon-wrap`, `input-error`,
  `checkbox-indeterminate`, `switch-{sm|md|lg}`, `stepper-progress`,
  `progress`, `progress-bar`, `rating-interactive`, `rating-disabled`,
  `combobox-async`, `combobox-loading`, `combobox-empty`, `combobox-multi`,
  `multiselect-*`, `kbd-combo`, `form-section`, `input-group`, `radio-*`,
  `datetime-picker`).
- No new icons added to the lucide registry — all `Icon { name = "…" }`
  uses names already present in the registry (`check`, `chevron-down`,
  etc.).
- No `dangerous_inner_html` outside the existing `QrCode` and the
  icon helper.
- No page files were edited.
- No BFF files were edited.

## `cargo check` results

### `cargo check -p epsx-dioxus-ui --all-targets` (last 10 lines)

```
warning: `epsx-dioxus-ui` (lib test) generated 60 warnings (60 duplicates)
warning: `epsx-dioxus-ui` (lib) generated 60 warnings (run `cargo fix --lib -p epsx-dioxus-ui` to apply 59 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.59s
```

All warnings are pre-existing (unused `crate::primitives::icon::Icon`
imports in `layout/navbar.rs`, `layout/footer.rs`, `layout/breadcrumbs.rs`,
`feedback/spinner.rs`, `auth/user.rs`; `unused_braces` warnings on
`card.rs`, `dropdown.rs`, `table.rs`, `filter_bar.rs`; `unused_imports`
in various `pages/*.rs`). No new warnings introduced by this track.

### `cargo check --workspace` (last 5 lines)

```
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.44s
```

### `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`

```
warning: `epsx-admin` (bin "bff-admin") generated 7 warnings (run `cargo fix --bin "bff-admin"` to apply 3 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.34s
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
| Branch | `wave1/track-a-form-input` |
| Commit hash | `e90468a8` + `64300cce` |
