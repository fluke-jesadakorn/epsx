'use client'

import { memo, useMemo, useState, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Shield, 
  Users, 
  Clock, 
  TrendingUp, 
  AlertTriangle,
  CheckCircle,
  XCircle,
  Plus,
  RefreshCw,
  Trash2,
  Edit3,
  Timer,
  Activity,
  UserCheck,
  ShieldCheck,
  Eye,
  Settings,
  Database
} from 'lucide-react'
import { LiveTile } from '@/components/tiles/LiveTile'
import { TileGrid } from '@/components/tiles/TileGrid'
import { useSmartPolling } from '@/hooks/useSmartPolling'
import { usePermissionStats, usePermissionManagement, usePermissionTemplates } from '@/hooks/usePermissionManagement'
import type { TileData } from '@/components/tiles/types'
import { cn } from '@/lib/utils'

// ============================================================================
// TYPES
// ============================================================================

interface PermissionStats {
  totalPermissions: number
  activeUsers: number
  expiring: number
  expired: number
  recentActivity: number
  bulkOperations: number
  platforms: string[]
  healthScore: number
}

interface PermissionActivity {
  id: string
  action: 'grant' | 'revoke' | 'extend' | 'cleanup'
  permission: string
  userId?: string
  timestamp: string
  status: 'success' | 'failed' | 'pending'
}

interface BulkOperation {
  id: string
  type: 'grant' | 'revoke' | 'extend' | 'cleanup'
  status: 'pending' | 'running' | 'completed' | 'failed'
  progress: number
  total: number
  completed: number
  failed: number
  startTime: string
}

export interface PermissionTilesProps {
  className?: string
}

// ============================================================================
// MOCK DATA & API SIMULATION
// ============================================================================

const mockStats: PermissionStats = {
  totalPermissions: 1847,
  activeUsers: 234,
  expiring: 12,
  expired: 3,
  recentActivity: 47,
  bulkOperations: 2,
  platforms: ['epsx', 'admin', 'epsx-pay'],
  healthScore: 92
}

const mockActivity: PermissionActivity[] = [
  {
    id: '1',
    action: 'grant',
    permission: 'epsx:analytics:view:1703980800',
    userId: 'user123',
    timestamp: '2024-12-29T14:30:00Z',
    status: 'success'
  },
  {
    id: '2',
    action: 'extend',
    permission: 'admin:users:manage:1703894400',
    userId: 'admin456',
    timestamp: '2024-12-29T13:15:00Z',
    status: 'success'
  },
  {
    id: '3',
    action: 'cleanup',
    permission: 'expired permissions',
    timestamp: '2024-12-29T12:00:00Z',
    status: 'success'
  }
]

const mockBulkOperations: BulkOperation[] = [
  {
    id: 'bulk1',
    type: 'grant',
    status: 'running',
    progress: 65,
    total: 150,
    completed: 98,
    failed: 2,
    startTime: '2024-12-29T14:00:00Z'
  },
  {
    id: 'bulk2',
    type: 'cleanup',
    status: 'completed',
    progress: 100,
    total: 45,
    completed: 43,
    failed: 2,
    startTime: '2024-12-29T11:30:00Z'
  }
]

// API functions with optimistic updates
const fetchPermissionStats = async (): Promise<PermissionStats> => {
  // Real API call would be:
  // const response = await fetch('/api/v1/admin/analytics/permissions')
  // return await response.json()
  
  // Simulate API delay and variability
  await new Promise(resolve => setTimeout(resolve, 300 + Math.random() * 200))
  
  // Simulate real-time changes
  return {
    ...mockStats,
    totalPermissions: mockStats.totalPermissions + Math.floor(Math.random() * 10) - 5,
    activeUsers: mockStats.activeUsers + Math.floor(Math.random() * 6) - 3,
    expiring: Math.max(0, mockStats.expiring + Math.floor(Math.random() * 4) - 2),
    expired: Math.max(0, mockStats.expired + Math.floor(Math.random() * 2) - 1),
    recentActivity: mockStats.recentActivity + Math.floor(Math.random() * 8) - 4,
    healthScore: Math.max(75, Math.min(100, mockStats.healthScore + Math.floor(Math.random() * 6) - 3))
  }
}

const fetchRecentActivity = async (): Promise<PermissionActivity[]> => {
  await new Promise(resolve => setTimeout(resolve, 200 + Math.random() * 100))
  return mockActivity.map(activity => ({
    ...activity,
    timestamp: new Date(Date.now() - Math.random() * 3600000).toISOString()
  }))
}

