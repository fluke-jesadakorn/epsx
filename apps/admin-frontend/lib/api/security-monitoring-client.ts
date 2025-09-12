import { fetchWithAuth } from './unified-admin-client';

// Security Event Types
export interface SecurityEvent {
  id: string;
  user_id: string;
  event_type: string;
  severity: string;
  description: string;
  risk_score: number;
  device_fingerprint?: string;
  ip_address: string;
  user_agent: string;
  timestamp: string;
  resolved: boolean;
  recommended_actions: string[];
  metadata: Record<string, string>;
}

export interface SecurityEventsQuery {
  limit?: number;
  severity?: string;
  event_type?: string;
  resolved?: boolean;
  user_id?: string;
}

export interface SecurityEventsResponse {
  events: SecurityEvent[];
  total_count: number;
  filters_applied: SecurityEventsQuery;
  timestamp: string;
}

// Security Metrics Types
export interface ThreatScoreRange {
  range: string;
  count: number;
  percentage: number;
}

export interface SecurityMetrics {
  total_events: number;
  active_threats: number;
  resolved_threats: number;
  avg_threat_score: number;
  events_by_severity: Record<string, number>;
  events_by_type: Record<string, number>;
  threat_score_distribution: ThreatScoreRange[];
}

export interface HourlyEventCount {
  hour: string;
  count: number;
  severity_breakdown: Record<string, number>;
}

export interface SeverityTrend {
  severity: string;
  trend: string;
  change_percentage: number;
}

export interface ThreatScoreTrend {
  timestamp: string;
  avg_score: number;
  max_score: number;
  min_score: number;
}

export interface SecurityTrends {
  hourly_events: HourlyEventCount[];
  severity_trends: SeverityTrend[];
  threat_score_trend: ThreatScoreTrend[];
}

export interface SecurityAlert {
  id: string;
  alert_type: string;
  message: string;
  severity: string;
  timestamp: string;
  auto_resolved: boolean;
  affected_users: string[];
}

export interface SecurityMetricsResponse {
  metrics: SecurityMetrics;
  trends: SecurityTrends;
  alerts: SecurityAlert[];
  timestamp: string;
}

// User Threat Assessment Types
export interface UserThreatResponse {
  user_id: string;
  current_threat_score: number;
  threat_level: string;
  is_under_threat: boolean;
  recent_events: SecurityEvent[];
  risk_factors: string[];
  recommendations: string[];
  last_assessed: string;
}

// API Client Class
export class SecurityMonitoringClient {
  private baseUrl: string;

  constructor(baseUrl: string = process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080') {
    this.baseUrl = `${baseUrl}/admin/security`;
  }

  /**
   * Get security events with optional filtering
   */
  async getSecurityEvents(query: SecurityEventsQuery = {}): Promise<SecurityEventsResponse> {
    const params = new URLSearchParams();
    
    if (query.limit) params.append('limit', query.limit.toString());
    if (query.severity) params.append('severity', query.severity);
    if (query.event_type) params.append('event_type', query.event_type);
    if (query.resolved !== undefined) params.append('resolved', query.resolved.toString());
    if (query.user_id) params.append('user_id', query.user_id);

    const url = `${this.baseUrl}/events${params.toString() ? `?${params}` : ''}`;
    return fetchWithAuth(url);
  }

  /**
   * Get security metrics and analytics
   */
  async getSecurityMetrics(): Promise<SecurityMetricsResponse> {
    return fetchWithAuth(`${this.baseUrl}/metrics`);
  }

  /**
   * Get threat assessment for a specific user
   */
  async getUserThreatAssessment(userId: string): Promise<UserThreatResponse> {
    const params = new URLSearchParams({ user_id: userId });
    return fetchWithAuth(`${this.baseUrl}/user-threat?${params}`);
  }

  /**
   * Get recent high-severity security events
   */
  async getHighSeverityEvents(limit: number = 10): Promise<SecurityEvent[]> {
    const response = await this.getSecurityEvents({
      severity: 'High',
      resolved: false,
      limit
    });
    return response.events;
  }

  /**
   * Get critical security alerts
   */
  async getCriticalAlerts(): Promise<SecurityAlert[]> {
    const response = await this.getSecurityMetrics();
    return response.alerts.filter(alert => 
      alert.severity === 'Critical' && !alert.auto_resolved
    );
  }

  /**
   * Get security events for a specific event type
   */
  async getEventsByType(eventType: string, limit: number = 20): Promise<SecurityEvent[]> {
    const response = await this.getSecurityEvents({
      event_type: eventType,
      limit
    });
    return response.events;
  }

  /**
   * Get unresolved security events
   */
  async getUnresolvedEvents(limit: number = 50): Promise<SecurityEvent[]> {
    const response = await this.getSecurityEvents({
      resolved: false,
      limit
    });
    return response.events;
  }

  /**
   * Get security trend summary
   */
  async getSecurityTrendSummary(): Promise<{
    totalEvents: number;
    activeThreats: number;
    avgThreatScore: number;
    trendingUp: boolean;
    criticalAlerts: number;
  }> {
    const response = await this.getSecurityMetrics();
    
    const trendingUp = response.trends.severity_trends
      .filter(trend => ['High', 'Critical'].includes(trend.severity))
      .some(trend => trend.trend === 'increasing');

    return {
      totalEvents: response.metrics.total_events,
      activeThreats: response.metrics.active_threats,
      avgThreatScore: response.metrics.avg_threat_score,
      trendingUp,
      criticalAlerts: response.alerts.filter(a => a.severity === 'Critical').length,
    };
  }

  /**
   * Check if system is under security alert
   */
  async isSystemUnderAlert(): Promise<boolean> {
    try {
      const response = await this.getSecurityMetrics();
      const criticalAlerts = response.alerts.filter(a => 
        a.severity === 'Critical' && !a.auto_resolved
      ).length;
      
      return criticalAlerts > 0 || response.metrics.active_threats > 10;
    } catch (error) {
      console.error('Failed to check system alert status:', error);
      return false;
    }
  }
}

// Export singleton instance
export const securityMonitoringClient = new SecurityMonitoringClient();

// Utility functions
export const getSeverityColor = (severity: string): string => {
  switch (severity.toLowerCase()) {
    case 'critical': return 'text-red-600';
    case 'high': return 'text-orange-600';
    case 'medium': return 'text-yellow-600';
    case 'low': return 'text-green-600';
    default: return 'text-gray-600';
  }
};

export const getSeverityBadgeColor = (severity: string): string => {
  switch (severity.toLowerCase()) {
    case 'critical': return 'bg-red-100 text-red-800 border-red-200';
    case 'high': return 'bg-orange-100 text-orange-800 border-orange-200';
    case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
    case 'low': return 'bg-green-100 text-green-800 border-green-200';
    default: return 'bg-gray-100 text-gray-800 border-gray-200';
  }
};

export const formatThreatScore = (score: number): string => {
  if (score >= 80) return `${score.toFixed(1)} (Critical)`;
  if (score >= 60) return `${score.toFixed(1)} (High)`;
  if (score >= 40) return `${score.toFixed(1)} (Medium)`;
  return `${score.toFixed(1)} (Low)`;
};

export const getEventTypeIcon = (eventType: string): string => {
  switch (eventType.toLowerCase()) {
    case 'suspiciouslogin': return '🔒';
    case 'tokenreuse': return '🔄';
    case 'devicemismatch': return '📱';
    case 'permissionescalation': return '⬆️';
    case 'ratelimitexceeded': return '🚫';
    case 'maliciouspayload': return '🛡️';
    default: return '⚠️';
  }
};