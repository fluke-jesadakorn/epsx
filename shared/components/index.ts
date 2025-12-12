/**
 * SHARED COMPONENTS INDEX
 * Unified export of all consolidated components
 * Replaces 68+ duplicate component implementations across frontend and admin apps
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
  type StockDataCardProps
} from './cards/StockDataCard'


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

// Modals - Import for local use and re-export
import {
  BaseModal,
  ConfirmModal,
  FormModal,
  type BaseModalProps,
  type ConfirmModalProps,
  type FormModalProps
} from './modals/BaseModal'

export {
  BaseModal,
  ConfirmModal,
  FormModal,
  type BaseModalProps,
  type ConfirmModalProps,
  type FormModalProps
}

// Themed Modals
export {
  PancakeCardModal,
  PancakeConfirmModal,
  PancakeFormModal, PancakeModal, PancakeSubscriptionModal, type PancakeCardModalProps, type PancakeConfirmModalProps,
  type PancakeFormModalProps, type PancakeModalProps, type PancakeSubscriptionModalProps
} from './modals/PancakeModal'

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

export {
  BaseForm as Form, FormControl,
  FormDescription, FormField, FormFieldWrapper, FormItem,
  FormLabel, FormMessage, useFormField,
  type BaseFormProps, type FormControlProps,
  type FormDescriptionProps, type FormFieldWrapperProps, type FormItemProps,
  type FormLabelProps, type FormMessageProps
}

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
  BaseInput, BaseCheckbox as Checkbox, BaseInput as Input, BaseRadio as Radio, BaseSelect as Select, BaseTextarea as Textarea, type BaseCheckboxProps, type BaseInputProps, type BaseRadioProps, type BaseSelectProps, type BaseTextareaProps
}

// Themed Forms (excluding duplicates with buttons/cards)
export {
  PancakeBadge, PancakeCheckbox, Checkbox as PancakeCheckboxAlias, PancakeForm, Form as PancakeFormAlias, PancakeFormField, FormField as PancakeFormFieldAlias, PancakeInput, Input as PancakeInputAlias, PancakeLabel, Label as PancakeLabelAlias, PancakeSelect, Select as PancakeSelectAlias, PancakeTextarea, Textarea as PancakeTextareaAlias, type PancakeBadgeProps, type PancakeFormFieldProps, type PancakeFormProps,
  type PancakeInputProps,
  type PancakeLabelProps, type PancakeSelectProps,
  type PancakeTextareaProps
} from './forms/PancakeForm'

// Note: PancakeButton is exported from buttons section to avoid duplicates

// ============================================================================
// NAVIGATION COMPONENTS
// ============================================================================

// Navigation System - Import for local use and re-export
import {
  BaseNavigation,
  BreadcrumbSeparator,
  NavigationContent,
  NavigationItem,
  NavigationLink,
  NavigationList,
  NavigationTrigger,
  useNavigation,
  type BaseNavigationProps,
  type BreadcrumbSeparatorProps,
  type NavigationContentProps,
  type NavigationItemProps,
  type NavigationLinkProps,
  type NavigationListProps,
  type NavigationTriggerProps
} from './navigation/BaseNavigation'

export {
  BreadcrumbSeparator, BaseNavigation as Navigation, NavigationContent, NavigationItem,
  NavigationLink, NavigationList, NavigationTrigger, useNavigation,
  type BaseNavigationProps, type BreadcrumbSeparatorProps, type NavigationContentProps, type NavigationItemProps, type NavigationItem as NavigationItemType, type NavigationLinkProps, type NavigationListProps, type NavigationTriggerProps
}

// Tabs System - Import for local use and re-export
import {
  BaseTabs,
  CounterTab,
  IconTab,
  TabsContent,
  TabsList,
  TabsTrigger,
  useTabActive,
  useTabs,
  useTabsState,
  type BaseTabsProps,
  type CounterTabProps,
  type IconTabProps,
  type TabsContentProps,
  type TabsListProps,
  type TabsTriggerProps
} from './navigation/BaseTabs'

export {
  CounterTab, IconTab, BaseTabs as Tabs, TabsContent, TabsList,
  TabsTrigger, useTabActive, useTabs,
  useTabsState, type BaseTabsProps, type CounterTabProps, type IconTabProps, type TabsContentProps, type TabsListProps,
  type TabsTriggerProps
}

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

  // Navigation
  get Navigation() { return BaseNavigation },
  get Tabs() { return BaseTabs }
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

  // Navigation components → Unified
  'MobileBottomNav': 'BaseNavigation',
  'NavigationMenu': 'BaseNavigation',
  'Tabs': 'BaseTabs',
  'TabsList': 'TabsList',
  'TabsTrigger': 'TabsTrigger',
  'TabsContent': 'TabsContent'
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