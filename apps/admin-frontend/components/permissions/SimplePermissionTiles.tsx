'use client'

import { memo, useMemo } from 'react'
import { 
  Shield, 
  Users, 
  Clock, 
  TrendingUp, 
  AlertTriangle,
  Plus,
  RefreshCw,
  Trash2,
  Timer,
  Activity,
  ShieldCheck,
  Eye,
  Settings
} from 'lucide-react'
import { TileGrid } from '@/components/tiles/TileGrid'
import type { TileData } from '@/components/tiles/types'
import { cn } from '@/lib/utils'

export interface SimplePermissionTilesProps {
  className?: string
}

// Mock data for testing
const mockStats = {
  totalPermissions: 1847,
  activeUsers: 234,
  expiring: 12,
  expired: 3,
  recentActivity: 47,
  bulkOperations: 2,
  platforms: ['epsx', 'admin', 'epsx-pay'],
  healthScore: 92
}

export const SimplePermissionTiles = memo(function SimplePermissionTiles({ 
  className 
}: SimplePermissionTilesProps) {
  
  // Generate tiles data with simple mock data
  const tiles: TileData[] = useMemo(() => [
    {
      id: 'total-permissions',
      title: 'Total Permissions',
      value: mockStats.totalPermissions.toLocaleString(),
      subtitle: `${mockStats.platforms.join(', ')} platforms`,
      icon: Shield,
      size: 'wide' as const,
      color: 'primary' as const,
      priority: 'important' as const
    },
    {
      id: 'active-users',
      title: 'Users with Permissions',
      value: mockStats.activeUsers,
      subtitle: 'currently active',
      icon: Users,
      size: 'square' as const,
      color: 'success' as const,
      priority: 'normal' as const
    },
    {
      id: 'expiring-permissions',
      title: 'Expiring Soon',
      value: mockStats.expiring,
      subtitle: 'next 24 hours',
      icon: Clock,
      size: 'square' as const,
      color: mockStats.expiring > 10 ? 'warning' as const : 'info' as const,
      priority: 'high' as const
    },
    {
      id: 'expired-permissions',
      title: 'Expired',
      value: mockStats.expired,
      subtitle: 'needs cleanup',
      icon: AlertTriangle,
      size: 'square' as const,
      color: mockStats.expired > 0 ? 'error' as const : 'success' as const,
      priority: 'critical' as const
    },
    {
      id: 'health-score',
      title: 'Permission Health',
      value: `${mockStats.healthScore}%`,
      subtitle: mockStats.healthScore > 90 ? 'excellent' : 
                mockStats.healthScore > 75 ? 'good' : 'needs attention',
      icon: TrendingUp,
      size: 'wide' as const,
      color: mockStats.healthScore > 90 ? 'success' as const : 
             mockStats.healthScore > 75 ? 'warning' as const : 'error' as const,
      priority: 'important' as const
    },
    {
      id: 'recent-activity',
      title: 'Recent Activity',
      value: mockStats.recentActivity,
      subtitle: 'last hour',
      icon: Activity,
      size: 'square' as const,
      color: 'info' as const,
      priority: 'normal' as const
    }
  ], [])

  // Simple click handler
  const handleTileClick = (tile: TileData) => {
    console.log('Tile clicked:', tile.id)
  }

  return (
    <div className={cn('space-y-6', className)}>
      {/* Header */}
      <div>
        <h2 className="text-2xl font-semibold text-gray-900 dark:text-white">
          Permission Management
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Manage embedded timestamp permissions across all platforms
        </p>
      </div>

      {/* Simple tile grid */}
      <TileGrid 
        tiles={tiles}
        onTileClick={handleTileClick}
        className="grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4"
        showControls={false}
      />
    </div>
  )
})

SimplePermissionTiles.displayName = 'SimplePermissionTiles'

export default SimplePermissionTiles