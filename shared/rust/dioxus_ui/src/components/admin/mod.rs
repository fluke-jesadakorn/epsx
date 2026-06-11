//! Admin sub-components — 1:1 extraction of named `#[component]`s from
//! the Next.js source (`apps-old/admin-frontend/components/{group}/*.tsx`).
//!
//! Each `pub mod` here is owned by exactly one wave-6C track:
//!   - `dashboard`, `analytics`, `policies` → Track B
//!   - `payments`, `wallet`, `settings`, `media` → Track C
//!   - `wallets`, `chat`, `developer`, `audit`, `auth`, `news`,
//!     `notifications` → Track D
//!
//! Tracks only add their own `pub mod` lines. The integration agent
//! concatenates them at merge time (the `// === wave6c-1to1-track-* ===`
//! marker convention from prior waves).

// Track C — Wave 6C admin sub-components for the financial-surface
// pages (payments, wallet) + the small settings/media pages. Each
// track adds its own pub mod lines inside its own `// === wave6c-...`
// marker region.

pub mod payments;
pub mod wallet;
pub mod settings;
pub mod media;
