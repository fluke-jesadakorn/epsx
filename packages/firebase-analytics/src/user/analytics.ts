import { logEvent } from 'firebase/analytics';
import { getFirebaseAnalytics } from '../core/firebase';

// User-specific analytics tracking functions
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