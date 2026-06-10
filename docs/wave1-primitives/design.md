# Wave 1 — Dioxus UI Primitive Parity

Shared design doc for all three coder tracks in the Wave 1 team plan.
Read this end-to-end before touching any primitive file.

## Goal

Bring `shared/rust/dioxus_ui/src/primitives/*` to **1:1 UX/UI parity** with
the Next.js shadcn/Radix components in `apps-old/frontend/components/ui/`
and `apps-old/admin-frontend/components/ui/`. The Dioxus primitives already
exist and many are reasonable — most of the work is **gap-fill, not
from-scratch**. After this wave, every component in the TS shadcn set
should have a Dioxus counterpart with a stable public API.

## Source of truth

| What | Where |
| --- | --- |
| Rust primitives (current state) | `shared/rust/dioxus_ui/src/primitives/*.rs` |
| TS shadcn source (target) | `shared/components/ui/*.tsx` (real files; `apps-old/*/components/ui/*` are re-exports) |
| Admin TS additions | `apps-old/admin-frontend/components/ui/*.tsx` (pancake-card, theme-toggle, etc.) |
| Pages that consume primitives | `shared/rust/dioxus_ui/src/pages/*.rs` |
| BFFs that render pages | `apps/frontend/src/`, `apps/admin/src/`, `apps/pay/src/`, `apps/preview/src/` |
| Icon helper | `epsx_templates::lucide(name, size, class)` returns SVG string |
| Design system CSS | `epsx_templates::design_system_head` — emits the Tailwind v2 CDN + EPSX tokens |

## Hard conventions — every primitive must follow

These are non-negotiable. If a track violates them, the verifier will FAIL.

### 1. Component signature

```rust
use dioxus::prelude::*;

#[component]
pub fn Button(
    kind: Option<ButtonKind>,                 // enum variants come first
    size: Option<ButtonSize>,                 // then size/position
    href: Option<String>,                     // then behavior
    r#type: Option<String>,
    disabled: Option<bool>,
    block: Option<bool>,
    loading: Option<bool>,
    class_name: Option<String>,               // then class/identity overrides
    id: Option<String>,
    onclick: Option<EventHandler<MouseEvent>>,
    children: Element,                        // children last
) -> Element { ... }
```

- `Option<T>` with `unwrap_or` defaults — never required booleans.
- Enums (e.g. `ButtonKind`, `BadgeKind`, `CardKind`) are `Clone, Copy, Debug, PartialEq` with a `classes()` method that returns `&'static str`.
- `class_name` (snake_case) is the public prop name, **not** `class` (Dioxus reserves `class` for the built-in attribute). Inside rsx! the attribute is still `class:`.
- `r#type` (raw identifier) is acceptable for the HTML `type` attribute to avoid the `type` keyword.
- `children: Element` is always the last parameter.
- For components that emit form fields, also accept `name`, `label`, `help`, `error`, `required`.

### 2. State handling

- **Controlled** (caller owns state): component takes `value: Option<String>`, `onchange: Option<EventHandler<FormEvent>>`.
- **Uncontrolled** (caller wants the component to manage its own state): use `use_signal` internally, expose `default_value: Option<String>`.
- **Hybrid** (e.g. `Tabs`, `Accordion`): use `use_signal` for open/active state but always allow `active: String` + `on_select: EventHandler<String>` so the parent can stay in control.
- Never call `use_signal` outside a `#[component]` function — keep all hooks inside the component.

### 3. Accessibility (a11y) — every interactive primitive

| Component | Required a11y |
| --- | --- |
| `Button` (when `href` set) | `role="link"` implicit via `<a>` |
| `Modal` / `Sheet` | `role="dialog"`, `aria-modal="true"`, focus trap on open, Escape closes |
| `Dropdown` | `role="menu"`, items have `role="menuitem"` |
| `Tabs` | `role="tablist"`, each trigger `role="tab"`, `aria-selected`, focus roving |
| `Tooltip` | `role="tooltip"`, hidden until trigger is focus/hover |
| `Alert` | `role="alert"` (or `role="status"` for non-critical) |
| `AlertDialog` | `role="alertdialog"`, `aria-modal`, `aria-labelledby` |
| `Popover` | `role="dialog"`, `aria-expanded` on trigger |
| `Switch` | `role="switch"`, `aria-checked` |
| `Progress` | `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax` |
| `Slider` | `role="slider"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax` |
| `Toast` region | `role="region"`, `aria-live="polite"` or `"assertive"` |
| `Separator` | `role="separator"`, `aria-orientation` |
| `Combobox` | `role="combobox"`, `aria-expanded`, `aria-autocomplete="list"`, listbox `role="listbox"`, options `role="option"`, `aria-selected` |

