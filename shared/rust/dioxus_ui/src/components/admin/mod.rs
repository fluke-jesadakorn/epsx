//! Admin-only shared components — Wave 37 T1 admin primitives port
//! + Wave 38a T1 admin wallet domain port.
//!
//! This module is the admin-specific component namespace, mirroring
//! `apps-old/admin-frontend/components/ui/` (primitives layer) and
//! `apps-old/admin-frontend/components/wallet/` (wallet-domain
//! layer). It re-exports the existing generic primitives under
//! `admin::*` names so admin pages can do `use
//! crate::components::admin::*` and reach every admin-side primitive
//! without crossing module boundaries.
//!
//! ## Two-layer strategy
//!
//! 1. **Re-exports** for primitives that already exist under
//!    `primitives::*` (button, badge, card, input, etc.) — these
//!    are pure aliases and have no new test surface. Re-exports
//!    preserve the original behavior exactly.
//!
//! 2. **New admin-specific primitives** for components that didn't
//!    exist in Dioxus yet — `AuthPageOverlay` (Wave 25 T3),
//!    `PancakeButton`, `PancakeCard`, `AnalyticsStatsCard`, the
//!    wallet-domain components (`WalletStatusBadge`,
//!    `WalletLabelBadge`, `AdminWalletStatsBar`, etc.), etc.
//!    Each has a colocated `#[cfg(test)] mod tests` with at least
//!    2 tests (smoke render + key prop handling).
//!
//! ## File inventory
//!
//! | File | Source | Wave |
//! | --- | --- | --- |
//! | `auth_page_overlay` | `auth/page.tsx` overlay + skeleton | 25 T3 |
//! | `analytics_card` | `analytics-card.tsx` | 37 T1 (NEW) |
//! | `form_components` | `form-components.tsx` (re-exports) | 37 T1 (NEW) |
//! | `label` | `label.tsx` (re-export) | 37 T1 (NEW) |
//! | `loading_spinner` | `loading-spinner.tsx` | 37 T1 (NEW) |
//! | `pancake_button` | `pancake-button.tsx` | 37 T1 (NEW) |
//! | `pancake_card` | `pancake-card.tsx` | 37 T1 (NEW) |
//! | `textarea` | `textarea.tsx` (wp variant default) | 37 T1 (NEW) |
//! | `theme_toggle` | `theme-toggle.tsx` (re-exports) | 37 T1 (NEW) |
//! | `toast` | `toast.tsx` (re-exports) | 37 T1 (NEW) |
//! | `disable_wallet_modal` | `disable-wallet-modal.tsx` | 38a T1 (NEW) |
//! | `reenable_wallet_modal` | `reenable-wallet-modal.tsx` | 38a T1 (NEW) |
//! | `wallet_card` | `wallet-card.tsx` | 38a T1 (NEW) |
//! | `wallet_detail_header` | `wallet-detail-header.tsx` | 38a T1 (NEW) |
//! | `wallet_filter_bar` | `wallet-filter-bar.tsx` | 38a T1 (NEW) |
//! | `wallet_header` | `wallet-header.tsx` | 38a T1 (NEW) |
//! | `wallet_label_badge` | `wallet-label-badge.tsx` | 38a T1 (NEW) |
//! | `wallet_management_tabs` | `wallet-management-tabs.tsx` | 38a T1 (NEW) |
//! | `wallet_section` | `wallet-section.tsx` | 38a T1 (NEW) |
//! | `wallet_stats_bar` | `wallet-stats-bar.tsx` | 38a T1 (NEW) |
//! | `wallet_status_badge` | `wallet-status-badge.tsx` | 38a T1 (NEW) |
//! | `wallet_table` | `wallet-table.tsx` | 38a T1 (NEW) |
//! | `wallet_table_row` | `wallet-table-row.tsx` | 38a T1 (NEW) |
//!
//! The remaining 20+ primitives (button, badge, card, etc.) are
//! re-exported from `crate::primitives::*` at the bottom of this
//! file.

// =====================================================================
// Admin-specific primitives (NEW in Wave 37 T1)
// =====================================================================

pub mod analytics_card;
pub mod form_components;
pub mod label;
pub mod loading_spinner;
pub mod pancake_button;
pub mod pancake_card;
pub mod textarea;
pub mod theme_toggle;
pub mod toast;

// Pre-existing admin primitives
pub mod auth_page_overlay;

// =====================================================================
// Wave 38a T1 — admin wallet domain components
// =====================================================================
//
// 12 NEW components ported from
// `apps-old/admin-frontend/components/wallet/`. Naming notes:
// - `WalletStatsBar` and `WalletTableRow` had inline collisions
//   with file-local `fn`s in
//   `pages::admin_pages::wallet_wallets`, so the shared versions
//   are renamed with the `Admin` prefix to keep both
//   co-existent during migration.
// - All other components use their natural source names
//   (`WalletStatusBadge`, `WalletLabelBadge`, `WalletCard`, etc.).
pub mod disable_wallet_modal;
pub mod reenable_wallet_modal;
pub mod wallet_card;
pub mod wallet_detail_header;
pub mod wallet_filter_bar;
pub mod wallet_header;
pub mod wallet_label_badge;
pub mod wallet_management_tabs;
pub mod wallet_section;
pub mod wallet_stats_bar;
pub mod wallet_status_badge;
pub mod wallet_table;
pub mod wallet_table_row;

