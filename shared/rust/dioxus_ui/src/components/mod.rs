//! User-side and admin-side sub-components extracted from the page
//! files during Wave 6C (1:1 component parity).
//!
//! Each track (B/C/D for admin, E for user) added a sub-module under
//! `components::` and a `pub mod` line in this file. The integration
//! gate then re-exported the whole tree from `lib.rs`.
//!
//! Module ownership:
//! - `components::user`         — Track E
//! - `components::admin`        — Tracks B + C + D (admin sub-components)
//! - `components::mod.rs` itself — all tracks share; the integration
//!   gate resolves any conflicts by concatenating `pub mod` lines
//!   in track order.

pub mod user;
