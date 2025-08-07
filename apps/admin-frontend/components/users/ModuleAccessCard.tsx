/**
 * Module Access Card Component
 * Shows module access information with management actions
 */

import { MoreHorizontal, AlertCircle, CheckCircle } from 'lucide-react'
import type { ModuleAccess } from '@/lib/types/unified-user'
import { UserStatusBadge } from './UserStatusBadge'

interface ModuleAccessCardProps {
  moduleAccess: ModuleAccess
  canManage: boolean
}

export function ModuleAccessCard({ moduleAccess, canManage }: ModuleAccessCardProps) {
  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  const getModuleIcon = () => {
    if (moduleAccess.isActive) {
      return <CheckCircle className="h-5 w-5 text-green-500" />
    }
    return <AlertCircle className="h-5 w-5 text-gray-400" />
  }

  const isExpiringSoon = moduleAccess.expiresAt && 
    new Date(moduleAccess.expiresAt).getTime() - Date.now() < 7 * 24 * 60 * 60 * 1000 // 7 days

  return (
    <div className="border border-muted rounded-lg p-4 hover:bg-muted/30 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          {getModuleIcon()}
          <div>
            <div className="flex items-center gap-2">
              <span className="font-medium">{moduleAccess.moduleName}</span>
              <UserStatusBadge 
                status={moduleAccess.isActive ? 'active' : 'disabled'} 
                size="sm" 
              />
            </div>
            <p className="text-xs text-muted-foreground">
              {moduleAccess.description || 'Module access permission'}
            </p>
          </div>
        </div>

        {canManage && (
          <button className="p-1 hover:bg-muted rounded-md transition-colors">
            <MoreHorizontal className="h-4 w-4" />
            <span className="sr-only">Module actions</span>
          </button>
        )}
      </div>

      <div className="space-y-2 text-xs text-muted-foreground">
        <div className="flex items-center justify-between">
          <span>Access Level:</span>
          <span className="font-medium capitalize">{moduleAccess.accessLevel || 'standard'}</span>
        </div>
        
        {moduleAccess.assignedAt && (
          <div className="flex items-center justify-between">
            <span>Assigned:</span>
            <span>{formatDate(moduleAccess.assignedAt)}</span>
          </div>
        )}
        
        {moduleAccess.expiresAt && (
          <div className="flex items-center justify-between">
            <span>Expires:</span>
            <span className={isExpiringSoon ? 'text-orange-600 font-medium' : ''}>
              {formatDate(moduleAccess.expiresAt)}
              {isExpiringSoon && ' (Soon)'}
            </span>
          </div>
        )}

        {moduleAccess.lastUsed && (
          <div className="flex items-center justify-between">
            <span>Last Used:</span>
            <span>{formatDate(moduleAccess.lastUsed)}</span>
          </div>
        )}
      </div>

      {isExpiringSoon && (
        <div className="mt-3 p-2 bg-orange-50 border border-orange-200 rounded text-xs text-orange-800">
          <AlertCircle className="h-3 w-3 inline mr-1" />
          Access expires soon
        </div>
      )}
    </div>
  )
}