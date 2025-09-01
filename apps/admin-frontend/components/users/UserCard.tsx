/**
 * User Card Component
 * Individual user card for the enhanced user list
 */

'use client'

import Link from 'next/link'
import { useSearchParams } from 'next/navigation'
import { User, Calendar, Shield, Package, Activity, MoreHorizontal } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'
import { adminCardVariants, cn } from '@/design-system'

interface UserCardProps {
  user: UnifiedUserData
}

export function UserCard({ user }: UserCardProps) {
  const searchParams = useSearchParams()
  const view = searchParams?.get('view')

  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  // Parse permissions with embedded timestamps
  const parsePermission = (permission: string) => {
    const parts = permission.split(':')
    if (parts.length >= 4 && /^\d+$/.test(parts[parts.length - 1])) {
      const timestamp = parseInt(parts[parts.length - 1])
      const permissionWithoutTimestamp = parts.slice(0, -1).join(':')
      const expiresAt = new Date(timestamp * 1000)
      const isExpired = expiresAt < new Date()
      return { permission: permissionWithoutTimestamp, expiresAt, isExpired, hasTimestamp: true }
    }
    return { permission, expiresAt: null, isExpired: false, hasTimestamp: false }
  }

  const permissions = user.permissions?.map(parsePermission) || []
  const activePermissions = permissions.filter(p => !p.isExpired).length
  const adminPermissions = permissions.filter(p => p.permission.startsWith('admin:') && !p.isExpired).length
  const platformPermissions = permissions.filter(p => p.permission.startsWith('epsx:') && !p.isExpired).length
  const expiredPermissions = permissions.filter(p => p.isExpired).length
  const expiringPermissions = permissions.filter(p => {
    if (!p.hasTimestamp || p.isExpired) return false
    const hoursUntilExpiry = (p.expiresAt!.getTime() - Date.now()) / (1000 * 60 * 60)
    return hoursUntilExpiry <= 24
  }).length
  const activeModules = user.moduleAccess?.filter(m => m.isActive).length || 0
  const activePackages = user.stockRankingPackages?.filter(p => p.isActive).length || 0

  // Context-aware link based on view parameter
  const getUserLink = () => {
    const baseUrl = `/users/${user.id}`
    switch (view) {
      case 'permissions': return `${baseUrl}/permissions`
      case 'modules': return `${baseUrl}/modules`
      case 'packages': return `${baseUrl}/packages`
      case 'billing': return `${baseUrl}/packages`
      default: return baseUrl
    }
  }

  return (
    <div className={cn(
      adminCardVariants({ 
        variant: 'pancake', 
        hover: 'intense',
        animation: 'subtle',
        selectable: 'enabled',
        size: 'default'
      }),
      'wp-scale-hover group/card relative overflow-hidden'
    )}>
      {/* Windows Phone accent indicator */}
      <div className="absolute top-0 left-0 w-1 h-full bg-gradient-to-b from-yellow-400 to-yellow-600 opacity-80" />
      
      {/* Live tile status indicator */}
      <div className="absolute top-2 right-2 w-2 h-2 bg-yellow-400/80 rounded-full animate-pulse-subtle" />

      <div className="flex items-start justify-between relative z-10">
        <div className="flex items-start gap-4 flex-1">
          {/* Windows Phone style user avatar tile */}
          <div className="w-14 h-14 bg-gradient-to-br from-blue-500/20 to-blue-600/30 border border-blue-400/30 flex items-center justify-center relative group-hover/card:shadow-md transition-all duration-300">
            <User className="h-6 w-6 text-blue-100" />
            <div className="absolute -top-0.5 -right-0.5 w-3 h-3 bg-yellow-400 opacity-60" />
          </div>

          {/* User Info with Windows Phone typography */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-3 mb-2">
              <Link 
                href={getUserLink()}
                className="font-light text-lg text-foreground hover:text-yellow-400 transition-colors tracking-wide"
              >
                {user.displayName || user.email}
              </Link>
              <UserStatusBadge status={user.status} />
              {view && (
                <span className="text-xs px-2 py-1 bg-yellow-400/20 text-yellow-200 uppercase tracking-wider font-light">
                  {view}
                </span>
              )}
            </div>
            
            <p className="text-sm text-muted-foreground mb-3 font-light">{user.email}</p>
            
            <div className="flex items-center gap-6 text-xs text-muted-foreground">
              <div className="flex items-center gap-1.5">
                <Calendar className="h-3 w-3 text-yellow-400/60" />
                <span className="font-light">joined {formatDate(user.createdAt)}</span>
              </div>
              
              {user.lastLogin && (
                <div className="flex items-center gap-1.5">
                  <Activity className="h-3 w-3 text-green-400/60" />
                  <span className="font-light">active {formatDate(user.lastLogin)}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Windows Phone Mini-Tiles Stats */}
        <div className="flex items-center gap-3">
          {/* Permissions mini-tile */}
          <div className="bg-gradient-to-br from-green-600/20 to-green-700/30 border border-green-400/20 px-3 py-2 min-w-[60px] text-center relative group-hover/card:shadow-sm transition-all duration-300">
            <div className="flex items-center justify-center gap-1 mb-1">
              <div className="text-lg font-light text-green-200 counter-animation">{activePermissions}</div>
              {expiredPermissions > 0 && (
                <div className="absolute -top-1 -right-1 w-2 h-2 bg-red-400 rounded-full animate-pulse" />
              )}
              {expiringPermissions > 0 && (
                <div className="absolute -top-1 -right-1 w-2 h-2 bg-yellow-400 rounded-full animate-pulse" />
              )}
            </div>
            <div className="text-xs text-green-300/80 uppercase tracking-wider font-light flex items-center justify-center gap-1">
              <Shield className="h-3 w-3" />
              perms
            </div>
            <div className="absolute bottom-0 right-0 w-1 h-1 bg-yellow-400/60" />
          </div>
          
          {/* Modules mini-tile */}
          <div className="bg-gradient-to-br from-blue-600/20 to-blue-700/30 border border-blue-400/20 px-3 py-2 min-w-[60px] text-center relative group-hover/card:shadow-sm transition-all duration-300">
            <div className="text-lg font-light text-blue-200 counter-animation mb-1">{activeModules}</div>
            <div className="text-xs text-blue-300/80 uppercase tracking-wider font-light flex items-center justify-center gap-1">
              <Package className="h-3 w-3" />
              mods
            </div>
            <div className="absolute bottom-0 right-0 w-1 h-1 bg-yellow-400/60" />
          </div>
          
          {/* Packages mini-tile */}
          <div className="bg-gradient-to-br from-purple-600/20 to-purple-700/30 border border-purple-400/20 px-3 py-2 min-w-[60px] text-center relative group-hover/card:shadow-sm transition-all duration-300">
            <div className="text-lg font-light text-purple-200 counter-animation mb-1">{activePackages}</div>
            <div className="text-xs text-purple-300/80 uppercase tracking-wider font-light flex items-center justify-center gap-1">
              <Package className="h-3 w-3" />
              pkgs
            </div>
            <div className="absolute bottom-0 right-0 w-1 h-1 bg-yellow-400/60" />
          </div>

          {/* Actions as mini-tiles */}
          <div className="flex items-center gap-2 ml-2">
            <Link 
              href={`/users/${user.id}/activity`}
              className="w-10 h-10 bg-gradient-to-br from-gray-600/20 to-gray-700/30 border border-gray-400/20 flex items-center justify-center hover:from-yellow-600/20 hover:to-yellow-700/30 hover:border-yellow-400/30 transition-all duration-300 group"
              title="View Activity Logs"
            >
              <Activity className="h-4 w-4 text-gray-300 group-hover:text-yellow-300" />
              <span className="sr-only">View activity logs</span>
            </Link>
            <button className="w-10 h-10 bg-gradient-to-br from-gray-600/20 to-gray-700/30 border border-gray-400/20 flex items-center justify-center hover:from-yellow-600/20 hover:to-yellow-700/30 hover:border-yellow-400/30 transition-all duration-300 group">
              <MoreHorizontal className="h-4 w-4 text-gray-300 group-hover:text-yellow-300" />
              <span className="sr-only">More actions</span>
            </button>
          </div>
        </div>
      </div>

      {/* Windows Phone Tile-Based Additional Info */}
      <div className="mt-6 pt-4 border-t border-yellow-400/20">
        <div className="grid grid-cols-3 gap-3">
          {/* Subscription tile */}
          <div className="bg-gradient-to-br from-indigo-600/15 to-indigo-700/25 border border-indigo-400/15 p-3 text-center relative group-hover/card:shadow-sm transition-all duration-300">
            <div className="text-sm font-light text-indigo-200 mb-1 uppercase tracking-wider">
              subscription
            </div>
            <div className="text-lg font-light text-indigo-100 capitalize">
              {user.billing?.tier || 'basic'}
            </div>
            <div className="absolute top-1 right-1 w-1.5 h-1.5 bg-yellow-400/60 rounded-full" />
          </div>
          
          {/* 2FA Status tile */}
          <div className={cn(
            "border p-3 text-center relative group-hover/card:shadow-sm transition-all duration-300",
            user.twoFactorEnabled 
              ? "bg-gradient-to-br from-green-600/15 to-green-700/25 border-green-400/15" 
              : "bg-gradient-to-br from-red-600/15 to-red-700/25 border-red-400/15"
          )}>
            <div className={cn(
              "text-sm font-light mb-1 uppercase tracking-wider",
              user.twoFactorEnabled ? "text-green-200" : "text-red-200"
            )}>
              2fa auth
            </div>
            <div className={cn(
              "text-lg font-light",
              user.twoFactorEnabled ? "text-green-100" : "text-red-100"
            )}>
              {user.twoFactorEnabled ? 'enabled' : 'disabled'}
            </div>
            <div className="absolute top-1 right-1 w-1.5 h-1.5 bg-yellow-400/60 rounded-full" />
            {user.twoFactorEnabled && (
              <div className="absolute -top-0.5 -right-0.5 w-2 h-2 bg-green-400 rounded-full animate-pulse-subtle" />
            )}
          </div>
          
          {/* API Usage tile */}
          <div className="bg-gradient-to-br from-orange-600/15 to-orange-700/25 border border-orange-400/15 p-3 text-center relative group-hover/card:shadow-sm transition-all duration-300">
            <div className="text-sm font-light text-orange-200 mb-1 uppercase tracking-wider">
              api calls
            </div>
            <div className="text-lg font-light text-orange-100 counter-animation">
              {((user.usageMetrics?.apiCallsThisMonth || 0) / 1000).toFixed(user.usageMetrics?.apiCallsThisMonth > 1000 ? 1 : 0)}
              {user.usageMetrics?.apiCallsThisMonth > 1000 ? 'k' : ''}
            </div>
            <div className="absolute top-1 right-1 w-1.5 h-1.5 bg-yellow-400/60 rounded-full" />
            {(user.usageMetrics?.apiCallsThisMonth || 0) > 5000 && (
              <div className="absolute -top-0.5 -right-0.5 w-2 h-2 bg-orange-400 rounded-full animate-pulse" />
            )}
          </div>
        </div>
      </div>

    </div>
  )
}