When in doubt, mirror what the TS shadcn source does — it uses Radix under the hood, so the a11y contract is well-defined.

### 4. Styling

- All visual styling is provided by the **global CSS emitted by `epsx_templates::design_system_head`** — Tailwind v2.2.19 CDN + EPSX design tokens + glassmorphism utilities. The Dioxus components just emit markup with the right class names.
- Reuse the **existing class conventions** already in use across primitives:
  - `btn`, `btn-primary`, `btn-outline`, `btn-ghost`, `btn-danger`, `btn-link`, `btn-sm`, `btn-lg`, `btn-icon`, `btn-block`, `btn-loading`
  - `card`, `card-glass`, `card-insight`, `card-primary-solid`, `card-stats`, `card-header`, `card-body`, `card-footer`, `card-title`, `card-description`, `card-icon`, `card-badge`
  - `badge`, `badge-primary`, `badge-success`, `badge-warn`, `badge-danger`, `badge-info`, `badge-brand`, `badge-cool`, `badge-warm`, `badge-purple`, `badge-outline`
  - `input`, `input-error`, `input-with-icon`, `input-icon-wrap`, `input-icon`
  - `field`, `field-label`, `field-required`, `field-help`, `field-error`
  - `modal`, `modal-overlay`, `modal-header`, `modal-title`, `modal-description`, `modal-body`, `modal-close`, `modal-sm`, `modal-lg`, `modal-xl`, `modal-full`
  - `dropdown`, `dropdown-trigger`, `dropdown-menu`, `dropdown-item`, `dropdown-item-icon`, `dropdown-item-danger`, `dropdown-separator`
  - `tab`, `tab-active`, `tab-icon`, `tablist`
  - `tooltip-wrapper`, `tooltip-content`
  - `popover`, `popover-trigger`, `popover-content`
  - `popover-content`, `hover-card`, `hover-card-content`
  - `accordion`, `accordion-item`, `accordion-trigger`, `accordion-icon`, `accordion-content`
  - `collapsible`, `collapsible-trigger`, `collapsible-content`
  - `command-palette-overlay`, `command-palette`, `command-input`, `command-list`, `command-item`, `command-empty`, `command-hint`
  - `toast`, `toast-container`, `toast-success`, `toast-warn`, `toast-error`, `toast-info`
  - `progress`, `progress-bar`
  - `skeleton`, `skeleton-line`, `skeleton-text`
  - `avatar`, `avatar-sm`, `avatar-md`, `avatar-lg`, `avatar-fallback`, `avatar-group`, `avatar-extra`
  - `separator`, `separator-vertical`
  - `switch`, `switch-thumb`, `switch-label`
  - `slider`
  - `table`, `table-striped`, `table-hover`, `table-wrap`, `data-table`, `data-table-toolbar`, `data-table-row`, `pagination`, `pagination-info`
  - `alert`, `alert-title`, `alert-description`, `alert-success`, `alert-warn`, `alert-error`, `alert-info`
  - `sheet`, `sheet-overlay`, `sheet-content`, `sheet-left`, `sheet-right`, `sheet-top`, `sheet-bottom`
  - `scroll-area`, `scroll-area-viewport`, `scroll-area-scrollbar`, `scroll-area-thumb`
  - `kbd`
  - `rating`, `rating-star`, `rating-star.filled`
  - `stepper`, `step`, `step-active`, `step-complete`, `step-circle`, `step-line`, `step-label`, `step-panel`, `step-nav`
  - `form`, `form-actions`, `form-section`
