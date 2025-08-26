/**
 * User Packages Content Component
 * Stock ranking packages management interface
 */

import { Star, Package, Calendar, Plus, TrendingUp, DollarSign } from 'lucide-react'
import type { UnifiedUserData } from '@/lib/types/unified-user'
import type { EnhancedAuthUser } from '@/lib/auth/server-auth'
import { StockRankingPackageCard } from './StockRankingPackageCard'
import { StatsCard } from '@/components/ui/StatsCard'
import { adminCardVariants, adminButtonVariants, cn } from '@/design-system'

interface UserPackagesContentProps {
  user: UnifiedUserData
  currentUser: EnhancedAuthUser
}

export function UserPackagesContent({ user, currentUser }: UserPackagesContentProps) {
  const canManagePackages = currentUser.role === 'admin' || currentUser.canManageUsers

  // Calculate package stats
  const activePackages = user.stockRankingPackages.filter(p => p.isActive).length
  const totalValue = user.stockRankingPackages.reduce((sum, p) => sum + (p.price || 0), 0)
  const expiringSoon = user.stockRankingPackages.filter(p => {
    if (!p.expiresAt) return false
    const daysUntilExpiry = Math.ceil((new Date(p.expiresAt).getTime() - Date.now()) / (1000 * 60 * 60 * 24))
    return daysUntilExpiry <= 30 && daysUntilExpiry > 0
  }).length

  const packageStats = [
    {
      title: 'Active Packages',
      value: activePackages,
      description: `of ${user.stockRankingPackages.length} total`,
      icon: Package,
      color: 'blue'
    },
    {
      title: 'Package Value',
      value: `$${totalValue.toLocaleString()}`,
      description: 'Total subscription value',
      icon: DollarSign,
      color: 'green'
    },
    {
      title: 'Expiring Soon',
      value: expiringSoon,
      description: 'Within 30 days',
      icon: Calendar,
      color: expiringSoon > 0 ? 'warning' : 'neutral'
    }
  ]

  // Group packages by status
  const activePackagesList = user.stockRankingPackages.filter(p => p.isActive)
  const inactivePackagesList = user.stockRankingPackages.filter(p => !p.isActive)

  return (
    <div className="space-y-6">
      {/* Package Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {packageStats.map((stat, index) => (
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

      {/* Active Packages */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Star className="h-5 w-5 text-warning-500" />
            Active Packages
          </h3>
          {canManagePackages && (
            <button className={cn(adminButtonVariants({ variant: 'primary', size: 'sm' }))}>
              <Plus className="h-4 w-4" />
              Add Package
            </button>
          )}
        </div>
        
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {activePackagesList.length > 0 ? (
            activePackagesList.map((pkg) => (
              <StockRankingPackageCard 
                key={pkg.id}
                package={pkg}
                canManage={canManagePackages}
              />
            ))
          ) : (
            <div className="col-span-2 text-center text-muted-foreground py-8">
              <Package className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No active packages</p>
            </div>
          )}
        </div>
      </div>

      {/* Inactive Packages */}
      {inactivePackagesList.length > 0 && (
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Package className="h-5 w-5 text-neutral-400" />
            Inactive Packages
          </h3>
          
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {inactivePackagesList.map((pkg) => (
              <StockRankingPackageCard 
                key={pkg.id}
                package={pkg}
                canManage={canManagePackages}
              />
            ))}
          </div>
        </div>
      )}

      {/* Package Usage Analytics */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          Package Usage Analytics
        </h3>
        
        {/* Placeholder for usage analytics */}
        <div className="text-center text-muted-foreground py-8">
          <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
          <p>Usage analytics will be shown here</p>
          <p className="text-xs">Track package utilization, feature usage, and value metrics</p>
        </div>
      </div>

      {/* Billing Integration */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <DollarSign className="h-5 w-5" />
          Billing Summary
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="p-4 bg-muted/30 rounded-lg">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Monthly Recurring</span>
              <span className="font-medium">
                ${user.stockRankingPackages
                  .filter(p => p.isActive && p.billingCycle === 'monthly')
                  .reduce((sum, p) => sum + (p.price || 0), 0)
                  .toLocaleString()}
              </span>
            </div>
          </div>
          
          <div className="p-4 bg-muted/30 rounded-lg">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Annual Recurring</span>
              <span className="font-medium">
                ${user.stockRankingPackages
                  .filter(p => p.isActive && p.billingCycle === 'annual')
                  .reduce((sum, p) => sum + (p.price || 0), 0)
                  .toLocaleString()}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}