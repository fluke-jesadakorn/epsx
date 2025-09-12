// Permission Audit Dashboard
// Real-time monitoring of granular permissions with security analytics

'use client';

import React, { useState, useEffect } from 'react';
import { useAuth } from '@/lib/auth';
import { useSecurityEvents, useSecurityMetrics, useCriticalAlerts } from '@/hooks/useSecurityMonitoring';
import { getSeverityBadgeColor, getEventTypeIcon, formatThreatScore } from '@/lib/api/security-monitoring-client';
import { logger } from '@/lib/logger';

interface PermissionAuditEvent {
  id: string;
  timestamp: Date;
  user_id: string;
  user_email: string;
  permission: string;
  action: 'granted' | 'revoked' | 'used' | 'expired' | 'denied';
  granted_by?: string;
  expires_at?: Date;
  ip_address?: string;
  user_agent?: string;
  security_level: number;
  threat_level: 'low' | 'medium' | 'high' | 'critical';
}

interface PermissionMetrics {
  total_permissions: number;
  active_permissions: number;
  expired_permissions: number;
  temporary_permissions: number;
  wildcard_permissions: number;
  high_privilege_users: number;
  permission_violations: number;
  avg_permission_lifetime: number;
}

interface SecurityAlert {
  id: string;
  timestamp: Date;
  type: 'permission_escalation' | 'suspicious_usage' | 'token_manipulation' | 'device_change';
  user_id: string;
  message: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  resolved: boolean;
}

