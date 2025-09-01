'use client'

import { ReactNode } from 'react'
import { usePermissionExpiry } from '@/hooks/usePermissionExpiry'
import { type TimestampedPermission } from '@/types/permissions'
import { deriveTierFromPermissions, type UserLevelType } from '@/lib/permission-utils'

// ============================================================================
// MAIN EXPIRY INDICATOR COMPONENT
// ============================================================================

interface PermissionExpiryIndicatorProps {
  variant?: 'badge' | 'banner' | 'card' | 'inline' | 'tooltip'
  size?: 'xs' | 'sm' | 'md' | 'lg'
  showDetails?: boolean
  showHealth?: boolean
  showPredictions?: boolean
  className?: string
  permission?: string // Show expiry for specific permission
  onExpired?: (permission: TimestampedPermission) => void
  onExpiringSoon?: (permission: TimestampedPermission) => void
}

export function PermissionExpiryIndicator({
  variant = 'badge',
  size = 'sm',
  showDetails = false,
  showHealth = false,
  showPredictions = false,
  className = '',
  permission,
  onExpired,
  onExpiringSoon
}: PermissionExpiryIndicatorProps) {
  const expiry = usePermissionExpiry()

  // Handle callbacks
  if (onExpired && expiry.hasExpired) {
    expiry.expiryInfo.expired.forEach(onExpired)
  }
  if (onExpiringSoon && expiry.hasExpiringSoon) {
    expiry.expiryInfo.expiringSoon.forEach(onExpiringSoon)
  }

  // Filter for specific permission if requested
  const filteredExpiry = permission 
    ? {
        ...expiry,
        allPermissionsWithExpiry: expiry.allPermissionsWithExpiry.filter(tp => 
          tp.basePermission === permission || tp.permission === permission
        ),
        hasExpired: expiry.expiryInfo.expired.some(tp => 
          tp.basePermission === permission || tp.permission === permission
        ),
        hasExpiringSoon: expiry.expiryInfo.expiringSoon.some(tp => 
          tp.basePermission === permission || tp.permission === permission
        )
      }
    : expiry

  // Don't render if no expiry issues and no specific permission requested
  if (!permission && !filteredExpiry.hasExpired && !filteredExpiry.hasExpiringSoon && !showHealth) {
    return null
  }

  switch (variant) {
    case 'badge':
      return <ExpiryBadge expiry={filteredExpiry} size={size} className={className} />
    case 'banner':
      return <ExpiryBanner expiry={filteredExpiry} showDetails={showDetails} className={className} />
    case 'card':
      return <ExpiryCard expiry={filteredExpiry} showDetails={showDetails} showHealth={showHealth} showPredictions={showPredictions} className={className} />
    case 'inline':
      return <ExpiryInline expiry={filteredExpiry} size={size} className={className} />
    case 'tooltip':
      return <ExpiryTooltip expiry={filteredExpiry} size={size} className={className} />
    default:
      return <ExpiryBadge expiry={filteredExpiry} size={size} className={className} />
  }
}

// ============================================================================
// BADGE VARIANT
// ============================================================================

function ExpiryBadge({ 
  expiry, 
  size, 
  className 
}: { 
  expiry: ReturnType<typeof usePermissionExpiry>
  size: string
  className: string 
}) {
  const urgency = getOverallUrgency(expiry)
  const colors = getUrgencyColors(urgency)
  const sizeClasses = getSizeClasses(size)

  if (urgency === 'none') return null

  return (
    <span className={`inline-flex items-center font-medium rounded-full ${colors} ${sizeClasses} ${className}`}>
      <ExpiryIcon urgency={urgency} size={size} />
      <span className="ml-1">
        {urgency === 'expired' && 'Expired'}
        {urgency === 'critical' && 'Expires Soon'}
        {urgency === 'warning' && 'Expiring'}
        {urgency === 'normal' && 'Valid'}
      </span>
    </span>
  )
}

// ============================================================================
// BANNER VARIANT
// ============================================================================

