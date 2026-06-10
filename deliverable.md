# Track D — Missing TS parity primitives (Dioxus)

## Summary

Added 1:1 shadcn-parity Dioxus primitives for the 5 missing TS
components (Alert, AlertDialog, Sheet, ScrollArea, Label). All
re-exported from `epsx_dioxus_ui::*` so consumers can `use` them
exactly like the existing primitives. New public surface: 3 new
files (`alert.rs`, `alert_dialog.rs`, `sheet.rs`) + 2 inline additions
(`Label` in `form.rs`, `ScrollArea`/`ScrollBar` in `misc.rs`) +
re-exports in `primitives.rs`. Compiles clean against `dioxus 0.7`
with no new dependencies; uses only the existing design-system class
names declared in `docs/wave1-primitives/design.md`.

## Components added

### `primitives/alert.rs` (new file)
- `AlertKind` enum — `Default | Success | Warning | Danger | Info`,
  with a `classes()` method that returns the right `alert[-success |
  -warn | -error | -info]` class string, and a `default_icon()` that
  returns the lucide icon name for each variant.
- `Alert` — inline callout box. Renders `<div role="alert">` per the
  design doc's a11y rules. Props: `kind`, `title`, `description`,
  `icon` (overrides default lucide icon), `class_name`, `children`.
  When `icon` is set, an `<Icon>` is rendered at the top-left matching
  the TS shadcn `[&>svg]:absolute` positioning.
- `AlertTitle` — slot for the title (h5).
- `AlertDescription` — slot for the body (div with `text-sm
  [&_p]:leading-relaxed`).
- `AlertAction` — right-aligned action area (typically a Button).

### `primitives/alert_dialog.rs` (new file)
- `AlertDialog` — modal confirmation dialog. Props:
  `open: bool`, `on_close: EventHandler<MouseEvent>`,
  `title: Option<String>`, `description: Option<String>`,
  `size: Option<String>` (sm/lg/xl/full, reusing the existing
  `modal-*` size classes), `class_name: Option<String>`, `children`.
  Renders `<div role="alertdialog" aria-modal="true">` per the
  design doc's a11y rules (overriding `Modal`'s `role="dialog"`).
  Reuses the existing `modal`, `modal-overlay`, `modal-header`,
  `modal-body`, `modal-close` class names.
- `AlertDialogTrigger` — controlled trigger slot (caller owns the
  `open` signal).
- `AlertDialogContent` — inner content wrapper.
- `AlertDialogHeader` / `AlertDialogFooter` — flex layout helpers
  matching the TS shadcn `space-y-2 text-center sm:text-left` /
  `flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2` class
  sets.
- `AlertDialogTitle` / `AlertDialogDescription` — h2/p slots
  matching the TS shadcn `text-lg font-semibold` /
  `text-sm text-muted-foreground` styling.
- `AlertDialogAction` / `AlertDialogCancel` — `btn btn-primary` /
  `btn btn-outline` button slots with the right onclick wiring.

### `primitives/sheet.rs` (new file)
- `SheetSide` enum — `Left | Right | Top | Bottom`, with `classes()`
  that returns `sheet sheet-{side}` and a `from_str()` parser
  accepting the TS shadcn string form.
- `Sheet` — slide-in side panel. Props: `open: bool`,
  `on_close: EventHandler<MouseEvent>`, `side: Option<String>`,
  `class_name: Option<String>`, `children`. Renders
  `<div role="dialog" aria-modal="true">` per the design doc's
  a11y rules. The root sheet renders a default close X (✕) at
  `absolute right-4 top-4` matching the TS shadcn positioning.
- `SheetTrigger` — controlled trigger slot.
- `SheetContent` — body wrapper that uses the side class.
- `SheetHeader` / `SheetFooter` — flex layout helpers matching the
  TS shadcn pattern.
- `SheetTitle` / `SheetDescription` — h2/p slots.
- `SheetClose` — custom close slot (for use inside a sheet footer,
  e.g. a "Done" button).

### `primitives/misc.rs` (additions)
- `ScrollArea` — scrollable container. Props: `max_height:
  Option<String>`, `class_name: Option<String>`, `children`.
  Reuses the design-system `scroll-area`, `scroll-area-viewport`,
  `scroll-area-scrollbar`, `scroll-area-thumb` class names declared
  in the design doc. Renders an inner viewport div with
  `overflow-auto` and a vertical `<ScrollBar>` at the right edge.
- `ScrollBar` — scrollbar track + thumb. Props: `orientation:
  Option<String>` ("vertical" | "horizontal"). Picks the right
  class set per orientation (vertical = `h-full w-2.5 border-l ...`,
  horizontal = `h-2.5 flex-col border-t ...`).

