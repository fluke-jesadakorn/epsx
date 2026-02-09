/**
 * EmptyState - Reusable empty state component
 * Shows helpful messaging when no data is available
 * Follows zero animation policy
 */

import type { LucideIcon} from 'lucide-react';
import { Inbox, Users, FileText, AlertCircle, Database, Search } from 'lucide-react'

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
    iconColor: 'text-purple-400',
    bgColor: 'bg-gradient-to-br from-purple-500/10 to-orange-500/10 border border-purple-500/20',
    titleColor: 'text-foreground',
    descColor: 'text-muted-foreground'
  },
  search: {
    iconColor: 'text-purple-400',
    bgColor: 'bg-gradient-to-br from-purple-500/10 to-orange-500/10 border border-purple-500/20',
    titleColor: 'text-foreground',
    descColor: 'text-muted-foreground'
  },
  error: {
    iconColor: 'text-red-400',
    bgColor: 'bg-gradient-to-br from-red-500/10 to-red-600/10 border border-red-500/20',
    titleColor: 'text-foreground',
    descColor: 'text-muted-foreground'
  },
  info: {
    iconColor: 'text-amber-400',
    bgColor: 'bg-gradient-to-br from-amber-500/10 to-orange-500/10 border border-amber-500/20',
    titleColor: 'text-foreground',
    descColor: 'text-muted-foreground'
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
      <div className={cn('p-4 rounded-2xl mb-4 backdrop-blur-sm', config.bgColor)}>
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
      action={onAction ? { label: 'Create user', onClick: onAction } : undefined}
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
      description={description ?? 'Data will appear here once available.'}
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
    <div className="text-center py-12 text-muted-foreground">
      <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-gradient-to-br from-purple-500/10 to-orange-500/10 border border-purple-500/20 flex items-center justify-center">
        <Inbox className="w-6 h-6 text-purple-400" aria-hidden="true" />
      </div>
      <p className="text-sm">{message}</p>
      {action && (
        <Button
          onClick={action.onClick}
          variant="glass"
          className="mt-4"
          aria-label={action.label}
        >
          {action.label}
        </Button>
      )}
    </div>
  )
}