function ExpiryBanner({ 
  expiry, 
  showDetails, 
  className 
}: { 
  expiry: ReturnType<typeof usePermissionExpiry>
  showDetails: boolean
  className: string 
}) {
  const urgency = getOverallUrgency(expiry)
  const colors = getBannerColors(urgency)

  if (urgency === 'none') return null

  return (
    <div className={`p-4 rounded-md ${colors} ${className}`}>
      <div className="flex">
        <div className="flex-shrink-0">
          <ExpiryIcon urgency={urgency} size="md" />
        </div>
        <div className="ml-3">
          <h3 className="text-sm font-medium">
            {urgency === 'expired' && 'Permissions Expired'}
            {urgency === 'critical' && 'Permissions Expiring Imminently'}
            {urgency === 'warning' && 'Permissions Expiring Soon'}
            {urgency === 'normal' && 'Permissions Valid'}
          </h3>
          {showDetails && (
            <div className="mt-2 text-sm">
              <ExpiryDetails expiry={expiry} />
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

// ============================================================================
// CARD VARIANT
// ============================================================================

function ExpiryCard({ 
  expiry, 
  showDetails, 
  showHealth, 
  showPredictions, 
  className 
}: { 
  expiry: ReturnType<typeof usePermissionExpiry>
  showDetails: boolean
  showHealth: boolean
  showPredictions: boolean
  className: string 
}) {
  return (
    <div className={`bg-white dark:bg-gray-800 rounded-lg shadow p-6 ${className}`}>
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <ExpiryIcon urgency={getOverallUrgency(expiry)} size="lg" />
          <div className="ml-3">
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
              Permission Status
            </h3>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {getStatusText(expiry)}
            </p>
          </div>
        </div>
        <ExpiryBadge expiry={expiry} size="md" className="" />
      </div>

      {showHealth && (
        <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <PermissionHealthDisplay expiry={expiry} />
        </div>
      )}

      {showDetails && (
        <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <ExpiryDetails expiry={expiry} />
        </div>
      )}

      {showPredictions && expiry.tierPrediction.willChange && (
        <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
          <TierChangePrediction prediction={expiry.tierPrediction} />
        </div>
      )}
    </div>
  )
}

// ============================================================================
// INLINE VARIANT
// ============================================================================

function ExpiryInline({ 
  expiry, 
  size, 
  className 
}: { 
  expiry: ReturnType<typeof usePermissionExpiry>
  size: string
  className: string 
}) {
  const urgency = getOverallUrgency(expiry)
  if (urgency === 'none') return null

  const nextExpiry = expiry.nextExpiringPermission
  const timeText = nextExpiry ? expiry.formatTimeUntilExpiry(nextExpiry.expiresAt) : 'Never'

  return (
    <span className={`inline-flex items-center text-sm ${className}`}>
      <ExpiryIcon urgency={urgency} size={size} />
      <span className="ml-1 text-gray-600 dark:text-gray-400">
        {urgency === 'expired' ? 'Expired' : `Expires in ${timeText}`}
      </span>
    </span>
  )
}

// ============================================================================
// TOOLTIP VARIANT
// ============================================================================

function ExpiryTooltip({ 
  expiry, 
  size, 
  className 
}: { 
  expiry: ReturnType<typeof usePermissionExpiry>
  size: string
  className: string 
}) {
  const urgency = getOverallUrgency(expiry)
  if (urgency === 'none') return null

  return (
    <div className={`relative inline-block ${className}`} title={getTooltipText(expiry)}>
      <ExpiryIcon urgency={urgency} size={size} />
    </div>
  )
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

function ExpiryDetails({ expiry }: { expiry: ReturnType<typeof usePermissionExpiry> }) {
  return (
    <div className="space-y-2">
      {expiry.hasExpired && (
        <div>
          <strong>Expired ({expiry.expiryInfo.expired.length}):</strong>
          <ul className="mt-1 ml-4 text-xs">
            {expiry.expiryInfo.expired.slice(0, 3).map((tp, i) => (
              <li key={i}>• {tp.basePermission}</li>
            ))}
            {expiry.expiryInfo.expired.length > 3 && (
              <li>• ... and {expiry.expiryInfo.expired.length - 3} more</li>
            )}
          </ul>
        </div>
      )}
      
      {expiry.hasExpiringSoon && (
        <div>
          <strong>Expiring Soon ({expiry.expiryInfo.expiringSoon.length}):</strong>
          <ul className="mt-1 ml-4 text-xs">
            {expiry.expiryInfo.expiringSoon.slice(0, 3).map((tp, i) => (
              <li key={i}>
                • {tp.basePermission} ({expiry.formatTimeUntilExpiry(tp.expiresAt)})
              </li>
            ))}
            {expiry.expiryInfo.expiringSoon.length > 3 && (
              <li>• ... and {expiry.expiryInfo.expiringSoon.length - 3} more</li>
            )}
          </ul>
        </div>
      )}
      
      {expiry.nextExpiringPermission && (
        <div>
          <strong>Next Expiry:</strong> {expiry.nextExpiringPermission.basePermission} in {expiry.formatTimeUntilExpiry(expiry.nextExpiringPermission.expiresAt)}
        </div>
      )}
    </div>
  )
}

function PermissionHealthDisplay({ expiry }: { expiry: ReturnType<typeof usePermissionExpiry> }) {
  const health = expiry.healthSummary

  return (
    <div className="grid grid-cols-2 gap-4 text-sm">
      <div>
        <span className="text-gray-500 dark:text-gray-400">Total:</span>
        <span className="ml-2 font-medium">{health.total}</span>
      </div>
      <div>
        <span className="text-gray-500 dark:text-gray-400">Active:</span>
        <span className="ml-2 font-medium text-green-600">{health.active}</span>
      </div>
      <div>
        <span className="text-gray-500 dark:text-gray-400">Expired:</span>
        <span className="ml-2 font-medium text-red-600">{health.expired}</span>
      </div>
      <div>
        <span className="text-gray-500 dark:text-gray-400">Expiring:</span>
        <span className="ml-2 font-medium text-yellow-600">{health.expiringSoon}</span>
      </div>
      <div className="col-span-2">
        <span className="text-gray-500 dark:text-gray-400">Health:</span>
        <span className={`ml-2 font-medium capitalize ${getHealthColor(health.healthScore)}`}>
          {health.healthScore}
        </span>
      </div>
    </div>
  )
}

function TierChangePrediction({ prediction }: { prediction: ReturnType<typeof usePermissionExpiry>['tierPrediction'] }) {
  return (
    <div className="text-sm">
      <h4 className="font-medium text-gray-900 dark:text-gray-100">Tier Changes</h4>
      <div className="mt-2 space-y-1">
        <div>
          <span className="text-gray-500 dark:text-gray-400">Current:</span>
          <span className="ml-2 font-medium">{prediction.currentTier}</span>
        </div>
        <div>
          <span className="text-gray-500 dark:text-gray-400">Will become:</span>
          <span className="ml-2 font-medium">{prediction.futureTier}</span>
        </div>
        {prediction.changeTime && (
          <div>
            <span className="text-gray-500 dark:text-gray-400">When:</span>
            <span className="ml-2 font-medium">{prediction.changeTime.toLocaleString()}</span>
          </div>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function ExpiryIcon({ urgency, size }: { urgency: string, size: string }) {
  const sizeClasses = {
    xs: 'h-3 w-3',
    sm: 'h-4 w-4',
    md: 'h-5 w-5',
    lg: 'h-6 w-6'
  }[size] || 'h-4 w-4'

  const colorClass = {
    expired: 'text-red-500',
    critical: 'text-red-400',
    warning: 'text-yellow-500',
    normal: 'text-green-500',
    none: 'text-gray-400'
  }[urgency] || 'text-gray-400'

  if (urgency === 'expired' || urgency === 'critical') {
    return (
      <svg className={`${sizeClasses} ${colorClass}`} fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
      </svg>
    )
  }

  if (urgency === 'warning') {
    return (
      <svg className={`${sizeClasses} ${colorClass}`} fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
      </svg>
    )
  }

  return (
    <svg className={`${sizeClasses} ${colorClass}`} fill="currentColor" viewBox="0 0 20 20">
      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
    </svg>
  )
}

function getOverallUrgency(expiry: ReturnType<typeof usePermissionExpiry>): 'expired' | 'critical' | 'warning' | 'normal' | 'none' {
  if (expiry.hasExpired) return 'expired'
  
  if (expiry.nextExpiringPermission) {
    const urgency = expiry.getExpiryUrgency(expiry.nextExpiringPermission.expiresAt)
    if (urgency === 'expired' || urgency === 'critical') return 'critical'
    if (urgency === 'warning') return 'warning'
  }
  
  if (expiry.hasExpiringSoon) return 'warning'
  if (expiry.allPermissionsWithExpiry.length > 0) return 'normal'
  
  return 'none'
}

function getUrgencyColors(urgency: string): string {
  return {
    expired: 'bg-red-100 text-red-800 border border-red-200 shadow-red-100/50 shadow-sm dark:bg-red-900 dark:text-red-200 dark:border-red-800 animate-pulse',
    critical: 'bg-red-100 text-red-800 border border-red-200 shadow-red-100/50 shadow-sm dark:bg-red-900 dark:text-red-200 dark:border-red-800 animate-pulse',
    warning: 'bg-yellow-100 text-yellow-800 border border-yellow-200 shadow-yellow-100/50 shadow-sm dark:bg-yellow-900 dark:text-yellow-200 dark:border-yellow-800',
    normal: 'bg-green-100 text-green-800 border border-green-200 shadow-green-100/50 shadow-sm dark:bg-green-900 dark:text-green-200 dark:border-green-800',
    none: 'bg-gray-100 text-gray-800 border border-gray-200 shadow-gray-100/50 shadow-sm dark:bg-gray-900 dark:text-gray-200 dark:border-gray-800'
  }[urgency] || 'bg-gray-100 text-gray-800 border border-gray-200 dark:bg-gray-900 dark:text-gray-200'
}

function getBannerColors(urgency: string): string {
  return {
    expired: 'bg-red-50 text-red-800 border border-red-200 shadow-red-100/50 shadow-lg dark:bg-red-900/20 dark:text-red-200 dark:border-red-800 border-l-4 border-l-red-500',
    critical: 'bg-red-50 text-red-800 border border-red-200 shadow-red-100/50 shadow-lg dark:bg-red-900/20 dark:text-red-200 dark:border-red-800 border-l-4 border-l-red-500 animate-pulse',
    warning: 'bg-yellow-50 text-yellow-800 border border-yellow-200 shadow-yellow-100/50 shadow-md dark:bg-yellow-900/20 dark:text-yellow-200 dark:border-yellow-800 border-l-4 border-l-yellow-500',
    normal: 'bg-green-50 text-green-800 border border-green-200 shadow-green-100/50 shadow-md dark:bg-green-900/20 dark:text-green-200 dark:border-green-800 border-l-4 border-l-green-500',
    none: 'bg-gray-50 text-gray-800 border border-gray-200 shadow-gray-100/50 shadow-sm dark:bg-gray-900/20 dark:text-gray-200 dark:border-gray-800'
  }[urgency] || 'bg-gray-50 text-gray-800 border border-gray-200'
}

function getSizeClasses(size: string): string {
  return {
    xs: 'px-2 py-1 text-xs',
    sm: 'px-2.5 py-1.5 text-xs',
    md: 'px-3 py-2 text-sm',
    lg: 'px-4 py-2 text-base'
  }[size] || 'px-2.5 py-1.5 text-xs'
}

function getHealthColor(health: string): string {
  return {
    excellent: 'text-green-600',
    good: 'text-blue-600',
    warning: 'text-yellow-600',
    critical: 'text-red-600'
  }[health] || 'text-gray-600'
}

function getStatusText(expiry: ReturnType<typeof usePermissionExpiry>): string {
  if (expiry.hasExpired) {
    return `${expiry.expiryInfo.expired.length} permission${expiry.expiryInfo.expired.length !== 1 ? 's' : ''} expired`
  }
  if (expiry.hasExpiringSoon) {
    return `${expiry.expiryInfo.expiringSoon.length} permission${expiry.expiryInfo.expiringSoon.length !== 1 ? 's' : ''} expiring soon`
  }
  return 'All permissions valid'
}

function getTooltipText(expiry: ReturnType<typeof usePermissionExpiry>): string {
  if (expiry.hasExpired) {
    return `${expiry.expiryInfo.expired.length} expired permissions`
  }
  if (expiry.hasExpiringSoon) {
    const next = expiry.nextExpiringPermission
    if (next) {
      return `${expiry.expiryInfo.expiringSoon.length} expiring soon. Next: ${next.basePermission} in ${expiry.formatTimeUntilExpiry(next.expiresAt)}`
    }
    return `${expiry.expiryInfo.expiringSoon.length} permissions expiring soon`
  }
  return 'Permissions valid'
}

// ============================================================================
// SPECIALIZED COMPONENTS
// ============================================================================

export function RankingExpiryIndicator({ className }: { className?: string }) {
  return (
    <PermissionExpiryIndicator 
      permission="epsx:rankings:view"
      variant="inline"
      size="sm"
      className={className}
    />
  )
}

export function AnalyticsExpiryIndicator({ className }: { className?: string }) {
  return (
    <PermissionExpiryIndicator 
      permission="epsx:analytics:view"
      variant="badge"
      size="sm"
      className={className}
    />
  )
}

export function GlobalExpiryBanner() {
  return (
    <PermissionExpiryIndicator 
      variant="banner"
      showDetails={true}
      className="mb-4"
    />
  )
}

export function PermissionHealthCard() {
  return (
    <PermissionExpiryIndicator 
      variant="card"
      showHealth={true}
      showDetails={true}
      showPredictions={true}
    />
  )
}