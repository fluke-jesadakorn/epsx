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

// Analytics event tracking functions
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

export const trackEPSAnalysis = (symbol: string, analysisType: string, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'eps_analysis', {
      stock_symbol: symbol,
      analysis_type: analysisType,
      user_id: userId
    });
  }
};

export const trackPatternRecognition = (patternType: string, confidence: number, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'pattern_recognition', {
      pattern_type: patternType,
      confidence_score: confidence,
      user_id: userId
    });
  }
};

export const trackMarketDataAccess = (dataType: string, symbols: string[], userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'market_data_access', {
      data_type: dataType,
      symbol_count: symbols.length,
      symbols: symbols.join(','),
      user_id: userId
    });
  }
};

export const trackSubscriptionEvent = (eventType: string, planType: string, userId?: string) => {
  const analytics = getFirebaseAnalytics();
  if (analytics) {
    logEvent(analytics, 'subscription_event', {
      event_type: eventType,
      plan_type: planType,
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