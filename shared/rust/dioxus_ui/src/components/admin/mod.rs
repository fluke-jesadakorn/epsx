//! Admin-only shared components — Wave 37 T1 admin primitives port.
//!
//! This module is the admin-specific component namespace, mirroring
//! `apps-old/admin-frontend/components/ui/`. It re-exports the
//! existing generic primitives under `admin::*` names so admin pages
//! can do `use crate::components::admin::*` and reach every
//! admin-side primitive without crossing module boundaries.
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
//!    `PancakeButton`, `PancakeCard`, `AnalyticsStatsCard`, etc.
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
