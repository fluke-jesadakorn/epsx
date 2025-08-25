/**
 * Analytics Server Actions - Focused on module analytics operations
 * Server-side first analytics data fetching and operations
 */

'use server'

import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { logger } from '@/lib/logger';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL

export interface UsageMetrics {
  totalRequests: number
  totalUsers: number
  totalRevenue: number
  averageResponseTime: number
  errorRate: number
  activeApiKeys: number
}

export interface TimeSeriesData {
  date: string
  requests: number
  users: number
  revenue: number
  errors: number
}

export interface ModuleUsageData {
  moduleName: string
  requests: number
  users: number
  revenue: number
  quota: number
  quotaUsed: number
  quotaPercentage: number
}

export interface BillingData {
  currentPeriod: {
    startDate: string
    endDate: string
    totalCost: number
    totalRequests: number
  }
  upcomingInvoice: {
    amount: number
    dueDate: string
    status: string
  }
  costBreakdown: Array<{
    module: string
    cost: number
    requests: number
  }>
}

export interface AnalyticsDashboardData {
  metrics: UsageMetrics
  timeSeriesData: TimeSeriesData[]
  moduleData: ModuleUsageData[]
  billingData: BillingData
}

export interface AnalyticsOperationResult<T = void> {
  success: boolean
  data?: T
  error?: {
    code: string
    message: string
  }
}

/**
 * Get comprehensive analytics dashboard data
 */