const fetchBulkOperations = async (): Promise<BulkOperation[]> => {
  await new Promise(resolve => setTimeout(resolve, 150))
  return mockBulkOperations.map(op => ({
    ...op,
    progress: op.status === 'running' ? Math.min(100, op.progress + Math.floor(Math.random() * 15)) : op.progress
  }))
}

// ============================================================================
// COMPONENT
// ============================================================================

export const PermissionTiles = memo(function PermissionTiles({ 
  className 
}: PermissionTilesProps) {
  const [optimisticStats, setOptimisticStats] = useState<PermissionStats | null>(null)
  const [isPerformingBulk, setIsPerformingBulk] = useState(false)

  // Real API hooks for permission management
  const { stats, isLoading: statsLoading, error: statsError, refresh: refreshStats } = usePermissionStats(15000)
  const { 
    isProcessing: isProcessingAction, 
    lastOperation,
    grantPermission,
    bulkGrantPermissions,
    cleanupExpired 
  } = usePermissionManagement()
  const { templates } = usePermissionTemplates()

  // Mock data for features not yet implemented in backend
  const {
    data: activity,
    isLoading: activityLoading,
    refresh: refreshActivity
  } = useSmartPolling(
    'permission-activity',
    fetchRecentActivity,
    { 
      priority: 'normal',
      customInterval: 30000 // 30 seconds for activity feed
    }
  )

  // Mock data for bulk operations tracking
  const {
    data: bulkOps,
    isLoading: bulkLoading,
    refresh: refreshBulkOps
  } = useSmartPolling(
    'bulk-operations',
    fetchBulkOperations,
    { 
      priority: 'high',
      customInterval: 5000 // 5 seconds for active bulk operations
    }
  )

  // Current stats with optimistic updates
  const currentStats = optimisticStats || stats || mockStats

  // Optimistic UI handlers
  const handleOptimisticAction = useCallback((action: string, delta: Partial<PermissionStats>) => {
    setOptimisticStats(prev => ({
      ...(prev || stats || mockStats),
      ...delta
    }))
    
    // Clear optimistic state after a delay
    setTimeout(() => {
      setOptimisticStats(null)
      refreshStats()
      refreshActivity()
    }, 2000)
  }, [stats, refreshStats, refreshActivity])

  const handleGrantPermissions = useCallback(async () => {
    try {
      // Optimistic update first
      handleOptimisticAction('grant', { 
        totalPermissions: currentStats.totalPermissions + 5,
        recentActivity: currentStats.recentActivity + 1
      })

      // Use template for demo - normally would show a permission grant modal
      const template = templates[0] // Admin full access template
      if (template) {
        const templateRequest = {
          user_ids: ['demo-user'], // Would come from user selection
          permissions: template.permissions.map(p => ({
            base_permission: `${p.platform}:${p.resource}:${p.action}`,
            platform: p.platform,
            resource: p.resource,
            action: p.action,
            expiry_timestamp: Math.floor(Date.now() / 1000) + (24 * 60 * 60) // 24 hours
          })),
          reason: 'Demo permission grant from tiles interface'
        }
        
        await bulkGrantPermissions(templateRequest)
      }
    } catch (error) {
      console.error('Failed to grant permissions:', error)
      // Reset optimistic update on error
      setOptimisticStats(null)
    }
  }, [currentStats, handleOptimisticAction, templates, bulkGrantPermissions])

  const handleBulkCleanup = useCallback(async () => {
    if (isPerformingBulk || isProcessingAction) return
    
    try {
      setIsPerformingBulk(true)
      
      // Optimistic update
      handleOptimisticAction('cleanup', { 
        expired: Math.max(0, currentStats.expired - 3),
        recentActivity: currentStats.recentActivity + 1
      })

      // Real API call
      const result = await cleanupExpired(false) // Not a dry run
      
      console.log('Cleanup completed:', result)
    } catch (error) {
      console.error('Failed to cleanup permissions:', error)
      // Reset optimistic update on error
      setOptimisticStats(null)
    } finally {
      setTimeout(() => {
        setIsPerformingBulk(false)
      }, 2000)
    }
  }, [currentStats, handleOptimisticAction, cleanupExpired, isPerformingBulk, isProcessingAction])

  // Generate tiles data
  const tiles: TileData[] = useMemo(() => [
    // Main stats tiles
    {
      id: 'total-permissions',
      title: 'Total Permissions',
      value: currentStats.totalPermissions.toLocaleString(),
      subtitle: `${currentStats.platforms.join(', ')} platforms`,
      icon: Shield,
      size: 'wide' as const,
      color: 'primary' as const,
      href: '/permissions/manage',
      isRealTime: true,
      priority: 'important' as const,
      refreshInterval: 15000
    },
    {
      id: 'active-users',
      title: 'Users with Permissions',
      value: currentStats.activeUsers,
      subtitle: 'currently active',
      icon: Users,
      size: 'square' as const,
      color: 'success' as const,
      href: '/permissions/users',
      isRealTime: true,
      priority: 'normal' as const
    },
    {
      id: 'expiring-permissions',
      title: 'Expiring Soon',
      value: currentStats.expiring,
      subtitle: 'next 24 hours',
      icon: Clock,
      size: 'square' as const,
      color: currentStats.expiring > 10 ? 'warning' as const : 'info' as const,
      href: '/permissions/expiring',
      isRealTime: true,
      priority: 'high' as const
    },
    {
      id: 'expired-permissions',
      title: 'Expired',
      value: currentStats.expired,
      subtitle: 'needs cleanup',
      icon: AlertTriangle,
      size: 'square' as const,
      color: currentStats.expired > 0 ? 'error' as const : 'success' as const,
      href: '/permissions/expired',
      isRealTime: true,
      priority: 'critical' as const
    },
    {
      id: 'health-score',
      title: 'Permission Health',
      value: `${currentStats.healthScore}%`,
      subtitle: currentStats.healthScore > 90 ? 'excellent' : 
                currentStats.healthScore > 75 ? 'good' : 'needs attention',
      icon: TrendingUp,
      size: 'wide' as const,
      color: currentStats.healthScore > 90 ? 'success' as const : 
             currentStats.healthScore > 75 ? 'warning' as const : 'error' as const,
      href: '/permissions/health',
      isRealTime: true,
      priority: 'important' as const
    },
    {
      id: 'recent-activity',
      title: 'Recent Activity',
      value: currentStats.recentActivity,
      subtitle: 'last hour',
      icon: Activity,
      size: 'square' as const,
      color: 'info' as const,
      href: '/permissions/activity',
      isRealTime: true,
      priority: 'normal' as const
    },

    // Action tiles
    {
      id: 'grant-permissions',
      title: 'Grant Permissions',
      value: 'Quick Grant',
      subtitle: 'bulk operations',
      icon: Plus,
      size: 'wide' as const,
      color: 'primary' as const,
      href: '/permissions/grant',
      priority: 'normal' as const
    },
    {
      id: 'bulk-cleanup',
      title: 'Cleanup Expired',
      value: isPerformingBulk ? 'Running...' : 'Run Cleanup',
      subtitle: `${currentStats.expired} expired`,
      icon: isPerformingBulk ? RefreshCw : Trash2,
      size: 'square' as const,
      color: isPerformingBulk ? 'warning' as const : 'error' as const,
      priority: 'high' as const
    },
    {
      id: 'extend-permissions',
      title: 'Extend Expiry',
      value: 'Bulk Extend',
      subtitle: 'prevent expiration',
      icon: Timer,
      size: 'square' as const,
      color: 'warning' as const,
      href: '/permissions/extend',
      priority: 'normal' as const
    },
    {
      id: 'permission-templates',
      title: 'Templates',
      value: 'Quick Setup',
      subtitle: 'common permissions',
      icon: ShieldCheck,
      size: 'square' as const,
      color: 'info' as const,
      href: '/permissions/templates',
      priority: 'low' as const
    },
    {
      id: 'audit-logs',
      title: 'Audit Logs',
      value: 'View History',
      subtitle: 'permission changes',
      icon: Eye,
      size: 'wide' as const,
      color: 'secondary' as const,
      href: '/permissions/audit',
      priority: 'low' as const
    },
    {
      id: 'settings',
      title: 'Settings',
      value: 'Configure',
      subtitle: 'system defaults',
      icon: Settings,
      size: 'square' as const,
      color: 'secondary' as const,
      href: '/permissions/settings',
      priority: 'low' as const
    }
  ], [currentStats, isPerformingBulk])

  // Tile click handlers with optimistic updates
  const handleTileClick = useCallback((tile: TileData) => {
    switch (tile.id) {
      case 'grant-permissions':
        handleGrantPermissions()
        break
      case 'bulk-cleanup':
        if (!isPerformingBulk) {
          handleBulkCleanup()
        }
        break
      default:
        if (tile.href) {
          // Navigation would happen here
          console.log(`Navigate to: ${tile.href}`)
        }
    }
  }, [handleGrantPermissions, handleBulkCleanup, isPerformingBulk])

  // Custom fetchers for tiles with real-time data
  const tileFetchers = useMemo(() => ({
    'total-permissions': () => fetchPermissionStats().then(s => ({ value: s.totalPermissions.toLocaleString() })),
    'active-users': () => fetchPermissionStats().then(s => ({ value: s.activeUsers })),
    'expiring-permissions': () => fetchPermissionStats().then(s => ({ value: s.expiring })),
    'expired-permissions': () => fetchPermissionStats().then(s => ({ value: s.expired })),
    'health-score': () => fetchPermissionStats().then(s => ({ value: `${s.healthScore}%` })),
    'recent-activity': () => fetchPermissionStats().then(s => ({ value: s.recentActivity }))
  }), [])

  // Enhanced tiles with fetcher functions
  const tilesWithFetchers = useMemo(() => 
    tiles.map(tile => ({
      ...tile,
      fetcher: tileFetchers[tile.id as keyof typeof tileFetchers]
    })), [tiles, tileFetchers]
  )

  if (statsError) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-red-500" />
            <span className="text-red-700">Failed to load permission data</span>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Mockup indicator */}
      <div className="flex items-center gap-2 text-xs">
        <span className="px-2 py-1 bg-red-100 text-red-600 rounded-md font-medium">
          *mockup
        </span>
        <span className="text-gray-500">Stats dashboard needs backend /admin/analytics/permission-stats endpoint</span>
      </div>

      {/* Header with actions */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-white">
            Permission Management
          </h2>
          <p className="text-gray-600 dark:text-gray-400">
            Manage embedded timestamp permissions across all platforms
          </p>
        </div>
        
        <div className="flex items-center gap-3">
          <button
            onClick={refreshStats}
            disabled={statsLoading}
            className="p-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
          >
            <RefreshCw className={cn(
              "h-4 w-4 text-gray-600 dark:text-gray-400",
              statsLoading && "animate-spin"
            )} />
          </button>
        </div>
      </div>

      {/* Main tile grid */}
      <TileGrid 
        tiles={tilesWithFetchers}
        onTileClick={handleTileClick}
        className="grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4"
        showControls={false}
      />

      {/* Status indicators */}
      {statsLoading && (
        <div className="flex items-center justify-center py-8">
          <RefreshCw className="h-6 w-6 animate-spin text-gray-400" />
          <span className="ml-2 text-gray-500">Loading permission data...</span>
        </div>
      )}

      {/* Activity feed (compact) */}
      {!activityLoading && activity && activity.length > 0 && (
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <h3 className="font-medium text-gray-900 dark:text-white">Recent Activity</h3>
              <span className="px-1.5 py-0.5 bg-red-100 text-red-600 rounded text-xs font-medium">
                *mockup
              </span>
            </div>
            <span className="text-xs text-gray-500">{activity.length} recent actions</span>
          </div>
          
          <div className="space-y-2 max-h-32 overflow-y-auto">
            {activity.slice(0, 3).map((act) => (
              <div key={act.id} className="flex items-center gap-3 text-sm">
                <div className={cn(
                  "w-2 h-2 rounded-full",
                  act.status === 'success' ? 'bg-green-400' :
                  act.status === 'failed' ? 'bg-red-400' : 'bg-yellow-400'
                )} />
                <span className="text-gray-600 dark:text-gray-400">
                  {act.action} permission {act.userId ? `for ${act.userId}` : ''}
                </span>
                <span className="text-xs text-gray-500 ml-auto">
                  {new Date(act.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Bulk operations status */}
      {!bulkLoading && bulkOps && bulkOps.some(op => op.status === 'running') && (
        <div className="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-4 border border-blue-200 dark:border-blue-800">
          <div className="flex items-center gap-2 mb-3">
            <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />
            <span className="font-medium text-blue-900 dark:text-blue-100">
              Bulk Operations Running
            </span>
          </div>
          
          {bulkOps.filter(op => op.status === 'running').map((op) => (
            <div key={op.id} className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-blue-800 dark:text-blue-200">
                  {op.type} - {op.completed}/{op.total}
                </span>
                <span className="text-blue-600">{op.progress}%</span>
              </div>
              <div className="w-full bg-blue-100 dark:bg-blue-800 rounded-full h-2">
                <div 
                  className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${op.progress}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
})

PermissionTiles.displayName = 'PermissionTiles'

export default PermissionTiles