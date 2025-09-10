// Token Health Monitor
// Real-time monitoring of JWT token health, refresh patterns, and security metrics

'use client';

import React, { useState, useEffect } from 'react';
import { useJWTParser } from '@/lib/auth/jwt-parser';
import { useSecurityEvents, useSecurityMetrics } from '@/hooks/useSecurityMonitoring';
import { getSeverityBadgeColor, getEventTypeIcon } from '@/lib/api/security-monitoring-client';

interface TokenHealth {
  user_id: string;
  user_email: string;
  token_id: string;
  issued_at: Date;
  expires_at: Date;
  last_refresh: Date;
  refresh_count: number;
  device_fingerprint: string;
  security_level: number;
  health_score: number;
  warnings: string[];
  is_suspicious: boolean;
  family_id: string;
  platform_context: string;
}

interface RefreshMetrics {
  total_active_tokens: number;
  healthy_tokens: number;
  warning_tokens: number;
  critical_tokens: number;
  avg_token_age: number;
  avg_refresh_frequency: number;
  suspicious_activity_count: number;
}

interface SecurityIncident {
  id: string;
  timestamp: Date;
  type: 'token_reuse' | 'device_change' | 'excessive_refresh' | 'integrity_violation';
  user_id: string;
  user_email: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  resolved: boolean;
  family_id?: string;
}

