/**
 * User Status Badge Component
 * Shows user account status with appropriate styling
 */

import type { UserStatus } from '@/lib/types/unified-user'

interface UserStatusBadgeProps {
  status?: UserStatus
  size?: 'sm' | 'md' | 'lg'
}

export function UserStatusBadge({ status, size = 'md' }: UserStatusBadgeProps) {
  // Safe size classes with fallback
  const getSizeClass = (size: string) => {
    switch (size) {
      case 'sm': return 'text-xs px-2 py-0.5'
      case 'lg': return 'text-base px-3 py-1.5'
      default: return 'text-sm px-2 py-1'
    }
  }

  // Safe status config with fallback
  const getStatusConfig = (status?: UserStatus) => {
    switch (status) {
      case 'active':
        return {
          label: 'Active',
          className: 'bg-green-100 text-green-800 border border-green-200'
        }
      case 'disabled':
        return {
          label: 'Disabled', 
          className: 'bg-red-100 text-red-800 border border-red-200'
        }
      case 'pending':
        return {
          label: 'Pending',
          className: 'bg-yellow-100 text-yellow-800 border border-yellow-200'
        }
      case 'suspended':
        return {
          label: 'Suspended',
          className: 'bg-orange-100 text-orange-800 border border-orange-200'
        }
      default:
        return {
          label: 'Unknown',
          className: 'bg-gray-100 text-gray-800 border border-gray-200'
        }
    }
  }

  const sizeClass = getSizeClass(size)
  const statusConfig = getStatusConfig(status)

  return (
    <span className={`inline-flex items-center rounded-full font-medium ${sizeClass} ${statusConfig.className}`}>
      {statusConfig.label}
    </span>
  )
}