export function PermissionAuditDashboard() {
  const [selectedTimeRange, setSelectedTimeRange] = useState('24h');
  const [filterUser, setFilterUser] = useState('');
  const [filterPermission, setFilterPermission] = useState('');

  // Get current user's permissions for access control
  const { can, isAdmin } = useAuth();

  // Use new security monitoring hooks
  const { events, totalCount, isLoading: eventsLoading, refresh: refreshEvents } = useSecurityEvents({
    limit: 50,
    resolved: false
  }, 30000);

  const { metrics, alerts, isLoading: metricsLoading, refresh: refreshMetrics } = useSecurityMetrics(30000);
  
  const { alerts: criticalAlerts, isLoading: alertsLoading, refresh: refreshAlerts } = useCriticalAlerts(10000);

  const isLoading = eventsLoading || metricsLoading || alertsLoading;

  // Refresh all data
  const refreshAll = () => {
    refreshEvents();
    refreshMetrics();
    refreshAlerts();
  };

  const handleResolveAlert = async (alertId: string) => {
    try {
      const response = await fetch(`/api/admin/security/alerts/${alertId}/resolve`, {
        method: 'POST',
        credentials: 'include',
      });
      
      if (response.ok) {
        setSecurityAlerts(alerts => 
          alerts.map(alert => 
            alert.id === alertId ? { ...alert, resolved: true } : alert
          )
        );
      }
    } catch (error) {
      logger.error('Failed to resolve alert', { alertId, error });
    }
  };

  const getThreatLevelColor = (level: string) => {
    switch (level) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200';
      case 'high': return 'bg-orange-100 text-orange-800 border-orange-200';
      case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      default: return 'bg-blue-100 text-blue-800 border-blue-200';
    }
  };

  const getActionColor = (action: string) => {
    switch (action) {
      case 'granted': return 'text-green-600';
      case 'revoked': return 'text-red-600';
      case 'expired': return 'text-orange-600';
      case 'denied': return 'text-red-500';
      case 'used': return 'text-blue-600';
      default: return 'text-gray-600';
    }
  };

  // Access control check
  if (!can('admin:audit:read') && !isAdmin()) {
    return (
      <div className="p-8 text-center">
        <div className="bg-red-50 border border-red-200 rounded-lg p-6">
          <h2 className="text-lg font-semibold text-red-800">Access Denied</h2>
          <p className="text-red-600 mt-2">
            You need 'admin:audit:read' permission to view the audit dashboard.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Permission Audit Dashboard</h1>
          <p className="text-gray-500 mt-1">Real-time monitoring of granular permissions</p>
        </div>
        
        <div className="flex items-center space-x-4">
          <select
            value={selectedTimeRange}
            onChange={(e) => setSelectedTimeRange(e.target.value)}
            className="border border-gray-300 rounded-md px-3 py-2 text-sm"
          >
            <option value="1h">Last Hour</option>
            <option value="24h">Last 24 Hours</option>
            <option value="7d">Last 7 Days</option>
            <option value="30d">Last 30 Days</option>
          </select>
          
          <button
            onClick={loadAuditData}
            disabled={isLoading}
            className="bg-blue-600 text-white px-4 py-2 rounded-md text-sm hover:bg-blue-700 disabled:opacity-50"
          >
            {isLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      {/* Metrics Overview */}
      {metrics && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Total Permissions</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-gray-900">{metrics.total_permissions.toLocaleString()}</p>
            </div>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Active Permissions</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-green-600">{metrics.active_permissions.toLocaleString()}</p>
              <p className="ml-2 text-sm text-gray-500">
                {metrics.total_permissions > 0 ? Math.round((metrics.active_permissions / metrics.total_permissions) * 100) : 0}%
              </p>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Expired Permissions</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-orange-600">{metrics.expired_permissions.toLocaleString()}</p>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Security Violations</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-red-600">{metrics.permission_violations.toLocaleString()}</p>
            </div>
          </div>
        </div>
      )}

      {/* Security Alerts */}
      {securityAlerts.length > 0 && (
        <div className="bg-white rounded-lg shadow border">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">Security Alerts</h2>
          </div>
          <div className="divide-y divide-gray-200">
            {securityAlerts.filter(alert => !alert.resolved).map((alert) => (
              <div key={alert.id} className="p-4 hover:bg-gray-50">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getThreatLevelColor(alert.severity)}`}>
                      {alert.severity.toUpperCase()}
                    </span>
                    <div>
                      <p className="text-sm font-medium text-gray-900">{alert.message}</p>
                      <p className="text-xs text-gray-500">
                        {alert.timestamp.toLocaleString()} • User: {alert.user_id} • Type: {alert.type}
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={() => handleResolveAlert(alert.id)}
                    className="text-sm text-blue-600 hover:text-blue-800"
                  >
                    Resolve
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="bg-white p-4 rounded-lg shadow border">
        <div className="flex items-center space-x-4">
          <input
            type="text"
            placeholder="Filter by user..."
            value={filterUser}
            onChange={(e) => setFilterUser(e.target.value)}
            className="border border-gray-300 rounded-md px-3 py-2 text-sm flex-1"
          />
          <input
            type="text"
            placeholder="Filter by permission..."
            value={filterPermission}
            onChange={(e) => setFilterPermission(e.target.value)}
            className="border border-gray-300 rounded-md px-3 py-2 text-sm flex-1"
          />
          <button
            onClick={loadAuditData}
            className="bg-gray-100 text-gray-700 px-4 py-2 rounded-md text-sm hover:bg-gray-200"
          >
            Apply Filters
          </button>
        </div>
      </div>

      {/* Audit Events Table */}
      <div className="bg-white rounded-lg shadow border">
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-lg font-semibold text-gray-900">Permission Audit Events</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Timestamp
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  User
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Permission
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Action
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Expires
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Threat Level
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Details
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {auditEvents.map((event) => (
                <tr key={event.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {event.timestamp.toLocaleString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div>
                      <div className="text-sm font-medium text-gray-900">{event.user_email}</div>
                      <div className="text-xs text-gray-500">{event.user_id}</div>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                      {event.permission}
                    </code>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`text-sm font-medium ${getActionColor(event.action)}`}>
                      {event.action.toUpperCase()}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {event.expires_at ? (
                      <span className={event.expires_at < new Date() ? 'text-red-500' : 'text-gray-600'}>
                        {event.expires_at.toLocaleDateString()}
                      </span>
                    ) : (
                      <span className="text-gray-400">Permanent</span>
                    )}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getThreatLevelColor(event.threat_level)}`}>
                      {event.threat_level}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    <div className="space-y-1">
                      {event.granted_by && (
                        <div>Granted by: {event.granted_by}</div>
                      )}
                      {event.ip_address && (
                        <div>IP: {event.ip_address}</div>
                      )}
                      <div>Security Level: {event.security_level}</div>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        
        {auditEvents.length === 0 && !isLoading && (
          <div className="text-center py-8">
            <p className="text-gray-500">No audit events found for the selected criteria.</p>
          </div>
        )}
      </div>

      {/* Permission Statistics */}
      {metrics && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Permission Distribution</h3>
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Permanent Permissions</span>
                <span className="text-sm font-medium">
                  {(metrics.total_permissions - metrics.temporary_permissions).toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Temporary Permissions</span>
                <span className="text-sm font-medium text-orange-600">
                  {metrics.temporary_permissions.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Wildcard Permissions</span>
                <span className="text-sm font-medium text-blue-600">
                  {metrics.wildcard_permissions.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">High Privilege Users</span>
                <span className="text-sm font-medium text-purple-600">
                  {metrics.high_privilege_users.toLocaleString()}
                </span>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Security Health</h3>
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Average Permission Lifetime</span>
                <span className="text-sm font-medium">
                  {Math.round(metrics.avg_permission_lifetime)} hours
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Permission Violations</span>
                <span className="text-sm font-medium text-red-600">
                  {metrics.permission_violations.toLocaleString()}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Security Score</span>
                <span className="text-sm font-medium text-green-600">
                  {Math.max(0, 100 - metrics.permission_violations * 2)}/100
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}