export function TokenHealthMonitor() {
  const [tokenHealth, setTokenHealth] = useState<TokenHealth[]>([]);
  const [metrics, setMetrics] = useState<RefreshMetrics | null>(null);
  const [incidents, setIncidents] = useState<SecurityIncident[]>([]);
  const [selectedTab, setSelectedTab] = useState<'health' | 'metrics' | 'incidents'>('health');
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [isLoading, setIsLoading] = useState(true);

  const { hasPermission, isAdmin } = useJWTParser();

  useEffect(() => {
    if (hasPermission('admin:security:read') || isAdmin()) {
      loadTokenHealthData();
      const interval = setInterval(loadTokenHealthData, 30000); // Refresh every 30 seconds
      return () => clearInterval(interval);
    }
  }, [hasPermission, isAdmin]);

  const loadTokenHealthData = async () => {
    try {
      setIsLoading(true);

      // Load token health data
      const healthResponse = await fetch('/api/admin/security/token-health', {
        credentials: 'include',
      });
      if (healthResponse.ok) {
        const healthData = await healthResponse.json();
        setTokenHealth(healthData.map((h: any) => ({
          ...h,
          issued_at: new Date(h.issued_at),
          expires_at: new Date(h.expires_at),
          last_refresh: new Date(h.last_refresh),
        })));
      }

      // Load metrics
      const metricsResponse = await fetch('/api/admin/security/refresh-metrics', {
        credentials: 'include',
      });
      if (metricsResponse.ok) {
        const metricsData = await metricsResponse.json();
        setMetrics(metricsData);
      }

      // Load security incidents
      const incidentsResponse = await fetch('/api/admin/security/incidents', {
        credentials: 'include',
      });
      if (incidentsResponse.ok) {
        const incidentsData = await incidentsResponse.json();
        setIncidents(incidentsData.map((i: any) => ({
          ...i,
          timestamp: new Date(i.timestamp),
        })));
      }
    } catch (error) {
      console.error('Failed to load token health data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRevokeToken = async (familyId: string, reason: string) => {
    try {
      const response = await fetch(`/api/admin/security/revoke-token-family/${familyId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify({ reason }),
      });

      if (response.ok) {
        // Refresh data after revocation
        loadTokenHealthData();
      }
    } catch (error) {
      console.error('Failed to revoke token:', error);
    }
  };

  const getHealthScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    if (score >= 40) return 'text-orange-600';
    return 'text-red-600';
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-100 text-red-800 border-red-200';
      case 'high': return 'bg-orange-100 text-orange-800 border-orange-200';
      case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      default: return 'bg-blue-100 text-blue-800 border-blue-200';
    }
  };

  const formatTimeAgo = (date: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMinutes < 60) return `${diffMinutes}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  };

  const filteredIncidents = incidents.filter(incident => 
    filterSeverity === 'all' || incident.severity === filterSeverity
  );

  // Access control check
  if (!hasPermission('admin:security:read') && !isAdmin()) {
    return (
      <div className="p-8 text-center">
        <div className="bg-red-50 border border-red-200 rounded-lg p-6">
          <h2 className="text-lg font-semibold text-red-800">Access Denied</h2>
          <p className="text-red-600 mt-2">
            You need 'admin:security:read' permission to view token health monitoring.
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
          <h1 className="text-2xl font-bold text-gray-900">Token Health Monitor</h1>
          <p className="text-gray-500 mt-1">Real-time JWT token security monitoring</p>
        </div>
        
        <button
          onClick={loadTokenHealthData}
          disabled={isLoading}
          className="bg-blue-600 text-white px-4 py-2 rounded-md text-sm hover:bg-blue-700 disabled:opacity-50"
        >
          {isLoading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {/* Metrics Overview */}
      {metrics && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Active Tokens</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-gray-900">{metrics.total_active_tokens.toLocaleString()}</p>
            </div>
          </div>
          
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Healthy Tokens</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-green-600">{metrics.healthy_tokens.toLocaleString()}</p>
              <p className="ml-2 text-sm text-gray-500">
                {metrics.total_active_tokens > 0 ? Math.round((metrics.healthy_tokens / metrics.total_active_tokens) * 100) : 0}%
              </p>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Warning Tokens</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-orange-600">{metrics.warning_tokens.toLocaleString()}</p>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-sm font-medium text-gray-500">Critical Tokens</h3>
            <div className="mt-2 flex items-baseline">
              <p className="text-2xl font-semibold text-red-600">{metrics.critical_tokens.toLocaleString()}</p>
            </div>
          </div>
        </div>
      )}

      {/* Tab Navigation */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {(['health', 'metrics', 'incidents'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setSelectedTab(tab)}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                selectedTab === tab
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
              {tab === 'incidents' && filteredIncidents.filter(i => !i.resolved).length > 0 && (
                <span className="ml-2 bg-red-100 text-red-800 text-xs font-medium px-2 py-0.5 rounded-full">
                  {filteredIncidents.filter(i => !i.resolved).length}
                </span>
              )}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      {selectedTab === 'health' && (
        <div className="bg-white rounded-lg shadow border">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">Token Health Status</h2>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    User
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Health Score
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Token Age
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Refresh Count
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Security Level
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Warnings
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {tokenHealth.map((token) => (
                  <tr key={token.token_id} className={`hover:bg-gray-50 ${token.is_suspicious ? 'bg-red-50' : ''}`}>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div>
                        <div className="text-sm font-medium text-gray-900">{token.user_email}</div>
                        <div className="text-xs text-gray-500">{token.user_id}</div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <span className={`text-2xl font-bold ${getHealthScoreColor(token.health_score)}`}>
                          {token.health_score}
                        </span>
                        <span className="text-sm text-gray-500 ml-1">/100</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatTimeAgo(token.issued_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      <div>
                        <span className="font-medium">{token.refresh_count}</span>
                        <div className="text-xs text-gray-500">
                          Last: {formatTimeAgo(token.last_refresh)}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                        Level {token.security_level}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      {token.warnings.length > 0 ? (
                        <div className="space-y-1">
                          {token.warnings.map((warning, index) => (
                            <div key={index} className="text-xs text-red-600 bg-red-50 px-2 py-1 rounded">
                              {warning}
                            </div>
                          ))}
                        </div>
                      ) : (
                        <span className="text-sm text-green-600">No warnings</span>
                      )}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm">
                      {token.is_suspicious && (
                        <button
                          onClick={() => handleRevokeToken(token.family_id, 'Suspicious activity detected')}
                          className="text-red-600 hover:text-red-800 font-medium"
                        >
                          Revoke
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {selectedTab === 'metrics' && metrics && (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Token Statistics</h3>
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Average Token Age</span>
                <span className="text-sm font-medium">{Math.round(metrics.avg_token_age)} hours</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Average Refresh Frequency</span>
                <span className="text-sm font-medium">{metrics.avg_refresh_frequency.toFixed(1)} times/day</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Suspicious Activity Count</span>
                <span className="text-sm font-medium text-red-600">{metrics.suspicious_activity_count}</span>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow border">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Health Distribution</h3>
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Healthy (80-100)</span>
                <div className="flex items-center">
                  <div className="w-32 bg-gray-200 rounded-full h-2 mr-2">
                    <div 
                      className="bg-green-600 h-2 rounded-full"
                      style={{ width: `${metrics.total_active_tokens > 0 ? (metrics.healthy_tokens / metrics.total_active_tokens) * 100 : 0}%` }}
                    ></div>
                  </div>
                  <span className="text-sm font-medium">{metrics.healthy_tokens}</span>
                </div>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Warning (60-79)</span>
                <div className="flex items-center">
                  <div className="w-32 bg-gray-200 rounded-full h-2 mr-2">
                    <div 
                      className="bg-yellow-600 h-2 rounded-full"
                      style={{ width: `${metrics.total_active_tokens > 0 ? (metrics.warning_tokens / metrics.total_active_tokens) * 100 : 0}%` }}
                    ></div>
                  </div>
                  <span className="text-sm font-medium">{metrics.warning_tokens}</span>
                </div>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Critical (0-59)</span>
                <div className="flex items-center">
                  <div className="w-32 bg-gray-200 rounded-full h-2 mr-2">
                    <div 
                      className="bg-red-600 h-2 rounded-full"
                      style={{ width: `${metrics.total_active_tokens > 0 ? (metrics.critical_tokens / metrics.total_active_tokens) * 100 : 0}%` }}
                    ></div>
                  </div>
                  <span className="text-sm font-medium">{metrics.critical_tokens}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {selectedTab === 'incidents' && (
        <div className="bg-white rounded-lg shadow border">
          <div className="px-6 py-4 border-b border-gray-200 flex justify-between items-center">
            <h2 className="text-lg font-semibold text-gray-900">Security Incidents</h2>
            <select
              value={filterSeverity}
              onChange={(e) => setFilterSeverity(e.target.value)}
              className="border border-gray-300 rounded-md px-3 py-1 text-sm"
            >
              <option value="all">All Severities</option>
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>
          </div>
          <div className="divide-y divide-gray-200">
            {filteredIncidents.map((incident) => (
              <div key={incident.id} className={`p-4 hover:bg-gray-50 ${!incident.resolved ? 'border-l-4 border-l-red-500' : ''}`}>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium border ${getSeverityColor(incident.severity)}`}>
                        {incident.severity.toUpperCase()}
                      </span>
                      <span className="text-sm font-medium text-gray-900">{incident.type.replace('_', ' ').toUpperCase()}</span>
                      {!incident.resolved && (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800">
                          Unresolved
                        </span>
                      )}
                    </div>
                    <p className="mt-2 text-sm text-gray-600">{incident.description}</p>
                    <p className="mt-1 text-xs text-gray-500">
                      {incident.timestamp.toLocaleString()} • User: {incident.user_email} ({incident.user_id})
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
          
          {filteredIncidents.length === 0 && (
            <div className="text-center py-8">
              <p className="text-gray-500">No security incidents found.</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}