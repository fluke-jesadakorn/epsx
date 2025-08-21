/**
 * Analytics Overview Server Component - Server-rendered analytics overview
 * Shows time series charts and module breakdowns
 */

import { TrendingUp, Users, Activity, BarChart3 } from 'lucide-react'
import type { TimeSeriesData, ModuleUsageData } from '@/lib/actions/analytics-actions'
import { adminCardVariants, cn } from '@/design-system'

interface AnalyticsOverviewServerProps {
  timeSeriesData: TimeSeriesData[]
  moduleData: ModuleUsageData[]
  dateRange: string
}

export function AnalyticsOverviewServer({ 
  timeSeriesData, 
  moduleData, 
  dateRange 
}: AnalyticsOverviewServerProps) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num)
  }

  // Calculate trends
  const totalRequests = timeSeriesData.reduce((sum, data) => sum + data.requests, 0)
  const totalRevenue = timeSeriesData.reduce((sum, data) => sum + data.revenue, 0)
  const totalUsers = Math.max(...timeSeriesData.map(data => data.users), 0)
  
  // Calculate growth rates (comparing first half vs second half of period)
  const midPoint = Math.floor(timeSeriesData.length / 2)
  const firstHalf = timeSeriesData.slice(0, midPoint)
  const secondHalf = timeSeriesData.slice(midPoint)
  
  const firstHalfRequests = firstHalf.reduce((sum, data) => sum + data.requests, 0)
  const secondHalfRequests = secondHalf.reduce((sum, data) => sum + data.requests, 0)
  const requestsGrowth = firstHalfRequests > 0 ? ((secondHalfRequests - firstHalfRequests) / firstHalfRequests) * 100 : 0

  return (
    <div className="space-y-6">
      {/* Time Series Overview */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Requests Timeline */}
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Request Timeline
          </h3>
          
          <div className="mb-4">
            <div className="text-2xl font-bold text-info-600">{formatNumber(totalRequests)}</div>
            <div className="text-sm text-neutral-600">Total requests in {dateRange}</div>
            <div className={`text-xs flex items-center gap-1 mt-1 ${
              requestsGrowth >= 0 ? 'text-success-600' : 'text-error-600'
            }`}>
              <TrendingUp className="w-3 h-3" />
              {Math.abs(requestsGrowth).toFixed(1)}% {requestsGrowth >= 0 ? 'growth' : 'decline'}
            </div>
          </div>
          
          <TimeSeriesChart data={timeSeriesData} dataKey="requests" color="#3B82F6" />
        </div>

        {/* Revenue Timeline */}
        <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Revenue Timeline
          </h3>
          
          <div className="mb-4">
            <div className="text-2xl font-bold text-success-600">{formatCurrency(totalRevenue)}</div>
            <div className="text-sm text-neutral-600">Total revenue in {dateRange}</div>
            <div className="text-xs text-neutral-500 mt-1">
              Avg: {formatCurrency(totalRevenue / timeSeriesData.length)} per day
            </div>
          </div>
          
          <TimeSeriesChart data={timeSeriesData} dataKey="revenue" color="#10B981" />
        </div>
      </div>

      {/* Module Usage Breakdown */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-6 flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          Module Usage Breakdown
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {moduleData.map((module, index) => (
            <ModuleUsageCard key={module.moduleName} module={module} />
          ))}
        </div>
      </div>

      {/* Top Performing Modules */}
      <div className={cn(adminCardVariants({ variant: 'pancake' }))}>
        <h3 className="text-lg font-semibold mb-4">Top Performing Modules</h3>
        <TopModulesTable moduleData={moduleData} />
      </div>
    </div>
  )
}

/**
 * Simple Time Series Chart using SVG - Server Component
 */