- **Don't invent new class names** — if a needed utility doesn't exist, add a doc comment noting the gap and use the closest existing class. CSS additions go in a separate track (out of scope for Wave 1).
- Tailwind v2 utilities (e.g. `flex`, `gap-2`, `text-sm`, `text-muted-foreground`, `rounded`, `border`, `border-border`, `text-red-500`, `text-center`, `py-8`, `text-2xl`, `font-semibold`, `mt-1`, `ml-1`, `opacity-30`, `cursor-pointer`, `select-none`) are available via the global CDN and are fair game.

### 5. Icons

- Use `<Icon name="..." size={Some(16)} />` from `super::icon::Icon` (or `crate::primitives::icon::Icon`).
- Icon names must be **lucide** and exist in `epsx_templates::lucide`. The current set is in the icon helper's internal registry; common ones: `check`, `chevron-down`, `chevron-up`, `chevron-left`, `chevron-right`, `x`, `search`, `user`, `settings`, `home`, `log-out`, `menu`, `plus`, `minus`, `trash`, `edit`, `copy`, `eye`, `eye-off`, `alert-circle`, `alert-triangle`, `info`, `check-circle`, `loader`, `arrow-right`, `arrow-left`, `arrow-up`, `arrow-down`, `external-link`, `github`, `twitter`, `discord`, `wallet`, `shield`, `lock`, `mail`, `bell`, `calendar`, `clock`, `file`, `image`, `download`, `upload`, `globe`, `star`, `heart`, `message-circle`, `send`, `filter`, `refresh`, `more-vertical`, `more-horizontal`.
- If you need an icon that's not in the registry, **don't add a new helper** — note it in a `// TODO: add to lucide registry:` comment. Wave 1 only consumes existing icons.

### 6. Event handlers

- All event handlers use `EventHandler<T>` (Dioxus 0.7 API) and are **always optional**: `onclick: Option<EventHandler<MouseEvent>>`. Inside `rsx!`, wrap calls in `if let Some(h) = &onclick { h.call(e); }`.
- For keyboard handlers on `Combobox`/`Tabs`, prefer `onkeydown` with `e.key()` returning a `Key` enum.
- For input change handlers, `oninput` is preferred over `onchange` for live updates; `onchange` for commit semantics.

### 7. Public API stability

- **Don't break** the function signature of any existing primitive that is consumed by `pages/*.rs`. If you need to add a parameter, give it a default (`#[props(default = ...)]`).
- New components go in the appropriate file (e.g. `Alert` and `AlertDialog` go in `overlays.rs`; `Sheet` and `ScrollArea` and `Label` go in new `composites.rs` or expand existing files).
- New enums go in the same file as the component that uses them.
- Re-export everything from `primitives.rs` (`pub use overlay::alert::*;` etc.).

### 8. Pages don't migrate in Wave 1

Pages in `shared/rust/dioxus_ui/src/pages/*.rs` may already use primitive APIs that you're changing. **You may not edit pages** to match a signature change. Instead, your changes must be **additive**: new params have defaults, removed params never happen, renamed params keep the old name as a deprecated alias.

Exception: if a primitive rename is unavoidable, add a `#[deprecated]` re-export at the bottom of the primitive file and keep the old function as a thin wrapper that calls the new one. Document the rename in your `deliverable.md`.

## Track scope

### Track A — Form & Input primitives
Owner: `coder` #1

Files:
- `primitives/input.rs` — add missing input kinds (file, color, hidden), improve icon positioning, controlled/uncontrolled
- `primitives/form.rs` — add `InputGroup`, `FormSection`, `FormRow`, `RadioGroup`, controlled state
- `primitives/select.rs` — add `MultiSelect`, controlled state
- `primitives/combobox.rs` — add `ComboboxAsync` (lazy-loaded options), `ComboboxMulti`
- `primitives/date_picker.rs` — add `DateTimePicker`
- `primitives/stepper.rs` — improve `Stepper` with progress bar, `Step` icon support
- `primitives/checkbox.rs` — add `indeterminate` state
- `primitives/switch.rs` — add size variants
- Add new components to `primitives/misc.rs`: `Slider` (full a11y), `Rating` (interactive), `Kbd` (combo keys)

