'use client'

import { useState, useCallback, useMemo } from 'react'
import useSWR, { mutate } from 'swr'

// ============================================================================
// TYPES - Match backend embedded permission types
// ============================================================================

export interface PermissionStats {
  totalPermissions: number
  activeUsers: number
  expiring: number
  expired: number
  recentActivity: number
  bulkOperations: number
  platforms: string[]
  healthScore: number
}

export interface PermissionExpiryInfo {
  permission: string
  base_permission: string
  expires_at?: number
  is_expired: boolean
  time_remaining?: number // milliseconds
  expires_in?: string // human readable
}

export interface ExpiryStatusResponse {
  user_id: string
  permissions: PermissionExpiryInfo[]
  health: ExpiryHealthInfo
}

export interface ExpiryHealthInfo {
  has_expired: boolean
  has_expiring_soon: boolean
  next_expiry?: number
  time_until_next_expiry?: number
}

export interface EmbeddedPermissionRequest {
  embedded_permission: string
  base_permission: string
  platform: string
  resource: string
  action: string
  expiry_timestamp: number
  reason?: string
  metadata?: Record<string, any>
}

export interface BulkPermissionRequest {
  user_ids: string[]
  permissions: Array<{
    base_permission: string
    platform: string
    resource: string
    action: string
    expiry_timestamp: number
  }>
  reason?: string
  metadata?: Record<string, any>
}

export interface ValidationResult {
  valid: string[]
  expired: Array<{
    permission: string
    base_permission: string
    expired_at: number
    expired_for: number
  }>
  expiring_soon: Array<{
    permission: string
    base_permission: string
    expires_at: number
    expires_in: number
  }>
  summary: {
    total: number
    valid_count: number
    expired_count: number
    expiring_soon_count: number
  }
}

export interface CleanupResponse {
  cleaned: number
  failed: number
  details: Array<{
    user_id: string
    permission: string
    expired_at: number
    status: 'cleaned' | 'failed'
    error?: string
  }>
}

// ============================================================================
// API CONFIGURATION
// ============================================================================

const API_BASE = '/api/v1/admin'

// Generic fetcher with error handling
const fetcher = async (url: string, options?: RequestInit) => {
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  })

  if (!response.ok) {
    const error = new Error(`HTTP ${response.status}: ${response.statusText}`)
    const errorData = await response.json().catch(() => ({}))
    Object.assign(error, { status: response.status, data: errorData })
    throw error
  }

  return response.json()
}

// ============================================================================
// API FUNCTIONS
// ============================================================================

