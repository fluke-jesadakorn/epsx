'use client';

import React, { useState } from 'react';
import { useJWTParser } from '@/lib/auth/jwt-parser';
import { 
  useSecurityEvents, 
  useSecurityMetrics, 
  useCriticalAlerts,
  useSecurityTrendSummary,
  useSystemAlertStatus 
} from '@/hooks/useSecurityMonitoring';
import { getSeverityBadgeColor, getEventTypeIcon, formatThreatScore } from '@/lib/api/security-monitoring-client';
import { PermissionAuditDashboard } from './PermissionAuditDashboard';
import { TokenHealthMonitor } from './TokenHealthMonitor';

interface SecurityOverviewCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  trend?: 'up' | 'down' | 'stable';
  severity?: 'low' | 'medium' | 'high' | 'critical';
  icon?: string;
}

function SecurityOverviewCard({ title, value, subtitle, trend, severity, icon }: SecurityOverviewCardProps) {
  const trendColors = {
    up: severity === 'high' || severity === 'critical' ? 'text-red-600' : 'text-green-600',
    down: severity === 'high' || severity === 'critical' ? 'text-green-600' : 'text-red-600',
    stable: 'text-gray-600',
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-600">{title}</p>
          <p className="text-3xl font-semibold text-gray-900 mt-2">{value}</p>
          {subtitle && (
            <p className={`text-sm mt-1 ${trend ? trendColors[trend] : 'text-gray-500'}`}>
              {trend && (
                <span className="mr-1">
                  {trend === 'up' ? '↗' : trend === 'down' ? '↘' : '→'}
                </span>
              )}
              {subtitle}
            </p>
          )}
        </div>
        {icon && (
          <div className="text-3xl opacity-50">
            {icon}
          </div>
        )}
      </div>
    </div>
  );
}

interface AlertBannerProps {
  isVisible: boolean;
  onDismiss: () => void;
  alerts: Array<{
    id: string;
    message: string;
    severity: string;
  }>;
}

function AlertBanner({ isVisible, onDismiss, alerts }: AlertBannerProps) {
  if (!isVisible || alerts.length === 0) return null;

  const criticalAlerts = alerts.filter(a => a.severity === 'Critical');
  
  return (
    <div className="bg-red-50 border-l-4 border-red-400 p-4 mb-6">
      <div className="flex justify-between">
        <div className="flex">
          <div className="flex-shrink-0">
            <span className="text-red-400 text-xl">🚨</span>
          </div>
          <div className="ml-3">
            <h3 className="text-sm font-medium text-red-800">
              Critical Security Alerts ({criticalAlerts.length})
            </h3>
            <div className="mt-2 text-sm text-red-700">
              {criticalAlerts.slice(0, 3).map(alert => (
                <p key={alert.id} className="mb-1">• {alert.message}</p>
              ))}
              {criticalAlerts.length > 3 && (
                <p className="text-red-600 font-medium">
                  +{criticalAlerts.length - 3} more alerts
                </p>
              )}
            </div>
          </div>
        </div>
        <div className="flex-shrink-0">
          <button
            type="button"
            className="bg-red-50 rounded-md p-1.5 text-red-500 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-red-600 focus:ring-offset-2 focus:ring-offset-red-50"
            onClick={onDismiss}
          >
            <span className="sr-only">Dismiss</span>
            <span className="text-sm">×</span>
          </button>
        </div>
      </div>
    </div>
  );
}

interface SecurityEventListProps {
  events: Array<{
    id: string;
    event_type: string;
    severity: string;
    description: string;
    user_id: string;
    timestamp: string;
    resolved: boolean;
  }>;
  isLoading: boolean;
}

