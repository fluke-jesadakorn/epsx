/**
 * Security Monitoring API
 * Uses unified admin API client
 */

'use client'

import { adminApiClient } from '../api-client'

// ============================================================================
// TYPES
// ============================================================================

export interface SecurityEvent {
  id: string
  user_id: string
  event_type: string
  severity: string
  description: string
  risk_score: number
  device_fingerprint?: string
  ip_address: string
  user_agent: string
  timestamp: string
  resolved: boolean
  recommended_actions: string[]
  metadata: Record<string, string>
}

export interface SecurityEventsResponse {
  events: SecurityEvent[]
  total_count: number
  filters_applied: any
  timestamp: string
}

export interface SecurityMetrics {
  total_events: number
  active_threats: number
  resolved_threats: number
  avg_threat_score: number
  events_by_severity: Record<string, number>
  events_by_type: Record<string, number>
  threat_score_distribution: Array<{ range: string; count: number; percentage: number }>
}

export interface SecurityTrends {
  hourly_events: Array<{ hour: string; count: number; severity_breakdown: Record<string, number> }>
  severity_trends: Array<{ severity: string; trend: string; change_percentage: number }>
  threat_score_trend: Array<{ timestamp: string; avg_score: number; max_score: number; min_score: number }>
}

export interface SecurityAlert {
  id: string
  alert_type: string
  message: string
  severity: string
  timestamp: string
  auto_resolved: boolean
  affected_users: string[]
}

export interface SecurityMetricsResponse {
  metrics: SecurityMetrics
  trends: SecurityTrends
  alerts: SecurityAlert[]
  timestamp: string
}

export interface UserThreatResponse {
  user_id: string
  current_threat_score: number
  threat_level: string
  is_under_threat: boolean
  recent_events: SecurityEvent[]
  risk_factors: string[]
  recommendations: string[]
  last_assessed: string
}

// ============================================================================
// SECURITY API
// ============================================================================

export const securityApi = {
  async getSecurityEvents(query: {
    limit?: number
    severity?: string
    event_type?: string
    resolved?: boolean
    user_id?: string
  } = {}): Promise<SecurityEventsResponse> {
    // Filter out undefined values
    const params = Object.fromEntries(
      Object.entries(query).filter(([, v]) => v !== undefined)
    )

    const res = await adminApiClient.get<SecurityEventsResponse>('/api/admin/security/events', params)
    return res.data!
  },

  async getSecurityMetrics(): Promise<SecurityMetricsResponse> {
    const res = await adminApiClient.get<SecurityMetricsResponse>('/api/admin/security/metrics')
    return res.data!
  },

  async getUserThreatAssessment(userId: string): Promise<UserThreatResponse> {
    const res = await adminApiClient.get<UserThreatResponse>('/api/admin/security/user-threat', { user_id: userId })
    return res.data!
  },

  async getHighSeverityEvents(limit = 10): Promise<SecurityEvent[]> {
    const res = await this.getSecurityEvents({ severity: 'High', resolved: false, limit })
    return res.events
  },

  async getCriticalAlerts(): Promise<SecurityAlert[]> {
    const res = await this.getSecurityMetrics()
    return res.alerts.filter(a => a.severity === 'Critical' && !a.auto_resolved)
  },

  async getEventsByType(eventType: string, limit = 20): Promise<SecurityEvent[]> {
    const res = await this.getSecurityEvents({ event_type: eventType, limit })
    return res.events
  },

  async getUnresolvedEvents(limit = 50): Promise<SecurityEvent[]> {
    const res = await this.getSecurityEvents({ resolved: false, limit })
    return res.events
  },

  async getSecurityTrendSummary(): Promise<{
    totalEvents: number
    activeThreats: number
    avgThreatScore: number
    trendingUp: boolean
    criticalAlerts: number
  }> {
    const res = await this.getSecurityMetrics()
    const trendingUp = res.trends.severity_trends
      .filter(t => ['High', 'Critical'].includes(t.severity))
      .some(t => t.trend === 'increasing')

    return {
      totalEvents: res.metrics.total_events,
      activeThreats: res.metrics.active_threats,
      avgThreatScore: res.metrics.avg_threat_score,
      trendingUp,
      criticalAlerts: res.alerts.filter(a => a.severity === 'Critical').length
    }
  },

  async isSystemUnderAlert(): Promise<boolean> {
    try {
      const res = await this.getSecurityMetrics()
      const criticalAlerts = res.alerts.filter(a => a.severity === 'Critical' && !a.auto_resolved).length
      return criticalAlerts > 0 || res.metrics.active_threats > 10
    } catch {
      return false
    }
  }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export const getSeverityColor = (severity: string): string => {
  switch (severity.toLowerCase()) {
    case 'critical': return 'text-red-600'
    case 'high': return 'text-orange-600'
    case 'medium': return 'text-yellow-600'
    case 'low': return 'text-green-600'
    default: return 'text-gray-600'
  }
}

export const getSeverityBadgeColor = (severity: string): string => {
  switch (severity.toLowerCase()) {
    case 'critical': return 'bg-red-100 text-red-800 border-red-200'
    case 'high': return 'bg-orange-100 text-orange-800 border-orange-200'
    case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200'
    case 'low': return 'bg-green-100 text-green-800 border-green-200'
    default: return 'bg-gray-100 text-gray-800 border-gray-200'
  }
}

export const formatThreatScore = (score: number): string => {
  if (score >= 80) return `${score.toFixed(1)} (Critical)`
  if (score >= 60) return `${score.toFixed(1)} (High)`
  if (score >= 40) return `${score.toFixed(1)} (Medium)`
  return `${score.toFixed(1)} (Low)`
}

export const getEventTypeIcon = (eventType: string): string => {
  switch (eventType.toLowerCase()) {
    case 'suspiciouslogin': return '🔒'
    case 'tokenreuse': return '🔄'
    case 'devicemismatch': return '📱'
    case 'permissionescalation': return '⬆️'
    case 'ratelimitexceeded': return '🚫'
    case 'maliciouspayload': return '🛡️'
    default: return '⚠️'
  }
}
