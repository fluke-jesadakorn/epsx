import { PermissionAuditLog } from '@/types/admin/iam-enhanced';
import { useEffect, useState } from 'react';

interface UseActivityLogsOptions {
  searchTerm: string;
  statusFilter: string;
  actionFilter: string;
  dateRange: string;
  userId?: string;
  startDate?: string;
  endDate?: string;
}

export const useActivityLogs = (options: UseActivityLogsOptions) => {
  const [logs, setLogs] = useState<PermissionAuditLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchLogs = async () => {
      setLoading(true);
      setError(null);

      try {
        const params = new URLSearchParams();
        params.append('limit', '100');

        if (options.userId && options.userId !== 'all') {
          params.append('userId', options.userId);
        }
        if (options.actionFilter && options.actionFilter !== 'all') {
          params.append('action', options.actionFilter);
        }
        if (options.statusFilter && options.statusFilter !== 'all') {
          params.append('status', options.statusFilter);
        }
        if (options.startDate) {
          params.append('startDate', options.startDate);
        }
        if (options.endDate) {
          params.append('endDate', options.endDate);
        }

        const response = await fetch(
          `/api/admin/iam/activity-logs?${params.toString()}`,
        );
        const data = await response.json();

        if (data.success) {
          let fetchedLogs = data.logs;

          // Apply client-side search filter
          if (options.searchTerm) {
            const searchLower = options.searchTerm.toLowerCase();
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

          setLogs(fetchedLogs);
        } else {
          throw new Error('Failed to fetch activity logs');
        }
      } catch (err) {
        console.error('Failed to fetch activity logs:', err);
        setError('Failed to load activity logs');
        setLogs([]);
      } finally {
        setLoading(false);
      }
    };

    const debounceTimer = setTimeout(fetchLogs, 300);
    return () => clearTimeout(debounceTimer);
  }, [
    options.searchTerm,
    options.statusFilter,
    options.actionFilter,
    options.dateRange,
    options.userId,
    options.startDate,
    options.endDate,
  ]);

  const exportLogs = async (
    exportOptions: {
      format?: 'csv' | 'json';
      filters?: any;
    } = {},
  ) => {
    try {
      const response = await fetch('/api/admin/iam/activity-logs', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          exportFormat: exportOptions.format || 'csv',
          filters: exportOptions.filters || {
            userId: options.userId !== 'all' ? options.userId : undefined,
            action:
              options.actionFilter !== 'all' ? options.actionFilter : undefined,
            status:
              options.statusFilter !== 'all' ? options.statusFilter : undefined,
            startDate: options.startDate || undefined,
            endDate: options.endDate || undefined,
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
        return true;
      } else {
        const data = await response.json();
        if (data.success) {
          return data.logs;
        }
        throw new Error('Export failed');
      }
    } catch (error) {
      console.error('Failed to export activity logs:', error);
      throw error;
    }
  };

  return {
    logs,
    loading,
    error,
    exportLogs,
  };
};
