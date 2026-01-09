/**
 * ADMIN SECURITY MONITORING CLIENT
 *
 * Re-exports from shared/api/security for backward compatibility.
 * This file is deprecated - import directly from @/shared/api instead.
 */

'use client';

// Re-export everything from shared security module
export {
  createSecurityClient, formatThreatScore,
  getEventTypeIcon, getSeverityBadgeColor, getSeverityColor, SecurityApi, type SecurityAlert, type SecurityEvent, type SecurityEventFilters, type SecurityEventsResponse,
  type SecurityMetrics, type SecurityMetricsResponse, type SecurityTrends, type SecurityTrendSummary, type UserThreatResponse
} from '@/shared/api/security';

// Create singleton instance for backward compatibility
import { createSecurityClient, type SecurityApi as SecurityApiType } from '@/shared/api/security';

let securityApiInstance: SecurityApiType | null = null;

function getSecurityApiInstance(): SecurityApiType {
  if (!securityApiInstance) {
    securityApiInstance = createSecurityClient({
      serverSide: typeof window === 'undefined',
    });
  }
  return securityApiInstance;
}

/**
 * Security API singleton for backward compatibility
 * @deprecated Use createSecurityClient() from @/shared/api instead
 */
export const securityApi = {
  async getSecurityEvents(filters?: Parameters<SecurityApiType['getSecurityEvents']>[0]) {
    const res = await getSecurityApiInstance().getSecurityEvents(filters);
    return res.data!;
  },
  async getSecurityMetrics() {
    const res = await getSecurityApiInstance().getSecurityMetrics();
    return res.data!;
  },
  async getUserThreatAssessment(userId: string) {
    const res = await getSecurityApiInstance().getUserThreatAssessment(userId);
    return res.data!;
  },
  async getHighSeverityEvents(limit = 10) {
    return getSecurityApiInstance().getHighSeverityEvents(limit);
  },
  async getCriticalAlerts() {
    return getSecurityApiInstance().getCriticalAlerts();
  },
  async getEventsByType(eventType: string, limit = 20) {
    return getSecurityApiInstance().getEventsByType(eventType, limit);
  },
  async getUnresolvedEvents(limit = 50) {
    return getSecurityApiInstance().getUnresolvedEvents(limit);
  },
  async getSecurityTrendSummary() {
    return getSecurityApiInstance().getSecurityTrendSummary();
  },
  async isSystemUnderAlert() {
    return getSecurityApiInstance().isSystemUnderAlert();
  },
};

export default securityApi;
