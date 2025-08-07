/**
 * User Overview Content Component
 * Shows detailed user information and recent activity
 */

import { User, Calendar, Clock, TrendingUp } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth-enhanced'
import { StatsCard } from '@/components/ui/StatsCard'
import { UserStatusBadge } from './UserStatusBadge'

interface UserOverviewContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function UserOverviewContent({ user, _currentUser }: UserOverviewContentProps) {
  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const userStats = [
    {
      title: 'Active Modules',
      value: user.moduleAccess.filter(m => m.isActive).length,
      description: 'Assigned modules',
      icon: User,
      color: 'blue'
    },
    {
      title: 'API Keys',
      value: user.apiKeys.filter(k => k.isActive).length,
      description: 'Active API keys',
      icon: TrendingUp,
      color: 'green'
    },
    {
      title: 'Sessions This Month',
      value: user.usageMetrics.sessionsThisMonth,
      description: 'Login sessions',
      icon: Clock,
      color: 'purple'
    },
    {
      title: 'API Calls',
      value: user.usageMetrics.apiCallsThisMonth,
      description: 'This month',
      icon: TrendingUp,
      color: 'orange'
    }
  ]

  return (
    <div className="space-y-6" data-testid="user-profile-overview">
      {/* User Stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {userStats.map((stat, index) => (
          <StatsCard
            key={index}
            title={stat.title}
            value={stat.value}
            description={stat.description}
            icon={stat.icon}
            color={stat.color}
            variant="simple"
          />
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Personal Information */}
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <User className="h-5 w-5" />
            Personal Information
          </h3>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Full Name</span>
              <span className="text-sm font-medium">
                {user.displayName || 'Not provided'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Email</span>
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">{user.email}</span>
                <UserStatusBadge 
                  status={user.emailVerified ? 'active' : 'pending'} 
                  size="sm"
                />
              </div>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Phone</span>
              <span className="text-sm font-medium">
                {user.phoneNumber || 'Not provided'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Timezone</span>
              <span className="text-sm font-medium">
                {user.timezone || 'UTC'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Language</span>
              <span className="text-sm font-medium">
                {user.language || 'English'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">2FA Enabled</span>
              <UserStatusBadge 
                status={user.twoFactorEnabled ? 'active' : 'disabled'} 
                size="sm"
              />
            </div>
          </div>
        </div>

        {/* Account Information */}
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Account Information
          </h3>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Account Status</span>
              <UserStatusBadge status={user.status} size="sm" />
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Created</span>
              <span className="text-sm font-medium">
                {formatDate(user.createdAt)}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Last Updated</span>
              <span className="text-sm font-medium">
                {formatDate(user.updatedAt)}
              </span>
            </div>

            {user.lastLogin && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Last Login</span>
                <span className="text-sm font-medium">
                  {formatDate(user.lastLogin)}
                </span>
              </div>
            )}

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Subscription Tier</span>
              <span className="text-sm font-medium capitalize">
                {user.billing.tier}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Payment Status</span>
              <UserStatusBadge 
                status={user.billing.paymentStatus === 'current' ? 'active' : 'pending'} 
                size="sm"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="pancake-card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Recent Activity
        </h3>
        
        <div className="space-y-3">
          {user.recentActivity.length > 0 ? (
            user.recentActivity.slice(0, 5).map((activity, index) => (
              <div key={index} className="flex items-center justify-between p-3 rounded-lg bg-muted/30">
                <div>
                  <div className="text-sm font-medium">{activity.action}</div>
                  <div className="text-xs text-muted-foreground">
                    {activity.resource} • {formatDate(activity.timestamp)}
                  </div>
                </div>
              </div>
            ))
          ) : (
            <div className="text-center text-muted-foreground py-8">
              <Clock className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No recent activity</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}