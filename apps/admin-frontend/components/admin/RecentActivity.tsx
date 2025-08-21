/**
 * Recent Activity Component
 * Shows recent system activities and events
 */

import { Clock, User, Settings, AlertTriangle, CheckCircle } from 'lucide-react'
import { adminCardVariants, adminBadgeVariants, cn } from '@/design-system'

interface ActivityItem {
  id: string
  type: 'user' | 'system' | 'security' | 'billing'
  action: string
  details: string
  timestamp: string
  status: 'success' | 'warning' | 'error'
}

export function RecentActivity() {
  // Mock data - in real implementation, this would come from props or server data
  const activities: ActivityItem[] = [
    {
      id: '1',
      type: 'user',
      action: 'User created',
      details: 'john.doe@example.com registered',
      timestamp: '2 minutes ago',
      status: 'success'
    },
    {
      id: '2',
      type: 'security',
      action: 'Failed login',
      details: 'Invalid credentials for admin@test.com',
      timestamp: '15 minutes ago',
      status: 'warning'
    },
    {
      id: '3',
      type: 'system',
      action: 'Module updated',
      details: 'Analytics module v2.1.3 deployed',
      timestamp: '1 hour ago',
      status: 'success'
    },
    {
      id: '4',
      type: 'billing',
      action: 'Subscription upgraded',
      details: 'Premium plan activated for user_456',
      timestamp: '2 hours ago',
      status: 'success'
    },
    {
      id: '5',
      type: 'system',
      action: 'Database backup',
      details: 'Daily backup completed successfully',
      timestamp: '3 hours ago',
      status: 'success'
    }
  ]

  const getIcon = (type: string, status: string) => {
    if (status === 'error') return AlertTriangle
    if (status === 'warning') return AlertTriangle
    
    switch (type) {
      case 'user': return User
      case 'system': return Settings
      case 'security': return AlertTriangle
      case 'billing': return CheckCircle
      default: return Clock
    }
  }

  const getStatusVariant = (status: string) => {
    switch (status) {
      case 'success': return 'success' as const
      case 'warning': return 'warning' as const
      case 'error': return 'error' as const
      default: return 'default' as const
    }
  }

  const getTypeVariant = (type: string) => {
    switch (type) {
      case 'user': return 'info' as const
      case 'system': return 'default' as const
      case 'security': return 'error' as const
      case 'billing': return 'success' as const
      default: return 'default' as const
    }
  }

  const getTypeColor = (type: string) => {
    switch (type) {
      case 'user': return 'bg-info-100 text-info-600'
      case 'system': return 'bg-neutral-100 text-neutral-600'
      case 'security': return 'bg-error-100 text-error-600'
      case 'billing': return 'bg-success-100 text-success-600'
      default: return 'bg-neutral-100 text-neutral-600'
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'success': return 'text-success-500'
      case 'warning': return 'text-warning-500'
      case 'error': return 'text-error-500'
      default: return 'text-neutral-500'
    }
  }

  return (
    <div className={cn(adminCardVariants({ variant: 'pancake', hover: 'both' }))}>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Recent Activity
        </h2>
      </div>
      
      <div className="space-y-3">
        {activities.map((activity) => {
          const Icon = getIcon(activity.type, activity.status)
          return (
            <div key={activity.id} className="flex items-start gap-3 p-3 rounded-lg bg-muted/30">
              <div className={`p-2 rounded-lg ${getTypeColor(activity.type)}`}>
                <Icon className="h-4 w-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-medium text-sm">{activity.action}</span>
                  <span className={`text-xs ${getStatusColor(activity.status)}`}>
                    ●
                  </span>
                </div>
                <p className="text-xs text-muted-foreground mb-1">
                  {activity.details}
                </p>
                <span className="text-xs text-muted-foreground">
                  {activity.timestamp}
                </span>
              </div>
            </div>
          )
        })}
      </div>
      
      <div className="mt-4 pt-3 border-t border-muted">
        <button className="text-sm text-info-600 hover:text-info-700">
          View all activity
        </button>
      </div>
    </div>
  )
}