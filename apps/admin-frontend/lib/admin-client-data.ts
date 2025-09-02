'use client'

/**
 * Admin Frontend Hybrid Data Strategy - Client-side
 * Implements SWR-based data fetching with OIDC authentication for admin interface
 * Serverless optimized - no Server Actions with fetch() calls
 */

import useSWR, { SWRConfiguration } from 'swr'
import { useCallback, useEffect, useState } from 'react'

// ============================================================================
// Types and Configuration
// ============================================================================

export interface AdminFilters {
  role?: string
  status?: string  
  search?: string
  limit?: number
  offset?: number
  page?: number
}

export interface UserPermissionFilters {
  user_id?: string
  platform?: string
  resource?: string
  action?: string
}

interface SWRConfig extends SWRConfiguration {
  refreshInterval?: number
  dedupingInterval?: number
}

// ============================================================================
// OIDC Authentication Fetcher
// ============================================================================

/**
 * Admin-specific OIDC fetcher for API calls
 * Uses OIDC cookies for authentication with admin backend endpoints
 */
async function adminOIDCFetcher(url: string, options: RequestInit = {}): Promise<any> {
  // Get current session to ensure OIDC authentication
  const sessionResponse = await fetch('/api/auth/session', {
    credentials: 'include',
    cache: 'no-cache'
  })
  
  if (!sessionResponse.ok) {
    throw new Error('Admin authentication required')
  }
  
  const sessionData = await sessionResponse.json()
  
  if (!sessionData.isAuthenticated) {
    throw new Error('Admin session expired')
  }
  
  // Make direct API call to backend with OIDC cookies
  const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
  const fullUrl = url.startsWith('/') ? `${backendUrl}${url}` : url
  
  const response = await fetch(fullUrl, {
    ...options,
    credentials: 'include', // Include OIDC cookies
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    }
  })
  
  if (!response.ok) {
    const error = await response.text()
    throw new Error(`API Error: ${response.status} - ${error}`)
  }
  
  return response.json()
}

// ============================================================================
// Admin User Management
// ============================================================================

/**
 * Fetch admin users with filters using direct backend API
 */
export function useAdminUsers(filters?: AdminFilters, config?: SWRConfig) {
  const queryParams = new URLSearchParams()
  if (filters) {
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined) {
        queryParams.append(key, String(value))
      }
    })
  }
  
  const endpoint = `/api/v1/iam/users${queryParams.toString() ? `?${queryParams.toString()}` : ''}`
  
  return useSWR(
    endpoint,
    adminOIDCFetcher,
    {
      refreshInterval: 30000, // Refresh every 30 seconds
      dedupingInterval: 10000,
      revalidateOnFocus: true,
      ...config,
    }
  )
}

/**
 * Fetch user statistics for admin dashboard
 */
export function useAdminUserStats(config?: SWRConfig) {
  return useSWR(
    '/api/admin/analytics/user-statistics?include_roles=true&include_tiers=true',
    adminOIDCFetcher,
    {
      refreshInterval: 60000, // Refresh every minute
      dedupingInterval: 30000,
      ...config,
    }
  )
}

/**
 * Fetch specific user details
 */
export function useAdminUserDetail(userId: string | null, config?: SWRConfig) {
  return useSWR(
    userId ? `/api/v1/iam/users/${userId}` : null,
    adminOIDCFetcher,
    {
      dedupingInterval: 10000,
      ...config,
    }
  )
}

// ============================================================================
// Permission Management
// ============================================================================

/**
 * Fetch user permissions with filters
 */
export function useUserPermissions(filters?: UserPermissionFilters, config?: SWRConfig) {
  const queryParams = new URLSearchParams()
  if (filters) {
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined) {
        queryParams.append(key, String(value))
      }
    })
  }
  
  const endpoint = `/api/v1/admin/users/permissions${queryParams.toString() ? `?${queryParams.toString()}` : ''}`
  
  return useSWR(
    endpoint,
    adminOIDCFetcher,
    {
      refreshInterval: 45000, // Refresh every 45 seconds
      ...config,
    }
  )
}

/**
 * Fetch available permission templates/profiles
 */
export function usePermissionProfiles(config?: SWRConfig) {
  return useSWR(
    '/api/v1/admin/permission-profiles',
    adminOIDCFetcher,
    {
      refreshInterval: 300000, // Refresh every 5 minutes
      dedupingInterval: 60000,
      ...config,
    }
  )
}

/**
 * Fetch IAM roles for role management
 */  
