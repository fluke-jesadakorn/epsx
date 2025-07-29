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
export async function getDashboardData(userId?: string) {
  try {
    // Use profile endpoint to get user data as basis for dashboard
    const profileResponse = await serverGet('/api/v1/auth/profile');
    
    // For now, construct dashboard data from profile and other available data
    return {
      overview: {
        userId: profileResponse?.user_id || userId,
        userLevel: profileResponse?.package_tier || 'BRONZE',
        hasAccess: profileResponse?.hasPaid || false,
        expiresAt: profileResponse?.expires_at
      },
      charts: [],
      alerts: []
    };
  } catch (error) {
    console.error('Error fetching dashboard data:', error);
    return {
      overview: {},
      charts: [],
      alerts: []
    };
  }
}

// Monitoring Actions
export async function trackError(error: {
  message: string;
  stack?: string;
  url?: string;
  userAgent?: string;
  userId?: string;
  severity?: 'low' | 'medium' | 'high' | 'critical';
  metadata?: Record<string, any>;
}) {
  try {
    console.error('Error Captured:', {
      timestamp: new Date().toISOString(),
      ...error,
    });

    // Send to external monitoring service if configured
    if (process.env.MONITORING_ENDPOINT) {
      await fetch(process.env.MONITORING_ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${process.env.MONITORING_API_KEY}`,
        },
        body: JSON.stringify({
          service: 'epsx',
          type: 'error',
          data: error,
          severity: error.severity,
        }),
      });
    }

    // Alert on critical errors
    if (error.severity === 'critical') {
      console.error('CRITICAL ERROR DETECTED:', error);
    }

    return await serverPost('/api/v1/monitoring/errors', error);
  } catch (err) {
    console.error('Failed to track error:', err);
    return { success: false };
  }
}

export async function trackPerformance(metric: {
  name: string;
  value: number;
  unit: string;
  url?: string;
  userAgent?: string;
  metadata?: Record<string, any>;
}) {
  try {
    console.log('Performance Metric:', {
      timestamp: new Date().toISOString(),
      ...metric,
    });

    // Send to external monitoring service if configured
    if (process.env.MONITORING_ENDPOINT) {
      await fetch(process.env.MONITORING_ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${process.env.MONITORING_API_KEY}`,
        },
        body: JSON.stringify({
          service: 'epsx',
          type: 'performance',
          data: metric,
        }),
      });
    }

    return await serverPost('/api/v1/monitoring/performance', metric);
  } catch (error) {
    console.error('Failed to track performance:', error);
    return { success: false };
  }
}

export async function getHealthCheck() {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || '1.0.0',
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      environment: process.env.NODE_ENV,
      checks: {
        server: true,
        database: await checkDatabase(),
        cache: await checkCache(),
        external_apis: await checkExternalAPIs(),
      }
    };

    const allHealthy = Object.values(health.checks).every(Boolean);
    
    return {
      ...health,
      status: allHealthy ? 'healthy' : 'degraded'
    };
  } catch (error) {
    console.error('Health check failed:', error);
    return {
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: error instanceof Error ? error.message : 'Unknown error',
      checks: {},
    };
  }
}

async function checkDatabase(): Promise<boolean> {
  try {
    // Add database connectivity check here
    return true;
  } catch {
    return false;
  }
}

async function checkCache(): Promise<boolean> {
  try {
    // Add cache connectivity check here
    return true;
  } catch {
    return false;
  }
}

async function checkExternalAPIs(): Promise<boolean> {
  try {
    // Add external API health checks here
    return true;
  } catch {
    return false;
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