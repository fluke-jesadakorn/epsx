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
  Card as PancakeCard,
  CardHeader as PancakeCardHeader,
  CardTitle as PancakeCardTitle,
  CardDescription as PancakeCardDescription,
  CardContent as PancakeCardContent,
  CardFooter as PancakeCardFooter,
  StatsCard as PancakeStatsCard,
  AnalyticsCard as PancakeAnalyticsCard,
  type PancakeCardProps,
  type StatsCardProps as PancakeStatsCardProps,
  type AnalyticsCardProps as PancakeAnalyticsCardProps
} from './cards/PancakeCard'

// Buttons - Import for local use and re-export
import {
  BaseButton,
  IconButton,
  LoadingButton,
  ActionButton,
  PaginationButton,
  type BaseButtonProps,
  type IconButtonProps,
  type LoadingButtonProps,
  type ActionButtonProps,
  type PaginationButtonProps
} from './buttons/BaseButton'

export {
  BaseButton,
  IconButton,
  LoadingButton,
  ActionButton,
  PaginationButton,
  type BaseButtonProps,
  type IconButtonProps,
  type LoadingButtonProps,
  type ActionButtonProps,
  type PaginationButtonProps
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
  PancakeModal,
  PancakeCardModal,
  PancakeConfirmModal,
  PancakeFormModal,
  PancakeSubscriptionModal,
  type PancakeModalProps,
  type PancakeConfirmModalProps,
  type PancakeFormModalProps,
  type PancakeCardModalProps,
  type PancakeSubscriptionModalProps
} from './modals/PancakeModal'

// ============================================================================
// FORM COMPONENTS
// ============================================================================

// Form System - Import for local use and re-export
import {
  BaseForm,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormDescription,
  FormMessage,
  FormFieldWrapper,
  useFormField,
  type BaseFormProps,
  type FormItemProps,
  type FormLabelProps,
  type FormControlProps,
  type FormDescriptionProps,
  type FormMessageProps,
  type FormFieldWrapperProps
} from './forms/BaseForm'

export {
  BaseForm as Form,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormDescription,
  FormMessage,
  FormFieldWrapper,
  useFormField,
  type BaseFormProps,
  type FormItemProps,
  type FormLabelProps,
  type FormControlProps,
  type FormDescriptionProps,
  type FormMessageProps,
  type FormFieldWrapperProps
}

// Input Components - Import for local use and re-export
import {
  BaseInput,
  BaseTextarea,
  BaseSelect,
  BaseCheckbox,
  BaseRadio,
  type BaseInputProps,
  type BaseTextareaProps,
  type BaseSelectProps,
  type BaseCheckboxProps,
  type BaseRadioProps
} from './forms/BaseInput'

export {
  BaseInput as Input,
  BaseInput,
  BaseTextarea as Textarea,
  BaseSelect as Select,
  BaseCheckbox as Checkbox,
  BaseRadio as Radio,
  type BaseInputProps,
  type BaseTextareaProps,
  type BaseSelectProps,
  type BaseCheckboxProps,
  type BaseRadioProps
}

// Themed Forms (excluding duplicates with buttons/cards)
export {
  PancakeForm,
  PancakeInput,
  PancakeLabel,
  PancakeBadge,
  PancakeSelect,
  PancakeCheckbox,
  PancakeTextarea,
  PancakeFormField,
  Form as PancakeFormAlias,
  Input as PancakeInputAlias,
  Label as PancakeLabelAlias,
  Select as PancakeSelectAlias,
  Checkbox as PancakeCheckboxAlias,
  Textarea as PancakeTextareaAlias,
  FormField as PancakeFormFieldAlias,
  type PancakeFormProps,
  type PancakeInputProps,
  type PancakeLabelProps,
  type PancakeBadgeProps,
  type PancakeSelectProps,
  type PancakeTextareaProps,
  type PancakeFormFieldProps
} from './forms/PancakeForm'

// Note: PancakeButton is exported from buttons section to avoid duplicates

// ============================================================================
// NAVIGATION COMPONENTS
// ============================================================================

// Navigation System - Import for local use and re-export
import {
  BaseNavigation,
  NavigationList,
  NavigationItem,
  NavigationLink,
  NavigationTrigger,
  NavigationContent,
  BreadcrumbSeparator,
  useNavigation,
  type BaseNavigationProps,
  type NavigationListProps,
  type NavigationItemProps,
  type NavigationLinkProps,
  type NavigationTriggerProps,
  type NavigationContentProps,
  type BreadcrumbSeparatorProps,
  type NavigationItem as NavigationItemType
} from './navigation/BaseNavigation'

export {
  BaseNavigation as Navigation,
  NavigationList,
  NavigationItem,
  NavigationLink,
  NavigationTrigger,
  NavigationContent,
  BreadcrumbSeparator,
  useNavigation,
  type BaseNavigationProps,
  type NavigationListProps,
  type NavigationItemProps,
  type NavigationLinkProps,
  type NavigationTriggerProps,
  type NavigationContentProps,
  type BreadcrumbSeparatorProps,
  type NavigationItem as NavigationItemType
}

// Tabs System - Import for local use and re-export
import {
  BaseTabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  IconTab,
  CounterTab,
  useTabs,
  useTabsState,
  useTabActive,
  type BaseTabsProps,
  type TabsListProps,
  type TabsTriggerProps,
  type TabsContentProps,
  type IconTabProps,
  type CounterTabProps
} from './navigation/BaseTabs'

export {
  BaseTabs as Tabs,
  TabsList,
  TabsTrigger,
  TabsContent,
  IconTab,
  CounterTab,
  useTabs,
  useTabsState,
  useTabActive,
  type BaseTabsProps,
  type TabsListProps,
  type TabsTriggerProps,
  type TabsContentProps,
  type IconTabProps,
  type CounterTabProps
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
export { BaseCard as Card } from './cards/BaseCard'
export { BaseButton as Button } from './buttons/BaseButton'
export { BaseModal as Modal } from './modals/BaseModal'
export { BaseForm } from './forms/BaseForm'

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