function SecurityEventList({ events, isLoading }: SecurityEventListProps) {
  if (isLoading) {
    return (
      <div className="space-y-3">
        {[...Array(5)].map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
            <div className="h-3 bg-gray-200 rounded w-1/2"></div>
          </div>
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {events.slice(0, 10).map(event => (
        <div key={event.id} className="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
          <div className="text-lg">
            {getEventTypeIcon(event.event_type)}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center space-x-2">
              <span className={`px-2 py-1 text-xs rounded-full border ${getSeverityBadgeColor(event.severity)}`}>
                {event.severity}
              </span>
              <span className="text-sm text-gray-500">
                {new Date(event.timestamp).toLocaleTimeString()}
              </span>
            </div>
            <p className="text-sm text-gray-900 mt-1">{event.description}</p>
            <p className="text-xs text-gray-500 mt-1">User: {event.user_id}</p>
          </div>
        </div>
      ))}
      {events.length === 0 && (
        <p className="text-center text-gray-500 py-8">
          No security events in the selected time range
        </p>
      )}
    </div>
  );
}

export default function SecurityDashboard() {
  const { hasPermission, isAdmin } = useJWTParser();
  const [selectedTab, setSelectedTab] = useState<'overview' | 'events' | 'audit' | 'tokens'>('overview');
  const [showAlertBanner, setShowAlertBanner] = useState(true);

  // Security monitoring hooks
  const { events, isLoading: eventsLoading } = useSecurityEvents({ limit: 20 });
  const { metrics, alerts = [], isLoading: metricsLoading } = useSecurityMetrics();
  const { alerts: criticalAlerts = [] } = useCriticalAlerts();
  const { summary, isLoading: summaryLoading } = useSecurityTrendSummary();
  const { isUnderAlert, lastChecked } = useSystemAlertStatus();

  // Check permissions
  if (!hasPermission('admin:security:read') && !isAdmin()) {
    return (
      <div className="min-h-screen bg-gray-50 px-4 py-16">
        <div className="max-w-md mx-auto text-center">
          <h1 className="text-xl font-semibold text-gray-900 mb-4">Access Denied</h1>
          <p className="text-gray-600">
            You don't have permission to access the security monitoring dashboard.
          </p>
        </div>
      </div>
    );
  }

  const tabs = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'events', label: 'Security Events', icon: '🔍' },
    { id: 'audit', label: 'Permission Audit', icon: '📋' },
    { id: 'tokens', label: 'Token Health', icon: '🔐' },
  ];

  return (
    <div className="min-h-screen bg-gray-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Security Monitoring</h1>
              <p className="mt-1 text-sm text-gray-600">
                Real-time security monitoring and threat detection
                {lastChecked && (
                  <span className="ml-2 text-gray-500">
                    • Last checked: {lastChecked.toLocaleTimeString()}
                  </span>
                )}
              </p>
            </div>
            <div className="flex items-center space-x-2">
              {isUnderAlert && (
                <div className="flex items-center space-x-1 text-red-600">
                  <span className="animate-pulse text-red-500">🚨</span>
                  <span className="text-sm font-medium">System Alert</span>
                </div>
              )}
              <div className={`w-3 h-3 rounded-full ${isUnderAlert ? 'bg-red-500' : 'bg-green-500'}`}></div>
            </div>
          </div>
        </div>

        <AlertBanner
          isVisible={showAlertBanner && criticalAlerts.length > 0}
          onDismiss={() => setShowAlertBanner(false)}
          alerts={criticalAlerts}
        />

        <div className="mb-8">
          <nav className="flex space-x-8" aria-label="Tabs">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setSelectedTab(tab.id as any)}
                className={`${
                  selectedTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                } whitespace-nowrap py-2 px-1 border-b-2 font-medium text-sm flex items-center space-x-2`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </nav>
        </div>

        {selectedTab === 'overview' && (
          <div className="space-y-8">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <SecurityOverviewCard
                title="Total Security Events"
                value={summary?.totalEvents || metrics?.total_events || 0}
                subtitle={summaryLoading ? 'Loading...' : 'Last 24 hours'}
                icon="📈"
              />
              <SecurityOverviewCard
                title="Active Threats"
                value={summary?.activeThreats || metrics?.active_threats || 0}
                subtitle={summary?.trendingUp ? '↗ Trending up' : '→ Stable'}
                severity={summary?.activeThreats && summary.activeThreats > 5 ? 'high' : 'low'}
                trend={summary?.trendingUp ? 'up' : 'stable'}
                icon="🛡️"
              />
              <SecurityOverviewCard
                title="Avg Threat Score"
                value={summary?.avgThreatScore ? formatThreatScore(summary.avgThreatScore) : 'N/A'}
                subtitle="System-wide average"
                icon="⚡"
              />
              <SecurityOverviewCard
                title="Critical Alerts"
                value={summary?.criticalAlerts || criticalAlerts.length || 0}
                subtitle="Requiring attention"
                severity={criticalAlerts.length > 0 ? 'critical' : 'low'}
                icon="🚨"
              />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
              <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Security Events</h3>
                <SecurityEventList events={events} isLoading={eventsLoading} />
              </div>

              <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Event Distribution</h3>
                {metrics && !metricsLoading ? (
                  <div className="space-y-3">
                    {Object.entries(metrics.events_by_type).map(([type, count]) => (
                      <div key={type} className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                          <span>{getEventTypeIcon(type)}</span>
                          <span className="text-sm text-gray-700">{type}</span>
                        </div>
                        <span className="text-sm font-medium text-gray-900">{count}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="animate-pulse space-y-3">
                    {[...Array(4)].map((_, i) => (
                      <div key={i} className="h-4 bg-gray-200 rounded w-full"></div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        )}

        {selectedTab === 'events' && (
          <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h3 className="text-lg font-medium text-gray-900 mb-6">Security Events</h3>
            <div className="space-y-4">
              {events.map(event => (
                <div key={event.id} className="border border-gray-200 rounded-lg p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex items-start space-x-3">
                      <div className="text-2xl">
                        {getEventTypeIcon(event.event_type)}
                      </div>
                      <div className="flex-1">
                        <div className="flex items-center space-x-2 mb-2">
                          <h4 className="text-sm font-medium text-gray-900">{event.event_type}</h4>
                          <span className={`px-2 py-1 text-xs rounded-full border ${getSeverityBadgeColor(event.severity)}`}>
                            {event.severity}
                          </span>
                          {event.resolved && (
                            <span className="px-2 py-1 text-xs rounded-full bg-green-100 text-green-800 border border-green-200">
                              Resolved
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-gray-700">{event.description}</p>
                        <div className="mt-2 text-xs text-gray-500 space-x-4">
                          <span>User: {event.user_id}</span>
                          <span>IP: {event.ip_address}</span>
                          <span>Time: {new Date(event.timestamp).toLocaleString()}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {selectedTab === 'audit' && <PermissionAuditDashboard />}
        
        {selectedTab === 'tokens' && <TokenHealthMonitor />}
      </div>
    </div>
  );
}