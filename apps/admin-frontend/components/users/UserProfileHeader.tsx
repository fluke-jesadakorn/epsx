/**
 * User Profile Header Component
 * Shows user basic info and status at the top of user profile pages
 */

import Image from 'next/image'
import { User, Mail, Shield, Calendar, CheckCircle, XCircle, AlertTriangle } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth-enhanced'
import { UserStatusBadge } from './UserStatusBadge'
import { QuickActions } from './QuickActions'

interface UserProfileHeaderProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function UserProfileHeader({ user, currentUser }: UserProfileHeaderProps) {
  const formatDate = (date: Date) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  return (
    <div className="pancake-card pancake-card-hover p-6">
      <div className="flex items-start justify-between">
        {/* User Info */}
        <div className="flex items-start gap-4">
          {/* Avatar */}
          <div className="relative">
            {user.avatar ? (
              <Image
                src={user.avatar}
                alt={user.displayName || user.email}
                width={64}
                height={64}
                className="w-16 h-16 rounded-full object-cover"
              />
            ) : (
              <div className="w-16 h-16 rounded-full bg-gradient-to-r from-blue-500 to-purple-500 flex items-center justify-center text-white text-xl font-semibold">
                {(user.displayName || user.email).charAt(0).toUpperCase()}
              </div>
            )}
            
            {/* Status Indicator */}
            <div className={`absolute -bottom-1 -right-1 w-5 h-5 rounded-full border-2 border-white ${
              user.status === 'active' ? 'bg-green-500' :
              user.status === 'disabled' ? 'bg-red-500' :
              user.status === 'suspended' ? 'bg-orange-500' :
              'bg-gray-500'
            }`} />
          </div>
          
          {/* User Details */}
          <div>
            <div className="flex items-center gap-3 mb-2">
              <h1 className="text-2xl font-bold text-foreground">
                {user.displayName || user.email}
              </h1>
              <UserStatusBadge status={user.status} />
            </div>
            
            <div className="space-y-1 text-sm text-muted-foreground">
              <div className="flex items-center gap-2">
                <Mail className="h-4 w-4" />
                <span>{user.email}</span>
                {user.emailVerified ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
              </div>
              
              {user.roles.length > 0 && (
                <div className="flex items-center gap-2">
                  <Shield className="h-4 w-4" />
                  <span>
                    {user.roles.map(role => role.name).join(', ')}
                  </span>
                </div>
              )}
              
              <div className="flex items-center gap-2">
                <Calendar className="h-4 w-4" />
                <span>Joined {formatDate(user.createdAt)}</span>
              </div>
              
              {user.lastLogin && (
                <div className="flex items-center gap-2">
                  <User className="h-4 w-4" />
                  <span>Last login {formatDate(user.lastLogin)}</span>
                </div>
              )}
            </div>
          </div>
        </div>
        
        {/* Quick Actions */}
        <QuickActions 
          user={user} 
          currentUser={currentUser}
        />
      </div>
      
      {/* Additional Info Row */}
      <div className="mt-6 pt-4 border-t border-muted flex items-center justify-between">
        <div className="flex items-center gap-6 text-sm">
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">Modules:</span>
            <span className="font-medium">{user.moduleAccess.length}</span>
          </div>
          
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">Tier:</span>
            <span className="font-medium capitalize">{user.billing.tier}</span>
          </div>
          
          <div className="flex items-center gap-2">
            <span className="text-muted-foreground">API Keys:</span>
            <span className="font-medium">{user.apiKeys.filter(key => key.isActive).length}</span>
          </div>
        </div>
        
        {/* Warning indicators */}
        <div className="flex items-center gap-2">
          {!user.emailVerified && (
            <div className="flex items-center gap-1 text-yellow-600">
              <AlertTriangle className="h-4 w-4" />
              <span className="text-xs">Email not verified</span>
            </div>
          )}
          
          {user.status === 'disabled' && (
            <div className="flex items-center gap-1 text-red-600">
              <XCircle className="h-4 w-4" />
              <span className="text-xs">Account disabled</span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}