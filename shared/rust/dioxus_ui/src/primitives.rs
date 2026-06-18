//! UI primitives — 1:1 mirror of the Next.js shadcn/Radix components.
//!
//! All visual styling is provided by the global Tailwind v2 CDN + design
//! system CSS emitted by `epsx_templates::design_system_head`. These
//! components add Dioxus interactivity (state, events, refs) on top of the
//! already-styled markup.

use dioxus::prelude::*;

pub mod button;
pub mod card;
pub mod badge;
pub mod input;
pub mod stat_card;
pub mod tabs;
pub mod skeleton;
pub mod icon;
pub mod dropdown;
pub mod modal;
pub mod checkbox;
pub mod switch;
pub mod select;
pub mod avatar;
pub mod progress;
pub mod separator;
pub mod tooltip;
pub mod table;
pub mod data_table;
pub mod form;
pub mod charts;
pub mod rich_text;
pub mod combobox;
pub mod date_picker;
pub mod stepper;
pub mod overlays;
pub mod misc;
pub mod alert;
pub mod alert_dialog;
pub mod sheet;
pub mod admin_metric_card;
pub mod admin_table;
// === Wave 36 T2 ports ===
pub mod safe_theme_script;
pub mod permission_badge;
pub mod toaster;
pub mod dropdown_menu;
pub mod dialog;
pub mod collapsible;
pub mod loading_button;
pub mod input_with_icon;
pub mod breadcrumb;
pub mod pagination_nav;
// === Wave 36b T2 ports — shadcn-namespace + new primitives ===
pub mod label;
pub mod popover;
pub mod scroll_area;
pub mod textarea;
pub mod toast;
pub mod kbd_shortcut;
pub mod section;
pub mod code_block;
pub mod callout;
pub mod pill;
pub mod progress_circle;
pub mod timeline;
pub mod toggle;
pub mod stack;
pub mod empty_state_compact;
pub mod chip;
pub mod list;
pub mod layout_utils;
pub mod metric_pill;
pub mod tag_input;
pub mod skeleton_variants;
pub mod cards_v2;

pub use button::*;
pub use card::*;
pub use badge::*;
pub use input::*;
pub use stat_card::*;
pub use tabs::*;
pub use skeleton::*;
pub use icon::*;
pub use dropdown::*;
pub use modal::*;
pub use checkbox::*;
pub use switch::*;
pub use select::*;
pub use avatar::*;
pub use progress::*;
pub use separator::*;
pub use tooltip::*;
pub use table::*;
pub use data_table::*;
pub use form::*;
pub use charts::*;
pub use rich_text::*;
pub use combobox::*;
pub use date_picker::*;
pub use stepper::*;
pub use overlays::*;
pub use misc::*;
pub use alert::*;
pub use alert_dialog::*;
pub use sheet::*;
pub use admin_metric_card::*;
pub use admin_table::*;
pub use safe_theme_script::*;
pub use permission_badge::*;
pub use toaster::*;
pub use dropdown_menu::*;
pub use dialog::*;
pub use collapsible::*;
pub use loading_button::*;
pub use input_with_icon::*;
pub use breadcrumb::*;
pub use pagination_nav::*;
// === Wave 36b T2 — exported via primitives namespace path
// (no `pub use` to avoid collisions with form::Label, misc::ScrollArea,
// overlays::Popover, form::Textarea, feedback::toast::ToastProvider etc.)
// Use `crate::primitives::label::Label`, `crate::primitives::popover::Popover`,
// etc. for the new shadcn-namespace components. ===
