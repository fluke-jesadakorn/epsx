// Server-safe exports - no React hooks or client-side code
// These are safe no-op functions that can be used in server contexts

// Safe no-op implementations for server environments
export const initializeFirebaseAnalytics = () => null;
export const getFirebaseAnalytics = () => null;

// Safe tracking functions that only work client-side
export const trackPageView = (pageName: string, userId?: string) => {
  // Only track if we're in a browser environment
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ trackPageView }) => trackPageView(pageName, userId));
  }
};

export const trackUserAction = (action: string, category: string, label?: string, value?: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ trackUserAction }) => trackUserAction(action, category, label, value, userId));
  }
};

export const setAnalyticsUserId = (userId: string) => {
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ setAnalyticsUserId }) => setAnalyticsUserId(userId));
  }
};

export const trackError = (errorMessage: string, errorCode?: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ trackError }) => trackError(errorMessage, errorCode, userId));
  }
};

export const trackPerformance = (metricName: string, value: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ trackPerformance }) => trackPerformance(metricName, value, userId));
  }
};

export const trackSecurityEvent = (action: string, resource: string, success: boolean, userId?: string, category?: string, severity?: string, additionalData?: Record<string, any>) => {
  if (typeof window !== 'undefined') {
    import('./core/common').then(({ trackSecurityEvent }) => trackSecurityEvent(action, resource, success, userId, category, severity, additionalData));
  }
};

// User analytics functions
export const trackEPSAnalysis = (symbol: string, analysisType: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./user/analytics').then(({ trackEPSAnalysis }) => trackEPSAnalysis(symbol, analysisType, userId));
  }
};

export const trackPatternRecognition = (patternType: string, confidence: number, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./user/analytics').then(({ trackPatternRecognition }) => trackPatternRecognition(patternType, confidence, userId));
  }
};

export const trackMarketDataAccess = (dataType: string, symbols: string[], userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./user/analytics').then(({ trackMarketDataAccess }) => trackMarketDataAccess(dataType, symbols, userId));
  }
};

export const trackSubscriptionEvent = (eventType: string, planType: string, userId?: string) => {
  if (typeof window !== 'undefined') {
    import('./user/analytics').then(({ trackSubscriptionEvent }) => trackSubscriptionEvent(eventType, planType, userId));
  }
};

// Admin analytics functions
export const trackAdminPageView = (pageName: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAdminPageView }) => trackAdminPageView(pageName, adminId));
  }
};

export const trackAdminAction = (action: string, category: string, details?: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAdminAction }) => trackAdminAction(action, category, details, adminId));
  }
};

export const trackUserManagement = (action: string, userId: string, changes?: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackUserManagement }) => trackUserManagement(action, userId, changes, adminId));
  }
};

export const trackRoleManagement = (action: string, roleId: string, permissions?: string[], adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackRoleManagement }) => trackRoleManagement(action, roleId, permissions, adminId));
  }
};

export const trackPermissionChange = (action: string, resourceId: string, permissions: string[], adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackPermissionChange }) => trackPermissionChange(action, resourceId, permissions, adminId));
  }
};

export const trackSystemConfiguration = (configType: string, changes: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackSystemConfiguration }) => trackSystemConfiguration(configType, changes, adminId));
  }
};

export const trackAdminSecurityEvent = (eventType: string, severity: string, details: any, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAdminSecurityEvent }) => trackAdminSecurityEvent(eventType, severity, details, adminId));
  }
};

export const trackAuditAction = (action: string, targetType: string, targetId: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAuditAction }) => trackAuditAction(action, targetType, targetId, adminId));
  }
};

export const trackDatabaseOperation = (operation: string, table: string, recordCount?: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackDatabaseOperation }) => trackDatabaseOperation(operation, table, recordCount, adminId));
  }
};

export const trackAPIUsage = (endpoint: string, method: string, responseTime: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAPIUsage }) => trackAPIUsage(endpoint, method, responseTime, adminId));
  }
};

export const setAdminAnalyticsUserId = (adminId: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ setAdminAnalyticsUserId }) => setAdminAnalyticsUserId(adminId));
  }
};

export const trackAdminError = (errorMessage: string, errorCode?: string, context?: string, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAdminError }) => trackAdminError(errorMessage, errorCode, context, adminId));
  }
};

export const trackAdminPerformance = (metricName: string, value: number, adminId?: string) => {
  if (typeof window !== 'undefined') {
    import('./admin/analytics').then(({ trackAdminPerformance }) => trackAdminPerformance(metricName, value, adminId));
  }
};