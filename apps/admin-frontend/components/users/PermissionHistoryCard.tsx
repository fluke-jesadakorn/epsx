/**
 * Permission History Card Component
 * Shows audit trail of permission changes
 */

'use client'

import { useState, useEffect } from 'react'
import { Calendar, User, Shield, Key, Users, Clock, AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { getPermissionHistory, type PermissionHistoryEntry } from '@/lib/actions/unified-user-actions'
import { formatDistanceToNow, format } from 'date-fns'

interface PermissionHistoryCardProps {
  userId: string
  className?: string
}

const ACTION_ICONS = {
  granted: Shield,
  revoked: AlertTriangle,
  modified: Key,
}

const ACTION_COLORS = {
  granted: 'text-green-600',
  revoked: 'text-red-600', 
  modified: 'text-blue-600',
}

const TYPE_ICONS = {
  role: Users,
  permission: Key,
  profile: Shield,
}

export function PermissionHistoryCard({ userId, className = '' }: PermissionHistoryCardProps) {
  const [history, setHistory] = useState<PermissionHistoryEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showAll, setShowAll] = useState(false)

  useEffect(() => {
    loadHistory()
  }, [userId])

  const loadHistory = async () => {
    try {
      setLoading(true)
      setError(null)
      
      const result = await getPermissionHistory(userId, 100)
      
      if (result.success) {
        setHistory(result.data || [])
      } else {
        setError(result.error?.message || 'Failed to load permission history')
      }
    } catch (err) {
      setError('An unexpected error occurred')
      console.error('Permission history load error:', err)
    } finally {
      setLoading(false)
    }
  }

  const displayHistory = showAll ? history : history.slice(0, 10)

  if (loading) {
    return (
      <div className={`pancake-card p-6 ${className}`}>
        <div className="flex items-center justify-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className={`pancake-card p-6 ${className}`}>
        <div className="text-center text-red-600 py-4">
          <AlertTriangle className="h-8 w-8 mx-auto mb-2" />
          <p>{error}</p>
          <Button 
            variant="outline" 
            size="sm" 
            onClick={loadHistory}
            className="mt-2"
          >
            Retry
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className={`pancake-card p-6 ${className}`}>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Permission History
        </h3>
        <Badge variant="secondary">
          {history.length} {history.length === 1 ? 'entry' : 'entries'}
        </Badge>
      </div>

      {history.length === 0 ? (
        <div className="text-center text-muted-foreground py-8">
          <Clock className="h-8 w-8 mx-auto mb-2 opacity-50" />
          <p>No permission changes recorded</p>
          <p className="text-xs">Permission changes will appear here</p>
        </div>
      ) : (
        <div className="space-y-3">
          {displayHistory.map((entry) => {
            const ActionIcon = ACTION_ICONS[entry.action]
            const TypeIcon = TYPE_ICONS[entry.type]
            const actionColor = ACTION_COLORS[entry.action]

            return (
              <div 
                key={entry.id}
                className="flex items-start gap-3 p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors"
              >
                <div className={`p-2 rounded-full bg-background border ${actionColor}`}>
                  <ActionIcon className="h-4 w-4" />
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <TypeIcon className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium text-sm">
                          {entry.action === 'granted' && 'Granted'}
                          {entry.action === 'revoked' && 'Revoked'}
                          {entry.action === 'modified' && 'Modified'}
                        </span>
                        <Badge variant="outline" size="sm">
                          {entry.type}
                        </Badge>
                      </div>

                      <div className="text-sm text-muted-foreground">
                        {entry.type === 'role' && entry.role && (
                          <span>Role: <span className="font-mono">{entry.role}</span></span>
                        )}
                        {entry.type === 'permission' && entry.resource && entry.permission && (
                          <span>Permission: <span className="font-mono">{entry.resource}:{entry.permission}</span></span>
                        )}
                        {entry.type === 'profile' && entry.profileId && (
                          <span>Profile: <span className="font-mono">{entry.profileId}</span></span>
                        )}
                      </div>

                      {entry.reason && (
                        <div className="text-xs text-muted-foreground mt-1 italic">
                          Reason: {entry.reason}
                        </div>
                      )}

                      <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <User className="h-3 w-3" />
                          <span>by {entry.grantedBy}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          <span title={format(entry.grantedAt, 'PPpp')}>
                            {formatDistanceToNow(entry.grantedAt, { addSuffix: true })}
                          </span>
                        </div>
                        {entry.expires && (
                          <div className="flex items-center gap-1 text-amber-600">
                            <Clock className="h-3 w-3" />
                            <span title={`Expires ${format(entry.expires, 'PPpp')}`}>
                              Expires {formatDistanceToNow(entry.expires, { addSuffix: true })}
                            </span>
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )
          })}

          {history.length > 10 && (
            <div className="text-center pt-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowAll(!showAll)}
              >
                {showAll ? 'Show Less' : `Show All ${history.length} Entries`}
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  )
}