import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SWRConfig } from 'swr';
import SecurityDashboard from '@/components/security/SecurityDashboard';

// Mock the security monitoring hooks
jest.mock('@/hooks/useSecurityMonitoring', () => ({
  useSecurityEvents: jest.fn(),
  useSecurityMetrics: jest.fn(),
  useCriticalAlerts: jest.fn(),
  useSecurityTrendSummary: jest.fn(),
  useSystemAlertStatus: jest.fn(),
}));

// Mock the JWT parser
jest.mock('@/lib/auth/jwt-parser', () => ({
  useJWTParser: jest.fn(),
}));

// Mock the API client
jest.mock('@/lib/api/security-monitoring-client', () => ({
  getSeverityBadgeColor: jest.fn((severity: string) => `mock-${severity}-color`),
  getEventTypeIcon: jest.fn((type: string) => '🔒'),
  formatThreatScore: jest.fn((score: number) => `${score.toFixed(1)} (Mock)`),
}));

import { 
  useSecurityEvents, 
  useSecurityMetrics, 
  useCriticalAlerts,
  useSecurityTrendSummary,
  useSystemAlertStatus 
} from '@/hooks/useSecurityMonitoring';
import { useJWTParser } from '@/lib/auth/jwt-parser';

const mockUseSecurityEvents = useSecurityEvents as jest.MockedFunction<typeof useSecurityEvents>;
const mockUseSecurityMetrics = useSecurityMetrics as jest.MockedFunction<typeof useSecurityMetrics>;
const mockUseCriticalAlerts = useCriticalAlerts as jest.MockedFunction<typeof useCriticalAlerts>;
const mockUseSecurityTrendSummary = useSecurityTrendSummary as jest.MockedFunction<typeof useSecurityTrendSummary>;
const mockUseSystemAlertStatus = useSystemAlertStatus as jest.MockedFunction<typeof useSystemAlertStatus>;
const mockUseJWTParser = useJWTParser as jest.MockedFunction<typeof useJWTParser>;

// Test data
const mockSecurityEvents = [
  {
    id: 'evt_001',
    user_id: 'user_123',
    event_type: 'SuspiciousLogin',
    severity: 'High',
    description: 'Login from unusual location',
    ip_address: '192.168.1.100',
    timestamp: '2024-01-15T10:30:00Z',
    resolved: false,
  },
  {
    id: 'evt_002',
    user_id: 'user_456',
    event_type: 'TokenReuse',
    severity: 'Critical',
    description: 'Refresh token reuse detected',
    ip_address: '10.0.0.50',
    timestamp: '2024-01-15T10:25:00Z',
    resolved: true,
  },
];