// Real API functions that match backend endpoints
const permissionApi = {
  // Get overall permission statistics (would need to be implemented in backend)
  getStats: async (): Promise<PermissionStats> => {
    // This would be a new endpoint: GET /api/v1/admin/analytics/permission-stats
    // For now, return mock data with note
    console.warn('🔴 MOCKUP: Permission stats endpoint needs backend implementation')
    
    // Mock data with realistic values
    return {
      totalPermissions: 1847 + Math.floor(Math.random() * 50),
      activeUsers: 234 + Math.floor(Math.random() * 10),
      expiring: Math.floor(Math.random() * 15),
      expired: Math.floor(Math.random() * 5),
      recentActivity: 47 + Math.floor(Math.random() * 20),
      bulkOperations: Math.floor(Math.random() * 3),
      platforms: ['epsx', 'admin', 'epsx-pay'],
      healthScore: 85 + Math.floor(Math.random() * 15)
    }
  },

  // Grant embedded permission (REAL ENDPOINT)
  grantPermission: async (userId: string, request: EmbeddedPermissionRequest) => {
    return fetcher(`${API_BASE}/users/${userId}/embedded-permissions`, {
      method: 'POST',
      body: JSON.stringify(request)
    })
  },

  // Bulk grant permissions (REAL ENDPOINT)
  bulkGrantPermissions: async (request: BulkPermissionRequest) => {
    return fetcher(`${API_BASE}/users/bulk/embedded-permissions`, {
      method: 'POST',
      body: JSON.stringify(request)
    })
  },

  // Get permission expiry status (REAL ENDPOINT)
  getExpiryStatus: async (userId: string): Promise<ExpiryStatusResponse> => {
    return fetcher(`${API_BASE}/users/${userId}/permissions/expiry-status`)
  },

  // Validate permissions (REAL ENDPOINT)
  validatePermissions: async (userId: string, permissions: string[]): Promise<ValidationResult> => {
    return fetcher(`${API_BASE}/users/${userId}/embedded-permissions/validate`, {
      method: 'POST',
      body: JSON.stringify({ permissions })
    })
  },

  // Extend permission (REAL ENDPOINT)
  extendPermission: async (userId: string, permission: string, newExpiryTimestamp: number) => {
    return fetcher(`${API_BASE}/users/${userId}/embedded-permissions/extend`, {
      method: 'POST',
      body: JSON.stringify({
        permission,
        new_expiry_timestamp: newExpiryTimestamp,
        reason: 'Extended via admin interface'
      })
    })
  },

  // Revoke permission (REAL ENDPOINT)
  revokePermission: async (userId: string, permission: string) => {
    return fetcher(`${API_BASE}/users/${userId}/embedded-permissions/revoke`, {
      method: 'POST',
      body: JSON.stringify({
        permission,
        reason: 'Revoked via admin interface'
      })
    })
  },

  // Cleanup expired permissions (REAL ENDPOINT)
  cleanupExpired: async (dryRun: boolean = false): Promise<CleanupResponse> => {
    return fetcher(`${API_BASE}/embedded-permissions/cleanup-expired`, {
      method: 'POST',
      body: JSON.stringify({
        dry_run: dryRun,
        batch_size: 100
      })
    })
  }
}

// ============================================================================
// CUSTOM HOOKS
// ============================================================================

export function usePermissionStats(refreshInterval: number = 30000) {
  const { data, error, isLoading, mutate: refresh } = useSWR(
    'permission-stats',
    permissionApi.getStats,
    {
      refreshInterval,
      revalidateOnFocus: true,
      revalidateOnReconnect: true,
      errorRetryCount: 3,
      errorRetryInterval: 5000
    }
  )

  return {
    stats: data,
    isLoading,
    error,
    refresh
  }
}

export function useUserPermissionExpiry(userId: string | null) {
  const { data, error, isLoading, mutate: refresh } = useSWR(
    userId ? `user-permission-expiry-${userId}` : null,
    () => userId ? permissionApi.getExpiryStatus(userId) : null,
    {
      refreshInterval: 60000, // 1 minute for expiry data
      revalidateOnFocus: true
    }
  )

  return {
    expiryData: data,
    isLoading,
    error,
    refresh
  }
}

