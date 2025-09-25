/**
 * Activity Timeline Card Component
 * Shows individual activity entries in timeline format
 */

import { Clock, Shield, Package, Settings, AlertCircle, CheckCircle, Info } from 'lucide-react'
import type { ActivityRecord } from '@/lib/types/unified-user'

interface ActivityTimelineCardProps {
  activity: ActivityRecord
  isLast?: boolean
  compact?: boolean
}

export function ActivityTimelineCard({ activity, isLast = false, compact = false }: ActivityTimelineCardProps) {
  const formatTimestamp = (timestamp: Date | string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

    if (diffDays > 7) {
      return date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined
      })
    } else if (diffDays > 0) {
      return `${diffDays}d ago`
    } else if (diffHours > 0) {
      return `${diffHours}h ago`
    } else {
      const diffMinutes = Math.floor(diffMs / (1000 * 60))
      return diffMinutes > 0 ? `${diffMinutes}m ago` : 'Just now'
    }
  }

  const getActivityIcon = () => {
    const iconClass = "h-4 w-4"
    
    switch ((activity as any).category || 'general') {
      case 'security':
        return <Shield className={`${iconClass} text-red-500`} />
      case 'permissions':
        return <Shield className={`${iconClass} text-blue-500`} />
      case 'modules':
        return <Package className={`${iconClass} text-green-500`} />
      case 'admin':
        return <Settings className={`${iconClass} text-purple-500`} />
      default:
        return <Info className={`${iconClass} text-gray-500`} />
    }
  }

  const getStatusIcon = () => {
    switch ((activity as any).status || 'info') {
      case 'success':
        return <CheckCircle className="h-3 w-3 text-green-500" />
      case 'error':
        return <AlertCircle className="h-3 w-3 text-red-500" />
      case 'warning':
        return <AlertCircle className="h-3 w-3 text-orange-500" />
      default:
        return null
    }
  }

  const getSeverityColor = () => {
    switch ((activity as any).severity || 'low') {
      case 'high':
        return 'border-l-red-500 bg-red-50'
      case 'medium':
        return 'border-l-orange-500 bg-orange-50'
      case 'low':
        return 'border-l-blue-500 bg-blue-50'
      default:
        return 'border-l-gray-300 bg-gray-50'
    }
  }

  if (compact) {
    return (
      <div className="flex items-center gap-3 p-2 hover:bg-muted/30 rounded-lg transition-colors">
        {getActivityIcon()}
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium truncate">{activity.action}</p>
          <p className="text-xs text-muted-foreground">{formatTimestamp(activity.timestamp)}</p>
        </div>
        {getStatusIcon()}
      </div>
    )
  }

  return (
    <div className="relative">
      {/* Timeline connector line */}
      {!isLast && (
        <div className="absolute left-6 top-12 bottom-0 w-px bg-muted" />
      )}
      
      <div className={`border-l-4 rounded-lg p-4 ${getSeverityColor()}`}>
        <div className="flex items-start gap-3">
          <div className="flex-shrink-0 mt-0.5">
            {getActivityIcon()}
          </div>
          
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <h4 className="text-sm font-medium">{activity.action}</h4>
              {getStatusIcon()}
            </div>
            
            <div className="mt-1 text-xs text-muted-foreground flex items-center gap-2">
              <Clock className="h-3 w-3" />
              <span>{formatTimestamp(activity.timestamp)}</span>
              {activity.resource && (
                <>
                  <span>•</span>
                  <span>{activity.resource}</span>
                </>
              )}
            </div>

            {activity.details && (
              <p className="mt-2 text-sm text-muted-foreground">{JSON.stringify(activity.details)}</p>
            )}

            {(activity as any).metadata && Object.keys((activity as any).metadata).length > 0 && (
              <div className="mt-2 space-y-1">
                {Object.entries((activity as any).metadata).map(([key, value]) => (
                  <div key={key} className="text-xs text-muted-foreground flex items-center gap-2">
                    <span className="capitalize">{key.replace('_', ' ')}:</span>
                    <span className="font-mono">{String(value)}</span>
                  </div>
                ))}
              </div>
            )}

            {activity.ipAddress && (
              <div className="mt-2 text-xs text-muted-foreground">
                IP: <span className="font-mono">{activity.ipAddress}</span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}