// Re-exports of the NEW admin-specific components for caller
// convenience: `use crate::components::admin::PancakeButton` works
// without specifying the module.
pub use analytics_card::{
    AnalyticsIcon, AnalyticsIconName, AnalyticsStatsCard, AnalyticsStatusColor,
    AnalyticsSummaryCard, AnalyticsTrend, AnalyticsUserCard,
};
pub use form_components::FormFieldWrapper;
pub use label::Label;
pub use loading_spinner::{
    ButtonLoadingSpinner, InlineLoading, LoadingSpinner, PageLoadingSpinner,
    SectionLoading,
};
pub use pancake_button::{PancakeButton, PancakeFAB, PancakeIconButton};
pub use pancake_card::{PancakeCard, PancakeFeatureCard, PancakeStatsCard};
pub use textarea::Textarea;
pub use theme_toggle::{
    AdminThemeToggle, AnimatedThemeToggle, GradientThemeToggle, MinimalThemeToggle,
    OptimizedThemeToggle, SimpleThemeToggle, ThemeToggle, ThemeToggleCSS,
};
pub use toast::{Toast, ToastDescription, ToastTitle, ToastViewport};

// Wave 38a T1 — admin wallet domain re-exports.
pub use disable_wallet_modal::{
    DisableDuration, DisablePlatform, DisableReasonCategory, DisableWalletData,
    DisableWalletModal,
};
pub use reenable_wallet_modal::{ReenableDisableInfo, ReenableWalletData, ReenableWalletModal};
pub use wallet_card::{WalletCard, WalletCardData};
pub use wallet_detail_header::WalletDetailHeader;
pub use wallet_filter_bar::{WalletFilterBar, WalletFilters};
pub use wallet_header::{WalletHeader, WalletHeaderData};
pub use wallet_label_badge::{WalletLabelBadge, WalletLabelSize};
pub use wallet_management_tabs::WalletManagementTabs;
pub use wallet_section::WalletSection;
pub use wallet_stats_bar::{
    AdminWalletStatsBar, PlatformDistributionPanel, WalletPlatformDistribution,
    WalletStatsChanges, WalletStatsData,
};
pub use wallet_status_badge::{WalletStatusBadge, WalletStatusKind};
pub use wallet_table::WalletTable;
pub use wallet_table_row::{AdminWalletTableRow, WalletRowData, WalletRowStatus};

// =====================================================================
// Re-exports of existing primitives (admin-namespaced aliases)
// =====================================================================
//
// These aliases give admin pages a single import surface:
// `use crate::components::admin::*` reaches every admin-side
// primitive (the 9 NEW ones above + the 22 below).
//
// No new behavior — these are pure `pub use` lines. They do NOT
// add to the test count because they're tested under their original
// `primitives::*` paths.

pub use crate::primitives::admin_metric_card::{AdminMetricCard, MetricTrend};
pub use crate::primitives::admin_table::AdminTable;
pub use crate::primitives::alert::{Alert, AlertAction, AlertDescription, AlertKind, AlertTitle};
pub use crate::primitives::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle,
    AlertDialogTrigger,
};
pub use crate::primitives::avatar::Avatar;
pub use crate::primitives::badge::{Badge, BadgeKind};
pub use crate::primitives::breadcrumb::BreadcrumbNav;
pub use crate::primitives::button::{Button, ButtonKind, ButtonSize};
pub use crate::primitives::card::{
    Card, CardBody, CardDescription, CardDivider, CardFooter, CardHeader, CardKind,
    CardLink, CardTitle,
};
pub use crate::primitives::charts::{ChartArea, ChartBar, ChartDonut, ChartLine, ChartStackedBar};
pub use crate::primitives::checkbox::Checkbox;
pub use crate::primitives::data_table::{Align, Column, DataTable, Row, SortDir};
pub use crate::primitives::dialog::{
    DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    DialogTrigger,
};
pub use crate::primitives::dropdown::{
    Dropdown, DropdownCheckboxItem, DropdownItem, DropdownLabel, DropdownSeparator,
};
pub use crate::primitives::dropdown_menu::{Content as DropdownMenuContent, DropdownMenuTrigger, Group as DropdownMenuGroup};
pub use crate::primitives::form::{
    Field, Form, FormActions, FormField, FormRow, FormSection, InputGroup, RadioGroup,
    SelectField,
};
pub use crate::primitives::icon::{Icon, IconButton};
pub use crate::primitives::input::{Input, InputKind};
pub use crate::primitives::loading_button::LoadingButton;
pub use crate::primitives::modal::{Modal, ModalBody, ModalFooter, ModalHeader};
pub use crate::primitives::pagination_nav::{PaginationKind, PaginationNav};
pub use crate::primitives::permission_badge::{
    PermissionBadge, PermissionBadgeProps, PermissionBadgeSize,
};
pub use crate::primitives::progress::Progress;
pub use crate::primitives::safe_theme_script::{
    SafeThemeScript, SafeThemeScriptWithNonce, theme_utils,
};
pub use crate::primitives::select::{MultiSelect, Select, SelectOption};
pub use crate::primitives::separator::Separator;
pub use crate::primitives::sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader,
    SheetSide, SheetTitle, SheetTrigger,
};
pub use crate::primitives::skeleton::{Skeleton, SkeletonBlock, SkeletonCircle, SkeletonGroup};
pub use crate::primitives::stat_card::StatCard;
pub use crate::primitives::switch::{Switch, SwitchSize};
pub use crate::primitives::table::{
    Table, TableCell, TableEmpty, TableFooter, TableLoading, TableRow,
};
pub use crate::primitives::tabs::{TabItem, Tabs};
pub use crate::primitives::toaster::{Toaster, ToasterPosition};
pub use crate::primitives::tooltip::Tooltip;
