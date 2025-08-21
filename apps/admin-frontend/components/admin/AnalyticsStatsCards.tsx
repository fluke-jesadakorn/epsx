/**
 * Analytics Stats Cards - Pure Server Component
 * Displays analytics statistics without client-side state
 */

import { TrendingUp, Users, DollarSign, Activity, Clock, AlertTriangle, Key } from 'lucide-react'
import { adminCardVariants, cn } from '@/design-system'

interface AnalyticsStatsCardsProps {
  totalRequests: number
  totalUsers: number
  totalRevenue: number
  averageResponseTime: number
  errorRate: number
  activeApiKeys: number
}

export function AnalyticsStatsCards({ 
  totalRequests, 
  totalUsers, 
  totalRevenue,
  averageResponseTime,
  errorRate,
  activeApiKeys
}: AnalyticsStatsCardsProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2
    }).format(amount)
  }

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num)
  }

  const getErrorRateColor = (rate: number) => {
    if (rate < 1) return 'text-success-600 bg-success-100'
    if (rate < 5) return 'text-warning-600 bg-warning-100'
    return 'text-error-600 bg-error-100'
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-6 gap-4">
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-info-600">{formatNumber(totalRequests)}</p>
            <p className="text-sm text-muted-foreground">Total Requests</p>
          </div>
          <div className="p-2 bg-info-100 rounded-lg">
            <Activity className="h-6 w-6 text-info-500" />
          </div>
        </div>
      </div>
      
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-success-600">{formatNumber(totalUsers)}</p>
            <p className="text-sm text-muted-foreground">Total Users</p>
          </div>
          <div className="p-2 bg-success-100 rounded-lg">
            <Users className="h-6 w-6 text-success-500" />
          </div>
        </div>
      </div>

      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-primary-600">{formatCurrency(totalRevenue)}</p>
            <p className="text-sm text-muted-foreground">Total Revenue</p>
          </div>
          <div className="p-2 bg-primary-100 rounded-lg">
            <DollarSign className="h-6 w-6 text-primary-500" />
          </div>
        </div>
      </div>

      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-warning-600">{averageResponseTime}ms</p>
            <p className="text-sm text-muted-foreground">Avg Response</p>
          </div>
          <div className="p-2 bg-warning-100 rounded-lg">
            <Clock className="h-6 w-6 text-warning-500" />
          </div>
        </div>
      </div>

      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-error-600">{errorRate.toFixed(2)}%</p>
            <p className="text-sm text-muted-foreground">Error Rate</p>
          </div>
          <div className={`p-2 rounded-lg ${getErrorRateColor(errorRate)}`}>
            <AlertTriangle className="h-6 w-6" />
          </div>
        </div>
      </div>

      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-neutral-600">{formatNumber(activeApiKeys)}</p>
            <p className="text-sm text-muted-foreground">Active API Keys</p>
          </div>
          <div className="p-2 bg-neutral-100 rounded-lg">
            <Key className="h-6 w-6 text-neutral-500" />
          </div>
        </div>
      </div>
    </div>
  )
}