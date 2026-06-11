//! Shared sub-components extracted from `pages/admin_pages/<page>.rs`
//! (Wave 6C 1:1 component parity) and user-side pages (Track E).
//!
//! Each admin page in `pages/admin_pages/` now composes sub-components
//! from `components::admin::<page>` instead of inlining the same JSX.
//! The sub-components mirror the Next.js source's
//! `apps-old/admin-frontend/components/<group>/*.tsx` tree 1:1.

pub mod admin;
