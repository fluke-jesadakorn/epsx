'use client'

import { useAuth } from '@/lib/auth'
import { useCallback, useEffect, useState, useMemo } from 'react'
import { 
  getPermissionExpiryInfo,
  getTimeUntilNextExpiry,
  filterValidPermissions,
  getAllPermissionsWithExpiry,
  willPermissionsChangeSoon,
  getEffectivePermissionsAtTime,
  predictTierChanges,
  getPermissionHealthSummary,
  hasExpiringSoonRankingPermissions,
  getNextExpiringRankingPermission,
  type TimestampedPermission,
  type PermissionExpiryInfo
} from '@/types/permissions'
import { deriveTierFromPermissions, type UserLevelType } from '@/lib/permission-utils'

export interface PermissionExpiryHookReturn {
  // Expiry information
  expiryInfo: PermissionExpiryInfo
  healthSummary: ReturnType<typeof getPermissionHealthSummary>
  timeUntilNextExpiry: number | null
  
  // Permission states
  hasExpired: boolean
  hasExpiringSoon: boolean
  hasExpiringSoonRanking: boolean
  nextExpiringPermission: TimestampedPermission | null
  nextExpiringRanking: TimestampedPermission | null
  
  // Predictions
  willChangeSoon: boolean
  tierPrediction: ReturnType<typeof predictTierChanges>
  
  // Valid permissions
  validPermissions: string[]
  allPermissionsWithExpiry: TimestampedPermission[]
  
  // Utilities
  checkExpiryAtTime: (time: Date) => string[]
  formatTimeUntilExpiry: (timestamp?: number) => string
  getExpiryUrgency: (timestamp?: number) => 'expired' | 'critical' | 'warning' | 'normal' | 'none'
  refreshExpiryData: () => void
  refresh: () => void
  
  // Real-time monitoring
  isMonitoring: boolean
  startMonitoring: () => void
  stopMonitoring: () => void
}

export function usePermissionExpiry(): PermissionExpiryHookReturn {
  const { user } = useAuth()
  const [lastRefresh, setLastRefresh] = useState(Date.now())
  const [isMonitoring, setIsMonitoring] = useState(false)
  
  // Add refresh method to the hook interface
  const refresh = useCallback(() => {
    setLastRefresh(Date.now())
  }, [])
  
  // Force refresh of expiry data
  const refreshExpiryData = useCallback(() => {
    setLastRefresh(Date.now())
  }, [])
  
  // Memoized calculations that update when user changes or refresh is triggered
  const expiryData = useMemo(() => {
    if (!user) {
      return {
        expiryInfo: {
          valid: [],
          expired: [],
          expiringSoon: [],
          nextExpiry: null,
          hasExpiringPermissions: false
        } as PermissionExpiryInfo,
        healthSummary: {
          total: 0,
          active: 0,
          expired: 0,
          expiringSoon: 0,
          permanent: 0,
          healthScore: 'excellent' as const
        },
        timeUntilNextExpiry: null,
        validPermissions: [],
        allPermissionsWithExpiry: [],
        tierPrediction: {
          currentTier: 'BRONZE' as UserLevelType,
          futureTier: 'BRONZE' as UserLevelType,
          willChange: false
        }
      }
    }
    
    // Trigger refresh dependency
    void lastRefresh
    
    const permissions = user.permissions || []
    
    return {
      expiryInfo: getPermissionExpiryInfo(user),
      healthSummary: getPermissionHealthSummary(permissions),
      timeUntilNextExpiry: getTimeUntilNextExpiry(user),
      validPermissions: filterValidPermissions(permissions),
      allPermissionsWithExpiry: getAllPermissionsWithExpiry(permissions),
      tierPrediction: predictTierChanges(permissions, 24)
    }
  }, [user, lastRefresh])
  
  // Computed states
  const hasExpired = expiryData.expiryInfo.expired.length > 0
  const hasExpiringSoon = expiryData.expiryInfo.hasExpiringPermissions
  const willChangeSoon = expiryData.tierPrediction.willChange
  
  // Ranking-specific expiry checks
  const hasExpiringSoonRanking = useMemo(() => {
    if (!user) return false
    return hasExpiringSoonRankingPermissions(user.permissions || [])
  }, [user])
  
  const nextExpiringRanking = useMemo(() => {
    if (!user) return null
    return getNextExpiringRankingPermission(user.permissions || [])
  }, [user])
  
  // Next expiring permission overall
  const nextExpiringPermission = useMemo(() => {
    return expiryData.allPermissionsWithExpiry
      .filter(tp => !tp.isExpired && tp.expiresAt)
      .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0] || null
  }, [expiryData.allPermissionsWithExpiry])
  
  // Utility functions
  const checkExpiryAtTime = useCallback((time: Date) => {
    if (!user) return []
    return getEffectivePermissionsAtTime(user.permissions || [], time)
  }, [user])
  
  const formatTimeUntilExpiry = useCallback((timestamp?: number): string => {
    if (!timestamp) return 'Never expires'
    
    const now = Date.now()
    const expiryTime = timestamp * 1000
    const diff = expiryTime - now
    
    if (diff <= 0) return 'Expired'
    
    const minutes = Math.floor(diff / (1000 * 60))
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)
    
    if (days > 0) {
      return `${days} day${days !== 1 ? 's' : ''}`
    } else if (hours > 0) {
      return `${hours} hour${hours !== 1 ? 's' : ''}`
    } else if (minutes > 0) {
      return `${minutes} minute${minutes !== 1 ? 's' : ''}`
    } else {
      return 'Less than 1 minute'
    }
  }, [])
  
  const getExpiryUrgency = useCallback((timestamp?: number): 'expired' | 'critical' | 'warning' | 'normal' | 'none' => {
    if (!timestamp) return 'none'
    
    const now = Date.now()
    const expiryTime = timestamp * 1000
    const diff = expiryTime - now
    
    if (diff <= 0) return 'expired'
    
    const hours = diff / (1000 * 60 * 60)
    
    if (hours <= 1) return 'critical'
    if (hours <= 24) return 'warning'
    if (hours <= 72) return 'normal'
    
    return 'none'
  }, [])
  
  // Real-time monitoring
  useEffect(() => {
    if (!isMonitoring) return
    
    const interval = setInterval(() => {
      refreshExpiryData()
    }, 60000) // Refresh every minute when monitoring
    
    return () => clearInterval(interval)
  }, [isMonitoring, refreshExpiryData])
  
  const startMonitoring = useCallback(() => {
    setIsMonitoring(true)
  }, [])
  
  const stopMonitoring = useCallback(() => {
    setIsMonitoring(false)
  }, [])
  
  // Auto-start monitoring if user has expiring permissions
  useEffect(() => {
    if (hasExpiringSoon && !isMonitoring) {
      startMonitoring()
    }
  }, [hasExpiringSoon, isMonitoring, startMonitoring])
  
  return {
    // Expiry information
    expiryInfo: expiryData.expiryInfo,
    healthSummary: expiryData.healthSummary,
    timeUntilNextExpiry: expiryData.timeUntilNextExpiry,
    
    // Permission states
    hasExpired,
    hasExpiringSoon,
    hasExpiringSoonRanking,
    nextExpiringPermission,
    nextExpiringRanking,
    
    // Predictions
    willChangeSoon,
    tierPrediction: expiryData.tierPrediction,
    
    // Valid permissions
    validPermissions: expiryData.validPermissions,
    allPermissionsWithExpiry: expiryData.allPermissionsWithExpiry,
    
    // Utilities
    checkExpiryAtTime,
    formatTimeUntilExpiry,
    getExpiryUrgency,
    refreshExpiryData,
    refresh,
    
    // Real-time monitoring
    isMonitoring,
    startMonitoring,
    stopMonitoring
  }
}

