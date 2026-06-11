//! Admin sub-component clusters — 1:1 mirrors of
//! `apps-old/admin-frontend/components/admin/*` and
//! `apps-old/admin-frontend/components/{analytics,policies,access-control}/*`.
//!
//! Each submodule is one source domain. Tracks B/C/D in the Wave 6C
//! plan extract per-page sub-components from the Wave 6B admin pages
//! into the matching cluster here:
//!
//! - `dashboard` — `AdminStatsCards`, `WalletsByChain`,
//!   `RecentTransactions`, `SystemAlerts`, `ActivityStream` +
//!   `AdminPulseHeader` (Wave 6B dashboard's pulse header).
//! - `analytics` — `AnalyticsHeader`, `AnalyticsCardGrid`,
//!   `AnalyticsChart`, `AnalyticsTable`, `AnalyticsFilterPanel`,
//!   `AnalyticsExportDialog`, `AnalyticsMetadata`.
//! - `policies` — `PolicyStatsBar`, `PolicyBuilder`, `PolicyMonitor`,
//!   `PolicyCard`, `PolicyFilters`, `PolicyList`.
//!
//! Future tracks (C, D) add `payments`, `wallet`, `settings`,
//! `media`, `wallets`, `chat`, `developer`, `audit`, `auth`, `news`,
//! `notifications` per the design doc's file-ownership table.

pub mod dashboard;
pub mod analytics;
pub mod policies;
