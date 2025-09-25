/**
 * Login History Card Component
 * Shows login session details
 */

import { LogIn, Smartphone, Monitor, Globe } from 'lucide-react'
import type { LoginRecord } from '@/lib/types/unified-user'

interface LoginHistoryCardProps {
  login: LoginRecord
}

export function LoginHistoryCard({ login }: LoginHistoryCardProps) {
  const formatTimestamp = (timestamp: Date | string) => {
    const date = new Date(timestamp)
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const getDeviceIcon = () => {
    const userAgent = login.userAgent?.toLowerCase() || ''
    if (userAgent.includes('mobile') || userAgent.includes('android') || userAgent.includes('iphone')) {
      return <Smartphone className="h-4 w-4 text-green-500" />
    } else if (userAgent.includes('desktop') || userAgent.includes('windows') || userAgent.includes('mac')) {
      return <Monitor className="h-4 w-4 text-blue-500" />
    } else {
      return <Globe className="h-4 w-4 text-gray-500" />
    }
  }

  const getSuccessIcon = () => {
    return login.success ? 
      <LogIn className="h-4 w-4 text-green-500" /> : 
      <LogIn className="h-4 w-4 text-red-500" />
  }

  return (
    <div className={`p-3 border rounded-lg hover:bg-muted/30 transition-colors ${
      !login.success ? 'border-red-200 bg-red-50' : 'border-muted'
    }`}>
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {getSuccessIcon()}
          <span className="text-sm font-medium">
            {login.success ? 'Login' : 'Failed Login'}
          </span>
        </div>
        <span className="text-xs text-muted-foreground">
          {formatTimestamp(login.timestamp)}
        </span>
      </div>

      <div className="space-y-1 text-xs text-muted-foreground">
        <div className="flex items-center gap-2">
          {getDeviceIcon()}
          <span>{login.userAgent ? 'Browser' : 'Unknown device'}</span>
          {login.userAgent && (
            <>
              <span>•</span>
              <span className="truncate max-w-32">{login.userAgent}</span>
            </>
          )}
        </div>

        {login.ipAddress && (
          <div className="flex items-center gap-2">
            <Globe className="h-3 w-3" />
            <span className="font-mono">{login.ipAddress}</span>
            {login.location && (
              <>
                <span>•</span>
                <span>{login.location.city ? `${login.location.city}, ${login.location.country}` : login.location.country}</span>
              </>
            )}
          </div>
        )}


        {!login.success && login.failureReason && (
          <div className="text-red-600">
            <span>Reason: {login.failureReason}</span>
          </div>
        )}
      </div>
    </div>
  )
}