const mockSecurityMetrics = {
  metrics: {
    total_events: 158,
    active_threats: 12,
    resolved_threats: 146,
    avg_threat_score: 34.5,
    events_by_severity: {
      'Low': 95,
      'Medium': 45,
      'High': 15,
      'Critical': 3,
    },
    events_by_type: {
      'SuspiciousLogin': 45,
      'TokenReuse': 18,
      'DeviceMismatch': 32,
      'PermissionEscalation': 8,
    },
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

const mockCriticalAlerts = [
  {
    id: 'alert_001',
    alert_type: 'HighThreatScore',
    message: 'Multiple users with threat scores above 80',
    severity: 'Critical',
    timestamp: '2024-01-15T10:30:00Z',
    auto_resolved: false,
    affected_users: ['user_123', 'user_456'],
  },
];

const mockTrendSummary = {
  totalEvents: 158,
  activeThreats: 12,
  avgThreatScore: 34.5,
  trendingUp: false,
  criticalAlerts: 1,
};

// Test wrapper with SWR config
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
  <SWRConfig value={{ provider: () => new Map() }}>
    {children}
  </SWRConfig>
);

describe('SecurityDashboard', () => {
  beforeEach(() => {
    // Reset all mocks
    jest.clearAllMocks();
    
    // Setup default mock implementations
    mockUseJWTParser.mockReturnValue({
      hasPermission: jest.fn(() => true),
      isAdmin: jest.fn(() => true),
      token: 'mock-token',
      claims: null,
      isExpired: false,
      expiresAt: null,
      permissions: ['admin:security:read'],
    });

    mockUseSecurityEvents.mockReturnValue({
      events: mockSecurityEvents,
      totalCount: 2,
      filtersApplied: {},
      isLoading: false,
      error: null,
      refresh: jest.fn(),
    });

    mockUseSecurityMetrics.mockReturnValue({
      metrics: mockSecurityMetrics.metrics,
      trends: mockSecurityMetrics.trends,
      alerts: mockSecurityMetrics.alerts,
      timestamp: mockSecurityMetrics.timestamp,
      isLoading: false,
      error: null,
      refresh: jest.fn(),
    });

    mockUseCriticalAlerts.mockReturnValue({
      alerts: mockCriticalAlerts,
      isLoading: false,
      error: null,
      refresh: jest.fn(),
    });

    mockUseSecurityTrendSummary.mockReturnValue({
      summary: mockTrendSummary,
      isLoading: false,
      error: null,
      refresh: jest.fn(),
    });

    mockUseSystemAlertStatus.mockReturnValue({
      isUnderAlert: false,
      lastChecked: new Date('2024-01-15T10:30:00Z'),
      refresh: jest.fn(),
    });
  });

  it('renders security dashboard with correct title', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    expect(screen.getByText('Security Monitoring')).toBeInTheDocument();
    expect(screen.getByText('Real-time security monitoring and threat detection')).toBeInTheDocument();
  });

  it('displays access denied for unauthorized users', async () => {
    mockUseJWTParser.mockReturnValue({
      hasPermission: jest.fn(() => false),
      isAdmin: jest.fn(() => false),
      token: null,
      claims: null,
      isExpired: true,
      expiresAt: null,
      permissions: [],
    });

    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    expect(screen.getByText('Access Denied')).toBeInTheDocument();
    expect(screen.getByText("You don't have permission to access the security monitoring dashboard.")).toBeInTheDocument();
  });

  it('displays overview metrics correctly', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Total Security Events')).toBeInTheDocument();
      expect(screen.getByText('158')).toBeInTheDocument();
      
      expect(screen.getByText('Active Threats')).toBeInTheDocument();
      expect(screen.getByText('12')).toBeInTheDocument();
      
      expect(screen.getByText('Critical Alerts')).toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument();
    });
  });

  it('shows critical alert banner when alerts exist', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Critical Security Alerts (1)')).toBeInTheDocument();
      expect(screen.getByText('• Multiple users with threat scores above 80')).toBeInTheDocument();
    });
  });

  it('can dismiss critical alert banner', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      const dismissButton = screen.getByRole('button', { name: 'Dismiss' });
      fireEvent.click(dismissButton);
    });

    // Alert banner should be hidden after dismiss
    await waitFor(() => {
      expect(screen.queryByText('Critical Security Alerts (1)')).not.toBeInTheDocument();
    });
  });

  it('switches between tabs correctly', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Default is overview tab
    expect(screen.getByText('Recent Security Events')).toBeInTheDocument();

    // Switch to events tab
    const eventsTab = screen.getByText('Security Events');
    fireEvent.click(eventsTab);

    await waitFor(() => {
      // Should show detailed event list
      expect(screen.getByText('SuspiciousLogin')).toBeInTheDocument();
      expect(screen.getByText('TokenReuse')).toBeInTheDocument();
    });
  });

  it('displays security events with correct information', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Switch to events tab to see detailed events
    fireEvent.click(screen.getByText('Security Events'));

    await waitFor(() => {
      expect(screen.getByText('Login from unusual location')).toBeInTheDocument();
      expect(screen.getByText('Refresh token reuse detected')).toBeInTheDocument();
      expect(screen.getByText('User: user_123')).toBeInTheDocument();
      expect(screen.getByText('IP: 192.168.1.100')).toBeInTheDocument();
    });
  });

  it('shows system alert status correctly', async () => {
    // Test when system is under alert
    mockUseSystemAlertStatus.mockReturnValue({
      isUnderAlert: true,
      lastChecked: new Date('2024-01-15T10:30:00Z'),
      refresh: jest.fn(),
    });

    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('System Alert')).toBeInTheDocument();
    });
  });

  it('displays loading states correctly', async () => {
    mockUseSecurityEvents.mockReturnValue({
      events: [],
      totalCount: 0,
      filtersApplied: {},
      isLoading: true,
      error: null,
      refresh: jest.fn(),
    });

    mockUseSecurityTrendSummary.mockReturnValue({
      summary: null,
      isLoading: true,
      error: null,
      refresh: jest.fn(),
    });

    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Should show loading states
    await waitFor(() => {
      expect(screen.getByText('Loading...')).toBeInTheDocument();
    });
  });

  it('handles empty security events state', async () => {
    mockUseSecurityEvents.mockReturnValue({
      events: [],
      totalCount: 0,
      filtersApplied: {},
      isLoading: false,
      error: null,
      refresh: jest.fn(),
    });

    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('No security events in the selected time range')).toBeInTheDocument();
    });
  });

  it('displays event type distribution correctly', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Event Distribution')).toBeInTheDocument();
      expect(screen.getByText('SuspiciousLogin')).toBeInTheDocument();
      expect(screen.getByText('45')).toBeInTheDocument(); // Event count
      expect(screen.getByText('TokenReuse')).toBeInTheDocument();
      expect(screen.getByText('18')).toBeInTheDocument();
    });
  });

  it('shows resolved and unresolved events correctly', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Switch to events tab
    fireEvent.click(screen.getByText('Security Events'));

    await waitFor(() => {
      // Should show resolved badge for resolved events
      expect(screen.getByText('Resolved')).toBeInTheDocument();
    });
  });

  it('renders permission audit tab', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Click on Permission Audit tab
    fireEvent.click(screen.getByText('Permission Audit'));

    // The PermissionAuditDashboard component should be rendered
    // Since it's mocked, we just verify the tab switch works
    await waitFor(() => {
      expect(screen.getByText('Permission Audit')).toBeInTheDocument();
    });
  });

  it('renders token health tab', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Click on Token Health tab
    fireEvent.click(screen.getByText('Token Health'));

    // The TokenHealthMonitor component should be rendered
    await waitFor(() => {
      expect(screen.getByText('Token Health')).toBeInTheDocument();
    });
  });

  it('handles API errors gracefully', async () => {
    mockUseSecurityEvents.mockReturnValue({
      events: [],
      totalCount: 0,
      filtersApplied: {},
      isLoading: false,
      error: new Error('API Error'),
      refresh: jest.fn(),
    });

    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    // Should still render without crashing
    expect(screen.getByText('Security Monitoring')).toBeInTheDocument();
  });

  it('formats timestamps correctly', async () => {
    render(
      <TestWrapper>
        <SecurityDashboard />
      </TestWrapper>
    );

    await waitFor(() => {
      // Check if timestamps are displayed (exact format may vary)
      const timeElements = screen.getAllByText(/\d{1,2}:\d{2}:\d{2}/);
      expect(timeElements.length).toBeGreaterThan(0);
    });
  });
});