export function usePermissionManagement() {
  const [isProcessing, setIsProcessing] = useState(false)
  const [lastOperation, setLastOperation] = useState<string | null>(null)

  // Grant permission with optimistic updates
  const grantPermission = useCallback(async (
    userId: string, 
    request: EmbeddedPermissionRequest
  ) => {
    setIsProcessing(true)
    setLastOperation('grant')
    
    try {
      const result = await permissionApi.grantPermission(userId, request)
      
      // Invalidate related caches
      await Promise.all([
        mutate('permission-stats'),
        mutate(`user-permission-expiry-${userId}`)
      ])
      
      return result
    } finally {
      setIsProcessing(false)
    }
  }, [])

  // Bulk grant permissions
  const bulkGrantPermissions = useCallback(async (request: BulkPermissionRequest) => {
    setIsProcessing(true)
    setLastOperation('bulk-grant')
    
    try {
      const result = await permissionApi.bulkGrantPermissions(request)
      
      // Invalidate stats cache
      await mutate('permission-stats')
      
      return result
    } finally {
      setIsProcessing(false)
    }
  }, [])

  // Extend permission
  const extendPermission = useCallback(async (
    userId: string, 
    permission: string, 
    newExpiryTimestamp: number
  ) => {
    setIsProcessing(true)
    setLastOperation('extend')
    
    try {
      const result = await permissionApi.extendPermission(userId, permission, newExpiryTimestamp)
      
      // Invalidate related caches
      await Promise.all([
        mutate('permission-stats'),
        mutate(`user-permission-expiry-${userId}`)
      ])
      
      return result
    } finally {
      setIsProcessing(false)
    }
  }, [])

  // Revoke permission
  const revokePermission = useCallback(async (userId: string, permission: string) => {
    setIsProcessing(true)
    setLastOperation('revoke')
    
    try {
      const result = await permissionApi.revokePermission(userId, permission)
      
      // Invalidate related caches
      await Promise.all([
        mutate('permission-stats'),
        mutate(`user-permission-expiry-${userId}`)
      ])
      
      return result
    } finally {
      setIsProcessing(false)
    }
  }, [])

  // Cleanup expired permissions
  const cleanupExpired = useCallback(async (dryRun: boolean = false) => {
    setIsProcessing(true)
    setLastOperation('cleanup')
    
    try {
      const result = await permissionApi.cleanupExpired(dryRun)
      
      // Invalidate stats cache
      await mutate('permission-stats')
      
      return result
    } finally {
      setIsProcessing(false)
    }
  }, [])

  // Validate permissions for a user
  const validatePermissions = useCallback(async (userId: string, permissions?: string[]) => {
    return permissionApi.validatePermissions(userId, permissions || [])
  }, [])

  return {
    // State
    isProcessing,
    lastOperation,
    
    // Actions
    grantPermission,
    bulkGrantPermissions,
    extendPermission,
    revokePermission,
    cleanupExpired,
    validatePermissions
  }
}

// ============================================================================
// UTILITY HOOKS
// ============================================================================

export function usePermissionTemplates() {
  // Common permission templates for quick setup
  const templates = useMemo(() => [
    {
      id: 'admin-full',
      name: 'Full Admin Access',
      permissions: [
        { platform: 'admin', resource: '*', action: '*' }
      ],
      duration: 30 * 24 * 60 * 60, // 30 days in seconds
      description: 'Complete administrative access to all systems'
    },
    {
      id: 'user-manager',
      name: 'User Manager',
      permissions: [
        { platform: 'admin', resource: 'users', action: 'manage' },
        { platform: 'admin', resource: 'users', action: 'view' }
      ],
      duration: 7 * 24 * 60 * 60, // 7 days
      description: 'Can manage users but not system configuration'
    },
    {
      id: 'analytics-viewer',
      name: 'Analytics Viewer',
      permissions: [
        { platform: 'epsx', resource: 'analytics', action: 'view' },
        { platform: 'admin', resource: 'analytics', action: 'view' }
      ],
      duration: 24 * 60 * 60, // 1 day
      description: 'Read-only access to analytics data'
    },
    {
      id: 'temporary-support',
      name: 'Temporary Support Access',
      permissions: [
        { platform: 'admin', resource: 'users', action: 'view' },
        { platform: 'epsx', resource: 'support', action: 'access' }
      ],
      duration: 4 * 60 * 60, // 4 hours
      description: 'Limited support access for troubleshooting'
    }
  ], [])

  const createFromTemplate = useCallback((templateId: string, userIds: string[]) => {
    const template = templates.find(t => t.id === templateId)
    if (!template) return null

    const expiryTimestamp = Math.floor(Date.now() / 1000) + template.duration

    return {
      user_ids: userIds,
      permissions: template.permissions.map(p => ({
        base_permission: `${p.platform}:${p.resource}:${p.action}`,
        platform: p.platform,
        resource: p.resource,
        action: p.action,
        expiry_timestamp: expiryTimestamp
      })),
      reason: `Applied template: ${template.name}`
    }
  }, [templates])

  return {
    templates,
    createFromTemplate
  }
}

export default usePermissionManagement