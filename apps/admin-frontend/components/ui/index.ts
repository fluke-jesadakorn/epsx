/**
 * ADMIN FRONTEND UI COMPONENTS INDEX
 * Re-exports from shared components where possible
 * Keeps admin-specific components local
 */

// ============================================================================
// BASIC PRIMITIVES - Re-export from shared
// ============================================================================
export * from '@/shared/components/ui/alert'
export * from '@/shared/components/ui/alert-dialog'
export * from '@/shared/components/ui/avatar'
export * from '@/shared/components/ui/badge'
export * from '@/shared/components/ui/button'
export * from '@/shared/components/ui/card'
export * from '@/shared/components/ui/dialog'
export * from '@/shared/components/ui/dropdown-menu'
export * from '@/shared/components/ui/input'
export * from '@/shared/components/ui/popover'
export * from '@/shared/components/ui/progress'
export * from '@/shared/components/ui/scroll-area'
export * from '@/shared/components/ui/skeleton'
export * from '@/shared/components/ui/table'
export * from '@/shared/components/ui/tabs'
export * from '@/shared/components/ui/toast'
export * from '@/shared/components/ui/toaster'
export * from '@/shared/components/ui/tooltip'

// ============================================================================
// UNIFIED COMPONENTS - Re-export from shared
// ============================================================================
export {
    AdminCard, EPSXCard, EPSXCardContent,
    EPSXCardFooter, EPSXCardHeader,
    // Legacy aliases
    MetroCard, MetroListCard, MetroStatsCard, PremiumCard, ProfessionalCard, ProfessionalFeatureCard, ProfessionalListCard, ProfessionalStatsCard, AnalyticsCard as UnifiedAnalyticsCard,
    // Main UnifiedCard
    UnifiedCard, UnifiedCardContent,
    UnifiedCardFooter, UnifiedCardHeader, UnifiedFeatureCard, UnifiedListCard,
    // Specialized variants
    PancakeCard as UnifiedPancakeCard,
    // Specialized cards
    UnifiedStatsCard, type AccentPosition, type UnifiedCardPadding,
    // Types
    type UnifiedCardProps,
    type UnifiedCardSectionProps, type UnifiedCardSize, type UnifiedCardVariant, type UnifiedFeatureCardProps, type UnifiedListCardProps,
    type UnifiedListItem, type UnifiedStatsCardProps
} from '@/shared/components/cards/CardVariants'

export {
    UnifiedLoader, UnifiedLoading, UnifiedProgressBar, UnifiedSkeleton,
    type UnifiedLoaderProps, type UnifiedLoadingProps, type UnifiedProgressBarProps, type UnifiedSkeletonProps
} from '@/shared/components/loaders/UnifiedLoader'

export {
    AdminThemeToggle, AnimatedThemeToggle, GradientThemeToggle,
    MinimalThemeToggle, OptimizedThemeToggle, SimpleThemeToggle, ThemeToggle,
    ThemeToggleCSS, UnifiedThemeToggle, type ThemeToggleIconType,
    type ThemeToggleSize, type ThemeToggleVariant, type UnifiedThemeToggleProps
} from '@/shared/components/ui/UnifiedThemeToggle'

export {
    MetroNotification, ProfessionalAlert, ProfessionalNotification, UnifiedAlert, UnifiedNotification, useAdminToast, useAnalyticsToast, useMetroToast, usePancakeToast, useProfessionalToast, useUnifiedToast, type UnifiedAlertProps, type UnifiedNotificationProps
} from '@/shared/components/notifications/UnifiedNotification'

// ============================================================================
// ADMIN-SPECIFIC LOCAL COMPONENTS
// ============================================================================
// Note: FormComponents uses shared components, but we keep local separator
export { Label } from '@/shared/components/ui/label'
export { Textarea } from '@/shared/components/ui/textarea'
export {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator,
    SelectTrigger,
    SelectValue
} from '@/shared/components/ui/select'
export { Switch } from '@/shared/components/ui/switch'
export { Separator } from './separator'

// Admin-specific presentational components
export * from './AnalyticsCard'
export * from './PancakeButton'
export * from './PancakeCard'
export * from './StatsCard'