Read first:
- `shared/components/ui/form.tsx` (290 lines)
- `shared/components/ui/select.tsx` (152 lines)
- `shared/components/ui/textarea.tsx` (72 lines)
- `shared/components/ui/input.tsx` (53 lines)
- `shared/components/ui/checkbox.tsx` (50 lines)
- `shared/components/ui/switch.tsx` (60 lines)
- `shared/components/ui/dropdown-menu.tsx` (190 lines) — for `RadioGroup` pattern

### Track B — Display primitives
Owner: `coder` #2

Files:
- `primitives/button.rs` — add `as_child` slot, `left_icon`/`right_icon` (currently only `loading`/`disabled`/`href`)
- `primitives/card.rs` — add `CardLink`, `CardDivider`, image variants
- `primitives/badge.rs` — add `dot` indicator, `truncate` mode
- `primitives/avatar.rs` — add `status` (online/offline), `ring` (border)
- `primitives/table.rs` — add `TableEmpty`, `TableLoading`, `TableFooter`, `caption`
- `primitives/data_table.rs` — add column `render` callback (for badges/buttons inside cells), `selectable` rows, `bulk_actions`
- `primitives/charts.rs` — add `ChartArea` (filled line), `ChartStackedBar`, axis labels
- `primitives/stat_card.rs` — add `trend` direction (auto-color up/down), `sparkline` slot
- `primitives/progress.rs` — add `indeterminate` (animated), size variants, label
- `primitives/skeleton.rs` — add `SkeletonCircle`, `SkeletonBlock`
- `primitives/separator.rs` — add `label` slot (e.g. "or" divider)
- `primitives/icon.rs` — add `IconButton` wrapper

Read first:
- `shared/components/ui/card.tsx` (78 lines)
- `shared/components/ui/table.tsx` (134 lines)
- `shared/components/ui/avatar.tsx` (51 lines)
- `shared/components/ui/badge.tsx`
- `shared/components/ui/skeleton.tsx`
- `shared/components/ui/scroll-area.tsx` (48 lines)
- `shared/components/ui/progress.tsx`

### Track C — Interactive primitives
Owner: `coder` #3

Files:
- `primitives/dropdown.rs` — add `DropdownTrigger` slot, controlled `open`, `align`/`side` props, `DropdownLabel`, `DropdownCheckboxItem`
- `primitives/modal.rs` — add focus trap, Escape handler, `on_open_change`, `ModalFooter` slot
- `primitives/tabs.rs` — add `vertical` orientation, `on_change`, scroll buttons
- `primitives/tooltip.rs` — add `side`, `align`, `delay`, controlled state, only-visible-on-hover/focus
- `primitives/overlays.rs` — improve `Popover` (controlled, side/align), `HoverCard` (open delay), `Accordion` (controlled), `Collapsible` (controlled), `CommandPalette` (keyboard nav, async items)
- `primitives/rich_text.rs` — add link/heading/list buttons (currently broken — toolbar uses unsafe `format!("{}**bold**", ...)` which puts cursor at start; needs proper selection-aware insertion)

Read first:
- `shared/components/ui/dialog.tsx` (142 lines)
- `shared/components/ui/sheet.tsx` (142 lines)
- `shared/components/ui/popover.tsx`
- `shared/components/ui/dropdown-menu.tsx` (190 lines)
- `shared/components/ui/tooltip.tsx`
- `shared/components/ui/tabs.tsx` (81 lines)
- `shared/components/ui/alert-dialog.tsx` (160 lines)
- `shared/components/ui/toast.tsx` (125 lines) + `toaster.tsx`
- `shared/components/ui/form.tsx` (290 lines — also has Collapsible/Command inside)

### Track D — Missing TS parity primitives (NEW FILES)
Owner: `coder` #4 (or fold into Track A/B/C if we want 3 tracks; pick one per file)

New files:
- `primitives/overlays.rs` — add `Alert`, `AlertTitle`, `AlertDescription`, `AlertAction` (in `overlays.rs` or a new `primitives/alert.rs`)
- `primitives/overlays.rs` — add `AlertDialog`, `AlertDialogTrigger`, `AlertDialogContent`, `AlertDialogHeader`, `AlertDialogFooter`, `AlertDialogTitle`, `AlertDialogDescription`, `AlertDialogAction`, `AlertDialogCancel` (in `overlays.rs` or a new `primitives/alert_dialog.rs`)
- `primitives/overlays.rs` — add `Sheet`, `SheetTrigger`, `SheetContent`, `SheetHeader`, `SheetTitle`, `SheetDescription`, `SheetClose` (in `overlays.rs` or a new `primitives/sheet.rs`)
- `primitives/misc.rs` — add `ScrollArea`, `ScrollBar`
- `primitives/form.rs` — add `Label` (standalone, not just `field-label`)

