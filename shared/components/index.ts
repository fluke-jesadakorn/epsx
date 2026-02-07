/**
 * SHARED COMPONENTS INDEX
 * Unified export of all consolidated components
 */

// ============================================================================
// CORE COMPONENTS
// ============================================================================

// Cards - Import for local use and re-export
import {
  BaseCard,
  DataCard,
  StatusCard,
  type BaseCardProps,
  type DataCardProps,
  type StatusCardProps
} from './cards/BaseCard'

// Buttons - Import for local use and re-export
import {
  ActionButton,
  BaseButton,
  IconButton,
  LoadingButton,
  PaginationButton,
  type ActionButtonProps,
  type BaseButtonProps,
  type IconButtonProps,
  type LoadingButtonProps,
  type PaginationButtonProps
} from './buttons/BaseButton'

// Modals - Import for local use and re-export
import {
  BaseModal,
  ConfirmModal,
  FormModal,
  type BaseModalProps,
  type ConfirmModalProps,
  type FormModalProps
} from './modals/BaseModal'

// ============================================================================
// FORM COMPONENTS
// ============================================================================

// Form System - Import for local use and re-export
import {
  BaseForm,
  FormControl,
  FormDescription,
  FormField,
  FormFieldWrapper,
  FormItem,
  FormLabel,
  FormMessage,
  useFormField,
  type BaseFormProps,
  type FormControlProps,
  type FormDescriptionProps,
  type FormFieldWrapperProps,
  type FormItemProps,
  type FormLabelProps,
  type FormMessageProps
} from './forms/BaseForm'

// Input Components - Import for local use and re-export
import {
  BaseCheckbox,
  BaseInput,
  BaseRadio,
  BaseSelect,
  BaseTextarea,
  type BaseCheckboxProps,
  type BaseInputProps,
  type BaseRadioProps,
  type BaseSelectProps,
  type BaseTextareaProps
} from './forms/BaseInput'

export {
  BaseCard,
  DataCard,
  StatusCard,
  type BaseCardProps,
  type DataCardProps,
  type StatusCardProps
}

// Themed Cards
export {
  AnalyticsCard as PancakeAnalyticsCard, Card as PancakeCard, CardContent as PancakeCardContent, CardDescription as PancakeCardDescription, CardFooter as PancakeCardFooter, CardHeader as PancakeCardHeader,
  CardTitle as PancakeCardTitle, StatsCard as PancakeStatsCard, type AnalyticsCardProps as PancakeAnalyticsCardProps, type PancakeCardProps,
  type StatsCardProps as PancakeStatsCardProps
} from './cards/PancakeCard'

// Stock Data Card
export {
  StockDataCard,
  StockDataCardSkeleton,
  type StockDataCardProps
} from './cards/StockDataCard'

// Unified Card Variants (Frontend-style cards)
export {
  AdminCard,
  AnalyticsCard, EPSXCard, EPSXCardContent,
  EPSXCardFooter, EPSXCardHeader,
  // Legacy aliases
  MetroCard, MetroListCard, MetroStatsCard, PremiumCard, ProfessionalCard, ProfessionalFeatureCard, ProfessionalListCard, ProfessionalStatsCard, UnifiedCard, UnifiedCardContent,
  UnifiedCardFooter, UnifiedCardHeader, UnifiedFeatureCard, UnifiedListCard, PancakeCard as UnifiedPancakeCard, UnifiedStatsCard, type AccentPosition, type UnifiedCardPadding,
  // Types
  type UnifiedCardProps,
  type UnifiedCardSectionProps, type UnifiedCardSize, type UnifiedCardVariant, type UnifiedFeatureCardProps, type UnifiedListCardProps,
  type UnifiedListItem, type UnifiedStatsCardProps
} from './cards/CardVariants'

export {
  ActionButton, BaseButton,
  IconButton,
  LoadingButton, PaginationButton, type ActionButtonProps, type BaseButtonProps,
  type IconButtonProps,
  type LoadingButtonProps, type PaginationButtonProps
}

// Themed Buttons
export {
  PancakeButton,
  type PancakeButtonProps
} from './buttons/PancakeButton'

export {
  BaseModal,
  ConfirmModal,
  FormModal,
  type BaseModalProps,
  type ConfirmModalProps,
  type FormModalProps
}

export {
  BaseForm as Form, FormControl,
  FormDescription, FormField, FormFieldWrapper, FormItem,
  FormLabel, FormMessage, useFormField,
  type BaseFormProps, type FormControlProps,
  type FormDescriptionProps, type FormFieldWrapperProps, type FormItemProps,
  type FormLabelProps, type FormMessageProps
}

export {
  BaseInput, BaseCheckbox as Checkbox, BaseInput as Input, BaseRadio as Radio, BaseSelect as Select, BaseTextarea as Textarea, type BaseCheckboxProps, type BaseInputProps, type BaseRadioProps, type BaseSelectProps, type BaseTextareaProps
}

// ============================================================================
// NAVIGATION COMPONENTS
// ============================================================================

// Chain Selector
export { ChainSelector } from './navigation/ChainSelector'

// ============================================================================
// DEVELOPER PORTAL COMPONENTS
// ============================================================================

export { DeveloperMobileHeader, DeveloperSidebar } from './developer'

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

// Most commonly used components for quick access
export const Components = {
  // Core - lazy loaded to avoid circular dependencies
  get Card() { return BaseCard },
  get Button() { return BaseButton },
  get Modal() { return BaseModal },

  // Forms
  get Form() { return BaseForm },
  get Input() { return BaseInput },
  get Select() { return BaseSelect },
  get Checkbox() { return BaseCheckbox },
}

// Legacy compatibility aliases
export { BaseButton as Button } from './buttons/BaseButton'
export { BaseCard as Card } from './cards/BaseCard'
export { BaseForm } from './forms/BaseForm'
export { BaseModal as Modal } from './modals/BaseModal'

// ============================================================================
// MIGRATION HELPERS
// ============================================================================

/**
 * Migration map for old component names to new unified components
 * Use this to find replacement components during migration
 */
export const MIGRATION_MAP = {
  // Admin components → Unified
  'AdminEPSCard': 'DataCard',
  'PermissionAssignmentCard': 'BaseCard',
  'SystemHealthCard': 'StatusCard',
  'AdminPaginationButton': 'PaginationButton',
  'EditProfileButton': 'ActionButton',
  'CleanupButton': 'ActionButton',

  // Frontend components → Unified
  'SubscriptionDetailsModal': 'BaseModal',
  'WalletConnectionModal': 'BaseModal',
  'StockCard': 'DataCard',
  'MetricCard': 'DataCard',
  'FinancialCard': 'DataCard',

  // Form components → Unified
  'Input': 'BaseInput',
  'Select': 'BaseSelect',
  'Checkbox': 'BaseCheckbox',
  'FormField': 'FormFieldWrapper',
} as const

/**
 * Get the unified component name for a legacy component
 */
export function getUnifiedComponent(legacyName: keyof typeof MIGRATION_MAP): string {
  return MIGRATION_MAP[legacyName]
}

/**
 * Check if a component has been migrated to the unified system
 */
export function isComponentMigrated(componentName: string): boolean {
  return Object.keys(MIGRATION_MAP).includes(componentName)
}

export default Components