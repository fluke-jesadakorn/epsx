/**
 * EmptyState - Reusable empty state component
 * Shows helpful messaging when no data is available
 * Follows zero animation policy
 */

import { LucideIcon, Inbox, Users, FileText, AlertCircle, Database, Search } from 'lucide-react'

import { Button } from './button'

import { cn } from '@/lib/utils'

interface EmptyStateProps {
  icon?: LucideIcon
  title: string
  description?: string
  action?: {
    label: string
    onClick: () => void
  }
  variant?: 'default' | 'search' | 'error' | 'info'
  className?: string
}

const variantConfig = {
  default: {
    iconColor: 'text-gray-400',
    bgColor: 'bg-gray-100 dark:bg-gray-800',
    titleColor: 'text-gray-900 dark:text-gray-100',
    descColor: 'text-gray-600 dark:text-gray-400'
  },
  search: {
    iconColor: 'text-blue-400',
    bgColor: 'bg-blue-50 dark:bg-blue-900/20',
    titleColor: 'text-blue-900 dark:text-blue-100',
    descColor: 'text-blue-600 dark:text-blue-400'
  },
  error: {
    iconColor: 'text-red-400',
    bgColor: 'bg-red-50 dark:bg-red-900/20',
    titleColor: 'text-red-900 dark:text-red-100',
    descColor: 'text-red-600 dark:text-red-400'
  },
  info: {
    iconColor: 'text-yellow-400',
    bgColor: 'bg-yellow-50 dark:bg-yellow-900/20',
    titleColor: 'text-yellow-900 dark:text-yellow-100',
    descColor: 'text-yellow-600 dark:text-yellow-400'
  }
}

/**
 * Empty state component with icon, title, description, and optional action
 * @param root0
 * @param root0.icon
 * @param root0.title
 * @param root0.description
 * @param root0.action
 * @param root0.variant
 * @param root0.className
 */
export function EmptyState({
  icon: Icon = Inbox,
  title,
  description,
  action,
  variant = 'default',
  className
}: EmptyStateProps) {
  const config = variantConfig[variant]

  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center p-8 sm:p-12 text-center',
        className
      )}
      role="status"
      aria-live="polite"
    >
      <div className={cn('p-4 rounded-full mb-4', config.bgColor)}>
        <Icon className={cn('w-12 h-12', config.iconColor)} aria-hidden="true" />
      </div>

      <h3 className={cn('text-lg font-semibold mb-2', config.titleColor)}>
        {title}
      </h3>

      {description && (
        <p className={cn('text-sm max-w-md mb-6', config.descColor)}>
          {description}
        </p>
      )}

      {action && (
        <Button
          onClick={action.onClick}
          variant={variant === 'error' ? 'destructive' : 'default'}
          aria-label={action.label}
        >
          {action.label}
        </Button>
      )}
    </div>
  )
}

/**
 * Predefined empty state variations
 */

/**
 *
 * @param root0
 * @param root0.onAction
 */
export function NoUsersState({ onAction }: { onAction?: () => void }) {
  return (
    <EmptyState
      icon={Users}
      title="No users found"
      description="There are no users matching your criteria. Try adjusting your filters or create a new user."
      action={onAction ? { label: 'Create User', onClick: onAction } : undefined}
    />
  )
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.description
 */
export function NoDataState({ title = 'No data available', description }: { title?: string; description?: string }) {
  return (
    <EmptyState
      icon={Database}
      title={title}
      description={description || 'Data will appear here once available.'}
    />
  )
}

/**
 *
 * @param root0
 * @param root0.query
 * @param root0.onClear
 */
export function NoSearchResultsState({ query, onClear }: { query?: string; onClear?: () => void }) {
  return (
    <EmptyState
      icon={Search}
      variant="search"
      title="No results found"
      description={query ? `No results match "${query}". Try different search terms.` : 'No results match your search criteria.'}
      action={onClear ? { label: 'Clear Search', onClick: onClear } : undefined}
    />
  )
}

/**
 *
 */
export function NoActivityState() {
  return (
    <EmptyState
      icon={FileText}
      title="No recent activity"
      description="Activity will appear here as users interact with the system."
    />
  )
}

/**
 *
 * @param root0
 * @param root0.message
 * @param root0.onRetry
 */
export function ErrorState({ message, onRetry }: { message: string; onRetry?: () => void }) {
  return (
    <EmptyState
      icon={AlertCircle}
      variant="error"
      title="Something went wrong"
      description={message}
      action={onRetry ? { label: 'Try Again', onClick: onRetry } : undefined}
    />
  )
}

/**
 * Table empty state - optimized for table contexts
 * @param root0
 * @param root0.message
 * @param root0.action
 * @param root0.action.label
 * @param root0.action.onClick
 */
export function TableEmptyState({
  message = 'No data available',
  action
}: {
  message?: string
  action?: { label: string; onClick: () => void }
}) {
  return (
    <div className="text-center py-12 text-gray-500 dark:text-gray-400">
      <Inbox className="w-12 h-12 mx-auto mb-3 text-gray-400" aria-hidden="true" />
      <p className="text-sm">{message}</p>
      {action && (
        <Button
          onClick={action.onClick}
          variant="outline"
          className="mt-4"
          aria-label={action.label}
        >
          {action.label}
        </Button>
      )}
    </div>
  )
}