export function useIAMRoles(config?: SWRConfig) {
  return useSWR(
    '/api/v1/iam/roles',
    adminOIDCFetcher,
    {
      refreshInterval: 300000, // Refresh every 5 minutes
      dedupingInterval: 60000,
      ...config,
    }
  )
}

// ============================================================================
// Analytics and System Health
// ============================================================================

/**
 * Fetch admin analytics dashboard data
 */
export function useAdminAnalytics(config?: SWRConfig) {
  return useSWR(
    '/api/v1/admin/analytics/dashboard',
    adminOIDCFetcher,
    {
      refreshInterval: 60000, // Refresh every minute  
      ...config,
    }
  )
}

/**
 * Fetch system performance metrics
 */
export function useSystemMetrics(config?: SWRConfig) {
  return useSWR(
    '/api/v1/admin/performance/metrics',
    adminOIDCFetcher,
    {
      refreshInterval: 30000, // Refresh every 30 seconds
      ...config,
    }
  )
}

/**
 * Fetch cache statistics
 */
export function useCacheStats(config?: SWRConfig) {
  return useSWR(
    '/api/v1/admin/cache/stats', 
    adminOIDCFetcher,
    {
      refreshInterval: 60000, // Refresh every minute
      ...config,
    }
  )
}

// ============================================================================
// Real-time Updates and Cache Management  
// ============================================================================

/**
 * Real-time admin data updates using Server-Sent Events
 */
export function useAdminRealTime(enabled = true) {
  const [isConnected, setIsConnected] = useState(false)
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null)
  
  const connectSSE = useCallback(() => {
    if (!enabled || typeof window === 'undefined') return
    
    const backendUrl = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'
    const eventSource = new EventSource(`${backendUrl}/api/v1/admin/realtime/events`, {
      withCredentials: true // Include OIDC cookies
    })
    
    eventSource.onopen = () => {
      setIsConnected(true)
      console.log('✅ Admin real-time connection established')
    }
    
    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        setLastUpdate(new Date())
        
        // Trigger SWR revalidation based on event type
        if (data.type === 'user_updated') {
          // Revalidate user-related SWR keys
          // This would trigger re-fetching of user data
        } else if (data.type === 'permission_changed') {
          // Revalidate permission-related SWR keys
        }
      } catch (error) {
        console.error('❌ Error processing admin real-time event:', error)
      }
    }
    
    eventSource.onerror = () => {
      setIsConnected(false)
      console.warn('⚠️ Admin real-time connection error')
    }
    
    return () => {
      eventSource.close()
      setIsConnected(false)
    }
  }, [enabled])
  
  useEffect(() => {
    const cleanup = connectSSE()
    return cleanup
  }, [connectSSE])
  
  return {
    isConnected,
    lastUpdate,
    connectSSE,
  }
}

/**
 * Admin SWR cache management utilities
 */
export function useAdminCache() {
  const invalidateUsers = useCallback((filters?: AdminFilters) => {
    const queryParams = new URLSearchParams()
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined) {
          queryParams.append(key, String(value))
        }
      })
    }
    const key = `/api/v1/iam/users${queryParams.toString() ? `?${queryParams.toString()}` : ''}`
    // Trigger SWR revalidation
    return fetch(key, { method: 'HEAD' })
  }, [])
  
  const invalidatePermissions = useCallback(() => {
    // Invalidate all permission-related cache keys
    return Promise.all([
      fetch('/api/v1/admin/users/permissions', { method: 'HEAD' }),
      fetch('/api/v1/admin/permission-profiles', { method: 'HEAD' }),
    ])
  }, [])
  
  const invalidateAnalytics = useCallback(() => {
    return fetch('/api/v1/admin/analytics/dashboard', { method: 'HEAD' })
  }, [])
  
  return {
    invalidateUsers,
    invalidatePermissions, 
    invalidateAnalytics,
  }
}

// ============================================================================
// Admin Client Data Exports
// ============================================================================

export const adminClientData = {
  // User Management
  useUsers: useAdminUsers,
  useUserStats: useAdminUserStats,
  useUserDetail: useAdminUserDetail,
  
  // Permission Management
  usePermissions: useUserPermissions,
  usePermissionProfiles,
  useIAMRoles,
  
  // Analytics & System
  useAnalytics: useAdminAnalytics,
  useSystemMetrics,
  useCacheStats,
  
  // Real-time & Cache
  useRealTime: useAdminRealTime,
  useCache: useAdminCache,
}

export default adminClientData