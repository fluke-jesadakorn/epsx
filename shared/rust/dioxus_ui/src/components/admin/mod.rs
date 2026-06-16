//! Admin-only shared components for the Wave 25 T3 admin page
//! porting work.
//!
//! Currently contains:
//! - `auth_page_overlay` — the fixed-position 60/40 split
//!   layout that mimics the prod
//!   `apps-old/admin-frontend/app/auth/page.tsx` "Admin Access"
//!   page. Paired with a `<SkeletonPage>` that the 4 T3 pages
//!   mount as their body when the overlay is active. See the
//!   per-component doc for the attempt-2 rationale.

pub mod auth_page_overlay;
