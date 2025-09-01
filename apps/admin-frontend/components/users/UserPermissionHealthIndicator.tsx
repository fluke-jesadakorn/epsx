'use client'

import { 
  AlertTriangle, 
  CheckCircle,
  XCircle,
  Clock,
  Shield,
  AlertCircle,
  Zap
} from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { parseEmbeddedPermissions } from '../auth/AdminPermissionExpiryIndicator'

// ============================================================================
// TYPES
// ============================================================================

interface UserPermissionHealthProps {
  userId: string
  permissions: string[]
  variant?: 'compact' | 'detailed' | 'inline' | 'avatar'
  showActions?: boolean
  onViewUser?: (userId: string) => void
  onExtendPermissions?: (userId: string) => void
  className?: string
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function UserPermissionHealthIndicator({
  userId,
  permissions,
  variant = 'compact',
  showActions = false,
  onViewUser,
  onExtendPermissions,
  className = ''
}: UserPermissionHealthProps) {
  
  const data = parseEmbeddedPermissions(permissions)
  
  switch (variant) {
    case 'compact':
      return <CompactHealthIndicator 
        userId={userId} 
        data={data} 
        showActions={showActions}
        onViewUser={onViewUser}
        onExtendPermissions={onExtendPermissions}
        className={className} 
      />
    case 'detailed':
      return <DetailedHealthIndicator 
        userId={userId} 
        data={data} 
        showActions={showActions}
        onViewUser={onViewUser}
        onExtendPermissions={onExtendPermissions}
        className={className} 
      />
    case 'inline':
      return <InlineHealthIndicator 
        userId={userId} 
        data={data} 
        className={className} 
      />
    case 'avatar':
      return <AvatarHealthIndicator 
        userId={userId} 
        data={data} 
        className={className} 
      />
    default:
      return <CompactHealthIndicator 
        userId={userId} 
        data={data} 
        showActions={showActions}
        onViewUser={onViewUser}
        onExtendPermissions={onExtendPermissions}
        className={className} 
      />
  }
}

// ============================================================================
// COMPACT VARIANT - For tables and lists
// ============================================================================

function CompactHealthIndicator({ 
  userId, 
  data, 
  showActions,
  onViewUser,
  onExtendPermissions,
  className 
}: {
  userId: string
  data: ReturnType<typeof parseEmbeddedPermissions>
  showActions?: boolean
  onViewUser?: (userId: string) => void
  onExtendPermissions?: (userId: string) => void
  className: string
}) {
  const { icon: Icon, color } = getHealthConfig(data.healthScore)
  const hasIssues = data.expiredCount > 0 || data.criticalCount > 0
  
  return (
    <div className={`flex items-center gap-2 ${className}`}>
      {/* Health Icon */}
      <div className="relative">
        <Icon className={`h-5 w-5 ${color}`} />
        {hasIssues && (
          <div className="absolute -top-1 -right-1 h-3 w-3 bg-red-500 rounded-full animate-pulse border-2 border-white"></div>
        )}
      </div>
      
      {/* Status Badges */}
      <div className="flex gap-1">
        {data.expiredCount > 0 && (
          <Badge variant="destructive" className="text-xs px-1.5 py-0 h-5 animate-pulse">
            {data.expiredCount}
          </Badge>
        )}
        {data.criticalCount > 0 && (
          <Badge className="bg-orange-100 text-orange-800 text-xs px-1.5 py-0 h-5 border border-orange-200">
            {data.criticalCount}
          </Badge>
        )}
        {data.expiringSoonCount > 0 && !data.criticalCount && (
          <Badge variant="outline" className="bg-yellow-50 text-yellow-800 text-xs px-1.5 py-0 h-5 border border-yellow-300">
            {data.expiringSoonCount}
          </Badge>
        )}
      </div>
      
      {/* Quick Actions */}
      {showActions && hasIssues && (
        <div className="flex gap-1">
          {onViewUser && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onViewUser(userId)}
              className="h-6 px-2 text-xs"
              title="View User Details"
            >
              View
            </Button>
          )}
          {onExtendPermissions && data.expiringSoonCount > 0 && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onExtendPermissions(userId)}
              className="h-6 px-2 text-xs text-blue-600 hover:text-blue-700"
              title="Extend Permissions"
            >
              <Zap className="h-3 w-3" />
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// DETAILED VARIANT - For user cards
// ============================================================================

function DetailedHealthIndicator({ 
  userId, 
  data, 
  showActions,
  onViewUser,
  onExtendPermissions,
  className 
}: {
  userId: string
  data: ReturnType<typeof parseEmbeddedPermissions>
  showActions?: boolean
  onViewUser?: (userId: string) => void
  onExtendPermissions?: (userId: string) => void
  className: string
}) {
  const { icon: Icon, color, bgColor } = getHealthConfig(data.healthScore)
  
  return (
    <div className={`p-3 rounded-lg ${bgColor} border ${className}`}>
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <Icon className={`h-5 w-5 ${color}`} />
          <span className="font-medium text-sm">Permission Health</span>
        </div>
        <Badge className={getHealthBadgeColor(data.healthScore)}>
          {data.healthScore}
        </Badge>
      </div>
      
      <div className="grid grid-cols-4 gap-2 text-xs mb-3">
        <div className="text-center">
          <div className="font-bold text-green-700">{data.totalCount - data.expiredCount}</div>
          <div className="text-green-600">Active</div>
        </div>
        <div className="text-center">
          <div className={`font-bold ${data.expiredCount > 0 ? 'text-red-700' : 'text-gray-500'}`}>
            {data.expiredCount}
          </div>
          <div className={`${data.expiredCount > 0 ? 'text-red-600' : 'text-gray-500'}`}>Expired</div>
        </div>
        <div className="text-center">
          <div className={`font-bold ${data.expiringSoonCount > 0 ? 'text-yellow-700' : 'text-gray-500'}`}>
            {data.expiringSoonCount}
          </div>
          <div className={`${data.expiringSoonCount > 0 ? 'text-yellow-600' : 'text-gray-500'}`}>Expiring</div>
        </div>
        <div className="text-center">
          <div className="font-bold text-blue-700">{data.permanentCount}</div>
          <div className="text-blue-600">Permanent</div>
        </div>
      </div>
      
      {data.nextExpiry && (
        <div className="text-xs text-gray-600 mb-2">
          <Clock className="h-3 w-3 inline mr-1" />
          Next expiry in {formatTimeRemaining(data.nextExpiry.timeRemaining)}
        </div>
      )}
      
      {showActions && (
        <div className="flex gap-2">
          {onViewUser && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onViewUser(userId)}
              className="text-xs h-7 flex-1"
            >
              View Details
            </Button>
          )}
          {onExtendPermissions && (data.expiredCount > 0 || data.expiringSoonCount > 0) && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onExtendPermissions(userId)}
              className="text-xs h-7 flex-1 text-blue-600"
            >
              <Zap className="h-3 w-3 mr-1" />
              Extend
            </Button>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// INLINE VARIANT - For status text
// ============================================================================

function InlineHealthIndicator({ 
  userId, 
  data, 
  className 
}: {
  userId: string
  data: ReturnType<typeof parseEmbeddedPermissions>
  className: string
}) {
  const { icon: Icon, color } = getHealthConfig(data.healthScore)
  
  return (
    <div className={`inline-flex items-center gap-2 text-sm ${className}`}>
      <Icon className={`h-4 w-4 ${color}`} />
      <span>
        {data.expiredCount > 0 && `${data.expiredCount} expired`}
        {!data.expiredCount && data.expiringSoonCount > 0 && `${data.expiringSoonCount} expiring`}
        {!data.expiredCount && !data.expiringSoonCount && 'All valid'}
      </span>
      
      {(data.expiredCount > 0 || data.expiringSoonCount > 0) && (
        <div className="flex gap-1">
          {data.expiredCount > 0 && (
            <div className="h-2 w-2 bg-red-500 rounded-full animate-pulse"></div>
          )}
          {data.expiringSoonCount > 0 && (
            <div className="h-2 w-2 bg-yellow-500 rounded-full"></div>
          )}
        </div>
      )}
    </div>
  )
}

// ============================================================================
// AVATAR VARIANT - For user avatars with overlay
// ============================================================================

function AvatarHealthIndicator({ 
  userId, 
  data, 
  className 
}: {
  userId: string
  data: ReturnType<typeof parseEmbeddedPermissions>
  className: string
}) {
  if (data.expiredCount === 0 && data.criticalCount === 0) return null
  
  const isExpired = data.expiredCount > 0
  const isCritical = data.criticalCount > 0
  
  return (
    <div className={`absolute -top-1 -right-1 ${className}`}>
      {isExpired && (
        <div className="h-5 w-5 bg-red-500 rounded-full flex items-center justify-center animate-pulse">
          <XCircle className="h-3 w-3 text-white" />
        </div>
      )}
      {!isExpired && isCritical && (
        <div className="h-5 w-5 bg-orange-500 rounded-full flex items-center justify-center">
          <AlertTriangle className="h-3 w-3 text-white" />
        </div>
      )}
      {!isExpired && !isCritical && data.expiringSoonCount > 0 && (
        <div className="h-5 w-5 bg-yellow-500 rounded-full flex items-center justify-center">
          <Clock className="h-3 w-3 text-white" />
        </div>
      )}
    </div>
  )
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function getHealthConfig(healthScore: string) {
  const configs = {
    excellent: {
      icon: CheckCircle,
      color: 'text-green-500',
      bgColor: 'bg-green-50 border-green-200'
    },
    good: {
      icon: Shield,
      color: 'text-blue-500',
      bgColor: 'bg-blue-50 border-blue-200'
    },
    warning: {
      icon: AlertTriangle,
      color: 'text-yellow-500',
      bgColor: 'bg-yellow-50 border-yellow-200'
    },
    critical: {
      icon: AlertCircle,
      color: 'text-red-500',
      bgColor: 'bg-red-50 border-red-200'
    }
  }
  return configs[healthScore] || configs.good
}

function getHealthBadgeColor(healthScore: string): string {
  return {
    excellent: 'bg-green-100 text-green-800 border border-green-200',
    good: 'bg-blue-100 text-blue-800 border border-blue-200',
    warning: 'bg-yellow-100 text-yellow-800 border border-yellow-200', 
    critical: 'bg-red-100 text-red-800 border border-red-200'
  }[healthScore] || 'bg-gray-100 text-gray-800 border border-gray-200'
}

function formatTimeRemaining(seconds: number): string {
  if (seconds <= 0) return 'Expired'
  
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `${days}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m`
  return `${seconds}s`
}

// ============================================================================
// TABLE-SPECIFIC COMPONENTS
// ============================================================================

export function UserTableHealthCell({ 
  userId, 
  permissions,
  onViewUser,
  onExtendPermissions 
}: {
  userId: string
  permissions: string[]
  onViewUser?: (userId: string) => void
  onExtendPermissions?: (userId: string) => void
}) {
  return (
    <UserPermissionHealthIndicator
      userId={userId}
      permissions={permissions}
      variant="compact"
      showActions={true}
      onViewUser={onViewUser}
      onExtendPermissions={onExtendPermissions}
    />
  )
}

export function UserCardHealthSection({ 
  userId, 
  permissions,
  onViewUser,
  onExtendPermissions 
}: {
  userId: string
  permissions: string[]
  onViewUser?: (userId: string) => void
  onExtendPermissions?: (userId: string) => void
}) {
  return (
    <UserPermissionHealthIndicator
      userId={userId}
      permissions={permissions}
      variant="detailed"
      showActions={true}
      onViewUser={onViewUser}
      onExtendPermissions={onExtendPermissions}
    />
  )
}

export function UserAvatarHealthOverlay({ 
  userId, 
  permissions,
  className 
}: {
  userId: string
  permissions: string[]
  className?: string
}) {
  return (
    <UserPermissionHealthIndicator
      userId={userId}
      permissions={permissions}
      variant="avatar"
      className={className}
    />
  )
}