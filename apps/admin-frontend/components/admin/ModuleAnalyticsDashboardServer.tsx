/**
 * Module Analytics Dashboard Server Component - Server-side first architecture
 * Fetches analytics data on the server and renders dashboard with client components for interactions
 */

import { TrendingUp, Users, DollarSign, Activity, AlertTriangle, BarChart3, RefreshCcw } from 'lucide-react'
import { getCurrentUser } from '@/lib/actions/server-auth'
import { getAnalyticsDashboardData } from '@/lib/actions/analytics-actions'
import { AnalyticsStatsCards } from './AnalyticsStatsCards'
import { AnalyticsTabNavigation } from './AnalyticsTabNavigation'
import { AnalyticsOverviewServer } from './AnalyticsOverviewServer'
import { AnalyticsUsageServer } from './AnalyticsUsageServer'
import { AnalyticsExportButton } from './AnalyticsExportButton'
import { AnalyticsRefreshButton } from './AnalyticsRefreshButton'

interface ModuleAnalyticsDashboardServerProps {
  searchParams: {
    tab?: string
    dateRange?: string
    selectedModule?: string
  }
}

export async function ModuleAnalyticsDashboardServer({ searchParams }: ModuleAnalyticsDashboardServerProps) {
  const currentUser = await getCurrentUser()
  
  // Check authentication
  if (!currentUser) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Authentication Required</h3>
          <p className="text-gray-600">Please sign in to access analytics.</p>
        </div>
      </div>
    )
  }

  // Check analytics permissions
  const canViewAnalytics = currentUser.admin && 
    (currentUser.admin_modules.includes('analytics_specialist') || 
     currentUser.admin_modules.includes('system_admin'))

  if (!canViewAnalytics) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Access Restricted</h3>
          <p className="text-gray-600">You don&apos;t have permission to view analytics.</p>
        </div>
      </div>
    )
  }

  // Fetch analytics data on the server
  const dateRange = searchParams.dateRange || '7d'
  const selectedModule = searchParams.selectedModule || 'all'
  const analyticsResult = await getAnalyticsDashboardData(dateRange, selectedModule)

  if (!analyticsResult.success) {
    return (
      <div className="max-w-7xl mx-auto p-6">
        <div className="pancake-card p-6 text-center">
          <AlertTriangle className="w-12 h-12 text-red-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Failed to Load Analytics Data</h3>
          <p className="text-gray-600">{analyticsResult.error?.message}</p>
        </div>
      </div>
    )
  }

  const { metrics, timeSeriesData, moduleData, billingData } = analyticsResult.data
  const activeTab = searchParams.tab || 'overview'

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-2">
            <BarChart3 className="h-8 w-8" />
            Module Analytics
          </h1>
          <p className="text-gray-600">Monitor usage, performance, and trends across all modules</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <AnalyticsFiltersForm 
            currentDateRange={dateRange}
            currentModule={selectedModule}
            availableModules={moduleData.map(m => m.moduleName)}
          />
          <AnalyticsRefreshButton dateRange={dateRange} selectedModule={selectedModule} />
          <AnalyticsExportButton dateRange={dateRange} selectedModule={selectedModule} />
        </div>
      </div>

      {/* Tab Navigation */}
      <AnalyticsTabNavigation activeTab={activeTab} />

      {/* Stats Cards - Always visible */}
      <AnalyticsStatsCards 
        totalRequests={metrics.totalRequests}
        totalUsers={metrics.totalUsers}
        totalRevenue={metrics.totalRevenue}
        averageResponseTime={metrics.averageResponseTime}
        errorRate={metrics.errorRate}
        activeApiKeys={metrics.activeApiKeys}
      />

      {/* Tab Content */}
      {activeTab === 'overview' && (
        <AnalyticsOverviewServer 
          timeSeriesData={timeSeriesData}
          moduleData={moduleData}
          dateRange={dateRange}
        />
      )}

      {activeTab === 'usage' && (
        <AnalyticsUsageServer 
          moduleData={moduleData}
          timeSeriesData={timeSeriesData}
          selectedModule={selectedModule}
        />
      )}

      {activeTab === 'billing' && (
        <div className="space-y-6">
          <div className="pancake-card p-6">
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <DollarSign className="h-5 w-5" />
              Billing Overview
            </h3>
            <BillingOverviewCards billingData={billingData} />
          </div>
          
          <div className="pancake-card p-6">
            <h3 className="text-lg font-semibold mb-4">Cost Breakdown by Module</h3>
            <CostBreakdownTable costBreakdown={billingData.costBreakdown} />
          </div>
        </div>
      )}

      {activeTab === 'performance' && (
        <div className="pancake-card p-6">
          <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Performance Metrics
          </h3>
          <PerformanceMetricsDisplay 
            averageResponseTime={metrics.averageResponseTime}
            errorRate={metrics.errorRate}
            timeSeriesData={timeSeriesData}
          />
        </div>
      )}
    </div>
  )
}

