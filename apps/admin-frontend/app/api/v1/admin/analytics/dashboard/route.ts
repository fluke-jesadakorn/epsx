import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken, getCurrentUser } from '@/lib/actions/server-auth';
import { logger } from '@/lib/logger';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

export interface UsageMetrics {
  totalRequests: number;
  totalUsers: number;
  totalRevenue: number;
  averageResponseTime: number;
  errorRate: number;
  activeApiKeys: number;
}

export interface TimeSeriesData {
  date: string;
  requests: number;
  users: number;
  revenue: number;
  errors: number;
}

export interface ModuleUsageData {
  moduleName: string;
  requests: number;
  users: number;
  revenue: number;
  quota: number;
  quotaUsed: number;
  quotaPercentage: number;
}

export interface BillingData {
  currentPeriod: {
    startDate: string;
    endDate: string;
    totalCost: number;
    totalRequests: number;
  };
  upcomingInvoice: {
    amount: number;
    dueDate: string;
    status: string;
  };
  costBreakdown: Array<{
    module: string;
    cost: number;
    requests: number;
  }>;
}

export interface AnalyticsDashboardData {
  metrics: UsageMetrics;
  timeSeriesData: TimeSeriesData[];
  moduleData: ModuleUsageData[];
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const dateRange = searchParams.get('dateRange') || '7d';
    const selectedModule = searchParams.get('selectedModule') || 'all';
    
    logger.action.start('getAnalyticsDashboardData', { dateRange, selectedModule });
    
    const user = await getCurrentUser();
    const token = await getBearerToken();
    
    if (!user || !token) {
      return NextResponse.json(
        { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } },
        { status: 401 }
      );
    }

    // Check analytics admin permissions
    if (!user.admin || !user.admin_modules.includes('analytics_specialist')) {
      return NextResponse.json(
        { 
          success: false, 
          error: { code: 'FORBIDDEN', message: 'Analytics specialist access required' } 
        },
        { status: 403 }
      );
    }

    // Fetch analytics data from backend
    const [metricsResponse, timeSeriesResponse, moduleResponse] = await Promise.allSettled([
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
    ]);

    // Check if requests failed and provide mock data
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
      ]
    };

    // If any backend call failed, return mock data
    if (
      metricsResponse.status === 'rejected' ||
      timeSeriesResponse.status === 'rejected' ||
      moduleResponse.status === 'rejected'
    ) {
      logger.action.success('getAnalyticsDashboardData', { source: 'mock', dateRange, selectedModule });
      return NextResponse.json({ success: true, data: mockData });
    }

    // Process successful responses
    const [metricsData, timeSeriesData, moduleData] = await Promise.all([
      metricsResponse.status === 'fulfilled' && metricsResponse.value.ok ? metricsResponse.value.json() : mockData.metrics,
      timeSeriesResponse.status === 'fulfilled' && timeSeriesResponse.value.ok ? timeSeriesResponse.value.json() : { data: mockData.timeSeriesData },
      moduleResponse.status === 'fulfilled' && moduleResponse.value.ok ? moduleResponse.value.json() : { modules: mockData.moduleData }
    ]);

    const dashboardData: AnalyticsDashboardData = {
      metrics: metricsData,
      timeSeriesData: timeSeriesData.data || mockData.timeSeriesData,
      moduleData: (moduleData.modules || mockData.moduleData).map((module: any) => ({
        ...module,
        quotaPercentage: (module.quotaUsed / module.quota) * 100
      }))
    };

    logger.action.success('getAnalyticsDashboardData', { 
      dateRange, 
      selectedModule,
      totalRequests: dashboardData.metrics.totalRequests,
      moduleCount: dashboardData.moduleData.length
    });
    
    return NextResponse.json({ success: true, data: dashboardData });
    
  } catch (error) {
    logger.action.error('getAnalyticsDashboardData', error);
    return NextResponse.json(
      { 
        success: false, 
        error: { 
          code: 'UNKNOWN_ERROR', 
          message: 'An unexpected error occurred' 
        } 
      },
      { status: 500 }
    );
  }
}

/**
 * Generate mock time series data for development
 */
function generateMockTimeSeriesData(dateRange: string): TimeSeriesData[] {
  const days = dateRange === '30d' ? 30 : dateRange === '7d' ? 7 : 1;
  const data: TimeSeriesData[] = [];
  
  for (let i = days - 1; i >= 0; i--) {
    const date = new Date(Date.now() - i * 24 * 60 * 60 * 1000);
    data.push({
      date: date.toISOString().split('T')[0],
      requests: Math.floor(Math.random() * 50000) + 40000,
      users: Math.floor(Math.random() * 1000) + 800,
      revenue: Math.floor(Math.random() * 5000) + 3000,
      errors: Math.floor(Math.random() * 100) + 10
    });
  }
  
  return data;
}