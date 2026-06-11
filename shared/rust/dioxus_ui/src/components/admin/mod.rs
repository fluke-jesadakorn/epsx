//! Admin sub-components — Wave 6C 1:1 component parity.
//!
//! Each module here mirrors a Next.js source's sub-component tree.
//! The parent page (`pages/admin_pages/<page>.rs`) composes these
//! instead of inlining the same JSX. The 7 modules here cover
//! the Wave 6C Track D scope:
//!   - `wallets`        — wallet_wallets page
//!   - `chat`           — chat page
//!   - `developer`      — developer_portal page
//!   - `audit`          — audit_log page
//!   - `auth`           — auth_page
//!   - `news`           — news page
//!   - `notifications`  — notifications page
//!
//! Tracks B and C add their own modules to this file. The
//! integration agent concatenates the `pub mod` lines on merge.

pub mod wallets;
pub mod chat;
pub mod developer;
pub mod audit;
pub mod auth;
pub mod news;
pub mod notifications;