Read first:
- `shared/components/ui/alert.tsx` (77 lines)
- `shared/components/ui/alert-dialog.tsx` (160 lines)
- `shared/components/ui/sheet.tsx` (142 lines)
- `shared/components/ui/scroll-area.tsx` (48 lines)
- `shared/components/ui/label.tsx` (~30 lines)

## Per-track deliverables

Each track produces:
1. **Code changes** in the files listed above.
2. **Re-exports** in `primitives.rs` for any new public items.
3. **A short `deliverable.md`** at the repo root (or track-specific path) listing:
   - Components added or enhanced
   - Any signatures changed (and the re-export alias path)
   - Any a11y gaps intentionally left for Wave 2 (with a one-line reason)
   - `cargo check --workspace` final result (paste the last 5 lines)

## Verifier gate

After all tracks report done, a single verifier run:
1. `cargo check --workspace` — must be green (warnings ok, errors not).
2. `cargo build --workspace --bins` — must succeed.
3. `cargo test -p epsx-dioxus-ui --lib` — if any `#[cfg(test)]` modules exist, they must pass.
4. `git diff migration/dioxus-microservices..HEAD --stat` — show file change summary.
5. Spot-check **3 random primitives** (verifier picks) — open the .rs file, confirm:
   - Follows the signature conventions above.
   - Has the required a11y attributes.
   - Uses existing class names (no invented class names).
6. Spot-check that **pages still compile** by running `cargo check -p epsx-frontend -p epsx-admin -p epsx-pay -p epsx-preview`.
7. Confirm no page files were edited (`git diff --stat HEAD -- shared/rust/dioxus_ui/src/pages/` should be empty).

## Commit & push

Per the user's push cadence ("per batch, after cargo green"):
- Each track commits locally as it goes (no need to wait for others).
- **The plan owner** (me) will fast-forward `migration/dioxus-microservices` to the combined worktree commit and push **after** all three tracks pass verification. Workers do not push to remote.

## What is out of scope for Wave 1

- Pages (`shared/rust/dioxus_ui/src/pages/*`) — not edited.
- Layout components (`shared/rust/dioxus_ui/src/layout/*`) — Wave 2.
- Auth components (`shared/rust/dioxus_ui/src/auth/*`) — Wave 3.
- CSS additions for new class names — separate track.
- BFF route changes — not needed for primitive parity.
- Removing deprecated shadcn re-exports in `apps-old/*` — that's TS cleanup, not porting.

## Reference: existing public API (don't break these)

```rust
// From primitives.rs re-exports
Button, ButtonKind, ButtonSize,
Card, CardKind, CardHeader, CardBody, CardFooter, CardTitle, CardDescription,
Badge, BadgeKind,
Input, InputKind,
StatCard,
Tabs, TabItem,
Skeleton, SkeletonGroup,
Icon,
Dropdown, DropdownItem, DropdownSeparator,
Modal,
Checkbox,
Switch,
Select, SelectOption,
Avatar,
Progress,
Separator,
Tooltip,
Table, TableRow, TableCell,
DataTable, Column, Row, SortDir, Align,
Form, Field, FormField, Textarea, SelectField, CheckboxField, FormActions,
ChartLine, ChartBar, ChartDonut, Series, DataPoint,
RichTextEditor,
Combobox,
DatePicker, DateRangePicker,
Stepper, StepPanel, StepNavigation,
Popover, HoverCard, Accordion, AccordionItem, Collapsible, CommandPalette, Command,
FileUpload, CopyButton, QrCode, AvatarGroup, Kbd, SkeletonText, ErrorBoundary_, LoadingState, Slider, Rating,
```

All of the above must remain importable from `epsx_dioxus_ui::*` after Wave 1.