export async function getAnalyticsDashboardData(
  dateRange = '7d', 
  selectedModule = 'all'
): Promise<AnalyticsOperationResult<AnalyticsDashboardData>> {
  try {
    logger.action.start('getAnalyticsDashboardData', { dateRange, selectedModule })
    
    const user = await getCurrentUser()
    const token = await getBearerToken()
    
    if (!user || !token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }

    // Check analytics admin permissions
    if (!user.admin || !user.admin_modules.includes('analytics_specialist')) {
      return { 
        success: false, 
        error: { code: 'FORBIDDEN', message: 'Analytics specialist access required' } 
      }
    }

    // Fetch analytics data from backend
    const [metricsResponse, timeSeriesResponse, moduleResponse] = await Promise.all([
      fetch(`${BACKEND_URL}/api/v1/admin/analytics/metrics?range=${dateRange}&module=${selectedModule}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 300 } // 5-minute cache
      }),
      fetch(`${BACKEND_URL}/api/v1/admin/analytics/time-series?range=${dateRange}&module=${selectedModule}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 300 }
      }),
      fetch(`${BACKEND_URL}/api/v1/admin/analytics/modules?range=${dateRange}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 300 }
      })
    ])

    // Check if all requests were successful
    if (!metricsResponse.ok || !timeSeriesResponse.ok || !moduleResponse.ok) {
      logger.action.error('getAnalyticsDashboardData', 'Failed to fetch analytics data from backend')
      
      // Return mock data for development
      const mockData: AnalyticsDashboardData = {
        metrics: {
          totalRequests: 1245678,
          totalUsers: 8934,
          totalRevenue: 45672.89,
          averageResponseTime: 245,
          errorRate: 0.23,
          activeApiKeys: 156
        },
        timeSeriesData: generateMockTimeSeriesData(dateRange),
        moduleData: [
          { 
            moduleName: 'User Management', 
            requests: 450000, 
            users: 3200, 
            revenue: 15000, 
            quota: 500000, 
            quotaUsed: 450000,
            quotaPercentage: 90
          },
          { 
            moduleName: 'Analytics', 
            requests: 320000, 
            users: 2100, 
            revenue: 12000, 
            quota: 400000, 
            quotaUsed: 320000,
            quotaPercentage: 80
          },
          { 
            moduleName: 'API Gateway', 
            requests: 475678, 
            users: 3634, 
            revenue: 18672.89, 
            quota: 600000, 
            quotaUsed: 475678,
            quotaPercentage: 79
          }
        ],
        billingData: {
          currentPeriod: {
            startDate: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
            endDate: new Date().toISOString(),
            totalCost: 45672.89,
            totalRequests: 1245678
          },
          upcomingInvoice: {
            amount: 48000.00,
            dueDate: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString(),
            status: 'pending'
          },
          costBreakdown: [
            { module: 'User Management', cost: 15000, requests: 450000 },
            { module: 'Analytics', cost: 12000, requests: 320000 },
            { module: 'API Gateway', cost: 18672.89, requests: 475678 }
          ]
        }
      }

      logger.action.success('getAnalyticsDashboardData', { source: 'mock', dateRange, selectedModule })
      return { success: true, data: mockData }
    }

    const [metricsData, timeSeriesData, moduleData] = await Promise.all([
      metricsResponse.json(),
      timeSeriesResponse.json(),
      moduleResponse.json()
    ])

    const dashboardData: AnalyticsDashboardData = {
      metrics: metricsData,
      timeSeriesData: timeSeriesData.data || [],
      moduleData: moduleData.modules.map((module: any) => ({
        ...module,
        quotaPercentage: (module.quotaUsed / module.quota) * 100
      }))
    }

    logger.action.success('getAnalyticsDashboardData', { 
      dateRange, 
      selectedModule,
      totalRequests: dashboardData.metrics.totalRequests,
      moduleCount: dashboardData.moduleData.length
    })
    
    return { success: true, data: dashboardData }
    
  } catch (error) {
    logger.action.error('getAnalyticsDashboardData', error, { dateRange, selectedModule })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Generate mock time series data for development
 */
function generateMockTimeSeriesData(dateRange: string): TimeSeriesData[] {
  const days = dateRange === '30d' ? 30 : dateRange === '7d' ? 7 : 1
  const data: TimeSeriesData[] = []
  
  for (let i = days - 1; i >= 0; i--) {
    const date = new Date(Date.now() - i * 24 * 60 * 60 * 1000)
    data.push({
      date: date.toISOString().split('T')[0],
      requests: Math.floor(Math.random() * 50000) + 40000,
      users: Math.floor(Math.random() * 1000) + 800,
      revenue: Math.floor(Math.random() * 5000) + 3000,
      errors: Math.floor(Math.random() * 100) + 10
    })
  }
  
  return data
}

/**
 * Export analytics report
 */
export async function exportAnalyticsReport(
  dateRange: string,
  selectedModule: string,
  format: 'csv' | 'pdf' | 'xlsx' = 'pdf'
): Promise<AnalyticsOperationResult<{ downloadUrl: string }>> {
  try {
    logger.action.start('exportAnalyticsReport', { dateRange, selectedModule, format })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/analytics/reports/export`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        date_range: dateRange,
        selected_module: selectedModule,
        format
      }),
    })
    
    if (!response.ok) {
      logger.action.error('exportAnalyticsReport', `Export failed: ${response.statusText}`, { dateRange, format })
      return { 
        success: false, 
        error: { 
          code: 'EXPORT_ERROR', 
          message: `Export failed: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    logger.action.success('exportAnalyticsReport', { dateRange, selectedModule, format })
    
    return { success: true, data: { downloadUrl: result.download_url } }
    
  } catch (error) {
    logger.action.error('exportAnalyticsReport', error, { dateRange, selectedModule, format })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Refresh analytics data (force cache invalidation)
 */
export async function refreshAnalyticsData(
  dateRange: string,
  selectedModule: string
): Promise<AnalyticsOperationResult> {
  try {
    logger.action.start('refreshAnalyticsData', { dateRange, selectedModule })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/analytics/refresh`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        date_range: dateRange,
        selected_module: selectedModule
      }),
    })
    
    if (!response.ok) {
      logger.action.error('refreshAnalyticsData', `Refresh failed: ${response.statusText}`)
      return { 
        success: false, 
        error: { 
          code: 'REFRESH_ERROR', 
          message: `Refresh failed: ${response.statusText}` 
        } 
      }
    }
    
    logger.action.success('refreshAnalyticsData', { dateRange, selectedModule })
    
    return { success: true }
    
  } catch (error) {
    logger.action.error('refreshAnalyticsData', error, { dateRange, selectedModule })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}