/**
 * Analytics Filters Form - Server Component with form submission
 */
function AnalyticsFiltersForm({ 
  currentDateRange, 
  currentModule, 
  availableModules 
}: { 
  currentDateRange: string
  currentModule: string
  availableModules: string[]
}) {
  return (
    <form method="get" className="flex gap-2">
      <select 
        name="dateRange"
        defaultValue={currentDateRange}
        className="border rounded-md px-3 py-2 text-sm"
      >
        <option value="1d">Last 24 Hours</option>
        <option value="7d">Last 7 Days</option>
        <option value="30d">Last 30 Days</option>
        <option value="90d">Last 90 Days</option>
      </select>
      
      <select 
        name="selectedModule"
        defaultValue={currentModule}
        className="border rounded-md px-3 py-2 text-sm"
      >
        <option value="all">All Modules</option>
        {availableModules.map(module => (
          <option key={module} value={module}>{module}</option>
        ))}
      </select>
      
      <button 
        type="submit"
        className="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700"
      >
        Apply
      </button>
    </form>
  )
}

/**
 * Billing Overview Cards - Server Component
 */
function BillingOverviewCards({ billingData }: { billingData: any }) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      <div className="bg-blue-50 p-4 rounded-lg">
        <div className="text-2xl font-bold text-blue-600">
          {formatCurrency(billingData.currentPeriod.totalCost)}
        </div>
        <div className="text-sm text-blue-700">Current Period Cost</div>
        <div className="text-xs text-blue-600 mt-1">
          {billingData.currentPeriod.totalRequests.toLocaleString()} requests
        </div>
      </div>
      
      <div className="bg-green-50 p-4 rounded-lg">
        <div className="text-2xl font-bold text-green-600">
          {formatCurrency(billingData.upcomingInvoice.amount)}
        </div>
        <div className="text-sm text-green-700">Upcoming Invoice</div>
        <div className="text-xs text-green-600 mt-1">
          Due: {new Date(billingData.upcomingInvoice.dueDate).toLocaleDateString()}
        </div>
      </div>
      
      <div className="bg-purple-50 p-4 rounded-lg">
        <div className="text-2xl font-bold text-purple-600">
          {billingData.costBreakdown.length}
        </div>
        <div className="text-sm text-purple-700">Active Modules</div>
        <div className="text-xs text-purple-600 mt-1">generating costs</div>
      </div>
    </div>
  )
}

/**
 * Cost Breakdown Table - Server Component
 */
function CostBreakdownTable({ costBreakdown }: { costBreakdown: any[] }) {
  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount)
  }

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Module</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Cost</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Requests</th>
            <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Cost per 1K</th>
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200">
          {costBreakdown.map((item, index) => (
            <tr key={index}>
              <td className="px-6 py-4 whitespace-nowrap font-medium">{item.module}</td>
              <td className="px-6 py-4 whitespace-nowrap">{formatCurrency(item.cost)}</td>
              <td className="px-6 py-4 whitespace-nowrap">{item.requests.toLocaleString()}</td>
              <td className="px-6 py-4 whitespace-nowrap">
                ${((item.cost / item.requests) * 1000).toFixed(4)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

/**
 * Performance Metrics Display - Server Component
 */
function PerformanceMetricsDisplay({ 
  averageResponseTime, 
  errorRate, 
  timeSeriesData 
}: { 
  averageResponseTime: number
  errorRate: number
  timeSeriesData: any[]
}) {
  const totalErrors = timeSeriesData.reduce((sum, data) => sum + data.errors, 0)
  const totalRequests = timeSeriesData.reduce((sum, data) => sum + data.requests, 0)
  const calculatedErrorRate = totalRequests > 0 ? (totalErrors / totalRequests) * 100 : 0

  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div className="text-center">
        <div className="text-3xl font-bold text-blue-600">{averageResponseTime}ms</div>
        <div className="text-sm text-gray-600">Avg Response Time</div>
      </div>
      
      <div className="text-center">
        <div className="text-3xl font-bold text-red-600">{errorRate.toFixed(2)}%</div>
        <div className="text-sm text-gray-600">Error Rate</div>
      </div>
      
      <div className="text-center">
        <div className="text-3xl font-bold text-green-600">{totalErrors.toLocaleString()}</div>
        <div className="text-sm text-gray-600">Total Errors</div>
      </div>
      
      <div className="text-center">
        <div className="text-3xl font-bold text-purple-600">
          {(99.99 - calculatedErrorRate).toFixed(2)}%
        </div>
        <div className="text-sm text-gray-600">Uptime</div>
      </div>
    </div>
  )
}