function TimeSeriesChart({ 
  data, 
  dataKey, 
  color 
}: { 
  data: TimeSeriesData[]
  dataKey: keyof TimeSeriesData
  color: string
}) {
  if (data.length === 0) {
    return (
      <div className="h-32 flex items-center justify-center text-neutral-400">
        <span>No data available</span>
      </div>
    )
  }

  const maxValue = Math.max(...data.map(d => Number(d[dataKey])))
  const minValue = Math.min(...data.map(d => Number(d[dataKey])))
  const range = maxValue - minValue
  
  const width = 300
  const height = 120
  const padding = 20

  const points = data.map((d, i) => {
    const x = padding + (i / (data.length - 1)) * (width - 2 * padding)
    const y = height - padding - ((Number(d[dataKey]) - minValue) / range) * (height - 2 * padding)
    return `${x},${y}`
  }).join(' ')

  return (
    <div className="h-32">
      <svg width="100%" height="100%" viewBox={`0 0 ${width} ${height}`} className="overflow-visible">
        <defs>
          <linearGradient id={`gradient-${dataKey}`} x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor={color} stopOpacity="0.3" />
            <stop offset="100%" stopColor={color} stopOpacity="0.05" />
          </linearGradient>
        </defs>
        
        {/* Grid lines */}
        <g className="stroke-neutral-200" strokeWidth="1">
          {[0, 25, 50, 75, 100].map(percent => {
            const y = height - padding - (percent / 100) * (height - 2 * padding)
            return <line key={percent} x1={padding} y1={y} x2={width - padding} y2={y} />
          })}
        </g>
        
        {/* Area fill */}
        <path
          d={`M ${padding},${height - padding} L ${points} L ${width - padding},${height - padding} Z`}
          fill={`url(#gradient-${dataKey})`}
        />
        
        {/* Line */}
        <polyline
          points={points}
          fill="none"
          stroke={color}
          strokeWidth="2"
          strokeLinejoin="round"
          strokeLinecap="round"
        />
        
        {/* Data points */}
        {data.map((d, i) => {
          const x = padding + (i / (data.length - 1)) * (width - 2 * padding)
          const y = height - padding - ((Number(d[dataKey]) - minValue) / range) * (height - 2 * padding)
          return (
            <circle
              key={i}
              cx={x}
              cy={y}
              r="3"
              fill={color}
              className="drop-shadow-sm"
            />
          )
        })}
      </svg>
    </div>
  )
}

/**
 * Module Usage Card - Server Component
 */
function ModuleUsageCard({ module }: { module: ModuleUsageData }) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num)
  }

  const getQuotaColor = (percentage: number) => {
    if (percentage < 50) return 'bg-success-500'
    if (percentage < 80) return 'bg-warning-500'
    return 'bg-error-500'
  }

  return (
    <div className="border border-neutral-200 rounded-lg p-4 bg-white">
      <div className="flex items-center justify-between mb-2">
        <h4 className="font-medium text-neutral-900">{module.moduleName}</h4>
        <div className={`w-2 h-2 rounded-full ${getQuotaColor(module.quotaPercentage)}`} />
      </div>
      
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-neutral-600">Requests:</span>
          <span className="font-medium">{formatNumber(module.requests)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-neutral-600">Users:</span>
          <span className="font-medium">{formatNumber(module.users)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-neutral-600">Revenue:</span>
          <span className="font-medium text-success-600">{formatCurrency(module.revenue)}</span>
        </div>
      </div>
      
      <div className="mt-3">
        <div className="flex justify-between text-xs text-neutral-600 mb-1">
          <span>Quota Usage</span>
          <span>{module.quotaPercentage.toFixed(1)}%</span>
        </div>
        <div className="w-full bg-neutral-200 rounded-full h-2">
          <div 
            className={`h-2 rounded-full ${getQuotaColor(module.quotaPercentage)}`}
            style={{ width: `${Math.min(module.quotaPercentage, 100)}%` }}
          />
        </div>
        <div className="text-xs text-neutral-500 mt-1">
          {formatNumber(module.quotaUsed)} / {formatNumber(module.quota)}
        </div>
      </div>
    </div>
  )
}

/**
 * Top Modules Table - Server Component
 */
function TopModulesTable({ moduleData }: { moduleData: ModuleUsageData[] }) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  const sortedModules = [...moduleData].sort((a, b) => b.revenue - a.revenue)

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full">
        <thead>
          <tr className="text-left text-xs font-medium text-neutral-500 uppercase tracking-wider">
            <th className="pb-2">Rank</th>
            <th className="pb-2">Module</th>
            <th className="pb-2">Requests</th>
            <th className="pb-2">Users</th>
            <th className="pb-2">Revenue</th>
            <th className="pb-2">Quota Usage</th>
          </tr>
        </thead>
        <tbody className="space-y-2">
          {sortedModules.map((module, index) => (
            <tr key={module.moduleName} className="border-t border-neutral-100">
              <td className="py-2 text-sm font-medium text-neutral-900">#{index + 1}</td>
              <td className="py-2 text-sm font-medium">{module.moduleName}</td>
              <td className="py-2 text-sm">{module.requests.toLocaleString()}</td>
              <td className="py-2 text-sm">{module.users.toLocaleString()}</td>
              <td className="py-2 text-sm font-medium text-success-600">
                {formatCurrency(module.revenue)}
              </td>
              <td className="py-2 text-sm">
                <span className={`px-2 py-1 rounded-full text-xs ${
                  module.quotaPercentage < 50 
                    ? 'bg-success-100 text-success-800'
                    : module.quotaPercentage < 80
                    ? 'bg-warning-100 text-warning-800'
                    : 'bg-error-100 text-error-800'
                }`}>
                  {module.quotaPercentage.toFixed(1)}%
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}