// ============================================================================
// FRONTEND PERMISSION GUARDS
// ============================================================================
// Frontend-specific permission guards using shared permission system

'use client'

import React from 'react'
import { 
  createBasePermissionGuard,
  BasePermissionGuardProps,
  BaseGuardContext 
} from '@/shared/permissions/guards'
import { FrontendUserClaims, FrontendGuardOptions } from './types'
import { frontendPermissionApiClient } from './api-client'
import { useFrontendGranularPermissions } from './hooks'

// ============================================================================
// FRONTEND GUARD CONTEXT
// ============================================================================

const frontendGuardContext: BaseGuardContext = {
  apiClient: frontendPermissionApiClient,
  usePermissions: useFrontendGranularPermissions,
  defaultErrorMessage: 'You need appropriate permissions to access this feature.',
  defaultUpgradeMessage: 'Upgrade your plan to access this feature.',
  getLoadingComponent: () => <div className="p-4 text-center text-sm text-muted-foreground">Loading permissions...</div>,
  getErrorComponent: (message: string) => (
    <div className="p-4 text-center border border-red-200 bg-red-50 rounded-lg">
      <p className="text-sm text-red-600">{message}</p>
    </div>
  ),
  getAccessDeniedComponent: (message: string, upgradeMessage?: string, canUpgrade?: boolean) => (
    <div className="p-6 text-center border border-amber-200 bg-amber-50 rounded-lg">
      <div className="space-y-3">
        <p className="text-sm text-amber-800 font-medium">{message}</p>
        {canUpgrade && upgradeMessage && (
          <>
            <p className="text-xs text-amber-700">{upgradeMessage}</p>
            <button 
              className="inline-flex items-center px-3 py-1.5 text-xs font-medium text-amber-900 bg-amber-100 hover:bg-amber-200 border border-amber-300 rounded-md transition-colors"
              onClick={() => window.location.href = '/billing'}
            >
              Upgrade Plan
            </button>
          </>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// BASE FRONTEND PERMISSION GUARD
// ============================================================================

interface FrontendPermissionGuardProps extends BasePermissionGuardProps {
  options?: FrontendGuardOptions
}

export const FrontendPermissionGuard = createBasePermissionGuard(
  frontendGuardContext,
  {
    trackAnalytics: true,
    showUpgradePrompt: true,
    enableLegacySupport: true
  }
)

// ============================================================================
// SPECIALIZED FRONTEND GUARDS
// ============================================================================

interface SinglePermissionGuardProps {
  permission: string
  children: React.ReactNode
  fallback?: React.ReactNode
  showUpgradePrompt?: boolean
  trackAnalytics?: boolean
}

export function RequireFrontendPermission({ 
  permission, 
  children, 
  fallback,
  showUpgradePrompt = true,
  trackAnalytics = true
}: SinglePermissionGuardProps) {
  return (
    <FrontendPermissionGuard
      permission={permission}
      fallback={fallback}
      options={{ showUpgradePrompt, trackAnalytics, enableLegacySupport: true }}
    >
      {children}
    </FrontendPermissionGuard>
  )
}

interface MultiplePermissionGuardProps {
  permissions: string[]
  requireAll?: boolean
  children: React.ReactNode
  fallback?: React.ReactNode
  showUpgradePrompt?: boolean
}

export function RequireAnyFrontendPermission({ 
  permissions, 
  children, 
  fallback,
  showUpgradePrompt = true
}: MultiplePermissionGuardProps) {
  return (
    <FrontendPermissionGuard
      permissions={permissions}
      requireAll={false}
      fallback={fallback}
      options={{ showUpgradePrompt, trackAnalytics: true, enableLegacySupport: true }}
    >
      {children}
    </FrontendPermissionGuard>
  )
}

export function RequireAllFrontendPermissions({ 
  permissions, 
  children, 
  fallback,
  showUpgradePrompt = true
}: MultiplePermissionGuardProps) {
  return (
    <FrontendPermissionGuard
      permissions={permissions}
      requireAll={true}
      fallback={fallback}
      options={{ showUpgradePrompt, trackAnalytics: true, enableLegacySupport: true }}
    >
      {children}
    </FrontendPermissionGuard>
  )
}

// ============================================================================
// FEATURE-SPECIFIC GUARDS
// ============================================================================

export function RequireAnalyticsAccess({ 
  children, 
  fallback,
  showUpgradePrompt = true
}: Omit<SinglePermissionGuardProps, 'permission'>) {
  return (
    <RequireAnyFrontendPermission
      permissions={[
        'epsx:analytics:view',
        'epsx:analytics:basic',
        'epsx:analytics:premium',
        'epsx:analytics:professional'
      ]}
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
    >
      {children}
    </RequireAnyFrontendPermission>
  )
}

export function RequireExportAccess({ 
  children, 
  fallback,
  showUpgradePrompt = true
}: Omit<SinglePermissionGuardProps, 'permission'>) {
  return (
    <RequireAnyFrontendPermission
      permissions={[
        'epsx:export:csv',
        'epsx:export:excel',
        'epsx:export:pdf',
        'epsx:export:unlimited'
      ]}
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
    >
      {children}
    </RequireAnyFrontendPermission>
  )
}

export function RequireRealtimeAccess({ 
  children, 
  fallback,
  showUpgradePrompt = true
}: Omit<SinglePermissionGuardProps, 'permission'>) {
  return (
    <RequireFrontendPermission
      permission="epsx:realtime:access"
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
      trackAnalytics={true}
    >
      {children}
    </RequireFrontendPermission>
  )
}

export function RequireAdvancedFilters({ 
  children, 
  fallback,
  showUpgradePrompt = true
}: Omit<SinglePermissionGuardProps, 'permission'>) {
  return (
    <RequireFrontendPermission
      permission="epsx:filters:advanced"
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
      trackAnalytics={true}
    >
      {children}
    </RequireFrontendPermission>
  )
}

export function RequireProfileManagement({ 
  children, 
  fallback 
}: Omit<SinglePermissionGuardProps, 'permission' | 'showUpgradePrompt'>) {
  return (
    <RequireFrontendPermission
      permission="epsx:profile:manage"
      fallback={fallback}
      showUpgradePrompt={false} // Profile management shouldn't show upgrade prompt
      trackAnalytics={false}
    >
      {children}
    </RequireFrontendPermission>
  )
}

export function RequireBillingAccess({ 
  children, 
  fallback 
}: Omit<SinglePermissionGuardProps, 'permission' | 'showUpgradePrompt'>) {
  return (
    <RequireFrontendPermission
      permission="epsx:billing:manage"
      fallback={fallback}
      showUpgradePrompt={false}
      trackAnalytics={false}
    >
      {children}
    </RequireFrontendPermission>
  )
}

// ============================================================================
// TIER-BASED GUARDS (Legacy Support)
// ============================================================================

interface TierGuardProps {
  requiredTier: 'free' | 'basic' | 'premium' | 'professional'
  children: React.ReactNode
  fallback?: React.ReactNode
  showUpgradePrompt?: boolean
}

export function RequireTierAccess({
  requiredTier,
  children,
  fallback,
  showUpgradePrompt = true
}: TierGuardProps) {
  // Map tiers to permission patterns
  const tierPermissions: Record<string, string[]> = {
    free: ['epsx:tier:free'],
    basic: ['epsx:tier:basic', 'epsx:tier:premium', 'epsx:tier:professional'],
    premium: ['epsx:tier:premium', 'epsx:tier:professional'],
    professional: ['epsx:tier:professional']
  }

  return (
    <RequireAnyFrontendPermission
      permissions={tierPermissions[requiredTier] || []}
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
    >
      {children}
    </RequireAnyFrontendPermission>
  )
}

// ============================================================================
// RANKING-BASED GUARDS (Legacy Support)
// ============================================================================

interface RankingGuardProps {
  minimumLimit?: number
  requireUnlimited?: boolean
  children: React.ReactNode
  fallback?: React.ReactNode
  showUpgradePrompt?: boolean
}

export function RequireRankingAccess({
  minimumLimit = 1,
  requireUnlimited = false,
  children,
  fallback,
  showUpgradePrompt = true
}: RankingGuardProps) {
  const { hasPermission } = useFrontendGranularPermissions()
  
  // Check for unlimited ranking first
  if (requireUnlimited) {
    return (
      <RequireFrontendPermission
        permission="epsx:ranking:unlimited"
        fallback={fallback}
        showUpgradePrompt={showUpgradePrompt}
        trackAnalytics={true}
      >
        {children}
      </RequireFrontendPermission>
    )
  }

  // Check for minimum ranking limit
  const rankingPermissions = [
    'epsx:ranking:unlimited',
    'epsx:ranking:1000',
    'epsx:ranking:500',
    'epsx:ranking:100',
    'epsx:ranking:50',
    'epsx:ranking:25'
  ].filter(permission => {
    if (permission === 'epsx:ranking:unlimited') return true
    const limit = parseInt(permission.split(':')[2])
    return limit >= minimumLimit
  })

  return (
    <RequireAnyFrontendPermission
      permissions={rankingPermissions}
      fallback={fallback}
      showUpgradePrompt={showUpgradePrompt}
    >
      {children}
    </RequireAnyFrontendPermission>
  )
}

// ============================================================================
// EXPORTS
// ============================================================================

export { FrontendPermissionGuard as default }

// Backward compatibility
export { FrontendPermissionGuard as GranularPermissionGuard }
export { RequireFrontendPermission as RequirePermission }
export { RequireAnyFrontendPermission as RequireAnyPermission }
export { RequireAllFrontendPermissions as RequireAllPermissions }