// Specialized hooks for specific use cases
export function useRankingExpiry() {
  const expiry = usePermissionExpiry()
  
  return {
    hasExpiringSoonRanking: expiry.hasExpiringSoonRanking,
    nextExpiringRanking: expiry.nextExpiringRanking,
    currentTier: expiry.tierPrediction.currentTier,
    futureTier: expiry.tierPrediction.futureTier,
    willTierChange: expiry.tierPrediction.willChange,
    changeTime: expiry.tierPrediction.changeTime,
    formatTimeUntilExpiry: expiry.formatTimeUntilExpiry,
    getExpiryUrgency: expiry.getExpiryUrgency,
    ...expiry
  }
}

export function usePermissionHealth() {
  const expiry = usePermissionExpiry()
  
  return {
    healthScore: expiry.healthSummary.healthScore,
    totalPermissions: expiry.healthSummary.total,
    activePermissions: expiry.healthSummary.active,
    expiredPermissions: expiry.healthSummary.expired,
    expiringSoonPermissions: expiry.healthSummary.expiringSoon,
    permanentPermissions: expiry.healthSummary.permanent,
    nextExpiry: expiry.healthSummary.nextExpiry,
    hasIssues: expiry.hasExpired || expiry.hasExpiringSoon,
    ...expiry
  }
}

// Hook for permission expiry notifications
export function useExpiryNotifications(options: {
  enableNotifications?: boolean
  notifyBeforeHours?: number[]
  onPermissionExpired?: (permission: TimestampedPermission) => void
  onPermissionExpiringSoon?: (permission: TimestampedPermission) => void
} = {}) {
  const {
    enableNotifications = true,
    notifyBeforeHours = [24, 6, 1],
    onPermissionExpired,
    onPermissionExpiringSoon
  } = options
  
  const expiry = usePermissionExpiry()
  const [notifiedPermissions] = useState<Set<string>>(new Set())
  
  useEffect(() => {
    if (!enableNotifications) return
    
    // Check for expired permissions
    expiry.expiryInfo.expired.forEach(tp => {
      const key = `${tp.permission}-expired`
      if (!notifiedPermissions.has(key) && onPermissionExpired) {
        onPermissionExpired(tp)
        notifiedPermissions.add(key)
      }
    })
    
    // Check for expiring soon permissions
    expiry.expiryInfo.expiringSoon.forEach(tp => {
      if (!tp.expiresAt) return
      
      const hoursUntilExpiry = ((tp.expiresAt * 1000) - Date.now()) / (1000 * 60 * 60)
      
      notifyBeforeHours.forEach(hours => {
        const key = `${tp.permission}-${hours}h`
        if (hoursUntilExpiry <= hours && !notifiedPermissions.has(key) && onPermissionExpiringSoon) {
          onPermissionExpiringSoon(tp)
          notifiedPermissions.add(key)
        }
      })
    })
  }, [expiry.expiryInfo, enableNotifications, notifyBeforeHours, onPermissionExpired, onPermissionExpiringSoon, notifiedPermissions])
  
  return {
    ...expiry,
    notificationCount: notifiedPermissions.size
  }
}