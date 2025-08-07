/**
 * User Status Badge Component
 * Shows user account status with appropriate styling
 */

import type { UserStatus } from '@/lib/types/unified-user'

interface UserStatusBadgeProps {
  status: UserStatus
  size?: 'sm' | 'md' | 'lg'
}

export function UserStatusBadge({ status, size = 'md' }: UserStatusBadgeProps) {
  const sizeClasses = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-2 py-1',
    lg: 'text-base px-3 py-1.5'
  }

  const statusConfig = {
    active: {
      label: 'Active',
      className: 'bg-green-100 text-green-800 border border-green-200'
    },
    disabled: {
      label: 'Disabled',
      className: 'bg-red-100 text-red-800 border border-red-200'
    },
    pending: {
      label: 'Pending',
      className: 'bg-yellow-100 text-yellow-800 border border-yellow-200'
    },
    suspended: {
      label: 'Suspended',
      className: 'bg-orange-100 text-orange-800 border border-orange-200'
    }
  }

  const config = statusConfig[status]

  return (
    <span className={`inline-flex items-center rounded-full font-medium ${sizeClasses[size]} ${config.className}`}>
      {config.label}
    </span>
  )
}