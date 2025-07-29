import { logEvent, setUserId } from 'firebase/analytics';
import { getFirebaseAnalytics } from '../core/firebase';

// Admin-specific analytics tracking functions
export const trackAdminPageView = (pageName: string, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'admin_page_view', {
      page_title: pageName,
      page_location: window.location.href,
      admin_id: adminId
    });
  }
};

export const trackAdminAction = (action: string, category: string, details?: any, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'admin_action', {
      action_type: action,
      category: category,
      details: JSON.stringify(details),
      admin_id: adminId
    });
  }
};

export const trackUserManagement = (action: string, userId: string, changes?: any, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'user_management', {
      action_type: action,
      target_user_id: userId,
      changes: JSON.stringify(changes),
      admin_id: adminId
    });
  }
};

export const trackRoleManagement = (action: string, roleId: string, permissions?: string[], adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'role_management', {
      action_type: action,
      role_id: roleId,
      permissions: permissions?.join(','),
      admin_id: adminId
    });
  }
};

export const trackPermissionChange = (action: string, resourceId: string, permissions: string[], adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'permission_change', {
      action_type: action,
      resource_id: resourceId,
      permissions: permissions.join(','),
      admin_id: adminId
    });
  }
};

export const trackSystemConfiguration = (configType: string, changes: any, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'system_configuration', {
      config_type: configType,
      changes: JSON.stringify(changes),
      admin_id: adminId
    });
  }
};

export const trackAdminSecurityEvent = (eventType: string, severity: string, details: any, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'admin_security_event', {
      event_type: eventType,
      severity: severity,
      details: JSON.stringify(details),
      admin_id: adminId
    });
  }
};

export const trackAuditAction = (action: string, targetType: string, targetId: string, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'audit_action', {
      action_type: action,
      target_type: targetType,
      target_id: targetId,
      admin_id: adminId
    });
  }
};

export const trackDatabaseOperation = (operation: string, table: string, recordCount?: number, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'database_operation', {
      operation_type: operation,
      table_name: table,
      record_count: recordCount,
      admin_id: adminId
    });
  }
};

export const trackAPIUsage = (endpoint: string, method: string, responseTime: number, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'api_usage', {
      endpoint: endpoint,
      method: method,
      response_time: responseTime,
      admin_id: adminId
    });
  }
};

export const setAdminAnalyticsUserId = (adminId: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    setUserId(analytics, `admin_${adminId}`);
  }
};

export const trackAdminError = (errorMessage: string, errorCode?: string, context?: string, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'admin_exception', {
      description: errorMessage,
      error_code: errorCode,
      context: context,
      fatal: false,
      admin_id: adminId
    });
  }
};

export const trackAdminPerformance = (metricName: string, value: number, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'admin_performance_metric', {
      metric_name: metricName,
      metric_value: value,
      admin_id: adminId
    });
  }
};