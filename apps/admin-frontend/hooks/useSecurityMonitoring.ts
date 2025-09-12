import { useState, useEffect, useCallback } from 'react';
import useSWR from 'swr';
import { 
  securityMonitoringClient, 
  SecurityEvent, 
  SecurityMetricsResponse, 
  UserThreatResponse,
  SecurityEventsQuery,
  SecurityAlert
} from '@/lib/api/security-monitoring-client';

// Hook for security events with filtering
export function useSecurityEvents(query: SecurityEventsQuery = {}, refreshInterval = 30000) {
  const queryKey = `/security/events?${JSON.stringify(query)}`;
  
  const { data, error, isLoading, mutate } = useSWR(
    queryKey,
    () => securityMonitoringClient.getSecurityEvents(query),
    {
      refreshInterval,
      dedupingInterval: 10000,
      errorRetryCount: 3,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    events: data?.events || [],
    totalCount: data?.total_count || 0,
    filtersApplied: data?.filters_applied || query,
    isLoading,
    error,
    refresh,
  };
}

// Hook for security metrics and trends
export function useSecurityMetrics(refreshInterval = 30000) {
  const { data, error, isLoading, mutate } = useSWR<SecurityMetricsResponse>(
    '/security/metrics',
    () => securityMonitoringClient.getSecurityMetrics(),
    {
      refreshInterval,
      dedupingInterval: 15000,
      errorRetryCount: 3,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    metrics: data?.metrics,
    trends: data?.trends,
    alerts: data?.alerts || [],
    timestamp: data?.timestamp,
    isLoading,
    error,
    refresh,
  };
}

// Hook for user threat assessment
export function useUserThreatAssessment(userId: string | null, refreshInterval = 60000) {
  const { data, error, isLoading, mutate } = useSWR<UserThreatResponse>(
    userId ? `/security/user-threat/${userId}` : null,
    () => userId ? securityMonitoringClient.getUserThreatAssessment(userId) : null,
    {
      refreshInterval,
      dedupingInterval: 30000,
      errorRetryCount: 2,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    threatAssessment: data,
    isLoading,
    error,
    refresh,
  };
}

// Hook for high-severity events
export function useHighSeverityEvents(limit = 10, refreshInterval = 15000) {
  const { data, error, isLoading, mutate } = useSWR<SecurityEvent[]>(
    `/security/events/high-severity?limit=${limit}`,
    () => securityMonitoringClient.getHighSeverityEvents(limit),
    {
      refreshInterval,
      dedupingInterval: 5000,
      errorRetryCount: 3,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    events: data || [],
    isLoading,
    error,
    refresh,
  };
}

// Hook for critical alerts
export function useCriticalAlerts(refreshInterval = 10000) {
  const { data, error, isLoading, mutate } = useSWR<SecurityAlert[]>(
    '/security/alerts/critical',
    () => securityMonitoringClient.getCriticalAlerts(),
    {
      refreshInterval,
      dedupingInterval: 5000,
      errorRetryCount: 5,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    alerts: data || [],
    isLoading,
    error,
    refresh,
  };
}

// Hook for security trend summary
export function useSecurityTrendSummary(refreshInterval = 30000) {
  const { data, error, isLoading, mutate } = useSWR(
    '/security/trends/summary',
    () => securityMonitoringClient.getSecurityTrendSummary(),
    {
      refreshInterval,
      dedupingInterval: 15000,
      errorRetryCount: 3,
    }
  );

  const refresh = useCallback(() => mutate(), [mutate]);

  return {
    summary: data,
    isLoading,
    error,
    refresh,
  };
}

// Hook for system alert status
export function useSystemAlertStatus(refreshInterval = 5000) {
  const [isUnderAlert, setIsUnderAlert] = useState(false);
  const [lastChecked, setLastChecked] = useState<Date | null>(null);

  const checkAlertStatus = useCallback(async () => {
    try {
      const status = await securityMonitoringClient.isSystemUnderAlert();
      setIsUnderAlert(status);
      setLastChecked(new Date());
    } catch (error) {
      console.error('Failed to check system alert status:', error);
    }
  }, []);

  useEffect(() => {
    checkAlertStatus();
    const interval = setInterval(checkAlertStatus, refreshInterval);
    return () => clearInterval(interval);
  }, [checkAlertStatus, refreshInterval]);

  return {
    isUnderAlert,
    lastChecked,
    refresh: checkAlertStatus,
  };
}

// Hook for real-time security notifications
export function useSecurityNotifications() {
  const [notifications, setNotifications] = useState<Array<{
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    title: string;
    message: string;
    timestamp: Date;
  }>>([]);

  const addNotification = useCallback((notification: {
    type: 'success' | 'error' | 'warning' | 'info';
    title: string;
    message: string;
  }) => {
    const id = Date.now().toString();
    setNotifications(prev => [...prev, {
      ...notification,
      id,
      timestamp: new Date(),
    }]);

    // Auto remove after 10 seconds
    setTimeout(() => {
      setNotifications(prev => prev.filter(n => n.id !== id));
    }, 10000);
  }, []);

  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id));
  }, []);

  const clearNotifications = useCallback(() => {
    setNotifications([]);
  }, []);

  return {
    notifications,
    addNotification,
    removeNotification,
    clearNotifications,
  };
}

// Hook for filtering and searching security events
export function useSecurityEventsFilter() {
  const [filters, setFilters] = useState<SecurityEventsQuery>({});
  const [searchTerm, setSearchTerm] = useState('');

  const updateFilter = useCallback((key: keyof SecurityEventsQuery, value: any) => {
    setFilters(prev => ({
      ...prev,
      [key]: value,
    }));
  }, []);

  const clearFilters = useCallback(() => {
    setFilters({});
    setSearchTerm('');
  }, []);

  const hasActiveFilters = Object.keys(filters).length > 0 || searchTerm.length > 0;

  return {
    filters,
    searchTerm,
    setSearchTerm,
    updateFilter,
    clearFilters,
    hasActiveFilters,
  };
}

// Hook for security event actions
export function useSecurityEventActions() {
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const resolveEvent = useCallback(async (eventId: string) => {
    setActionLoading(eventId);
    try {
      // In a real implementation, this would call an API to resolve the event
      await new Promise(resolve => setTimeout(resolve, 1000));
      return { success: true };
    } catch (error) {
      console.error('Failed to resolve event:', error);
      return { success: false, error };
    } finally {
      setActionLoading(null);
    }
  }, []);

  const escalateEvent = useCallback(async (eventId: string) => {
    setActionLoading(eventId);
    try {
      // In a real implementation, this would call an API to escalate the event
      await new Promise(resolve => setTimeout(resolve, 1000));
      return { success: true };
    } catch (error) {
      console.error('Failed to escalate event:', error);
      return { success: false, error };
    } finally {
      setActionLoading(null);
    }
  }, []);

  const assignEvent = useCallback(async (eventId: string, assigneeId: string) => {
    setActionLoading(eventId);
    try {
      // In a real implementation, this would call an API to assign the event
      await new Promise(resolve => setTimeout(resolve, 1000));
      return { success: true };
    } catch (error) {
      console.error('Failed to assign event:', error);
      return { success: false, error };
    } finally {
      setActionLoading(null);
    }
  }, []);

  return {
    actionLoading,
    resolveEvent,
    escalateEvent,
    assignEvent,
  };
}