/**
 * User Card Component - PancakeSwap Style
 * Individual user card for the enhanced user list with PancakeSwap theme
 */

'use client'

import Link from 'next/link'
import { useSearchParams } from 'next/navigation'
import { User, Calendar, Shield, Package, Activity, MoreHorizontal, Edit } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'
import { PancakeUserCard } from '@/components/ui/PancakeCard'

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

  // Determine user status for PancakeUserCard
  const getUserStatus = (): 'active' | 'inactive' | 'pending' => {
    if (user.status === 'active') return 'active'
    if (user.status === 'pending') return 'pending'
    return 'inactive'
  }

  return (
    <Link href={getUserLink()} className="block">
      <PancakeUserCard
        name={user.displayName || user.email.split('@')[0]}
        email={user.email}
        role={view || 'user'}
        status={getUserStatus()}
        permissions={activePermissions}
        actions={
          <div className="flex items-center gap-2">
            {/* Permissions Stats Badge */}
            <div className="flex items-center gap-1 px-2 py-1 bg-orange-100 dark:bg-orange-900/50 rounded-full">
              <Shield className="h-3 w-3 text-orange-600 dark:text-orange-400" />
              <span className="text-xs font-medium text-orange-700 dark:text-orange-300">
                {activePermissions}
              </span>
              {expiredPermissions > 0 && (
                <span className="w-1.5 h-1.5 bg-red-400 rounded-full animate-pulse" />
              )}
              {expiringPermissions > 0 && (
                <span className="w-1.5 h-1.5 bg-yellow-400 rounded-full animate-pulse" />
              )}
            </div>

            {/* Modules Stats Badge */}
            <div className="flex items-center gap-1 px-2 py-1 bg-orange-100 dark:bg-orange-900/50 rounded-full">
              <Package className="h-3 w-3 text-orange-600 dark:text-orange-400" />
              <span className="text-xs font-medium text-orange-700 dark:text-orange-300">
                {activeModules}
              </span>
            </div>

            {/* 2FA Status */}
            {user.twoFactorEnabled && (
              <div className="flex items-center gap-1 px-2 py-1 bg-green-100 dark:bg-green-900/50 rounded-full">
                <span className="text-xs font-medium text-green-700 dark:text-green-300">
                  2FA
                </span>
                <span className="w-1.5 h-1.5 bg-green-400 rounded-full" />
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex items-center gap-1 ml-2">
              <div className="w-8 h-8 bg-orange-100 dark:bg-orange-900/50 rounded-full flex items-center justify-center hover:bg-orange-200 dark:hover:bg-orange-800/70 transition-colors">
                <Activity className="h-3 w-3 text-orange-600 dark:text-orange-400" />
              </div>
              <div className="w-8 h-8 bg-orange-100 dark:bg-orange-900/50 rounded-full flex items-center justify-center hover:bg-orange-200 dark:hover:bg-orange-800/70 transition-colors">
                <Edit className="h-3 w-3 text-orange-600 dark:text-orange-400" />
              </div>
            </div>
          </div>
        }
      />
    </Link>
  )
}