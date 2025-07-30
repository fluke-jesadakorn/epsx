// Server-safe exports - no React hooks or client-side code
// These are safe no-op functions that can be used in server contexts

// Safe no-op implementations for server environments
export const initializeFirebaseAnalytics = () => null;
export const getFirebaseAnalytics = () => null;

// Helper function to safely handle dynamic imports
const safeImport = (modulePath: string, functionName: string, ...args: any[]): void => {
  import(modulePath)
    .then((module: any) => {
      if (module[functionName]) {
        module[functionName](...args);
      }
    })
    .catch((error: any) => {
      console.warn(`Failed to load analytics module ${modulePath}:`, error);
    });
};

// Safe tracking functions that only work client-side
export const trackPageView = (pageName: string, userId?: string) => {
  // Only track if we're in a browser environment
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'trackPageView', pageName, userId);
  }
};

export const trackUserAction = (action: string, category: string, label?: string, value?: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'trackUserAction', action, category, label, value, userId);
  }
};

export const setAnalyticsUserId = (userId: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'setAnalyticsUserId', userId);
  }
};

export const trackError = (errorMessage: string, errorCode?: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'trackError', errorMessage, errorCode, userId);
  }
};

export const trackPerformance = (metricName: string, value: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'trackPerformance', metricName, value, userId);
  }
};

export const trackSecurityEvent = (action: string, resource: string, success: boolean, userId?: string, category?: string, severity?: string, additionalData?: Record<string, any>) => {
  if (typeof window !== 'undefined') {
    safeImport('./core/common', 'trackSecurityEvent', action, resource, success, userId, category, severity, additionalData);
  }
};

// User analytics functions
export const trackEPSAnalysis = (symbol: string, analysisType: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./user/analytics', 'trackEPSAnalysis', symbol, analysisType, userId);
  }
};

export const trackPatternRecognition = (patternType: string, confidence: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./user/analytics', 'trackPatternRecognition', patternType, confidence, userId);
  }
};

export const trackMarketDataAccess = (dataType: string, symbols: string[], userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./user/analytics', 'trackMarketDataAccess', dataType, symbols, userId);
  }
};

export const trackSubscriptionEvent = (eventType: string, planType: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./user/analytics', 'trackSubscriptionEvent', eventType, planType, userId);
  }
};

// Admin analytics functions
export const trackAdminPageView = (pageName: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAdminPageView', pageName, adminId);
  }
};

export const trackAdminAction = (action: string, category: string, details?: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAdminAction', action, category, details, adminId);
  }
};

export const trackUserManagement = (action: string, userId: string, changes?: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackUserManagement', action, userId, changes, adminId);
  }
};

export const trackRoleManagement = (action: string, roleId: string, permissions?: string[], adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackRoleManagement', action, roleId, permissions, adminId);
  }
};

export const trackPermissionChange = (action: string, resourceId: string, permissions: string[], adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackPermissionChange', action, resourceId, permissions, adminId);
  }
};

export const trackSystemConfiguration = (configType: string, changes: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackSystemConfiguration', configType, changes, adminId);
  }
};

export const trackAdminSecurityEvent = (eventType: string, severity: string, details: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAdminSecurityEvent', eventType, severity, details, adminId);
  }
};

export const trackAuditAction = (action: string, targetType: string, targetId: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAuditAction', action, targetType, targetId, adminId);
  }
};

export const trackDatabaseOperation = (operation: string, table: string, recordCount?: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackDatabaseOperation', operation, table, recordCount, adminId);
  }
};

export const trackAPIUsage = (endpoint: string, method: string, responseTime: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAPIUsage', endpoint, method, responseTime, adminId);
  }
};

export const setAdminAnalyticsUserId = (adminId: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'setAdminAnalyticsUserId', adminId);
  }
};

export const trackAdminError = (errorMessage: string, errorCode?: string, context?: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAdminError', errorMessage, errorCode, context, adminId);
  }
};

export const trackAdminPerformance = (metricName: string, value: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    safeImport('./admin/analytics', 'trackAdminPerformance', metricName, value, adminId);
  }
};