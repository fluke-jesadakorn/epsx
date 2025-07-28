'use server';

import { serverGet, serverPost } from '../core/request';
import { withServerAction, type ServerActionResult } from '../core/error-handler';
import { validateSchema, analyticsFiltersSchema, generateReportSchema } from '../core/validation';

// Helper function to serialize complex parameters for query strings
function serializeFilters(filters: any): Record<string, string | number | boolean> | undefined {
  if (!filters) return undefined;
  
  const serialized: Record<string, string | number | boolean> = {};
  
  for (const [key, value] of Object.entries(filters)) {
    if (value === undefined || value === null) continue;
    
    if (typeof value === 'object' && !Array.isArray(value)) {
      // Handle nested objects like dateRange
      for (const [nestedKey, nestedValue] of Object.entries(value)) {
        if (nestedValue !== undefined && nestedValue !== null) {
          serialized[`${key}.${nestedKey}`] = String(nestedValue);
        }
      }
    } else if (Array.isArray(value)) {
      // Handle arrays
      serialized[key] = value.join(',');
    } else {
      // Handle primitives
      serialized[key] = value as string | number | boolean;
    }
  }
  
  return Object.keys(serialized).length > 0 ? serialized : undefined;
}

// Analytics Data Actions
export async function getAnalyticsData(filters?: {
  dateRange?: { start: string; end: string };
  metrics?: string[];
  groupBy?: string;
  userId?: string;
}): Promise<ServerActionResult<{
  data: any[];
  summary: Record<string, any>;
  trends: Record<string, any>;
}>> {
  return withServerAction(async () => {
    const validatedFilters = filters ? validateSchema(analyticsFiltersSchema, filters, 'getAnalyticsData') : undefined;
    const result = await serverGet('/api/v1/analytics/data', serializeFilters(validatedFilters));
    return result || {
      data: [],
      summary: {},
      trends: {}
    };
  }, 'getAnalyticsData');
}

export async function getUserAnalytics(userId: string, filters?: {
  dateRange?: { start: string; end: string };
  metrics?: string[];
}) {
  try {
    return await serverGet(`/api/v1/analytics/users/${userId}`, serializeFilters(filters));
  } catch (error) {
    console.error('Error fetching user analytics:', error);
    return {
      usage: {},
      performance: {},
      engagement: {}
    };
  }
}

export async function getSystemMetrics(filters?: {
  dateRange?: { start: string; end: string };
  interval?: 'hour' | 'day' | 'week' | 'month';
}) {
  try {
    return await serverGet('/api/v1/analytics/system/metrics', serializeFilters(filters));
  } catch (error) {
    console.error('Error fetching system metrics:', error);
    return {
      performance: {},
      usage: {},
      errors: {}
    };
  }
}

export async function getRevenueAnalytics(filters?: {
  dateRange?: { start: string; end: string };
  packageTier?: string;
  groupBy?: 'day' | 'week' | 'month';
}) {
  try {
    return await serverGet('/api/v1/analytics/revenue', serializeFilters(filters));
  } catch (error) {
    console.error('Error fetching revenue analytics:', error);
    return {
      total: 0,
      breakdown: {},
      trends: {}
    };
  }
}

// Report Generation Actions
export async function generateReports(data: {
  reportType: 'user_activity' | 'system_performance' | 'revenue' | 'security';
  dateRange: { start: string; end: string };
  format: 'pdf' | 'csv' | 'excel';
  filters?: Record<string, any>;
  recipients?: string[];
}): Promise<ServerActionResult<{ reportId: string; status: string }>> {
  return withServerAction(async () => {
    const validatedData = validateSchema(generateReportSchema, data, 'generateReports');
    return await serverPost('/api/v1/analytics/reports/generate', validatedData);
  }, 'generateReports');
}

export async function getScheduledReports() {
  try {
    return await serverGet('/api/v1/analytics/reports/scheduled');
  } catch (error) {
    console.error('Error fetching scheduled reports:', error);
    return [];
  }
}

export async function scheduleReport(data: {
  reportType: string;
  schedule: string; // cron expression
  format: 'pdf' | 'csv' | 'excel';
  recipients: string[];
  filters?: Record<string, any>;
}) {
  try {
    return await serverPost('/api/v1/analytics/reports/schedule', data);
  } catch (error) {
    console.error('Error scheduling report:', error);
    throw error;
  }
}

// Dashboard Data Actions
export async function getDashboardData(userId: string) {
  try {
    return await serverGet(`/api/v1/analytics/dashboard/${userId}`);
  } catch (error) {
    console.error('Error fetching dashboard data:', error);
    return {
      overview: {},
      charts: [],
      alerts: []
    };
  }
}

// Real-time Analytics
export async function getRealtimeMetrics() {
  try {
    return await serverGet('/api/v1/analytics/realtime');
  } catch (error) {
    console.error('Error fetching realtime metrics:', error);
    return {
      activeUsers: 0,
      requests: 0,
      errors: 0
    };
  }
}