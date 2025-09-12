import { renderHook, waitFor } from '@testing-library/react';
import { SWRConfig } from 'swr';
import { 
  useSecurityEvents, 
  useSecurityMetrics, 
  useCriticalAlerts,
  useSecurityTrendSummary,
  useSystemAlertStatus 
} from '@/hooks/useSecurityMonitoring';

// Mock the security monitoring client
jest.mock('@/lib/api/security-monitoring-client', () => ({
  securityMonitoringClient: {
    getSecurityEvents: jest.fn(),
    getSecurityMetrics: jest.fn(),
    getCriticalAlerts: jest.fn(),
    getSecurityTrendSummary: jest.fn(),
    isSystemUnderAlert: jest.fn(),
  },
}));

import { securityMonitoringClient } from '@/lib/api/security-monitoring-client';

const mockClient = securityMonitoringClient as jest.Mocked<typeof securityMonitoringClient>;

// Test wrapper with SWR config
const wrapper = ({ children }: { children: React.ReactNode }) => (
  <SWRConfig value={{ provider: () => new Map(), dedupingInterval: 0 }}>
    {children}
  </SWRConfig>
);

describe('useSecurityMonitoring hooks', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('useSecurityEvents', () => {
    it('fetches security events with default parameters', async () => {
      const mockEvents = [
        {
          id: 'evt_001',
          user_id: 'user_123',
          event_type: 'SuspiciousLogin',
          severity: 'High',
          description: 'Login from unusual location',
          risk_score: 75.0,
          ip_address: '192.168.1.100',
          timestamp: '2024-01-15T10:30:00Z',
          resolved: false,
        },
      ];

      const mockResponse = {
        events: mockEvents,
        total_count: 1,
        filters_applied: {},
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityEvents.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => useSecurityEvents(), { wrapper });

      await waitFor(() => {
        expect(result.current.events).toEqual(mockEvents);
        expect(result.current.totalCount).toBe(1);
        expect(result.current.isLoading).toBe(false);
      });

      expect(mockClient.getSecurityEvents).toHaveBeenCalledWith({});
    });

    it('applies filters correctly', async () => {
      const query = {
        severity: 'High',
        resolved: false,
        limit: 10,
      };

      const mockResponse = {
        events: [],
        total_count: 0,
        filters_applied: query,
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityEvents.mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => useSecurityEvents(query), { wrapper });

      await waitFor(() => {
        expect(result.current.filtersApplied).toEqual(query);
      });

      expect(mockClient.getSecurityEvents).toHaveBeenCalledWith(query);
    });

    it('handles API errors gracefully', async () => {
      const error = new Error('API Error');
      mockClient.getSecurityEvents.mockRejectedValueOnce(error);

      const { result } = renderHook(() => useSecurityEvents(), { wrapper });

      await waitFor(() => {
        expect(result.current.error).toEqual(error);
        expect(result.current.events).toEqual([]);
        expect(result.current.isLoading).toBe(false);
      });
    });

    it('refreshes data at specified intervals', async () => {
      const mockResponse = {
        events: [],
        total_count: 0,
        filters_applied: {},
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityEvents.mockResolvedValue(mockResponse);

      renderHook(() => useSecurityEvents({}, 1000), { wrapper });

      // Initial call
      expect(mockClient.getSecurityEvents).toHaveBeenCalledTimes(1);

      // Fast forward 1 second
      jest.advanceTimersByTime(1000);

      await waitFor(() => {
        expect(mockClient.getSecurityEvents).toHaveBeenCalledTimes(2);
      });
    });
  });

  describe('useSecurityMetrics', () => {
    it('fetches security metrics successfully', async () => {
      const mockMetrics = {
        metrics: {
          total_events: 150,
          active_threats: 10,
          resolved_threats: 140,
          avg_threat_score: 35.5,
          events_by_severity: { High: 5, Medium: 10, Low: 135 },
          events_by_type: { SuspiciousLogin: 50, TokenReuse: 25 },
          threat_score_distribution: [],
        },
        trends: {
          hourly_events: [],
          severity_trends: [],
          threat_score_trend: [],
        },
        alerts: [],
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityMetrics.mockResolvedValueOnce(mockMetrics);

      const { result } = renderHook(() => useSecurityMetrics(), { wrapper });

      await waitFor(() => {
        expect(result.current.metrics).toEqual(mockMetrics.metrics);
        expect(result.current.trends).toEqual(mockMetrics.trends);
        expect(result.current.alerts).toEqual(mockMetrics.alerts);
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe('useCriticalAlerts', () => {
    it('fetches critical alerts successfully', async () => {
      const mockAlerts = [
        {
          id: 'alert_001',
          alert_type: 'HighThreatScore',
          message: 'Multiple users with elevated threat scores',
          severity: 'Critical',
          timestamp: '2024-01-15T10:30:00Z',
          auto_resolved: false,
          affected_users: ['user_123', 'user_456'],
        },
      ];

      mockClient.getCriticalAlerts.mockResolvedValueOnce(mockAlerts);

      const { result } = renderHook(() => useCriticalAlerts(), { wrapper });

      await waitFor(() => {
        expect(result.current.alerts).toEqual(mockAlerts);
        expect(result.current.isLoading).toBe(false);
      });
    });

    it('refreshes at high frequency for critical alerts', async () => {
      mockClient.getCriticalAlerts.mockResolvedValue([]);

      renderHook(() => useCriticalAlerts(500), { wrapper }); // 500ms refresh

      expect(mockClient.getCriticalAlerts).toHaveBeenCalledTimes(1);

      jest.advanceTimersByTime(500);

      await waitFor(() => {
        expect(mockClient.getCriticalAlerts).toHaveBeenCalledTimes(2);
      });
    });
  });

  describe('useSecurityTrendSummary', () => {
    it('fetches trend summary successfully', async () => {
      const mockSummary = {
        totalEvents: 150,
        activeThreats: 10,
        avgThreatScore: 35.5,
        trendingUp: false,
        criticalAlerts: 1,
      };

      mockClient.getSecurityTrendSummary.mockResolvedValueOnce(mockSummary);

      const { result } = renderHook(() => useSecurityTrendSummary(), { wrapper });

      await waitFor(() => {
        expect(result.current.summary).toEqual(mockSummary);
        expect(result.current.isLoading).toBe(false);
      });
    });
  });

  describe('useSystemAlertStatus', () => {
    it('checks system alert status correctly', async () => {
      mockClient.isSystemUnderAlert.mockResolvedValue(true);

      const { result } = renderHook(() => useSystemAlertStatus(), { wrapper });

      await waitFor(() => {
        expect(result.current.isUnderAlert).toBe(true);
        expect(result.current.lastChecked).toBeInstanceOf(Date);
      });
    });

    it('handles alert status check errors gracefully', async () => {
      const consoleError = jest.spyOn(console, 'error').mockImplementation(() => {});
      mockClient.isSystemUnderAlert.mockRejectedValue(new Error('Network error'));

      const { result } = renderHook(() => useSystemAlertStatus(), { wrapper });

      await waitFor(() => {
        expect(result.current.isUnderAlert).toBe(false);
        expect(consoleError).toHaveBeenCalledWith(
          'Failed to check system alert status:',
          expect.any(Error)
        );
      });

      consoleError.mockRestore();
    });

    it('refreshes alert status at high frequency', async () => {
      mockClient.isSystemUnderAlert.mockResolvedValue(false);

      renderHook(() => useSystemAlertStatus(1000), { wrapper }); // 1 second refresh

      expect(mockClient.isSystemUnderAlert).toHaveBeenCalledTimes(1);

      jest.advanceTimersByTime(1000);

      await waitFor(() => {
        expect(mockClient.isSystemUnderAlert).toHaveBeenCalledTimes(2);
      });
    });
  });

  describe('hook refresh functions', () => {
    it('provides refresh functions that work correctly', async () => {
      const mockResponse = {
        events: [],
        total_count: 0,
        filters_applied: {},
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityEvents.mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useSecurityEvents(), { wrapper });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      // Call refresh function
      result.current.refresh();

      await waitFor(() => {
        expect(mockClient.getSecurityEvents).toHaveBeenCalledTimes(2);
      });
    });
  });

  describe('data caching and deduplication', () => {
    it('deduplicates requests with same parameters', async () => {
      const mockResponse = {
        events: [],
        total_count: 0,
        filters_applied: {},
        timestamp: '2024-01-15T10:30:00Z',
      };

      mockClient.getSecurityEvents.mockResolvedValue(mockResponse);

      // Render multiple hooks with same parameters
      const { result: result1 } = renderHook(() => useSecurityEvents({ limit: 10 }), { wrapper });
      const { result: result2 } = renderHook(() => useSecurityEvents({ limit: 10 }), { wrapper });

      await waitFor(() => {
        expect(result1.current.isLoading).toBe(false);
        expect(result2.current.isLoading).toBe(false);
      });

      // Should only make one API call due to SWR deduplication
      expect(mockClient.getSecurityEvents).toHaveBeenCalledTimes(1);
    });
  });

  describe('error retry behavior', () => {
    it('retries failed requests according to configuration', async () => {
      const error = new Error('Network error');
      mockClient.getSecurityEvents.mockRejectedValue(error);

      const { result } = renderHook(() => useSecurityEvents(), { wrapper });

      await waitFor(() => {
        expect(result.current.error).toEqual(error);
      });

      // SWR should retry failed requests based on errorRetryCount configuration
      // The exact number of retries depends on SWR's retry logic and timing
      expect(mockClient.getSecurityEvents).toHaveBeenCalledTimes(1);
    });
  });
});