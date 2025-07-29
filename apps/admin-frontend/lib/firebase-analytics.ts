import { initializeApp, getApps } from 'firebase/app';
import { getAnalytics, Analytics, logEvent, setUserId } from 'firebase/analytics';

const firebaseConfig = {
  apiKey: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
  authDomain: process.env.NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN,
  projectId: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
  storageBucket: process.env.NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET,
  messagingSenderId: process.env.NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID,
  appId: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
  measurementId: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID
};

let analytics: Analytics | null = null;

export const initializeFirebaseAnalytics = (): Analytics | null => {
  if (typeof window !== 'undefined' && !getApps().length) {
    const app = initializeApp(firebaseConfig);
    analytics = getAnalytics(app);
    return analytics;
  }
  return analytics;
};

export const getFirebaseAnalytics = (): Analytics | null => {
  if (typeof window !== 'undefined' && !analytics) {
    analytics = initializeFirebaseAnalytics();
  }
  return analytics;
};

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

export const trackSecurityEvent = (eventType: string, severity: string, details: any, adminId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'security_event', {
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