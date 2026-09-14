//! UI primitives — 1:1 mirror of the Next.js shadcn/Radix components.
//!
//! All visual styling is provided by the global Tailwind v2 CDN + design
//! system CSS emitted by `epsx_templates::design_system_head`. These
//! components add Dioxus interactivity (state, events, refs) on top of the
//! already-styled markup.

pub mod admin_metric_card;
pub mod admin_table;
pub mod alert;
pub mod alert_dialog;
pub mod avatar;
pub mod badge;
pub mod button;
pub mod card;
pub mod charts;
pub mod checkbox;
pub mod combobox;
pub mod data_table;
pub mod date_picker;
pub mod dropdown;
pub mod form;
pub mod icon;
pub mod input;
pub mod misc;
pub mod modal;
pub mod overlays;
pub mod progress;
pub mod rich_text;
pub mod select;
pub mod separator;
pub mod sheet;
pub mod skeleton;
pub mod stat_card;
pub mod stepper;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod tooltip;
// === Wave 36 T2 ports ===
pub mod breadcrumb;
pub mod collapsible;
pub mod dialog;
pub mod dropdown_menu;
pub mod input_with_icon;
pub mod loading_button;
pub mod pagination_nav;
pub mod permission_badge;
pub mod toaster;
// === Wave 36b T2 ports — shadcn-namespace + new primitives ===
pub mod callout;
pub mod cards_v2;
pub mod chip;
pub mod code_block;
pub mod empty_state_compact;
pub mod kbd_shortcut;
pub mod label;
pub mod layout_utils;
pub mod list;
pub mod metric_pill;
pub mod pill;
pub mod popover;
pub mod progress_circle;
pub mod scroll_area;
pub mod section;
pub mod skeleton_variants;
pub mod stack;
pub mod tag_input;
pub mod textarea;
pub mod timeline;
pub mod toast;
pub mod toggle;

pub use admin_metric_card::*;
pub use admin_table::*;
pub use alert::*;
pub use alert_dialog::*;
pub use avatar::*;
pub use badge::*;
pub use breadcrumb::*;
pub use button::*;
pub use card::*;
pub use charts::*;
pub use checkbox::*;
pub use collapsible::*;
pub use combobox::*;
pub use data_table::*;
pub use date_picker::*;
pub use dialog::*;
pub use dropdown::*;
pub use dropdown_menu::*;
pub use form::*;
pub use icon::*;
pub use input::*;
pub use input_with_icon::*;
pub use loading_button::*;
pub use misc::*;
pub use modal::*;
pub use overlays::*;
pub use pagination_nav::*;
pub use permission_badge::*;
pub use progress::*;
pub use rich_text::*;
pub use select::*;
pub use separator::*;
pub use sheet::*;
pub use skeleton::*;
pub use stat_card::*;
pub use stepper::*;
pub use switch::*;
pub use table::*;
pub use tabs::*;
pub use toaster::*;
pub use tooltip::*;
// === Wave 36b T2 — exported via primitives namespace path
// (no `pub use` to avoid collisions with form::Label, misc::ScrollArea,
// overlays::Popover, form::Textarea, feedback::toast::ToastProvider etc.)
// Use `crate::primitives::label::Label`, `crate::primitives::popover::Popover`,
// etc. for the new shadcn-namespace components. ===
