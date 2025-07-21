'use client';

import { PermissionAuditLog } from '@/types/admin/iam-enhanced';
import {
  AlertCircle,
  Calendar as CalendarIcon,
  CheckCircle,
  ChevronDown,
  Clock,
  Download,
  Eye,
  Filter,
  Info,
  RefreshCw,
  Search,
  Shield,
  User,
  Users,
  XCircle,
} from 'lucide-react';
import React, { useEffect, useState } from 'react';

interface ActivityLogsFilters {
  searchQuery: string;
  action: string;
  status: string;
  userId: string;
  startDate: string;
  endDate: string;
}

export const ActivityLogs: React.FC = () => {
  const [logs, setLogs] = useState<PermissionAuditLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [filters, setFilters] = useState<ActivityLogsFilters>({
    searchQuery: '',
    action: 'all',
    status: 'all',
    userId: 'all',
    startDate: '',
    endDate: '',
  });

  // Available filter options
  const actionOptions = [
    { value: 'all', label: 'All Actions' },
    { value: 'package_upgrade', label: 'Package Upgrade' },
    { value: 'permission_granted', label: 'Permission Granted' },
    { value: 'permission_revoked', label: 'Permission Revoked' },
    { value: 'login_attempt', label: 'Login Attempt' },
    { value: 'subscription_renewal', label: 'Subscription Renewal' },
    { value: 'user_created', label: 'User Created' },
    { value: 'password_reset', label: 'Password Reset' },
    { value: 'bulk_permission_update', label: 'Bulk Permission Update' },
  ];

  const statusOptions = [
    { value: 'all', label: 'All Statuses' },
    { value: 'success', label: 'Success' },
    { value: 'warning', label: 'Warning' },
    { value: 'error', label: 'Error' },
    { value: 'info', label: 'Info' },
  ];

  useEffect(() => {
    fetchActivityLogs();
  }, [filters]);

  const fetchActivityLogs = async () => {
    try {
      setLoading(true);
      setError(null);

      const params = new URLSearchParams();
      params.append('limit', '100');

      if (filters.userId !== 'all') params.append('userId', filters.userId);
      if (filters.action !== 'all') params.append('action', filters.action);
      if (filters.status !== 'all') params.append('status', filters.status);
      if (filters.startDate) params.append('startDate', filters.startDate);
      if (filters.endDate) params.append('endDate', filters.endDate);

      const response = await fetch(
        `/api/admin/iam/activity-logs?${params.toString()}`,
      );
      const data = await response.json();

      if (data.success) {
        let fetchedLogs = data.logs;

        // Apply client-side search filter
        if (filters.searchQuery) {
          const searchLower = filters.searchQuery.toLowerCase();
          fetchedLogs = fetchedLogs.filter((log: PermissionAuditLog) => {
            const userName = log.metadata?.userName || '';
            const performedByName = log.metadata?.performedByName || '';
            const details = log.metadata?.details || log.action;

            return (
              log.action.toLowerCase().includes(searchLower) ||
              userName.toLowerCase().includes(searchLower) ||
              performedByName.toLowerCase().includes(searchLower) ||
              details.toLowerCase().includes(searchLower)
            );
          });
        }

        // Convert string timestamps to Date objects if needed
        fetchedLogs = fetchedLogs.map((log: any) => ({
          ...log,
          timestamp:
            typeof log.timestamp === 'string'
              ? new Date(log.timestamp)
              : log.timestamp,
        }));

        setLogs(fetchedLogs);

        if (data.fallback) {
          setError('Using demo data - Firebase connection unavailable');
        }
      } else {
        throw new Error('Failed to fetch activity logs');
      }
    } catch (err) {
      console.error('Error fetching activity logs:', err);
      setError('Failed to load activity logs');
      setLogs([]);
    } finally {
      setLoading(false);
    }
  };

  const handleExport = async () => {
    try {
      setExporting(true);

      const response = await fetch('/api/admin/iam/activity-logs', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          exportFormat: 'csv',
          filters: {
            userId: filters.userId !== 'all' ? filters.userId : undefined,
            action: filters.action !== 'all' ? filters.action : undefined,
            status: filters.status !== 'all' ? filters.status : undefined,
            startDate: filters.startDate || undefined,
            endDate: filters.endDate || undefined,
          },
        }),
      });

      if (response.headers.get('content-type')?.includes('text/csv')) {
        // Handle CSV download
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `activity-logs-${new Date().toISOString().split('T')[0]}.csv`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
      } else {
        throw new Error('Export failed');
      }
    } catch (err) {
      console.error('Error exporting logs:', err);
      setError('Failed to export activity logs');
    } finally {
      setExporting(false);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success':
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case 'warning':
        return <AlertCircle className="h-4 w-4 text-yellow-500" />;
      case 'error':
        return <XCircle className="h-4 w-4 text-red-500" />;
      default:
        return <Info className="h-4 w-4 text-blue-500" />;
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'success':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400';
      case 'warning':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400';
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400';
      default:
        return 'bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400';
    }
  };

  const formatAction = (action: string) => {
    return action
      .split('_')
      .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  };

  const formatTimestamp = (timestamp: Date | string) => {
    const date =
      typeof timestamp === 'string' ? new Date(timestamp) : timestamp;
    const now = new Date();
    const diffInHours = Math.floor(
      (now.getTime() - date.getTime()) / (1000 * 60 * 60),
    );

    if (diffInHours < 1) {
      const diffInMinutes = Math.floor(
        (now.getTime() - date.getTime()) / (1000 * 60),
      );
      return `${diffInMinutes} minutes ago`;
    } else if (diffInHours < 24) {
      return `${diffInHours} hours ago`;
    } else {
      const diffInDays = Math.floor(diffInHours / 24);
      if (diffInDays === 1) {
        return 'Yesterday';
      } else if (diffInDays < 7) {
        return `${diffInDays} days ago`;
      } else {
        return date.toLocaleDateString();
      }
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row gap-4 justify-between">
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">
            Activity Logs
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Track all system activities and user actions
          </p>
        </div>

        <div className="flex gap-2">
          <button
            onClick={fetchActivityLogs}
            disabled={loading}
            className="inline-flex items-center px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50"
          >
            <RefreshCw
              className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`}
            />
            Refresh
          </button>
          <button
            onClick={handleExport}
            disabled={exporting || logs.length === 0}
            className="inline-flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50"
          >
            <Download
              className={`h-4 w-4 mr-2 ${exporting ? 'animate-pulse' : ''}`}
            />
            {exporting ? 'Exporting...' : 'Export CSV'}
          </button>
        </div>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <div className="flex items-center">
            <AlertCircle className="h-5 w-5 text-yellow-400 mr-3" />
            <div>
              <h3 className="text-sm font-medium text-yellow-800 dark:text-yellow-200">
                Notice
              </h3>
              <p className="text-sm text-yellow-700 dark:text-yellow-300 mt-1">
                {error}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Search and Filter */}
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
          <input
            type="text"
            placeholder="Search by user, action, or details..."
            value={filters.searchQuery}
            onChange={(e) =>
              setFilters((prev) => ({ ...prev, searchQuery: e.target.value }))
            }
            className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          />
        </div>

        <button
          onClick={() => setShowFilters(!showFilters)}
          className={`inline-flex items-center px-4 py-2 border rounded-lg text-sm font-medium transition-colors ${
            showFilters
              ? 'border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-600 dark:bg-blue-900/20 dark:text-blue-400'
              : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700'
          }`}
        >
          <Filter className="h-4 w-4 mr-2" />
          Filters
          <ChevronDown
            className={`h-4 w-4 ml-2 transition-transform ${showFilters ? 'rotate-180' : ''}`}
          />
        </button>
      </div>
      {/* Filters Panel */}
      {showFilters && (
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-5 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Action Type
              </label>
              <select
                value={filters.action}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, action: e.target.value }))
                }
                className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                {actionOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Status
              </label>
              <select
                value={filters.status}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, status: e.target.value }))
                }
                className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              >
                {statusOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Start Date
              </label>
              <input
                type="date"
                value={filters.startDate}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, startDate: e.target.value }))
                }
                className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                End Date
              </label>
              <input
                type="date"
                value={filters.endDate}
                onChange={(e) =>
                  setFilters((prev) => ({ ...prev, endDate: e.target.value }))
                }
                className="w-full border border-gray-300 dark:border-gray-600 rounded-lg px-3 py-2 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
              />
            </div>

            <div className="flex items-end">
              <button
                onClick={() =>
                  setFilters({
                    searchQuery: '',
                    action: 'all',
                    status: 'all',
                    userId: 'all',
                    startDate: '',
                    endDate: '',
                  })
                }
                className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                Clear Filters
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Activity Logs */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        {loading ? (
          <div className="p-8 text-center">
            <RefreshCw className="h-8 w-8 text-gray-400 mx-auto mb-4 animate-spin" />
            <p className="text-gray-500 dark:text-gray-400">
              Loading activity logs...
            </p>
          </div>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead className="bg-gray-50 dark:bg-gray-900">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      <div className="flex items-center">
                        <Clock className="h-4 w-4 mr-1" />
                        Timestamp
                      </div>
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      <div className="flex items-center">
                        <Shield className="h-4 w-4 mr-1" />
                        Action
                      </div>
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      <div className="flex items-center">
                        <User className="h-4 w-4 mr-1" />
                        User
                      </div>
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      <div className="flex items-center">
                        <Users className="h-4 w-4 mr-1" />
                        Performed By
                      </div>
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      Status
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      Details
                    </th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                  {logs.map((log) => {
                    const userName = log.metadata?.userName || 'Unknown User';
                    const performedByName =
                      log.metadata?.performedByName || 'Unknown';
                    const status = log.metadata?.status || 'success';
                    const details = log.metadata?.details || log.action;

                    return (
                      <tr
                        key={log.id}
                        className="hover:bg-gray-50 dark:hover:bg-gray-700"
                      >
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                          <div className="flex items-center">
                            <Clock className="h-4 w-4 mr-2 text-gray-400" />
                            <div>
                              <div>{formatTimestamp(log.timestamp)}</div>
                              <div className="text-xs text-gray-400">
                                {typeof log.timestamp === 'string'
                                  ? new Date(log.timestamp).toLocaleDateString()
                                  : log.timestamp.toLocaleDateString()}{' '}
                                {typeof log.timestamp === 'string'
                                  ? new Date(log.timestamp).toLocaleTimeString()
                                  : log.timestamp.toLocaleTimeString()}
                              </div>
                            </div>
                          </div>
                        </td>

                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex items-center">
                            <Shield className="h-4 w-4 mr-2 text-gray-400" />
                            <span className="text-sm font-medium text-gray-900 dark:text-white">
                              {formatAction(log.action)}
                            </span>
                          </div>
                        </td>

                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex items-center">
                            <div className="h-8 w-8 rounded-full bg-blue-100 dark:bg-blue-900/20 flex items-center justify-center mr-3">
                              <User className="h-4 w-4 text-blue-600 dark:text-blue-400" />
                            </div>
                            <div>
                              <div className="text-sm font-medium text-gray-900 dark:text-white">
                                {userName.split('@')[0]}
                              </div>
                              <div className="text-sm text-gray-500 dark:text-gray-400">
                                {userName}
                              </div>
                            </div>
                          </div>
                        </td>

                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
                          <div className="flex items-center">
                            <div className="h-6 w-6 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center mr-2">
                              <Users className="h-3 w-3 text-gray-600 dark:text-gray-400" />
                            </div>
                            {performedByName}
                          </div>
                        </td>

                        <td className="px-6 py-4 whitespace-nowrap">
                          <div className="flex items-center">
                            {getStatusIcon(status)}
                            <span
                              className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(status)}`}
                            >
                              {status.charAt(0).toUpperCase() + status.slice(1)}
                            </span>
                          </div>
                        </td>

                        <td className="px-6 py-4 text-sm text-gray-900 dark:text-white max-w-xs">
                          <div className="truncate" title={details}>
                            {details}
                          </div>
                          {log.metadata?.ipAddress && (
                            <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                              IP: {log.metadata.ipAddress}
                            </div>
                          )}
                        </td>

                        <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            className="text-blue-600 dark:text-blue-400 hover:text-blue-900 dark:hover:text-blue-300"
                            title="View details"
                          >
                            <Eye className="h-4 w-4" />
                          </button>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>

            {logs.length === 0 && (
              <div className="text-center py-12">
                <CalendarIcon className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                  No activity logs found
                </h3>
                <p className="text-gray-500 dark:text-gray-400">
                  {filters.searchQuery ||
                  filters.action !== 'all' ||
                  filters.status !== 'all' ||
                  filters.startDate ||
                  filters.endDate
                    ? 'Try adjusting your search or filters'
                    : 'Activity logs will appear here as users interact with the system'}
                </p>
              </div>
            )}
          </>
        )}
      </div>

      {/* Pagination and Summary */}
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-700 dark:text-gray-300">
          Showing {logs.length} entries
          {logs.length > 0 && logs[0] && (
            <span className="ml-2 text-gray-500 dark:text-gray-400">
              • Most recent activity: {formatTimestamp(logs[0].timestamp)}
            </span>
          )}
        </div>

        {logs.length > 20 && (
          <div className="flex items-center space-x-2">
            <button className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700">
              Previous
            </button>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              Page 1 of 1
            </span>
            <button className="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700">
              Next
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
