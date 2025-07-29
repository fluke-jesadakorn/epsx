import { useEffect, useCallback } from 'react';
import { usePathname } from 'next/navigation';
import { 
  initializeFirebaseAnalytics, 
  trackPageView, 
  trackUserAction, 
  setAnalyticsUserId,
  trackError,
  trackPerformance
} from '../core';
import { 
  trackEPSAnalysis,
  trackPatternRecognition,
  trackMarketDataAccess,
  trackSubscriptionEvent
} from '../user';

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