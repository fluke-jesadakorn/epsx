/**
 * UNIFIED SECURITY API CLIENT
 *
 * Security monitoring and threat assessment endpoints.
 * Migrated from admin-frontend/lib/api/security-monitoring-client.ts
 *
 * Features:
 * - Security event monitoring
 * - Threat assessment and metrics
 * - Alert management
 */

import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

import { createAdminApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

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

export interface SecurityEventsResponse {
    events: SecurityEvent[];
    total_count: number;
    filters_applied: Record<string, unknown>;
    timestamp: string;
}

export interface SecurityMetrics {
    total_events: number;
    active_threats: number;
    resolved_threats: number;
    avg_threat_score: number;
    events_by_severity: Record<string, number>;
    events_by_type: Record<string, number>;
    threat_score_distribution: Array<{ range: string; count: number; percentage: number }>;
}

export interface SecurityTrends {
    hourly_events: Array<{ hour: string; count: number; severity_breakdown: Record<string, number> }>;
    severity_trends: Array<{ severity: string; trend: string; change_percentage: number }>;
    threat_score_trend: Array<{ timestamp: string; avg_score: number; max_score: number; min_score: number }>;
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

export interface SecurityEventFilters {
    limit?: number;
    severity?: string;
    event_type?: string;
    resolved?: boolean;
    user_id?: string;
}

export interface SecurityTrendSummary {
    totalEvents: number;
    activeThreats: number;
    avgThreatScore: number;
    trendingUp: boolean;
    criticalAlerts: number;
}

// ============================================================================
// SECURITY API CLASS
// ============================================================================

export class SecurityApi {
    private client: UnifiedApiClient;

    constructor(client: UnifiedApiClient) {
        this.client = client;
    }

    // -------------------------------------------------------------------------
    // CORE API METHODS
    // -------------------------------------------------------------------------

    /**
     * Get security events with optional filtering
     */
    async getSecurityEvents(filters: SecurityEventFilters = {}): Promise<ApiResponse<SecurityEventsResponse>> {
        const params = Object.fromEntries(
            Object.entries(filters).filter(([, v]) => v !== undefined)
        );
        return this.client.get<SecurityEventsResponse>('/api/admin/security/events', params);
    }

    /**
     * Get security metrics and trends
     */
    async getSecurityMetrics(): Promise<ApiResponse<SecurityMetricsResponse>> {
        return this.client.get<SecurityMetricsResponse>('/api/admin/security/metrics');
    }

    /**
     * Get threat assessment for a specific user
     */
    async getUserThreatAssessment(userId: string): Promise<ApiResponse<UserThreatResponse>> {
        return this.client.get<UserThreatResponse>('/api/admin/security/user-threat', { user_id: userId });
    }

    // -------------------------------------------------------------------------
    // CONVENIENCE METHODS
    // -------------------------------------------------------------------------

    /**
     * Get high severity unresolved events
     */
    async getHighSeverityEvents(limit = 10): Promise<SecurityEvent[]> {
        const res = await this.getSecurityEvents({ severity: 'High', resolved: false, limit });
        return res.data?.events || [];
    }

    /**
     * Get critical unresolved alerts
     */
    async getCriticalAlerts(): Promise<SecurityAlert[]> {
        const res = await this.getSecurityMetrics();
        return res.data?.alerts.filter(a => a.severity === 'Critical' && !a.auto_resolved) || [];
    }

    /**
     * Get events filtered by type
     */
    async getEventsByType(eventType: string, limit = 20): Promise<SecurityEvent[]> {
        const res = await this.getSecurityEvents({ event_type: eventType, limit });
        return res.data?.events || [];
    }

    /**
     * Get all unresolved security events
     */
    async getUnresolvedEvents(limit = 50): Promise<SecurityEvent[]> {
        const res = await this.getSecurityEvents({ resolved: false, limit });
        return res.data?.events || [];
    }

    /**
     * Get a summary of security trends
     */
    async getSecurityTrendSummary(): Promise<SecurityTrendSummary> {
        const res = await this.getSecurityMetrics();
        const data = res.data;

        if (!data) {
            return {
                totalEvents: 0,
                activeThreats: 0,
                avgThreatScore: 0,
                trendingUp: false,
                criticalAlerts: 0,
            };
        }

        const trendingUp = data.trends.severity_trends
            .filter(t => ['High', 'Critical'].includes(t.severity))
            .some(t => t.trend === 'increasing');

        return {
            totalEvents: data.metrics.total_events,
            activeThreats: data.metrics.active_threats,
            avgThreatScore: data.metrics.avg_threat_score,
            trendingUp,
            criticalAlerts: data.alerts.filter(a => a.severity === 'Critical').length,
        };
    }

    /**
     * Check if system is under active alert
     */
    async isSystemUnderAlert(): Promise<boolean> {
        try {
            const res = await this.getSecurityMetrics();
            if (!res.data) {return false;}

            const criticalAlerts = res.data.alerts.filter(
                a => a.severity === 'Critical' && !a.auto_resolved
            ).length;
            return criticalAlerts > 0 || res.data.metrics.active_threats > 10;
        } catch {
            return false;
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Get text color class for severity level
 */
export const getSeverityColor = (severity: string): string => {
    switch (severity.toLowerCase()) {
        case 'critical': return 'text-red-600';
        case 'high': return 'text-orange-600';
        case 'medium': return 'text-yellow-600';
        case 'low': return 'text-green-600';
        default: return 'text-gray-600';
    }
};

/**
 * Get badge color classes for severity level
 */
export const getSeverityBadgeColor = (severity: string): string => {
    switch (severity.toLowerCase()) {
        case 'critical': return 'bg-red-100 text-red-800 border-red-200';
        case 'high': return 'bg-orange-100 text-orange-800 border-orange-200';
        case 'medium': return 'bg-yellow-100 text-yellow-800 border-yellow-200';
        case 'low': return 'bg-green-100 text-green-800 border-green-200';
        default: return 'bg-gray-100 text-gray-800 border-gray-200';
    }
};

/**
 * Format threat score with level indicator
 */
export const formatThreatScore = (score: number): string => {
    if (score >= 80) {return `${score.toFixed(1)} (Critical)`;}
    if (score >= 60) {return `${score.toFixed(1)} (High)`;}
    if (score >= 40) {return `${score.toFixed(1)} (Medium)`;}
    return `${score.toFixed(1)} (Low)`;
};

/**
 * Get icon for event type
 */
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

/**
 * Create a security API client
 */
export function createSecurityClient(options?: {
    baseURL?: string;
    token?: string;
    serverSide?: boolean;
}): SecurityApi {
    const client = createAdminApiClient(options);
    return new SecurityApi(client);
}
