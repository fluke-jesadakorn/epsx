import { useState, useEffect } from 'react';

interface ActivityLog {
  id: string;
  action: string;
  user: string;
  userId: string;
  timestamp: string;
  status: 'success' | 'warning' | 'error';
  details: string;
  ipAddress: string;
  userAgent: string;
}

interface UseActivityLogsOptions {
  searchTerm: string;
  statusFilter: string;
  actionFilter: string;
  dateRange: string;
}

export const useActivityLogs = (options: UseActivityLogsOptions) => {
  const [logs, setLogs] = useState<ActivityLog[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchLogs = async () => {
      setLoading(true);
      try {
        // Mock data for now - in real implementation this would fetch from API
        let mockLogs: ActivityLog[] = [
          {
            id: '1',
            action: 'User Login',
            user: 'John Doe',
            userId: '1',
            timestamp: '2024-01-15 10:30:00',
            status: 'success',
            details: 'User successfully logged in',
            ipAddress: '192.168.1.100',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
          },
          {
            id: '2',
            action: 'Permission Change',
            user: 'Admin User',
            userId: 'admin1',
            timestamp: '2024-01-15 09:45:00',
            status: 'success',
            details: 'Updated user permissions for Jane Smith',
            ipAddress: '192.168.1.101',
            userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
          },
          {
            id: '3',
            action: 'Failed Login Attempt',
            user: 'Unknown',
            userId: 'unknown',
            timestamp: '2024-01-15 08:15:00',
            status: 'error',
            details: 'Failed login attempt with invalid credentials',
            ipAddress: '203.0.113.1',
            userAgent: 'Mozilla/5.0 (X11; Linux x86_64)',
          },
          {
            id: '4',
            action: 'Package Upgrade',
            user: 'Jane Smith',
            userId: '2',
            timestamp: '2024-01-14 16:20:00',
            status: 'success',
            details: 'User upgraded to premium package',
            ipAddress: '192.168.1.102',
            userAgent: 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)',
          },
          {
            id: '5',
            action: 'Template Applied',
            user: 'Support Agent',
            userId: 'support1',
            timestamp: '2024-01-14 14:10:00',
            status: 'warning',
            details: 'Applied support template with temporary permissions',
            ipAddress: '192.168.1.103',
            userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)',
          },
        ];

        // Apply filters
        if (options.searchTerm) {
          const searchLower = options.searchTerm.toLowerCase();
          mockLogs = mockLogs.filter(
            (log) =>
              log.action.toLowerCase().includes(searchLower) ||
              log.user.toLowerCase().includes(searchLower) ||
              log.details.toLowerCase().includes(searchLower),
          );
        }

        if (options.statusFilter !== 'all') {
          mockLogs = mockLogs.filter(
            (log) => log.status === options.statusFilter,
          );
        }

        if (options.actionFilter !== 'all') {
          const actionMap: { [key: string]: string[] } = {
            login: ['User Login', 'Failed Login Attempt'],
            permission_change: ['Permission Change'],
            user_update: ['Package Upgrade'],
            template_apply: ['Template Applied'],
          };

          const allowedActions = actionMap[options.actionFilter] || [];
          mockLogs = mockLogs.filter((log) =>
            allowedActions.includes(log.action),
          );
        }

        // Date range filtering would be implemented here in real app
        // For now, we'll just use all logs

        setLogs(mockLogs);
      } catch (error) {
        console.error('Failed to fetch activity logs:', error);
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
  ]);

  const exportLogs = async (exportOptions: any) => {
    try {
      // Mock export functionality
      console.log('Exporting logs with options:', exportOptions);

      // In real implementation, this would trigger a download
      const csvContent = logs
        .map(
          (log) =>
            `${log.timestamp},${log.action},${log.user},${log.status},${log.details}`,
        )
        .join('\n');

      const blob = new Blob([csvContent], { type: 'text/csv' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `activity-logs-${new Date().toISOString().split('T')[0]}.csv`;
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export logs:', error);
      throw error;
    }
  };

  return { logs, loading, exportLogs };
};
