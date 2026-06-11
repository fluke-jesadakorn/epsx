//! Sub-components extracted from the user-side pages (Wave 6A → 6C).
//!
//! The page files in `crate::pages::*` keep their public `pub fn render`
//! entry point and `pub use` the sub-components from this module. The
//! extraction makes the source → port diff a 1:1 walk through named
//! components (per `docs/wave6c-live-render-and-1to1/design.md`).
//!
//! Conflict-avoidance with Tracks B/C/D (admin sub-components):
//! - This file is owned exclusively by Track E.
//! - Tracks B/C/D will add `pub mod admin;` to a sibling `admin/mod.rs`
//!   during their own work, and the integration gate will concat the
//!   `pub mod` lines into this file.
//! - The integration gate also re-exports `components::*` from
//!   `lib.rs` (out of scope for Track E).

// === wave6c-1to1-track-e user sub-modules ===
//
// One `pub mod` per user page that has local `#[component]`s to lift.
// Pages with no extractable sub-components (about, contact, access_denied
// have only section blocks, not named sub-components) are still reachable
// as their own modules so the integration agent can see the surface.
//
// The integration gate adds `pub mod admin;` here for Tracks B/C/D.

pub mod access_denied;
pub mod news_detail;
pub mod about;
pub mod news;
pub mod contact;
// end wave6c-1to1-track-e
