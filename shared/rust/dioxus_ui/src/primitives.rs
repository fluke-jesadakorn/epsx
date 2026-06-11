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
pub mod admin_table;

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
pub use admin_table::*;
