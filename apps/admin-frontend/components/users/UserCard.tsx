/**
 * User Card Component
 * Individual user card for the enhanced user list
 */

'use client'

import Link from 'next/link'
import { useSearchParams } from 'next/navigation'
import { User, Calendar, Shield, Package, MoreHorizontal } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'

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

  const activeRoles = user.roles?.filter(r => r.isActive).length || 0
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
    <div className="pancake-card p-6 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-4 flex-1">
          {/* User Avatar */}
          <div className="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
            <User className="h-6 w-6 text-blue-600" />
          </div>

          {/* User Info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <Link 
                href={getUserLink()}
                className="font-semibold text-blue-600 hover:text-blue-700 transition-colors"
              >
                {user.displayName || user.email}
              </Link>
              <UserStatusBadge status={user.status} />
              {view && (
                <span className="text-xs px-2 py-1 bg-blue-100 text-blue-700 rounded-full">
                  View {view}
                </span>
              )}
            </div>
            
            <p className="text-sm text-muted-foreground mb-2">{user.email}</p>
            
            <div className="flex items-center gap-4 text-xs text-muted-foreground">
              <div className="flex items-center gap-1">
                <Calendar className="h-3 w-3" />
                <span>Joined {formatDate(user.createdAt)}</span>
              </div>
              
              {user.lastLogin && (
                <div className="flex items-center gap-1">
                  <User className="h-3 w-3" />
                  <span>Last login {formatDate(user.lastLogin)}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Quick Stats */}
        <div className="flex items-center gap-6 text-sm text-muted-foreground">
          <div className="text-center">
            <div className="font-medium text-foreground">{activeRoles}</div>
            <div className="text-xs flex items-center gap-1">
              <Shield className="h-3 w-3" />
              Roles
            </div>
          </div>
          
          <div className="text-center">
            <div className="font-medium text-foreground">{activeModules}</div>
            <div className="text-xs flex items-center gap-1">
              <Package className="h-3 w-3" />
              Modules
            </div>
          </div>
          
          <div className="text-center">
            <div className="font-medium text-foreground">{activePackages}</div>
            <div className="text-xs flex items-center gap-1">
              <Package className="h-3 w-3" />
              Packages
            </div>
          </div>

          {/* Actions */}
          <button className="p-2 hover:bg-muted rounded-lg transition-colors">
            <MoreHorizontal className="h-4 w-4" />
            <span className="sr-only">User actions</span>
          </button>
        </div>
      </div>

      {/* Additional Info */}
      <div className="mt-4 pt-4 border-t border-muted">
        <div className="flex items-center justify-between text-sm">
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">Subscription:</span>
            <span className="capitalize font-medium">{user.billing?.tier || 'basic'}</span>
          </div>
          
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">2FA:</span>
            <UserStatusBadge 
              status={user.twoFactorEnabled ? 'active' : 'disabled'} 
              size="sm" 
            />
          </div>
          
          <div className="flex items-center gap-4">
            <span className="text-muted-foreground">API Calls:</span>
            <span className="font-medium">
              {user.usageMetrics?.apiCallsThisMonth?.toLocaleString() || 0}
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}