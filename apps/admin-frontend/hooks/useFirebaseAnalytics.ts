import { useEffect, useCallback } from 'react';
import { usePathname } from 'next/navigation';
import { 
  initializeFirebaseAnalytics, 
  trackAdminPageView, 
  trackAdminAction, 
  trackUserManagement,
  trackRoleManagement,
  trackPermissionChange,
  trackSystemConfiguration,
  trackSecurityEvent,
  trackAuditAction,
  trackDatabaseOperation,
  trackAPIUsage,
  setAdminAnalyticsUserId,
  trackAdminError,
  trackAdminPerformance
} from '../lib/firebase-analytics';

interface UseAdminFirebaseAnalyticsReturn {
  trackAction: (action: string, category: string, details?: any) => void;
  trackUserMgmt: (action: string, userId: string, changes?: any) => void;
  trackRoleMgmt: (action: string, roleId: string, permissions?: string[]) => void;
  trackPermissions: (action: string, resourceId: string, permissions: string[]) => void;
  trackSysConfig: (configType: string, changes: any) => void;
  trackSecurity: (eventType: string, severity: string, details: any) => void;
  trackAudit: (action: string, targetType: string, targetId: string) => void;
  trackDB: (operation: string, table: string, recordCount?: number) => void;
  trackAPI: (endpoint: string, method: string, responseTime: number) => void;
  trackErrorEvent: (errorMessage: string, errorCode?: string, context?: string) => void;
  trackPerformanceMetric: (metricName: string, value: number) => void;
  setAdminId: (adminId: string) => void;
}

export const useAdminFirebaseAnalytics = (adminId?: string): UseAdminFirebaseAnalyticsReturn => {
  const pathname = usePathname();

  // Initialize Firebase Analytics on mount
  useEffect(() => {
    initializeFirebaseAnalytics();
  }, []);

  // Track admin page views
  useEffect(() => {
    if (pathname) {
      trackAdminPageView(pathname, adminId);
    }
  }, [pathname, adminId]);

  // Set admin ID when provided
  useEffect(() => {
    if (adminId) {
      setAdminAnalyticsUserId(adminId);
    }
  }, [adminId]);

  const trackAction = useCallback((action: string, category: string, details?: any) => {
    trackAdminAction(action, category, details, adminId);
  }, [adminId]);

  const trackUserMgmt = useCallback((action: string, userId: string, changes?: any) => {
    trackUserManagement(action, userId, changes, adminId);
  }, [adminId]);

  const trackRoleMgmt = useCallback((action: string, roleId: string, permissions?: string[]) => {
    trackRoleManagement(action, roleId, permissions, adminId);
  }, [adminId]);

  const trackPermissions = useCallback((action: string, resourceId: string, permissions: string[]) => {
    trackPermissionChange(action, resourceId, permissions, adminId);
  }, [adminId]);

  const trackSysConfig = useCallback((configType: string, changes: any) => {
    trackSystemConfiguration(configType, changes, adminId);
  }, [adminId]);

  const trackSecurity = useCallback((eventType: string, severity: string, details: any) => {
    trackSecurityEvent(eventType, severity, details, adminId);
  }, [adminId]);

  const trackAudit = useCallback((action: string, targetType: string, targetId: string) => {
    trackAuditAction(action, targetType, targetId, adminId);
  }, [adminId]);

  const trackDB = useCallback((operation: string, table: string, recordCount?: number) => {
    trackDatabaseOperation(operation, table, recordCount, adminId);
  }, [adminId]);

  const trackAPI = useCallback((endpoint: string, method: string, responseTime: number) => {
    trackAPIUsage(endpoint, method, responseTime, adminId);
  }, [adminId]);

  const trackErrorEvent = useCallback((errorMessage: string, errorCode?: string, context?: string) => {
    trackAdminError(errorMessage, errorCode, context, adminId);
  }, [adminId]);

  const trackPerformanceMetric = useCallback((metricName: string, value: number) => {
    trackAdminPerformance(metricName, value, adminId);
  }, [adminId]);

  const setAdminId = useCallback((newAdminId: string) => {
    setAdminAnalyticsUserId(newAdminId);
  }, []);

  return {
    trackAction,
    trackUserMgmt,
    trackRoleMgmt,
    trackPermissions,
    trackSysConfig,
    trackSecurity,
    trackAudit,
    trackDB,
    trackAPI,
    trackErrorEvent,
    trackPerformanceMetric,
    setAdminId
  };
};

// Specialized hooks for different admin features
export const useUserManagementAnalytics = (adminId?: string) => {
  const { trackUserMgmt, trackAction } = useAdminFirebaseAnalytics(adminId);

  const trackUserCreation = useCallback((userId: string, userType: string) => {
    trackUserMgmt('create', userId, { userType });
    trackAction('user_created', 'user_management', { userId, userType });
  }, [trackUserMgmt, trackAction]);

  const trackUserStatusChange = useCallback((userId: string, oldStatus: string, newStatus: string) => {
    trackUserMgmt('status_change', userId, { oldStatus, newStatus });
    trackAction('user_status_changed', 'user_management', { userId, oldStatus, newStatus });
  }, [trackUserMgmt, trackAction]);

  const trackBulkUserOperation = useCallback((operation: string, userCount: number) => {
    trackAction('bulk_user_operation', 'user_management', { operation, userCount });
  }, [trackAction]);

  return {
    trackUserCreation,
    trackUserStatusChange,
    trackBulkUserOperation
  };
};

export const useIAMAnalytics = (adminId?: string) => {
  const { trackRoleMgmt, trackPermissions, trackAction } = useAdminFirebaseAnalytics(adminId);

  const trackRoleCreation = useCallback((roleId: string, permissions: string[]) => {
    trackRoleMgmt('create', roleId, permissions);
    trackAction('role_created', 'iam', { roleId, permissionCount: permissions.length });
  }, [trackRoleMgmt, trackAction]);

  const trackRoleAssignment = useCallback((userId: string, roleId: string) => {
    trackAction('role_assigned', 'iam', { userId, roleId });
  }, [trackAction]);

  const trackPermissionUpdate = useCallback((resourceId: string, permissions: string[]) => {
    trackPermissions('update', resourceId, permissions);
    trackAction('permissions_updated', 'iam', { resourceId, permissionCount: permissions.length });
  }, [trackPermissions, trackAction]);

  return {
    trackRoleCreation,
    trackRoleAssignment,
    trackPermissionUpdate
  };
};

export const useSecurityAnalytics = (adminId?: string) => {
  const { trackSecurity, trackAction } = useAdminFirebaseAnalytics(adminId);

  const trackSecurityIncident = useCallback((incidentType: string, severity: 'low' | 'medium' | 'high' | 'critical', details: any) => {
    trackSecurity(incidentType, severity, details);
    trackAction('security_incident_logged', 'security', { incidentType, severity });
  }, [trackSecurity, trackAction]);

  const trackAccessViolation = useCallback((userId: string, resource: string, attemptedAction: string) => {
    trackSecurity('access_violation', 'medium', { userId, resource, attemptedAction });
    trackAction('access_violation_detected', 'security', { userId, resource });
  }, [trackSecurity, trackAction]);

  const trackLoginAttempt = useCallback((userId: string, success: boolean, ipAddress?: string) => {
    trackAction('login_attempt', 'security', { userId, success, ipAddress });
  }, [trackAction]);

  return {
    trackSecurityIncident,
    trackAccessViolation,
    trackLoginAttempt
  };
};