/**
 * Module Quota Card Component
 * Shows quota usage with progress indicators
 */

import { MoreHorizontal, BarChart3, AlertTriangle, TrendingUp } from 'lucide-react'
import type { ModuleQuota } from '@/lib/types/unified-user'

interface ModuleQuotaCardProps {
  quota: ModuleQuota
  canManage: boolean
}

export function ModuleQuotaCard({ quota, canManage }: ModuleQuotaCardProps) {
  const usagePercent = Math.round((quota.used / quota.limit) * 100)
  const isNearLimit = usagePercent >= 80
  const isOverLimit = usagePercent >= 100

  const getProgressColor = () => {
    if (isOverLimit) return 'bg-red-500'
    if (isNearLimit) return 'bg-orange-500'
    return 'bg-green-500'
  }

  const formatNumber = (num: number) => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`
    return num.toString()
  }

  return (
    <div className="border border-muted rounded-lg p-4 hover:bg-muted/30 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5 text-blue-500" />
          <div>
            <span className="font-medium">{quota.moduleName}</span>
            <p className="text-xs text-muted-foreground">
              {quota.quotaType} quota
            </p>
          </div>
        </div>

        {canManage && (
          <button className="p-1 hover:bg-muted rounded-md transition-colors">
            <MoreHorizontal className="h-4 w-4" />
            <span className="sr-only">Quota actions</span>
          </button>
        )}
      </div>

      {/* Usage Progress Bar */}
      <div className="mb-3">
        <div className="flex items-center justify-between text-sm mb-1">
          <span>Usage</span>
          <span className="font-medium">
            {formatNumber(quota.used)} / {formatNumber(quota.limit)}
          </span>
        </div>
        <div className="w-full bg-muted rounded-full h-2">
          <div 
            className={`h-2 rounded-full transition-all ${getProgressColor()}`}
            style={{ width: `${Math.min(usagePercent, 100)}%` }}
          />
        </div>
        <div className="flex items-center justify-between mt-1">
          <span className="text-xs text-muted-foreground">
            {usagePercent}% used
          </span>
          {quota.resetDate && (
            <span className="text-xs text-muted-foreground">
              Resets {new Date(quota.resetDate).toLocaleDateString()}
            </span>
          )}
        </div>
      </div>

      {/* Warning Messages */}
      {isOverLimit && (
        <div className="flex items-center gap-1 p-2 bg-red-50 border border-red-200 rounded text-xs text-red-800 mb-2">
          <AlertTriangle className="h-3 w-3" />
          <span>Quota exceeded</span>
        </div>
      )}

      {isNearLimit && !isOverLimit && (
        <div className="flex items-center gap-1 p-2 bg-orange-50 border border-orange-200 rounded text-xs text-orange-800 mb-2">
          <AlertTriangle className="h-3 w-3" />
          <span>Approaching quota limit</span>
        </div>
      )}

      {/* Usage Trends */}
      {quota.dailyUsage && (
        <div className="text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <TrendingUp className="h-3 w-3" />
            <span>Daily average: {formatNumber(quota.dailyUsage)}</span>
          </div>
        </div>
      )}
    </div>
  )
}