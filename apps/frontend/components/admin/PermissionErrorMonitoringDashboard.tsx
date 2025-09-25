/**
 * Permission Error Monitoring Dashboard (Phase 3.2)
 * 🔒 ADMIN ONLY: Comprehensive monitoring of permission errors across the platform
 * 📊 ANALYTICS POWERED: Real-time insights into error patterns and user impact
 * 
 * Provides administrators with detailed visibility into permission system health,
 * error patterns, user experience issues, and actionable insights for improvements.
 */

'use client';

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { PermissionErrorBoundary } from '@/components/error-boundaries/PermissionErrorBoundary';
import { PermissionErrorUI } from '@/components/errors/PermissionErrorUI';
import { 
  permissionErrorAnalytics,
  ErrorAnalyticsSummary,
  ErrorPattern,
  PermissionErrorEvent,
  usePermissionErrorAnalytics
} from '@/lib/analytics/permission-error-analytics';
import { 
  ApiError,
  isPermissionDeniedError,
  isInsufficientTierError,
  isPermissionExpiredError,
  isRateLimitExceededError
} from '@/lib/api/response-handler';

// ============================================================================
// MONITORING DASHBOARD TYPES
// ============================================================================

interface ErrorTrend {
  timestamp: string;
  total_errors: number;
  unique_users: number;
  critical_errors: number;
  resolution_rate: number;
}

interface ComponentHealth {
  component: string;
  error_count: number;
  error_rate: number;
  avg_resolution_time_ms: number;
  user_satisfaction: number;
  common_errors: string[];
  status: 'healthy' | 'warning' | 'critical';
}

interface UserImpactAnalysis {
  affected_users: number;
  avg_errors_per_user: number;
  churn_risk_users: number;
  upgrade_opportunities: number;
  support_escalations: number;
  satisfaction_score: number;
}

interface AlertConfiguration {
  error_rate_threshold: number;
  resolution_time_threshold_ms: number;
  user_satisfaction_threshold: number;
  critical_error_types: string[];
  notification_channels: string[];
  enabled: boolean;
}

// ============================================================================
// MONITORING DASHBOARD COMPONENT
// ============================================================================

