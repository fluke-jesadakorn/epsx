'use client'

import { ReactNode } from 'react'
import { 
  Clock, 
  AlertTriangle, 
  CheckCircle,
  XCircle,
  Zap,
  Calendar,
  TrendingUp,
  Shield,
  Eye,
  AlertCircle
} from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Separator } from '@/components/ui/separator'
import { format, formatDistance } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface EmbeddedPermissionInfo {
  permission: string
  basePermission: string
  platform: string
  resource: string
  action: string
  expiresAt?: number
  isExpired: boolean
  isExpiringSoon: boolean
  timeRemaining: number
  urgencyLevel: 'expired' | 'critical' | 'warning' | 'normal' | 'permanent'
}

interface PermissionExpiryData {
  permissions: EmbeddedPermissionInfo[]
  totalCount: number
  expiredCount: number
  expiringSoonCount: number
  criticalCount: number
  permanentCount: number
  healthScore: 'excellent' | 'good' | 'warning' | 'critical'
  nextExpiry?: EmbeddedPermissionInfo
}

interface AdminPermissionExpiryIndicatorProps {
  variant?: 'badge' | 'banner' | 'card' | 'compact' | 'detailed' | 'dashboard'
  data: PermissionExpiryData
  size?: 'xs' | 'sm' | 'md' | 'lg'
  showActions?: boolean
  showDetails?: boolean
  showHealth?: boolean
  className?: string
  onExtendPermission?: (permission: EmbeddedPermissionInfo) => void
  onRevokePermission?: (permission: EmbeddedPermissionInfo) => void
  onViewDetails?: (permission: EmbeddedPermissionInfo) => void
}

// ============================================================================
// MAIN INDICATOR COMPONENT
// ============================================================================

