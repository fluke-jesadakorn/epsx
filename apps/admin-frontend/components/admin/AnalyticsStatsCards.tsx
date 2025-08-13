/**
 * Analytics Stats Cards - Pure Server Component
 * Displays analytics statistics without client-side state
 */

import { TrendingUp, Users, DollarSign, Activity, Clock, AlertTriangle, Key } from 'lucide-react'

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
    if (rate < 1) return 'text-green-600 bg-green-100'
    if (rate < 5) return 'text-yellow-600 bg-yellow-100'
    return 'text-red-600 bg-red-100'
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-6 gap-4">
      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-blue-600">{formatNumber(totalRequests)}</p>
            <p className="text-sm text-muted-foreground">Total Requests</p>
          </div>
          <div className="p-2 bg-blue-100 rounded-lg">
            <Activity className="h-6 w-6 text-blue-500" />
          </div>
        </div>
      </div>
      
      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-green-600">{formatNumber(totalUsers)}</p>
            <p className="text-sm text-muted-foreground">Total Users</p>
          </div>
          <div className="p-2 bg-green-100 rounded-lg">
            <Users className="h-6 w-6 text-green-500" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-purple-600">{formatCurrency(totalRevenue)}</p>
            <p className="text-sm text-muted-foreground">Total Revenue</p>
          </div>
          <div className="p-2 bg-purple-100 rounded-lg">
            <DollarSign className="h-6 w-6 text-purple-500" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-orange-600">{averageResponseTime}ms</p>
            <p className="text-sm text-muted-foreground">Avg Response</p>
          </div>
          <div className="p-2 bg-orange-100 rounded-lg">
            <Clock className="h-6 w-6 text-orange-500" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-red-600">{errorRate.toFixed(2)}%</p>
            <p className="text-sm text-muted-foreground">Error Rate</p>
          </div>
          <div className={`p-2 rounded-lg ${getErrorRateColor(errorRate)}`}>
            <AlertTriangle className="h-6 w-6" />
          </div>
        </div>
      </div>

      <div className="pancake-card p-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xl font-bold text-gray-600">{formatNumber(activeApiKeys)}</p>
            <p className="text-sm text-muted-foreground">Active API Keys</p>
          </div>
          <div className="p-2 bg-gray-100 rounded-lg">
            <Key className="h-6 w-6 text-gray-500" />
          </div>
        </div>
      </div>
    </div>
  )
}