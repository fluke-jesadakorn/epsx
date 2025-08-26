/**
 * User Modules Content Component
 * Module assignment and quota management interface
 */

import { Package, BarChart3, Settings, Plus, TrendingUp } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth'
import { ModuleAccessCard } from './ModuleAccessCard'
import { ModuleQuotaCard } from './ModuleQuotaCard'
import { StatsCard } from '@/components/ui/StatsCard'
import { adminCardVariants, adminButtonVariants, cn } from '@/design-system'

interface UserModulesContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function UserModulesContent({ user, currentUser }: UserModulesContentProps) {
  const canManageModules = currentUser.role === 'admin' || currentUser.canManageUsers

  // Calculate module stats
  const activeModules = user.moduleAccess.filter(m => m.isActive).length
  const totalQuotaUsed = user.moduleQuotas.reduce((sum, q) => sum + q.used, 0)
  const totalQuotaLimit = user.moduleQuotas.reduce((sum, q) => sum + q.limit, 0)
  const quotaUsagePercent = totalQuotaLimit > 0 ? Math.round((totalQuotaUsed / totalQuotaLimit) * 100) : 0

  const moduleStats = [
    {
      title: 'Active Modules',
      value: activeModules,
      description: `of ${user.moduleAccess.length} total`,
      icon: Package,
      color: 'blue'
    },
    {
      title: 'Quota Usage',
      value: `${quotaUsagePercent}%`,
      description: `${totalQuotaUsed}/${totalQuotaLimit}`,
      icon: BarChart3,
      color: quotaUsagePercent > 80 ? 'error' : quotaUsagePercent > 60 ? 'warning' : 'success'
    },
    {
      title: 'API Calls Today',
      value: user.usageMetrics.apiCallsToday || 0,
      description: 'Across all modules',
      icon: TrendingUp,
      color: 'purple'
    }
  ]

  return (
    <div className="space-y-6">
      {/* Module Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {moduleStats.map((stat, index) => (
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

      {/* Module Access */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Package className="h-5 w-5" />
            Module Access
          </h3>
          {canManageModules && (
            <button className={cn(adminButtonVariants({ variant: 'primary', size: 'sm' }))}>
              <Plus className="h-4 w-4" />
              Assign Module
            </button>
          )}
        </div>
        
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {user.moduleAccess.length > 0 ? (
            user.moduleAccess.map((moduleAccess) => (
              <ModuleAccessCard 
                key={moduleAccess.id}
                moduleAccess={moduleAccess}
                canManage={canManageModules}
              />
            ))
          ) : (
            <div className="col-span-2 text-center text-muted-foreground py-8">
              <Package className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No modules assigned</p>
            </div>
          )}
        </div>
      </div>

      {/* Module Quotas */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <BarChart3 className="h-5 w-5" />
            Usage Quotas
          </h3>
          {canManageModules && (
            <button className={cn(adminButtonVariants({ variant: 'success', size: 'sm' }))}>
              <Settings className="h-4 w-4" />
              Manage Quotas
            </button>
          )}
        </div>
        
        <div className="space-y-4">
          {user.moduleQuotas.length > 0 ? (
            user.moduleQuotas.map((quota) => (
              <ModuleQuotaCard 
                key={quota.id}
                quota={quota}
                canManage={canManageModules}
              />
            ))
          ) : (
            <div className="text-center text-muted-foreground py-8">
              <BarChart3 className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No quotas configured</p>
            </div>
          )}
        </div>
      </div>

      {/* Module Usage History */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          Usage History
        </h3>
        
        {/* Placeholder for usage charts/history */}
        <div className="text-center text-muted-foreground py-8">
          <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
          <p>Usage analytics will be shown here</p>
          <p className="text-xs">Track module usage patterns and quota consumption</p>
        </div>
      </div>
    </div>
  )
}