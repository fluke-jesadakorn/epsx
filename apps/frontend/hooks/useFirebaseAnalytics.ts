import { useEffect, useCallback } from 'react';
import { usePathname } from 'next/navigation';
import { 
  initializeFirebaseAnalytics, 
  trackPageView, 
  trackUserAction, 
  trackEPSAnalysis,
  trackPatternRecognition,
  trackMarketDataAccess,
  trackSubscriptionEvent,
  setAnalyticsUserId,
  trackError,
  trackPerformance
} from '../lib/firebase-analytics';

interface UseFirebaseAnalyticsReturn {
  trackAction: (action: string, category: string, label?: string, value?: number) => void;
  trackEPS: (symbol: string, analysisType: string) => void;
  trackPattern: (patternType: string, confidence: number) => void;
  trackMarketData: (dataType: string, symbols: string[]) => void;
  trackSubscription: (eventType: string, planType: string) => void;
  trackErrorEvent: (errorMessage: string, errorCode?: string) => void;
  trackPerformanceMetric: (metricName: string, value: number) => void;
  setUserId: (userId: string) => void;
}

export const useFirebaseAnalytics = (userId?: string): UseFirebaseAnalyticsReturn => {
  const pathname = usePathname();

  // Initialize Firebase Analytics on mount
  useEffect(() => {
    initializeFirebaseAnalytics();
  }, []);

  // Track page views
  useEffect(() => {
    if (pathname) {
      trackPageView(pathname, userId);
    }
  }, [pathname, userId]);

  // Set user ID when provided
  useEffect(() => {
    if (userId) {
      setAnalyticsUserId(userId);
    }
  }, [userId]);

  const trackAction = useCallback((action: string, category: string, label?: string, value?: number) => {
    trackUserAction(action, category, label, value, userId);
  }, [userId]);

  const trackEPS = useCallback((symbol: string, analysisType: string) => {
    trackEPSAnalysis(symbol, analysisType, userId);
  }, [userId]);

  const trackPattern = useCallback((patternType: string, confidence: number) => {
    trackPatternRecognition(patternType, confidence, userId);
  }, [userId]);

  const trackMarketData = useCallback((dataType: string, symbols: string[]) => {
    trackMarketDataAccess(dataType, symbols, userId);
  }, [userId]);

  const trackSubscription = useCallback((eventType: string, planType: string) => {
    trackSubscriptionEvent(eventType, planType, userId);
  }, [userId]);

  const trackErrorEvent = useCallback((errorMessage: string, errorCode?: string) => {
    trackError(errorMessage, errorCode, userId);
  }, [userId]);

  const trackPerformanceMetric = useCallback((metricName: string, value: number) => {
    trackPerformance(metricName, value, userId);
  }, [userId]);

  const setUserId = useCallback((newUserId: string) => {
    setAnalyticsUserId(newUserId);
  }, []);

  return {
    trackAction,
    trackEPS,
    trackPattern,
    trackMarketData,
    trackSubscription,
    trackErrorEvent,
    trackPerformanceMetric,
    setUserId
  };
};

// Specialized hooks for different features
export const useEPSAnalytics = (userId?: string) => {
  const { trackEPS, trackAction } = useFirebaseAnalytics(userId);

  const trackEPSFormSubmit = useCallback((symbol: string) => {
    trackEPS(symbol, 'form_submit');
    trackAction('eps_analysis_submit', 'analytics', symbol);
  }, [trackEPS, trackAction]);

  const trackEPSChartView = useCallback((symbol: string, chartType: string) => {
    trackEPS(symbol, `chart_view_${chartType}`);
    trackAction('eps_chart_view', 'analytics', `${symbol}_${chartType}`);
  }, [trackEPS, trackAction]);

  const trackEPSExport = useCallback((symbol: string, format: string) => {
    trackEPS(symbol, `export_${format}`);
    trackAction('eps_export', 'analytics', `${symbol}_${format}`);
  }, [trackEPS, trackAction]);

  return {
    trackEPSFormSubmit,
    trackEPSChartView,
    trackEPSExport
  };
};

export const usePatternAnalytics = (userId?: string) => {
  const { trackPattern, trackAction } = useFirebaseAnalytics(userId);

  const trackPatternDetection = useCallback((patternType: string, confidence: number, symbol: string) => {
    trackPattern(patternType, confidence);
    trackAction('pattern_detected', 'analytics', `${patternType}_${symbol}`, confidence);
  }, [trackPattern, trackAction]);

  const trackPatternAlert = useCallback((patternType: string, symbol: string) => {
    trackAction('pattern_alert_created', 'alerts', `${patternType}_${symbol}`);
  }, [trackAction]);

  return {
    trackPatternDetection,
    trackPatternAlert
  };
};

export const useMarketDataAnalytics = (userId?: string) => {
  const { trackMarketData, trackAction } = useFirebaseAnalytics(userId);

  const trackDataRequest = useCallback((dataType: string, symbols: string[]) => {
    trackMarketData(dataType, symbols);
    trackAction('market_data_request', 'data', dataType, symbols.length);
  }, [trackMarketData, trackAction]);

  const trackWatchlistAction = useCallback((action: string, symbol: string) => {
    trackAction(`watchlist_${action}`, 'watchlist', symbol);
  }, [trackAction]);

  const trackFilterUsage = useCallback((filterType: string, filterValues: string[]) => {
    trackAction('filter_applied', 'market_data', filterType, filterValues.length);
  }, [trackAction]);

  return {
    trackDataRequest,
    trackWatchlistAction,
    trackFilterUsage
  };
};