export function AdminPermissionExpiryIndicator({
  variant = 'badge',
  data,
  size = 'sm',
  showActions = false,
  showDetails = false,
  showHealth = false,
  className = '',
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: AdminPermissionExpiryIndicatorProps) {
  
  switch (variant) {
    case 'badge':
      return <ExpiryBadge data={data} size={size} className={className} />
    case 'banner':
      return <ExpiryBanner data={data} showDetails={showDetails} className={className} />
    case 'card':
      return <ExpiryCard 
        data={data} 
        showActions={showActions}
        showDetails={showDetails} 
        showHealth={showHealth} 
        className={className}
        onExtendPermission={onExtendPermission}
        onRevokePermission={onRevokePermission}
        onViewDetails={onViewDetails}
      />
    case 'compact':
      return <ExpiryCompact data={data} size={size} className={className} />
    case 'detailed':
      return <ExpiryDetailed 
        data={data} 
        showActions={showActions}
        className={className}
        onExtendPermission={onExtendPermission}
        onRevokePermission={onRevokePermission}
        onViewDetails={onViewDetails}
      />
    case 'dashboard':
      return <ExpiryDashboard data={data} showHealth={showHealth} className={className} />
    default:
      return <ExpiryBadge data={data} size={size} className={className} />
  }
}

// ============================================================================
// BADGE VARIANT
// ============================================================================

function ExpiryBadge({ 
  data, 
  size, 
  className 
}: { 
  data: PermissionExpiryData
  size: string
  className: string 
}) {
  const overallUrgency = getOverallUrgency(data)
  const { colors, icon: Icon } = getUrgencyConfig(overallUrgency)
  const sizeClasses = getSizeClasses(size)

  if (overallUrgency === 'permanent' && data.expiredCount === 0) return null

  return (
    <Badge className={`inline-flex items-center font-medium ${colors} ${sizeClasses} ${className}`}>
      <Icon className={`${getIconSize(size)} mr-1`} />
      <span>
        {overallUrgency === 'expired' && `${data.expiredCount} Expired`}
        {overallUrgency === 'critical' && `${data.criticalCount} Critical`}
        {overallUrgency === 'warning' && `${data.expiringSoonCount} Expiring`}
        {overallUrgency === 'normal' && 'Valid'}
        {overallUrgency === 'permanent' && `${data.permanentCount} Permanent`}
      </span>
    </Badge>
  )
}

// ============================================================================
// BANNER VARIANT
// ============================================================================

function ExpiryBanner({ 
  data, 
  showDetails, 
  className 
}: { 
  data: PermissionExpiryData
  showDetails: boolean
  className: string 
}) {
  const overallUrgency = getOverallUrgency(data)
  const { colors, bgColors, icon: Icon } = getUrgencyConfig(overallUrgency)

  if (overallUrgency === 'permanent' && data.expiredCount === 0) return null

  return (
    <Alert className={`${bgColors} ${className}`}>
      <Icon className="h-5 w-5" />
      <AlertDescription>
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-medium mb-1">
              {overallUrgency === 'expired' && 'Permissions Have Expired'}
              {overallUrgency === 'critical' && 'Critical Permission Issues'}
              {overallUrgency === 'warning' && 'Permissions Expiring Soon'}
              {overallUrgency === 'normal' && 'Permissions Valid'}
            </h3>
            {showDetails && <ExpiryDetails data={data} compact={true} />}
          </div>
          
          <div className="text-right text-sm">
            <div className="font-medium">{data.totalCount} Total</div>
            <div className="text-muted-foreground text-xs">
              Health: <span className={getHealthTextColor(data.healthScore)}>{data.healthScore}</span>
            </div>
          </div>
        </div>
      </AlertDescription>
    </Alert>
  )
}

// ============================================================================
// CARD VARIANT
// ============================================================================

function ExpiryCard({ 
  data, 
  showActions,
  showDetails, 
  showHealth, 
  className,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: { 
  data: PermissionExpiryData
  showActions?: boolean
  showDetails: boolean
  showHealth: boolean
  className: string
  onExtendPermission?: (permission: EmbeddedPermissionInfo) => void
  onRevokePermission?: (permission: EmbeddedPermissionInfo) => void
  onViewDetails?: (permission: EmbeddedPermissionInfo) => void
}) {
  const overallUrgency = getOverallUrgency(data)
  const { icon: Icon } = getUrgencyConfig(overallUrgency)

  return (
    <Card className={`${className}`}>
      <CardContent className="p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center">
            <Icon className={`h-6 w-6 mr-3 ${getUrgencyIconColor(overallUrgency)}`} />
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Permission Status
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {getStatusText(data)}
              </p>
            </div>
          </div>
          <ExpiryBadge data={data} size="md" className="" />
        </div>

        {showHealth && (
          <>
            <PermissionHealthDisplay data={data} />
            <Separator className="my-4" />
          </>
        )}

        {showDetails && (
          <>
            <ExpiryDetails 
              data={data} 
              showActions={showActions}
              onExtendPermission={onExtendPermission}
              onRevokePermission={onRevokePermission}
              onViewDetails={onViewDetails}
            />
            <Separator className="my-4" />
          </>
        )}

        {data.nextExpiry && (
          <div className="flex items-center text-sm text-gray-600 dark:text-gray-400">
            <Clock className="h-4 w-4 mr-2" />
            <span>
              Next expiry: <strong>{data.nextExpiry.basePermission}</strong> in{' '}
              <strong>{formatTimeRemaining(data.nextExpiry.timeRemaining)}</strong>
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

// ============================================================================
// COMPACT VARIANT
// ============================================================================

function ExpiryCompact({ 
  data, 
  size, 
  className 
}: { 
  data: PermissionExpiryData
  size: string
  className: string 
}) {
  const overallUrgency = getOverallUrgency(data)
  const { icon: Icon } = getUrgencyConfig(overallUrgency)

  return (
    <div className={`inline-flex items-center text-sm ${className}`}>
      <Icon className={`${getIconSize(size)} mr-2 ${getUrgencyIconColor(overallUrgency)}`} />
      <span className="font-medium mr-2">{data.totalCount}</span>
      {data.expiredCount > 0 && (
        <Badge variant="destructive" className="text-xs px-1 py-0 h-5">
          {data.expiredCount}
        </Badge>
      )}
      {data.criticalCount > 0 && (
        <Badge variant="secondary" className="text-xs px-1 py-0 h-5 ml-1 bg-red-100 text-red-800">
          {data.criticalCount}
        </Badge>
      )}
      {data.expiringSoonCount > 0 && (
        <Badge variant="outline" className="text-xs px-1 py-0 h-5 ml-1 bg-yellow-100 text-yellow-800">
          {data.expiringSoonCount}
        </Badge>
      )}
    </div>
  )
}

// ============================================================================
// DETAILED VARIANT
// ============================================================================

function ExpiryDetailed({ 
  data, 
  showActions,
  className,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: { 
  data: PermissionExpiryData
  showActions?: boolean
  className: string
  onExtendPermission?: (permission: EmbeddedPermissionInfo) => void
  onRevokePermission?: (permission: EmbeddedPermissionInfo) => void
  onViewDetails?: (permission: EmbeddedPermissionInfo) => void
}) {
  return (
    <div className={`space-y-4 ${className}`}>
      <PermissionHealthDisplay data={data} />
      <Separator />
      <ExpiryDetails 
        data={data} 
        showActions={showActions}
        onExtendPermission={onExtendPermission}
        onRevokePermission={onRevokePermission}
        onViewDetails={onViewDetails}
      />
    </div>
  )
}

// ============================================================================
// DASHBOARD VARIANT
// ============================================================================

function ExpiryDashboard({ 
  data, 
  showHealth, 
  className 
}: { 
  data: PermissionExpiryData
  showHealth: boolean
  className: string 
}) {
  return (
    <div className={`grid grid-cols-2 md:grid-cols-4 gap-4 ${className}`}>
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <div>
              <p className="text-2xl font-bold">{data.totalCount - data.expiredCount}</p>
              <p className="text-sm text-muted-foreground">Active</p>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            <div>
              <p className="text-2xl font-bold">{data.expiringSoonCount}</p>
              <p className="text-sm text-muted-foreground">Expiring</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-red-500" />
            <div>
              <p className="text-2xl font-bold">{data.expiredCount}</p>
              <p className="text-sm text-muted-foreground">Expired</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-500" />
            <div>
              <p className="text-2xl font-bold">{data.permanentCount}</p>
              <p className="text-sm text-muted-foreground">Permanent</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {showHealth && (
        <Card className="col-span-2 md:col-span-4">
          <CardContent className="p-4">
            <PermissionHealthDisplay data={data} />
          </CardContent>
        </Card>
      )}
    </div>
  )
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

function ExpiryDetails({ 
  data,
  compact = false,
  showActions = false,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: { 
  data: PermissionExpiryData
  compact?: boolean
  showActions?: boolean
  onExtendPermission?: (permission: EmbeddedPermissionInfo) => void
  onRevokePermission?: (permission: EmbeddedPermissionInfo) => void
  onViewDetails?: (permission: EmbeddedPermissionInfo) => void
}) {
  const expiredPermissions = data.permissions.filter(p => p.isExpired)
  const criticalPermissions = data.permissions.filter(p => p.urgencyLevel === 'critical')
  const warningPermissions = data.permissions.filter(p => p.urgencyLevel === 'warning')

  return (
    <div className="space-y-3">
      {expiredPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-red-800 dark:text-red-200 mb-2">
            Expired ({expiredPermissions.length})
          </h4>
          <div className="space-y-1">
            {expiredPermissions.slice(0, compact ? 3 : 10).map((permission, i) => (
              <PermissionItem 
                key={i} 
                permission={permission} 
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
            {expiredPermissions.length > (compact ? 3 : 10) && (
              <p className="text-xs text-muted-foreground">
                ... and {expiredPermissions.length - (compact ? 3 : 10)} more
              </p>
            )}
          </div>
        </div>
      )}
      
      {criticalPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-red-600 dark:text-red-300 mb-2">
            Critical - Expiring Soon ({criticalPermissions.length})
          </h4>
          <div className="space-y-1">
            {criticalPermissions.slice(0, compact ? 3 : 10).map((permission, i) => (
              <PermissionItem 
                key={i} 
                permission={permission} 
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
          </div>
        </div>
      )}

      {warningPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-yellow-700 dark:text-yellow-300 mb-2">
            Warning - Expiring ({warningPermissions.length})
          </h4>
          <div className="space-y-1">
            {warningPermissions.slice(0, compact ? 3 : 5).map((permission, i) => (
              <PermissionItem 
                key={i} 
                permission={permission} 
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

function PermissionItem({ 
  permission,
  showActions = false,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: {
  permission: EmbeddedPermissionInfo
  showActions?: boolean
  onExtendPermission?: (permission: EmbeddedPermissionInfo) => void
  onRevokePermission?: (permission: EmbeddedPermissionInfo) => void
  onViewDetails?: (permission: EmbeddedPermissionInfo) => void
}) {
  return (
    <div className="flex items-center justify-between text-xs bg-gray-50 dark:bg-gray-800 p-2 rounded">
      <div className="flex-1">
        <div className="font-mono text-gray-900 dark:text-gray-100">
          {permission.basePermission}
        </div>
        {permission.expiresAt && (
          <div className="text-muted-foreground">
            {permission.isExpired 
              ? `Expired ${formatDistance(new Date(permission.expiresAt * 1000), new Date(), { addSuffix: true })}`
              : `Expires ${formatTimeRemaining(permission.timeRemaining)}`
            }
          </div>
        )}
      </div>
      
      {showActions && (
        <div className="flex gap-1 ml-2">
          {onViewDetails && (
            <button
              onClick={() => onViewDetails(permission)}
              className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
              title="View Details"
            >
              <Eye className="h-3 w-3" />
            </button>
          )}
          {onExtendPermission && !permission.isExpired && (
            <button
              onClick={() => onExtendPermission(permission)}
              className="p-1 hover:bg-blue-100 dark:hover:bg-blue-900 rounded text-blue-600"
              title="Extend Permission"
            >
              <Zap className="h-3 w-3" />
            </button>
          )}
          {onRevokePermission && (
            <button
              onClick={() => onRevokePermission(permission)}
              className="p-1 hover:bg-red-100 dark:hover:bg-red-900 rounded text-red-600"
              title="Revoke Permission"
            >
              <XCircle className="h-3 w-3" />
            </button>
          )}
        </div>
      )}
    </div>
  )
}

function PermissionHealthDisplay({ data }: { data: PermissionExpiryData }) {
  const healthPercentage = Math.round(((data.totalCount - data.expiredCount) / data.totalCount) * 100)
  
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
      <div>
        <span className="text-muted-foreground">Total:</span>
        <span className="ml-2 font-medium">{data.totalCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Active:</span>
        <span className="ml-2 font-medium text-green-600">{data.totalCount - data.expiredCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Expired:</span>
        <span className="ml-2 font-medium text-red-600">{data.expiredCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Expiring:</span>
        <span className="ml-2 font-medium text-yellow-600">{data.expiringSoonCount}</span>
      </div>
      <div className="col-span-2 md:col-span-4 pt-2 border-t">
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground">Health Score:</span>
          <div className="flex items-center gap-2">
            <span className={`font-medium capitalize ${getHealthTextColor(data.healthScore)}`}>
              {data.healthScore}
            </span>
            <span className="text-xs text-muted-foreground">({healthPercentage}%)</span>
          </div>
        </div>
      </div>
    </div>
  )
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function getOverallUrgency(data: PermissionExpiryData): 'expired' | 'critical' | 'warning' | 'normal' | 'permanent' {
  if (data.expiredCount > 0) return 'expired'
  if (data.criticalCount > 0) return 'critical'
  if (data.expiringSoonCount > 0) return 'warning'
  if (data.totalCount > data.permanentCount) return 'normal'
  return 'permanent'
}

function getUrgencyConfig(urgency: string) {
  const configs = {
    expired: {
      colors: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      bgColors: 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-200',
      icon: XCircle
    },
    critical: {
      colors: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      bgColors: 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-200',
      icon: AlertCircle
    },
    warning: {
      colors: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      bgColors: 'bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-900/20 dark:border-yellow-800 dark:text-yellow-200',
      icon: AlertTriangle
    },
    normal: {
      colors: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      bgColors: 'bg-green-50 border-green-200 text-green-800 dark:bg-green-900/20 dark:border-green-800 dark:text-green-200',
      icon: CheckCircle
    },
    permanent: {
      colors: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
      bgColors: 'bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-200',
      icon: Shield
    }
  }
  return configs[urgency] || configs.normal
}

function getUrgencyIconColor(urgency: string): string {
  return {
    expired: 'text-red-500',
    critical: 'text-red-400', 
    warning: 'text-yellow-500',
    normal: 'text-green-500',
    permanent: 'text-blue-500'
  }[urgency] || 'text-gray-400'
}

function getSizeClasses(size: string): string {
  return {
    xs: 'px-2 py-1 text-xs',
    sm: 'px-2.5 py-1.5 text-xs',
    md: 'px-3 py-2 text-sm',
    lg: 'px-4 py-2 text-base'
  }[size] || 'px-2.5 py-1.5 text-xs'
}

function getIconSize(size: string): string {
  return {
    xs: 'h-3 w-3',
    sm: 'h-4 w-4', 
    md: 'h-5 w-5',
    lg: 'h-6 w-6'
  }[size] || 'h-4 w-4'
}

function getHealthTextColor(health: string): string {
  return {
    excellent: 'text-green-600',
    good: 'text-blue-600', 
    warning: 'text-yellow-600',
    critical: 'text-red-600'
  }[health] || 'text-gray-600'
}

function getStatusText(data: PermissionExpiryData): string {
  if (data.expiredCount > 0) {
    return `${data.expiredCount} permission${data.expiredCount !== 1 ? 's' : ''} expired`
  }
  if (data.expiringSoonCount > 0) {
    return `${data.expiringSoonCount} permission${data.expiringSoonCount !== 1 ? 's' : ''} expiring soon`
  }
  return 'All permissions valid'
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
// UTILITY FUNCTIONS FOR PARSING EMBEDDED PERMISSIONS
// ============================================================================

export function parseEmbeddedPermissions(permissions: string[]): PermissionExpiryData {
  const now = Math.floor(Date.now() / 1000)
  const processedPermissions: EmbeddedPermissionInfo[] = []
  
  for (const permission of permissions) {
    const parts = permission.split(':')
    const lastPart = parts[parts.length - 1]
    const timestamp = parseInt(lastPart, 10)
    
    let permissionInfo: EmbeddedPermissionInfo
    
    if (!isNaN(timestamp) && timestamp > 1000000000) { // Valid Unix timestamp
      const basePermission = parts.slice(0, -1).join(':')
      const [platform, resource, action] = basePermission.split(':')
      const timeRemaining = timestamp - now
      const isExpired = timeRemaining <= 0
      const isExpiringSoon = !isExpired && timeRemaining <= 86400 // 24 hours
      
      let urgencyLevel: EmbeddedPermissionInfo['urgencyLevel'] = 'normal'
      if (isExpired) urgencyLevel = 'expired'
      else if (timeRemaining <= 3600) urgencyLevel = 'critical' // 1 hour
      else if (isExpiringSoon) urgencyLevel = 'warning'
      
      permissionInfo = {
        permission,
        basePermission,
        platform: platform || '',
        resource: resource || '',
        action: action || '',
        expiresAt: timestamp,
        isExpired,
        isExpiringSoon,
        timeRemaining,
        urgencyLevel
      }
    } else {
      // Permanent permission
      const [platform, resource, action] = parts
      permissionInfo = {
        permission,
        basePermission: permission,
        platform: platform || '',
        resource: resource || '',
        action: action || '',
        isExpired: false,
        isExpiringSoon: false,
        timeRemaining: Infinity,
        urgencyLevel: 'permanent'
      }
    }
    
    processedPermissions.push(permissionInfo)
  }
  
  const expiredCount = processedPermissions.filter(p => p.isExpired).length
  const expiringSoonCount = processedPermissions.filter(p => p.isExpiringSoon && !p.isExpired).length
  const criticalCount = processedPermissions.filter(p => p.urgencyLevel === 'critical').length
  const permanentCount = processedPermissions.filter(p => p.urgencyLevel === 'permanent').length
  
  // Calculate health score
  const totalActive = processedPermissions.length - expiredCount
  const healthRatio = totalActive / processedPermissions.length
  let healthScore: PermissionExpiryData['healthScore'] = 'excellent'
  
  if (expiredCount > 0 || criticalCount > 2) healthScore = 'critical'
  else if (expiringSoonCount > 3 || healthRatio < 0.8) healthScore = 'warning'
  else if (healthRatio < 0.95) healthScore = 'good'
  
  const nextExpiry = processedPermissions
    .filter(p => !p.isExpired && p.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0]
  
  return {
    permissions: processedPermissions,
    totalCount: processedPermissions.length,
    expiredCount,
    expiringSoonCount,
    criticalCount,
    permanentCount,
    healthScore,
    nextExpiry
  }
}

// ============================================================================
// SPECIALIZED ADMIN COMPONENTS  
// ============================================================================

export function UserPermissionHealthCard({ 
  permissions,
  userId,
  className
}: { 
  permissions: string[]
  userId: string
  className?: string 
}) {
  const data = parseEmbeddedPermissions(permissions)
  
  return (
    <AdminPermissionExpiryIndicator
      variant="card"
      data={data}
      showHealth={true}
      showDetails={true}
      showActions={true}
      className={className}
    />
  )
}

export function AdminPermissionBanner({ 
  permissions,
  className 
}: { 
  permissions: string[]
  className?: string 
}) {
  const data = parseEmbeddedPermissions(permissions)
  
  if (data.expiredCount === 0 && data.criticalCount === 0) return null
  
  return (
    <AdminPermissionExpiryIndicator
      variant="banner"
      data={data}
      showDetails={true}
      className={className}
    />
  )
}

export function AdminPermissionDashboard({ 
  permissions,
  className 
}: { 
  permissions: string[]
  className?: string 
}) {
  const data = parseEmbeddedPermissions(permissions)
  
  return (
    <AdminPermissionExpiryIndicator
      variant="dashboard"
      data={data}
      showHealth={true}
      className={className}
    />
  )
}