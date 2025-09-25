/**
 * Stock Ranking Package Card Component
 * Shows package details with management actions
 */

import { MoreHorizontal, Star, Calendar, AlertTriangle, CheckCircle } from 'lucide-react'
import type { StockRankingPackage } from '@/lib/types/unified-user'
import { Badge } from '@/components/ui/badge'

interface StockRankingPackageCardProps {
  package: StockRankingPackage
  canManage: boolean
}

export function StockRankingPackageCard({ package: pkg, canManage }: StockRankingPackageCardProps) {
  const formatDate = (date: Date | string) => {
    return new Date(date).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    })
  }

  const formatPrice = (price: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(price)
  }

  const getPackageIcon = () => {
    if (pkg.isActive) {
      return <Star className="h-5 w-5 text-yellow-500" />
    }
    return <Star className="h-5 w-5 text-gray-400" />
  }

  // Check if package is expiring soon (within 30 days)
  const isExpiringSoon = pkg.expiresAt && 
    new Date(pkg.expiresAt).getTime() - Date.now() < 30 * 24 * 60 * 60 * 1000

  const daysUntilExpiry = pkg.expiresAt 
    ? Math.ceil((new Date(pkg.expiresAt).getTime() - Date.now()) / (1000 * 60 * 60 * 24))
    : null

  return (
    <div className="border border-muted rounded-lg p-4 hover:bg-muted/30 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          {getPackageIcon()}
          <div>
            <div className="flex items-center gap-2">
              <span className="font-medium">{pkg.name}</span>
              <Badge 
                variant={pkg.isActive ? 'default' : 'secondary'}
              >
                {pkg.isActive ? 'Active' : 'Disabled'}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground">
              Stock ranking package
            </p>
          </div>
        </div>

        {canManage && (
          <button className="min-h-[44px] min-w-[44px] p-2 hover:bg-muted rounded-xl transition-all duration-300 hover:scale-105 active:scale-95 flex items-center justify-center">
            <MoreHorizontal className="h-5 w-5" />
            <span className="sr-only">Package actions</span>
          </button>
        )}
      </div>

      {/* Package Details */}
      <div className="space-y-2 text-xs text-muted-foreground mb-3">
        {pkg.tier && (
          <div className="flex items-center justify-between">
            <span>Tier:</span>
            <span className="font-medium capitalize">{pkg.tier}</span>
          </div>
        )}

        <div className="flex items-center justify-between">
          <span>Assigned:</span>
          <span>{formatDate(pkg.assignedAt)}</span>
        </div>
        
        {pkg.expiresAt && (
          <div className="flex items-center justify-between">
            <span>Expires:</span>
            <span className={isExpiringSoon ? 'text-orange-600 font-medium' : ''}>
              {formatDate(pkg.expiresAt)}
              {daysUntilExpiry !== null && daysUntilExpiry > 0 && (
                <span className="ml-1">
                  ({daysUntilExpiry} days)
                </span>
              )}
            </span>
          </div>
        )}

      </div>

      {/* Package Features */}
      {pkg.features && pkg.features.length > 0 && (
        <div className="mb-3">
          <p className="text-xs font-medium text-muted-foreground mb-1">Features:</p>
          <div className="flex flex-wrap gap-1">
            {pkg.features.slice(0, 3).map((feature, index) => (
              <span 
                key={index}
                className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
              >
                {feature}
              </span>
            ))}
            {pkg.features.length > 3 && (
              <span className="text-xs text-muted-foreground">
                +{pkg.features.length - 3} more
              </span>
            )}
          </div>
        </div>
      )}

      {/* Status Indicators */}
      <div className="space-y-2">
        {isExpiringSoon && daysUntilExpiry !== null && daysUntilExpiry > 0 && (
          <div className="flex items-center gap-1 p-2 bg-orange-50 border border-orange-200 rounded text-xs text-orange-800">
            <AlertTriangle className="h-3 w-3" />
            <span>Expires in {daysUntilExpiry} days</span>
          </div>
        )}


        {!pkg.isActive && (
          <div className="flex items-center gap-1 p-2 bg-gray-50 border border-gray-200 rounded text-xs text-gray-600">
            <Calendar className="h-3 w-3" />
            <span>Package inactive</span>
          </div>
        )}
      </div>
    </div>
  )
}