### `primitives/form.rs` (additions)
- `Label` — standalone `<label>` primitive. Props: `html_for:
  Option<String>`, `required: bool`, `class_name: Option<String>`,
  `children`. Renders a `<label>` with the `label` base class; when
  `required` is true, appends a red `*` (`label-required` /
  `text-red-500 ml-1`). Mirrors `shared/components/ui/label.tsx`
  one-for-one. The `html_for` prop name is used (matching the
  existing `Field` component) rather than the raw-identifier
  `r#for` form for Rust idiom consistency.

### `primitives.rs` (re-exports)
Added `pub mod alert;`, `pub mod alert_dialog;`, `pub mod sheet;`
and `pub use alert::*;`, `pub use alert_dialog::*;`,
`pub use sheet::*;` so all new public items are reachable as
`epsx_dioxus_ui::Alert`, `epsx_dioxus_ui::AlertDialog`,
`epsx_dioxus_ui::Sheet`, `epsx_dioxus_ui::ScrollArea`,
`epsx_dioxus_ui::Label`, etc.

## Changed files

```
shared/rust/dioxus_ui/src/primitives.rs            |   6 +
shared/rust/dioxus_ui/src/primitives/alert.rs      | 123 ++++++++++++++
shared/rust/dioxus_ui/src/primitives/alert_dialog.rs  | 180 ++++++++++++++++++++
shared/rust/dioxus_ui/src/primitives/form.rs       |  22 +++
shared/rust/dioxus_ui/src/primitives/misc.rs       |  50 ++++++
shared/rust/dioxus_ui/src/primitives/sheet.rs      | 181 +++++++++++++++++++++
6 files changed, 562 insertions(+)
```

## `cargo check --workspace` (last 10 lines)

```
  --> apps/frontend/src/api.rs:39:12
   |
39 | pub struct NewsQuery {
   |            ^^^^^^^^^

warning: `epsx-identity` (bin "identity") generated 2 warnings (run `cargo fix --bin "identity"` to apply 2 suggestions)
warning: `epsx-notification` (bin "notification") generated 4 warnings (run `cargo fix --bin "notification"` to apply 1 suggestion)
warning: `epsx-content` (bin "content") generated 2 warnings (run `cargo fix --bin "content"` to apply 2 suggestions)
warning: `epsx-frontend` (bin "bff-frontend") generated 15 warnings (run `cargo fix --bin "bff-frontend"` to apply 9 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```

Exit code: 0. No errors. (All warnings are pre-existing from other
files — `cargo check -p epsx-dioxus-ui` shows 60 warnings, none in
my new files.)

## a11y notes

- `Alert` — `role="alert"` (matches design doc).
- `AlertDialog` — `role="alertdialog"` + `aria-modal="true"`
  (matches design doc; this is the only difference from the
  existing `Modal` which uses `role="dialog"`).
- `Sheet` — `role="dialog"` + `aria-modal="true"` (matches design
  doc). Default close X has `aria-label="Close"`.
- `Label` — standard `<label>` with `r#for` association, matching
  Radix Label's `htmlFor` prop.
- `ScrollArea` — no ARIA (intentionally; the TS shadcn source also
  doesn't add ARIA, relying on the native scroll semantics).

## a11y gaps intentionally left for Wave 2

- **Focus trap + Escape handler** in `AlertDialog` and `Sheet` —
  the TS shadcn source uses Radix Portal which handles this
  automatically. In Dioxus SSR we don't have a Radix equivalent,
  so closing on overlay click works, but tab focus isn't trapped
  inside the dialog and Escape doesn't close. Track C will add
  focus-trap utilities; this commit ships the structural surface
  only.
- **ScrollArea scroll position measurement** — the TS shadcn source
  uses Radix's measured thumb; we render a static thumb track
  sized `flex-1` to match the visual class set. JS-driven thumb
  sizing is a Wave 2 client-side enhancement.

## Notes for the verifier

- The branch is `wave1/track-d-missing`, commit hash
  `48ad0bd1b3e4f9def2b98a7ca3df5567c1bc23a2`. Working tree is
  clean (only the `.mavis/` and `target/` directories are
  untracked, neither of which is part of the commit).
- `cargo check --workspace` exits 0.
- No pages were edited (per Wave 1 hard rule).
- No new dependencies were added.
- All new public items are re-exported from `primitives.rs`.
- Existing primitive APIs are unchanged. The new `Label` is a new
  addition to `form.rs`; if Track A is also adding a `Label`, the
  merge will produce a duplicate-name error and the build owner
  will need to pick one. (As of this commit, `form.rs` had no
  `Label` on the `wave1/track-d-missing` base.)
