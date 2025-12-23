// UI Components Index (Refactored to Shared)
// Shared UI components (primary exports)
export * from '@/shared/components/ui/alert'
export * from '@/shared/components/ui/badge'
export * from '@/shared/components/ui/button'
export * from '@/shared/components/ui/card'
export * from '@/shared/components/ui/dialog'
export * from '@/shared/components/ui/input'
export * from '@/shared/components/ui/progress'
export * from '@/shared/components/ui/skeleton'
export * from '@/shared/components/ui/table'
export * from '@/shared/components/ui/tabs'

// Admin-specific local components - avoid conflicting names
// Note: Checkbox from FormComponents, Select from select are used in admin
export { Checkbox, Textarea } from './FormComponents'
export {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel, SelectScrollDownButton, SelectScrollUpButton, SelectSeparator,
    SelectTrigger,
    SelectValue
} from './select'
export * from './separator'
export * from './switch'

// Admin-specific presentational components
export * from './AnalyticsCard'
export * from './PancakeButton'
export * from './PancakeCard'
export * from './StatsCard'
export * from './ThemeToggle'

