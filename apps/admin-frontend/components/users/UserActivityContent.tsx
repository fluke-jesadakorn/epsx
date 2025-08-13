/**
 * User Activity Content Component
 * Comprehensive activity history and audit trail
 */

import { Clock, LogIn, Settings, Shield, TrendingUp, Filter } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth'
import { ActivityTimelineCard } from './ActivityTimelineCard'
import { LoginHistoryCard } from './LoginHistoryCard'
import { StatsCard } from '@/components/ui/StatsCard'

interface UserActivityContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function UserActivityContent({ user, currentUser: _currentUser }: UserActivityContentProps) {
  // Calculate activity stats
  const totalActivities = user.recentActivity.length
  const todayActivities = user.recentActivity.filter(activity => {
    const today = new Date()
    const activityDate = new Date(activity.timestamp)
    return activityDate.toDateString() === today.toDateString()
  }).length

  const uniqueLogins = user.loginHistory.length
  const lastLogin = user.loginHistory[0]?.timestamp

  const activityStats = [
    {
      title: 'Total Activities',
      value: totalActivities,
      description: 'All recorded actions',
      icon: Clock,
      color: 'blue'
    },
    {
      title: 'Today',
      value: todayActivities,
      description: 'Activities today',
      icon: TrendingUp,
      color: 'green'
    },
    {
      title: 'Login Sessions',
      value: uniqueLogins,
      description: 'Recorded logins',
      icon: LogIn,
      color: 'purple'
    }
  ]

  // Group activities by type for filtering
  const _activityTypes = ['all', 'auth', 'permissions', 'modules', 'billing', 'api']
  
  return (
    <div className="space-y-6">
      {/* Activity Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {activityStats.map((stat, index) => (
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

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Recent Activity Timeline */}
        <div className="lg:col-span-2">
          <div className="pancake-card p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold flex items-center gap-2">
                <Clock className="h-5 w-5" />
                Activity Timeline
              </h3>
              <button className="inline-flex items-center gap-1 px-3 py-1 text-sm border border-muted-foreground rounded-md hover:bg-muted transition-colors">
                <Filter className="h-4 w-4" />
                Filter
              </button>
            </div>
            
            <div className="space-y-4">
              {user.recentActivity.length > 0 ? (
                user.recentActivity.map((activity, index) => (
                  <ActivityTimelineCard 
                    key={index}
                    activity={activity}
                    isLast={index === user.recentActivity.length - 1}
                  />
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

        {/* Login History */}
        <div>
          <div className="pancake-card p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <LogIn className="h-5 w-5" />
              Login History
            </h3>
            
            <div className="space-y-3">
              {user.loginHistory.length > 0 ? (
                user.loginHistory.slice(0, 10).map((login, index) => (
                  <LoginHistoryCard 
                    key={index}
                    login={login}
                  />
                ))
              ) : (
                <div className="text-center text-muted-foreground py-8">
                  <LogIn className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p>No login history</p>
                </div>
              )}
            </div>

            {user.loginHistory.length > 10 && (
              <div className="mt-4 text-center">
                <button className="text-sm text-blue-600 hover:text-blue-700">
                  View all login history
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Activity Categories */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Security Activities */}
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Shield className="h-5 w-5 text-red-500" />
            Security Activities
          </h3>
          
          <div className="space-y-3">
            {user.recentActivity
              .filter(activity => 
                activity.category === 'security' || 
                activity.action.toLowerCase().includes('login') ||
                activity.action.toLowerCase().includes('password') ||
                activity.action.toLowerCase().includes('2fa')
              )
              .slice(0, 5)
              .map((activity, index) => (
                <ActivityTimelineCard 
                  key={index}
                  activity={activity}
                  compact
                />
              ))}
            
            {user.recentActivity.filter(activity => 
              activity.category === 'security' || 
              activity.action.toLowerCase().includes('login')
            ).length === 0 && (
              <div className="text-center text-muted-foreground py-4">
                <Shield className="h-6 w-6 mx-auto mb-2 opacity-50" />
                <p className="text-sm">No security activities</p>
              </div>
            )}
          </div>
        </div>

        {/* Administrative Changes */}
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Settings className="h-5 w-5 text-blue-500" />
            Administrative Changes
          </h3>
          
          <div className="space-y-3">
            {user.recentActivity
              .filter(activity => 
                activity.category === 'admin' || 
                activity.action.toLowerCase().includes('role') ||
                activity.action.toLowerCase().includes('permission') ||
                activity.action.toLowerCase().includes('module')
              )
              .slice(0, 5)
              .map((activity, index) => (
                <ActivityTimelineCard 
                  key={index}
                  activity={activity}
                  compact
                />
              ))}
            
            {user.recentActivity.filter(activity => 
              activity.category === 'admin'
            ).length === 0 && (
              <div className="text-center text-muted-foreground py-4">
                <Settings className="h-6 w-6 mx-auto mb-2 opacity-50" />
                <p className="text-sm">No admin changes</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Usage Patterns */}
      <div className="pancake-card p-6">
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          Usage Patterns
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="p-4 bg-muted/30 rounded-lg text-center">
            <div className="text-2xl font-bold text-blue-600">{user.usageMetrics.sessionsThisMonth || 0}</div>
            <div className="text-sm text-muted-foreground">Sessions This Month</div>
          </div>
          
          <div className="p-4 bg-muted/30 rounded-lg text-center">
            <div className="text-2xl font-bold text-green-600">{user.usageMetrics.apiCallsThisMonth || 0}</div>
            <div className="text-sm text-muted-foreground">API Calls This Month</div>
          </div>
          
          <div className="p-4 bg-muted/30 rounded-lg text-center">
            <div className="text-2xl font-bold text-purple-600">
              {user.usageMetrics.avgSessionDuration ? `${Math.round(user.usageMetrics.avgSessionDuration)}m` : 'N/A'}
            </div>
            <div className="text-sm text-muted-foreground">Avg Session Duration</div>
          </div>
          
          <div className="p-4 bg-muted/30 rounded-lg text-center">
            <div className="text-2xl font-bold text-orange-600">
              {lastLogin ? new Date(lastLogin).toLocaleDateString() : 'Never'}
            </div>
            <div className="text-sm text-muted-foreground">Last Login</div>
          </div>
        </div>
      </div>
    </div>
  )
}