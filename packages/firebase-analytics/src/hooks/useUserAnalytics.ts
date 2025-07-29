import { useCallback } from 'react';
import { useFirebaseAnalytics } from './useFirebaseAnalytics';

// Specialized hooks for different user features
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