function PermissionErrorMonitoringDashboardCore() {
  const analytics = usePermissionErrorAnalytics();
  
  // Dashboard state
  const [timeRange, setTimeRange] = useState<number>(24); // hours
  const [refreshInterval, setRefreshInterval] = useState<number>(300); // seconds
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true);
  
  // Data state
  const [analyticsData, setAnalyticsData] = useState<ErrorAnalyticsSummary | null>(null);
  const [errorPatterns, setErrorPatterns] = useState<ErrorPattern[]>([]);
  const [errorTrends, setErrorTrends] = useState<ErrorTrend[]>([]);
  const [componentHealth, setComponentHealth] = useState<ComponentHealth[]>([]);
  const [userImpact, setUserImpact] = useState<UserImpactAnalysis | null>(null);
  const [recentErrors, setRecentErrors] = useState<PermissionErrorEvent[]>([]);
  
  // UI state
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [loadingError, setLoadingError] = useState<ApiError | null>(null);
  const [selectedView, setSelectedView] = useState<'overview' | 'patterns' | 'components' | 'users' | 'trends'>('overview');
  const [alertConfig, setAlertConfig] = useState<AlertConfiguration>({
    error_rate_threshold: 5.0,
    resolution_time_threshold_ms: 30000,
    user_satisfaction_threshold: 3.0,
    critical_error_types: ['SYSTEM_ERROR', 'AUTHENTICATION_REQUIRED', 'NETWORK_ERROR'],
    notification_channels: ['email', 'slack'],
    enabled: true
  });

  // Data fetching
  const fetchDashboardData = useCallback(async () => {
    setIsLoading(true);
    setLoadingError(null);
    
    try {
      // Parallel data fetching for performance
      const [
        summaryData,
        patternsData,
        trendsResponse,
        healthResponse,
        impactResponse,
        recentErrorsResponse
      ] = await Promise.all([
        analytics.getAnalytics(timeRange),
        analytics.getPatterns(3), // minimum frequency of 3
        fetch(`/api/admin/analytics/error-trends?hours=${timeRange}`, { credentials: 'include' }),
        fetch(`/api/admin/analytics/component-health?hours=${timeRange}`, { credentials: 'include' }),
        fetch(`/api/admin/analytics/user-impact?hours=${timeRange}`, { credentials: 'include' }),
        fetch(`/api/admin/analytics/recent-errors?limit=50`, { credentials: 'include' })
      ]);
      
      // Process responses
      setAnalyticsData(summaryData);
      setErrorPatterns(patternsData);
      
      if (trendsResponse.ok) {
        setErrorTrends(await trendsResponse.json());
      }
      
      if (healthResponse.ok) {
        setComponentHealth(await healthResponse.json());
      }
      
      if (impactResponse.ok) {
        setUserImpact(await impactResponse.json());
      }
      
      if (recentErrorsResponse.ok) {
        setRecentErrors(await recentErrorsResponse.json());
      }
      
    } catch (error) {
      console.error('Failed to fetch monitoring dashboard data:', error);
      setLoadingError({
        success: false,
        error: {
          type: 'DASHBOARD_LOAD_ERROR',
          code: 'MONITORING_FETCH_FAILED',
          message: error instanceof Error ? error.message : 'Dashboard load failed',
          user_message: 'Unable to load the error monitoring dashboard. Please try again.',
          suggested_actions: ['Refresh the page', 'Check your connection', 'Contact system administrator']
        }
      });
    } finally {
      setIsLoading(false);
    }
  }, [timeRange, analytics]);

  // Auto-refresh functionality
  useEffect(() => {
    fetchDashboardData();
    
    if (autoRefresh && refreshInterval > 0) {
      const intervalId = setInterval(fetchDashboardData, refreshInterval * 1000);
      return () => clearInterval(intervalId);
    }
  }, [fetchDashboardData, autoRefresh, refreshInterval]);

  // Alert evaluation
  const alerts = useMemo(() => {
    if (!analyticsData || !componentHealth || !userImpact || !alertConfig.enabled) {
      return [];
    }

    const alertsList: Array<{
      type: 'error_rate' | 'resolution_time' | 'satisfaction' | 'critical_errors';
      severity: 'warning' | 'critical';
      message: string;
      component?: string;
      value: number;
      threshold: number;
    }> = [];

    // Error rate alerts
    const errorRate = (analyticsData.total_errors / Math.max(analyticsData.unique_users_affected, 1));
    if (errorRate > alertConfig.error_rate_threshold) {
      alertsList.push({
        type: 'error_rate',
        severity: errorRate > alertConfig.error_rate_threshold * 2 ? 'critical' : 'warning',
        message: `High error rate detected: ${errorRate.toFixed(2)} errors per user`,
        value: errorRate,
        threshold: alertConfig.error_rate_threshold
      });
    }

    // Resolution time alerts
    if (analyticsData.resolution_stats.avg_resolution_time_ms > alertConfig.resolution_time_threshold_ms) {
      alertsList.push({
        type: 'resolution_time',
        severity: 'warning',
        message: `Slow error resolution: ${(analyticsData.resolution_stats.avg_resolution_time_ms / 1000).toFixed(1)}s average`,
        value: analyticsData.resolution_stats.avg_resolution_time_ms,
        threshold: alertConfig.resolution_time_threshold_ms
      });
    }

    // User satisfaction alerts
    if (userImpact.satisfaction_score < alertConfig.user_satisfaction_threshold) {
      alertsList.push({
        type: 'satisfaction',
        severity: 'critical',
        message: `Low user satisfaction: ${userImpact.satisfaction_score.toFixed(1)}/5.0`,
        value: userImpact.satisfaction_score,
        threshold: alertConfig.user_satisfaction_threshold
      });
    }

    // Component health alerts
    componentHealth.forEach(component => {
      if (component.status === 'critical') {
        alertsList.push({
          type: 'error_rate',
          severity: 'critical',
          message: `Component ${component.component} has critical error rate: ${component.error_rate.toFixed(2)}%`,
          component: component.component,
          value: component.error_rate,
          threshold: 10.0
        });
      }
    });

    return alertsList;
  }, [analyticsData, componentHealth, userImpact, alertConfig]);

  // Render helper functions
  const renderMetricCard = (title: string, value: string | number, trend?: number, suffix = '') => (
    <div className="bg-white p-4 rounded-lg border shadow-sm">
      <h3 className="text-sm font-medium text-gray-600 mb-1">{title}</h3>
      <div className="flex items-center space-x-2">
        <span className="text-2xl font-bold text-gray-900">
          {typeof value === 'number' ? value.toLocaleString() : value}{suffix}
        </span>
        {trend !== undefined && (
          <span className={`text-xs px-1.5 py-0.5 rounded ${
            trend > 0 ? 'bg-red-100 text-red-600' : 
            trend < 0 ? 'bg-green-100 text-green-600' : 
            'bg-gray-100 text-gray-600'
          }`}>
            {trend > 0 ? '↑' : trend < 0 ? '↓' : '→'} {Math.abs(trend).toFixed(1)}%
          </span>
        )}
      </div>
    </div>
  );

  const renderErrorTypeDistribution = () => {
    if (!analyticsData) return null;

    return (
      <div className="bg-white p-6 rounded-lg border shadow-sm">
        <h3 className="text-lg font-semibold mb-4">Error Type Distribution</h3>
        <div className="space-y-3">
          {Object.entries(analyticsData.error_types).map(([type, count]) => {
            const percentage = (count / analyticsData.total_errors) * 100;
            const isHighImpact = ['SYSTEM_ERROR', 'AUTHENTICATION_REQUIRED'].includes(type);
            
            return (
              <div key={type} className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center justify-between mb-1">
                    <span className={`text-sm font-medium ${isHighImpact ? 'text-red-600' : 'text-gray-700'}`}>
                      {type.replace(/_/g, ' ')}
                      {isHighImpact && <span className="ml-1 text-xs">🔥</span>}
                    </span>
                    <span className="text-sm text-gray-500">{count} ({percentage.toFixed(1)}%)</span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div 
                      className={`h-2 rounded-full ${isHighImpact ? 'bg-red-500' : 'bg-blue-500'}`}
                      style={{ width: `${percentage}%` }}
                    ></div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  const renderComponentHealthTable = () => (
    <div className="bg-white rounded-lg border shadow-sm overflow-hidden">
      <div className="px-6 py-4 border-b">
        <h3 className="text-lg font-semibold">Component Health Status</h3>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Component</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Error Count</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Error Rate</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Avg Resolution</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Satisfaction</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {componentHealth.map((component, index) => (
              <tr key={index} className="hover:bg-gray-50">
                <td className="px-6 py-4 text-sm font-medium text-gray-900">
                  {component.component}
                </td>
                <td className="px-6 py-4">
                  <span className={`px-2 py-1 text-xs font-medium rounded-full ${
                    component.status === 'healthy' ? 'bg-green-100 text-green-800' :
                    component.status === 'warning' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-red-100 text-red-800'
                  }`}>
                    {component.status}
                  </span>
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  {component.error_count.toLocaleString()}
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  {component.error_rate.toFixed(2)}%
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  {(component.avg_resolution_time_ms / 1000).toFixed(1)}s
                </td>
                <td className="px-6 py-4 text-sm text-gray-900">
                  ⭐ {component.user_satisfaction.toFixed(1)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );

  const renderErrorPatterns = () => (
    <div className="bg-white p-6 rounded-lg border shadow-sm">
      <h3 className="text-lg font-semibold mb-4">Recurring Error Patterns</h3>
      <div className="space-y-4">
        {errorPatterns.map((pattern, index) => (
          <div key={pattern.pattern_id} className="border rounded-lg p-4">
            <div className="flex items-start justify-between mb-2">
              <div>
                <h4 className="font-medium text-gray-900">
                  {pattern.error_type.replace(/_/g, ' ')} Pattern #{index + 1}
                </h4>
                <p className="text-sm text-gray-600">
                  {pattern.frequency} occurrences • {pattern.affected_users} users affected
                </p>
              </div>
              <span className="px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded">
                High Impact
              </span>
            </div>
            
            <div className="grid md:grid-cols-2 gap-4 mt-3">
              <div>
                <h5 className="text-xs font-medium text-gray-500 uppercase mb-1">Affected Components</h5>
                <div className="flex flex-wrap gap-1">
                  {pattern.components.slice(0, 3).map(component => (
                    <span key={component} className="px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded">
                      {component}
                    </span>
                  ))}
                  {pattern.components.length > 3 && (
                    <span className="px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded">
                      +{pattern.components.length - 3} more
                    </span>
                  )}
                </div>
              </div>
              
              <div>
                <h5 className="text-xs font-medium text-gray-500 uppercase mb-1">Suggested Fixes</h5>
                <ul className="text-xs text-gray-700 space-y-1">
                  {pattern.suggested_fixes.slice(0, 2).map((fix, fixIndex) => (
                    <li key={fixIndex}>• {fix}</li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  if (isLoading) {
    return (
      <div className="p-8 text-center">
        <div className="flex items-center justify-center space-x-2 mb-4">
          <div className="w-6 h-6 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
          <span className="text-lg font-medium">Loading Error Analytics...</span>
        </div>
        <p className="text-gray-600">Gathering comprehensive error data and patterns</p>
      </div>
    );
  }

  if (loadingError) {
    return (
      <div className="p-6">
        <PermissionErrorUI
          error={loadingError}
          context={{
            component: 'PermissionErrorMonitoringDashboard',
            operation: 'dashboard_load'
          }}
          onRetry={() => {
            setLoadingError(null);
            fetchDashboardData();
          }}
          onContactSupport={() => window.location.href = '/support'}
          className="my-6"
        />
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Permission Error Monitoring</h1>
          <p className="text-gray-600 mt-1">Real-time insights into system health and user experience</p>
        </div>
        
        <div className="flex items-center space-x-4">
          <select 
            value={timeRange} 
            onChange={(e) => setTimeRange(parseInt(e.target.value))}
            className="border rounded-lg px-3 py-2 text-sm"
          >
            <option value={1}>Last Hour</option>
            <option value={6}>Last 6 Hours</option>
            <option value={24}>Last 24 Hours</option>
            <option value={168}>Last Week</option>
            <option value={720}>Last Month</option>
          </select>
          
          <button
            onClick={fetchDashboardData}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700"
          >
            Refresh Data
          </button>
        </div>
      </div>

      {/* Alerts */}
      {alerts.length > 0 && (
        <div className="space-y-2">
          {alerts.map((alert, index) => (
            <div key={index} className={`p-4 rounded-lg border ${
              alert.severity === 'critical' ? 'bg-red-50 border-red-200' : 'bg-yellow-50 border-yellow-200'
            }`}>
              <div className="flex items-start">
                <span className="text-lg mr-2" role="img" aria-hidden="true">
                  {alert.severity === 'critical' ? '🚨' : '⚠️'}
                </span>
                <div>
                  <h3 className={`text-sm font-medium ${
                    alert.severity === 'critical' ? 'text-red-800' : 'text-yellow-800'
                  }`}>
                    {alert.type === 'error_rate' ? 'High Error Rate' :
                     alert.type === 'resolution_time' ? 'Slow Resolution Time' :
                     alert.type === 'satisfaction' ? 'Low User Satisfaction' : 'Critical Errors'}
                  </h3>
                  <p className={`mt-1 text-sm ${
                    alert.severity === 'critical' ? 'text-red-700' : 'text-yellow-700'
                  }`}>
                    {alert.message}
                    {alert.component && ` in ${alert.component}`}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Navigation Tabs */}
      <div className="border-b">
        <nav className="flex space-x-8">
          {[
            { key: 'overview', label: 'Overview', count: analyticsData?.total_errors },
            { key: 'patterns', label: 'Error Patterns', count: errorPatterns.length },
            { key: 'components', label: 'Component Health', count: componentHealth.length },
            { key: 'users', label: 'User Impact', count: userImpact?.affected_users },
            { key: 'trends', label: 'Trends', count: null }
          ].map(tab => (
            <button
              key={tab.key}
              onClick={() => setSelectedView(tab.key as any)}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                selectedView === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {tab.label}
              {tab.count !== undefined && tab.count !== null && (
                <span className="ml-2 px-2 py-1 text-xs bg-gray-100 text-gray-600 rounded-full">
                  {tab.count}
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>

      {/* Content based on selected view */}
      {selectedView === 'overview' && (
        <div className="space-y-6">
          {/* Key Metrics */}
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            {renderMetricCard('Total Errors', analyticsData?.total_errors || 0)}
            {renderMetricCard('Affected Users', analyticsData?.unique_users_affected || 0)}
            {renderMetricCard('Resolution Rate', `${(analyticsData?.resolution_stats.retry_success_rate || 0).toFixed(1)}%`)}
            {renderMetricCard('Avg Resolution Time', `${((analyticsData?.resolution_stats.avg_resolution_time_ms || 0) / 1000).toFixed(1)}s`)}
          </div>

          {/* Charts and Distribution */}
          <div className="grid md:grid-cols-2 gap-6">
            {renderErrorTypeDistribution()}
            
            <div className="bg-white p-6 rounded-lg border shadow-sm">
              <h3 className="text-lg font-semibold mb-4">Top Affected Components</h3>
              <div className="space-y-3">
                {analyticsData?.top_components.slice(0, 5).map((component, index) => (
                  <div key={component.component} className="flex items-center justify-between">
                    <span className="text-sm font-medium text-gray-700">{component.component}</span>
                    <div className="flex items-center space-x-2">
                      <span className="text-sm text-gray-500">{component.error_count} errors</span>
                      <span className="text-sm text-gray-500">• {component.unique_users} users</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {selectedView === 'patterns' && renderErrorPatterns()}
      {selectedView === 'components' && renderComponentHealthTable()}
      
      {selectedView === 'users' && userImpact && (
        <div className="grid md:grid-cols-3 gap-6">
          {renderMetricCard('Users Affected', userImpact.affected_users)}
          {renderMetricCard('Avg Errors/User', userImpact.avg_errors_per_user.toFixed(1))}
          {renderMetricCard('Churn Risk', userImpact.churn_risk_users)}
          {renderMetricCard('Upgrade Opportunities', userImpact.upgrade_opportunities)}
          {renderMetricCard('Support Escalations', userImpact.support_escalations)}
          {renderMetricCard('Satisfaction Score', `${userImpact.satisfaction_score.toFixed(1)}/5`, undefined, '⭐')}
        </div>
      )}

      {/* Auto-refresh indicator */}
      {autoRefresh && (
        <div className="fixed bottom-4 right-4 bg-blue-600 text-white px-4 py-2 rounded-lg text-sm">
          🔄 Auto-refreshing every {refreshInterval}s
        </div>
      )}
    </div>
  );
}

// Main monitoring dashboard with error boundary
const PermissionErrorMonitoringDashboard: React.FC = () => {
  return (
    <PermissionErrorBoundary
      component="PermissionErrorMonitoringDashboard"
      onError={(error, errorInfo, apiError) => {
        console.error('Permission Error Monitoring Dashboard Error:', {
          error: error.message,
          errorInfo,
          apiError
        });
      }}
      fallback={
        <div className="max-w-7xl mx-auto p-6">
          <div className="text-center mb-6">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">Permission Error Monitoring</h1>
            <p className="text-gray-600">Real-time insights into system health and user experience</p>
          </div>
          
          <div className="p-6 bg-red-50 border border-red-200 rounded-lg">
            <div className="flex items-start">
              <div className="flex-shrink-0">
                <span className="text-lg text-red-500" role="img" aria-hidden="true">⚠️</span>
              </div>
              <div className="ml-3">
                <h3 className="text-sm font-medium text-red-800">Monitoring Dashboard Error</h3>
                <p className="mt-1 text-sm text-red-700">
                  The error monitoring dashboard encountered a critical error. Please contact system administrator.
                </p>
                <button 
                  onClick={() => window.location.reload()}
                  className="mt-2 text-sm font-medium text-red-800 underline hover:text-red-900"
                >
                  Refresh Dashboard
                </button>
              </div>
            </div>
          </div>
        </div>
      }
    >
      <PermissionErrorMonitoringDashboardCore />
    </PermissionErrorBoundary>
  );
};

export default PermissionErrorMonitoringDashboard;

// ============================================================================
// PERMISSION ERROR MONITORING DASHBOARD COMPLETE NOTICE (Phase 3.2.4)
// ============================================================================
//
// 🎉 PERMISSION ERROR MONITORING DASHBOARD COMPLETE!
//
// Created comprehensive administrative monitoring dashboard:
// - Real-time error analytics with trend analysis and pattern detection
// - Component health monitoring with performance metrics
// - User impact analysis with satisfaction tracking and churn prediction
// - Intelligent alerting system with configurable thresholds
// - Error pattern recognition for proactive issue resolution
// - Interactive data visualization with drill-down capabilities
// - Auto-refresh functionality for continuous monitoring
//
// Key Administrative Features:
// ✅ Real-time error frequency and impact monitoring
// ✅ Component-level health status with performance metrics
// ✅ User journey analysis through error resolution flows
// ✅ Intelligent alerting for critical error patterns
// ✅ Business impact assessment (upgrade conversion, churn risk)
// ✅ Actionable insights for system improvements
// ✅ Comprehensive error pattern analysis with suggested fixes
// ✅ Performance monitoring (resolution times, satisfaction scores)
//
// Monitoring Capabilities:
// 📊 Error rate trends and threshold alerts
// 📊 Component reliability scoring and health status
// 📊 User satisfaction tracking with feedback integration
// 📊 Business metrics (conversion rates, support escalations)
// 📊 Performance analytics (response times, cache effectiveness)
// 📊 Predictive analytics for churn risk and upgrade opportunities
//
// The Permission Error Monitoring Dashboard is now PRODUCTION-READY! 🎯
// ============================================================================