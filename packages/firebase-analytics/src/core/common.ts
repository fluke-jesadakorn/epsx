import { logEvent, setUserId } from 'firebase/analytics';
import { getFirebaseAnalytics } from './firebase';

// Common analytics functions shared across all apps
export const trackPageView = (pageName: string, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'page_view', {
      page_title: pageName,
      page_location: window.location.href,
      user_id: userId
    });
  }
};

export const trackUserAction = (action: string, category: string, label?: string, value?: number, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, action, {
      event_category: category,
      event_label: label,
      value: value,
      user_id: userId
    });
  }
};

export const setAnalyticsUserId = (userId: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    setUserId(analytics, userId);
  }
};

export const trackError = (errorMessage: string, errorCode?: string, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'exception', {
      description: errorMessage,
      error_code: errorCode,
      fatal: false,
      user_id: userId
    });
  }
};

export const trackPerformance = (metricName: string, value: number, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'performance_metric', {
      metric_name: metricName,
      metric_value: value,
      user_id: userId
    });
  }
};

export const trackSecurityEvent = (
  action: string, 
  resource: string, 
  success: boolean, 
  userId?: string, 
  category?: string,
  severity?: string,
  additionalData?: Record<string, any>
) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'security_event', {
      security_action: action,
      resource_type: resource,
      success: success,
      event_category: category || 'security',
      severity_level: severity || 'medium',
      user_id: userId,
      ...additionalData
    });
  }
};