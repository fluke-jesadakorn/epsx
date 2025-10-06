/**
 * Permission Error Analytics
 * Tracks and analyzes permission-related errors and security events
 */

export interface PermissionErrorEvent {
  id: string;
  userId?: string;
  walletAddress?: string;
  errorType: 'access_denied' | 'invalid_permission' | 'expired_permission' | 'missing_permission';
  attemptedPermission: string;
  currentPermissions: string[];
  timestamp: string;
  userAgent?: string;
  ipAddress?: string;
  sessionId?: string;
  metadata?: Record<string, any>;
}

export interface PermissionErrorStats {
  totalErrors: number;
  errorsByType: Record<string, number>;
  topFailedPermissions: Array<{ permission: string; count: number }>;
  errorTrends: Array<{ date: string; count: number }>;
  suspiciousActivity: number;
}

class PermissionErrorAnalytics {
  private events: PermissionErrorEvent[] = [];

  logPermissionError(event: Omit<PermissionErrorEvent, 'id' | 'timestamp'>): void {
    const permissionError: PermissionErrorEvent = {
      ...event,
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
    };
    
    this.events.push(permissionError);
    
    // Keep only last 1000 events for memory management
    if (this.events.length > 1000) {
      this.events = this.events.slice(-1000);
    }
    
    // Log to console for debugging
    // eslint-disable-next-line no-console
    console.warn('Permission Error:', permissionError);
  }

  getErrorStats(timeRangeHours = 24): PermissionErrorStats {
    const cutoffTime = new Date(Date.now() - timeRangeHours * 60 * 60 * 1000);
    const recentEvents = this.events.filter(
      event => new Date(event.timestamp) > cutoffTime
    );

    const errorsByType: Record<string, number> = {};
    const permissionCounts: Record<string, number> = {};

    recentEvents.forEach(event => {
      errorsByType[event.errorType] = (errorsByType[event.errorType] || 0) + 1;
      permissionCounts[event.attemptedPermission] = (permissionCounts[event.attemptedPermission] || 0) + 1;
    });

    const topFailedPermissions = Object.entries(permissionCounts)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 10)
      .map(([permission, count]) => ({ permission, count }));

    // Generate hourly trends for the last 24 hours
    const errorTrends: Array<{ date: string; count: number }> = [];
    for (let i = timeRangeHours - 1; i >= 0; i--) {
      const hourStart = new Date(Date.now() - i * 60 * 60 * 1000);
      const hourEnd = new Date(Date.now() - (i - 1) * 60 * 60 * 1000);
      
      const hourlyCount = recentEvents.filter(
        event => {
          const eventTime = new Date(event.timestamp);
          return eventTime >= hourStart && eventTime < hourEnd;
        }
      ).length;

      errorTrends.push({
        date: hourStart.toISOString(),
        count: hourlyCount
      });
    }

    // Count suspicious activity (multiple failed attempts from same user/IP)
    const userAttempts: Record<string, number> = {};
    const ipAttempts: Record<string, number> = {};
    
    recentEvents.forEach(event => {
      if (event.userId) {
        userAttempts[event.userId] = (userAttempts[event.userId] || 0) + 1;
      }
      if (event.ipAddress) {
        ipAttempts[event.ipAddress] = (ipAttempts[event.ipAddress] || 0) + 1;
      }
    });

    const suspiciousActivity = Object.values(userAttempts).filter(count => count > 5).length +
                             Object.values(ipAttempts).filter(count => count > 10).length;

    return {
      totalErrors: recentEvents.length,
      errorsByType,
      topFailedPermissions,
      errorTrends,
      suspiciousActivity
    };
  }

  getRecentErrors(limit = 50): PermissionErrorEvent[] {
    return this.events
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, limit);
  }

  clearOldEvents(daysToKeep = 7): void {
    const cutoffTime = new Date(Date.now() - daysToKeep * 24 * 60 * 60 * 1000);
    this.events = this.events.filter(
      event => new Date(event.timestamp) > cutoffTime
    );
  }

  exportEvents(): PermissionErrorEvent[] {
    return [...this.events];
  }

  importEvents(events: PermissionErrorEvent[]): void {
    this.events = [...events];
  }
}

// Singleton instance
export const permissionErrorAnalytics = new PermissionErrorAnalytics();

// Helper functions
/**
 *
 * @param attemptedPermission
 * @param currentPermissions
 * @param userId
 * @param metadata
 */
export function logPermissionDenied(
  attemptedPermission: string,
  currentPermissions: string[],
  userId?: string,
  metadata?: Record<string, any>
): void {
  permissionErrorAnalytics.logPermissionError({
    errorType: 'access_denied',
    attemptedPermission,
    currentPermissions,
    userId,
    metadata
  });
}

/**
 *
 * @param attemptedPermission
 * @param reason
 * @param userId
 */
export function logInvalidPermission(
  attemptedPermission: string,
  reason: string,
  userId?: string
): void {
  permissionErrorAnalytics.logPermissionError({
    errorType: 'invalid_permission',
    attemptedPermission,
    currentPermissions: [],
    userId,
    metadata: { reason }
  });
}

/**
 *
 * @param expiredPermission
 * @param userId
 */
export function logExpiredPermission(
  expiredPermission: string,
  userId?: string
): void {
  permissionErrorAnalytics.logPermissionError({
    errorType: 'expired_permission',
    attemptedPermission: expiredPermission,
    currentPermissions: [],
    userId
  });
}

/**
 *
 * @param timeRangeHours
 */
export function getPermissionErrorStats(timeRangeHours?: number): PermissionErrorStats {
  return permissionErrorAnalytics.getErrorStats(timeRangeHours);
}

/**
 *
 * @param limit
 */
export function getRecentPermissionErrors(limit?: number): PermissionErrorEvent[] {
  return permissionErrorAnalytics.getRecentErrors(limit);
}