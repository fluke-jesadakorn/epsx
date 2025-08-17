'use client';

import React, { createContext, useContext, useCallback, useEffect, useMemo, useRef } from 'react';
import { useAppState } from './app-state';
import { TradingState as _TradingState, StockItem, PortfolioItem, StockRanking, PriceAlert } from '@/lib/state/types';
import { useOptimisticUpdates, withAsyncState as _withAsyncState } from '@/lib/state/core';
import { createApiClient, isApiError } from '@/lib/api-client';
import type {WatchlistAddRequest as _WatchlistAddRequest, PriceAlertCreateRequest} from '@/lib/api-client';

interface TradingContextType {
  // Data
  watchlist: StockItem[];
  portfolio: PortfolioItem[];
  rankings: StockRanking[];
  alerts: PriceAlert[];
  recentSearches: string[];
  
  // Loading states
  loading: boolean;
  error: string | null;
  lastUpdated: number | null;
  
  // Realtime
  realtimeConnected: boolean;
  realtimeSubscriptions: string[];
  
  // Actions
  setWatchlist: (watchlist: StockItem[]) => void;
  addToWatchlist: (item: StockItem, optimistic?: boolean) => Promise<void>;
  removeFromWatchlist: (symbol: string, optimistic?: boolean) => Promise<void>;
  updateStockPrice: (symbol: string, price: number, change: number) => void;
  setPortfolio: (portfolio: PortfolioItem[]) => void;
  setRankings: (rankings: StockRanking[]) => void;
  addPriceAlert: (alert: Omit<PriceAlert, 'id' | 'createdAt'>, optimistic?: boolean) => Promise<void>;
  removePriceAlert: (id: string, optimistic?: boolean) => Promise<void>;
  addRecentSearch: (symbol: string) => void;
  refreshData: () => Promise<void>;
  
  // Realtime
  subscribeToSymbol: (symbol: string) => void;
  unsubscribeFromSymbol: (symbol: string) => void;
  
  // Helpers
  isInWatchlist: (symbol: string) => boolean;
  getStockPrice: (symbol: string) => number | null;
  getPortfolioItem: (symbol: string) => PortfolioItem | null;
  getTotalPortfolioValue: () => number;
  getPortfolioGainLoss: () => { gain: number; percentage: number };
}

const TradingContext = createContext<TradingContextType | undefined>(undefined);

interface TradingProviderProps {
  children: React.ReactNode;
}

export function TradingProvider({ children }: TradingProviderProps) {
  const { state, actions } = useAppState();
  const { trading } = state;
  const wsRef = useRef<WebSocket | null>(null);
  
  const {
    startOptimisticUpdate,
    confirmOptimisticUpdate,
    rollbackOptimisticUpdate
  } = useOptimisticUpdates();

  // Initialize API client to use Next.js API routes for trading API access
  const tradingApiClient = useMemo(() => {
    return createApiClient('/api');
  }, []);

  // WebSocket connection for real-time data
  const connectWebSocket = useCallback(() => {
    if (typeof window === 'undefined' || wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const wsUrl = process.env.NODE_ENV === 'development' 
        ? 'ws://localhost:8080/ws/trading'
        : 'wss://api.epsx.com/ws/trading';
      
      wsRef.current = new WebSocket(wsUrl);
      
      wsRef.current.onopen = () => {
        // WebSocket connected
        actions.trading.setRealtimeStatus({ connected: true });
        
        // Resubscribe to all symbols
        if (trading.realtime.subscriptions.length > 0) {
          wsRef.current?.send(JSON.stringify({
            type: 'subscribe',
            symbols: trading.realtime.subscriptions
          }));
        }
      };

      wsRef.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          
          switch (data.type) {
            case 'price_update':
              actions.trading.updateStockPrice(data.symbol, data.price, data.change);
              break;
            case 'portfolio_update':
              // Handle portfolio updates
              break;
            case 'ranking_update':
              // Handle ranking updates
              break;
          }
        } catch (error) {
          console.error('Error parsing WebSocket message:', error);
        }
      };

      wsRef.current.onclose = () => {
        // WebSocket disconnected
        actions.trading.setRealtimeStatus({ connected: false });
        
        // Reconnect after 5 seconds
        setTimeout(connectWebSocket, 5000);
      };

      wsRef.current.onerror = (error) => {
        console.error('Trading WebSocket error:', error);
        actions.trading.setRealtimeStatus({ connected: false });
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
    }
  }, [actions.trading, trading.realtime.subscriptions]);

  // Initialize WebSocket connection
  useEffect(() => {
    connectWebSocket();
    
    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [connectWebSocket]);

  // API calls using unified API client
  const addToWatchlistAPI = useCallback(async (item: StockItem) => {
    const response = await tradingApiClient.addToWatchlist({ symbol: item.symbol });
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to add to watchlist');
    }
    
    return response.data;
  }, [tradingApiClient]);

  const removeFromWatchlistAPI = useCallback(async (symbol: string) => {
    const response = await tradingApiClient.removeFromWatchlist(symbol);
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to remove from watchlist');
    }
  }, [tradingApiClient]);

  const addPriceAlertAPI = useCallback(async (alert: Omit<PriceAlert, 'id' | 'createdAt'>) => {
    const alertRequest: PriceAlertCreateRequest = {
      symbol: alert.symbol,
      type: alert.type,
      targetPrice: alert.targetPrice
    };
    
    const response = await tradingApiClient.addPriceAlert(alertRequest);
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to add price alert');
    }
    
    return response.data;
  }, [tradingApiClient]);

  const removePriceAlertAPI = useCallback(async (id: string) => {
    const response = await tradingApiClient.removePriceAlert(id);
    
    if (isApiError(response)) {
      throw new Error(response.error || 'Failed to remove price alert');
    }
  }, [tradingApiClient]);

  // Actions with optimistic updates
  const addToWatchlist = useCallback(async (item: StockItem, optimistic = true) => {
    if (!optimistic) {
      await addToWatchlistAPI(item);
      actions.trading.addToWatchlist(item);
      return;
    }

    const updateId = Math.random().toString(36);
    
    startOptimisticUpdate(
      updateId,
      () => actions.trading.addToWatchlist(item),
      () => actions.trading.removeFromWatchlist(item.symbol)
    );

    try {
      await addToWatchlistAPI(item);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [actions.trading, addToWatchlistAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const removeFromWatchlist = useCallback(async (symbol: string, optimistic = true) => {
    const currentItem = trading.data?.watchlist.find(item => item.symbol === symbol);
    
    if (!optimistic) {
      await removeFromWatchlistAPI(symbol);
      actions.trading.removeFromWatchlist(symbol);
      return;
    }

    const updateId = Math.random().toString(36);
    
    startOptimisticUpdate(
      updateId,
      () => actions.trading.removeFromWatchlist(symbol),
      () => currentItem && actions.trading.addToWatchlist(currentItem)
    );

    try {
      await removeFromWatchlistAPI(symbol);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [trading.data?.watchlist, actions.trading, removeFromWatchlistAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const addPriceAlert = useCallback(async (alert: Omit<PriceAlert, 'id' | 'createdAt'>, optimistic = true) => {
    const newAlert: PriceAlert = {
      ...alert,
      id: Math.random().toString(36),
      createdAt: new Date().toISOString()
    };

    if (!optimistic) {
      const result = await addPriceAlertAPI(alert);
      actions.trading.addPriceAlert(result);
      return;
    }

    const updateId = Math.random().toString(36);
    
    startOptimisticUpdate(
      updateId,
      () => actions.trading.addPriceAlert(newAlert),
      () => actions.trading.removePriceAlert(newAlert.id)
    );

    try {
      const result = await addPriceAlertAPI(alert);
      // Update with server-generated ID
      actions.trading.removePriceAlert(newAlert.id);
      actions.trading.addPriceAlert(result);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [actions.trading, addPriceAlertAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  const removePriceAlert = useCallback(async (id: string, optimistic = true) => {
    const currentAlert = trading.data?.alerts.find(alert => alert.id === id);
    
    if (!optimistic) {
      await removePriceAlertAPI(id);
      actions.trading.removePriceAlert(id);
      return;
    }

    const updateId = Math.random().toString(36);
    
    startOptimisticUpdate(
      updateId,
      () => actions.trading.removePriceAlert(id),
      () => currentAlert && actions.trading.addPriceAlert(currentAlert)
    );

    try {
      await removePriceAlertAPI(id);
      confirmOptimisticUpdate(updateId);
    } catch (error) {
      rollbackOptimisticUpdate(updateId);
      throw error;
    }
  }, [trading.data?.alerts, actions.trading, removePriceAlertAPI, startOptimisticUpdate, confirmOptimisticUpdate, rollbackOptimisticUpdate]);

  // Refresh all trading data
  const refreshData = useCallback(async () => {
    try {
      const [watchlistRes, portfolioRes, rankingsRes, alertsRes] = await Promise.all([
        tradingApiClient.getWatchlist(),
        tradingApiClient.getPortfolio(),
        tradingApiClient.getRankings(),
        tradingApiClient.getPriceAlerts()
      ]);

      // Handle each response, setting empty arrays if there are errors
      const watchlist = isApiError(watchlistRes) ? [] : watchlistRes.data || [];
      const portfolio = isApiError(portfolioRes) ? [] : portfolioRes.data || [];
      const rankings = isApiError(rankingsRes) ? [] : rankingsRes.data || [];
      const _alerts = isApiError(alertsRes) ? [] : alertsRes.data || [];

      actions.trading.setWatchlist(watchlist);
      actions.trading.setPortfolio(portfolio);
      actions.trading.setRankings(rankings);
      // Set alerts through a new action we need to add - for now we'll handle this separately
      
      // Log any API errors without failing the entire refresh
      if (isApiError(watchlistRes)) console.error('Failed to fetch watchlist:', watchlistRes.error);
      if (isApiError(portfolioRes)) console.error('Failed to fetch portfolio:', portfolioRes.error);
      if (isApiError(rankingsRes)) console.error('Failed to fetch rankings:', rankingsRes.error);
      if (isApiError(alertsRes)) console.error('Failed to fetch alerts:', alertsRes.error);
      
    } catch (error) {
      console.error('Failed to refresh trading data:', error);
      throw error;
    }
  }, [actions.trading, tradingApiClient]);

  // WebSocket subscription management
  const subscribeToSymbol = useCallback((symbol: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'subscribe',
        symbols: [symbol]
      }));
    }
    
    const currentSubscriptions = trading.realtime.subscriptions;
    if (!currentSubscriptions.includes(symbol)) {
      actions.trading.setRealtimeStatus({
        connected: trading.realtime.connected,
        subscriptions: [...currentSubscriptions, symbol]
      });
    }
  }, [wsRef, trading.realtime, actions.trading]);

  const unsubscribeFromSymbol = useCallback((symbol: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'unsubscribe',
        symbols: [symbol]
      }));
    }
    
    const currentSubscriptions = trading.realtime.subscriptions;
    actions.trading.setRealtimeStatus({
      connected: trading.realtime.connected,
      subscriptions: currentSubscriptions.filter(s => s !== symbol)
    });
  }, [wsRef, trading.realtime, actions.trading]);

  // Helper functions
  const isInWatchlist = useCallback((symbol: string) => {
    return trading.data?.watchlist.some(item => item.symbol === symbol) || false;
  }, [trading.data?.watchlist]);

  const getStockPrice = useCallback((symbol: string) => {
    const item = trading.data?.watchlist.find(item => item.symbol === symbol) ||
                 trading.data?.portfolio.find(item => item.symbol === symbol);
    return item?.price || null;
  }, [trading.data?.watchlist, trading.data?.portfolio]);

  const getPortfolioItem = useCallback((symbol: string) => {
    return trading.data?.portfolio.find(item => item.symbol === symbol) || null;
  }, [trading.data?.portfolio]);

  const getTotalPortfolioValue = useCallback(() => {
    return trading.data?.portfolio.reduce((total, item) => total + item.totalValue, 0) || 0;
  }, [trading.data?.portfolio]);

  const getPortfolioGainLoss = useCallback(() => {
    const portfolio = trading.data?.portfolio || [];
    const totalValue = getTotalPortfolioValue();
    const totalCost = portfolio.reduce((total, item) => total + (item.avgCost * item.shares), 0);
    const gain = totalValue - totalCost;
    const percentage = totalCost > 0 ? (gain / totalCost) * 100 : 0;
    
    return { gain, percentage };
  }, [trading.data?.portfolio, getTotalPortfolioValue]);

  const contextValue = useMemo(() => ({
    // Data
    watchlist: trading.data?.watchlist || [],
    portfolio: trading.data?.portfolio || [],
    rankings: trading.data?.rankings || [],
    alerts: trading.data?.alerts || [],
    recentSearches: trading.data?.recentSearches || [],
    
    // Loading states
    loading: trading.loading,
    error: trading.error,
    lastUpdated: trading.lastUpdated,
    
    // Realtime
    realtimeConnected: trading.realtime.connected,
    realtimeSubscriptions: trading.realtime.subscriptions,
    
    // Actions
    setWatchlist: actions.trading.setWatchlist,
    addToWatchlist,
    removeFromWatchlist,
    updateStockPrice: actions.trading.updateStockPrice,
    setPortfolio: actions.trading.setPortfolio,
    setRankings: actions.trading.setRankings,
    addPriceAlert,
    removePriceAlert,
    addRecentSearch: actions.trading.addRecentSearch,
    refreshData,
    
    // Realtime
    subscribeToSymbol,
    unsubscribeFromSymbol,
    
    // Helpers
    isInWatchlist,
    getStockPrice,
    getPortfolioItem,
    getTotalPortfolioValue,
    getPortfolioGainLoss
  }), [
    trading,
    actions.trading,
    addToWatchlist,
    removeFromWatchlist,
    addPriceAlert,
    removePriceAlert,
    refreshData,
    subscribeToSymbol,
    unsubscribeFromSymbol,
    isInWatchlist,
    getStockPrice,
    getPortfolioItem,
    getTotalPortfolioValue,
    getPortfolioGainLoss
  ]);

  return (
    <TradingContext.Provider value={contextValue}>
      {children}
    </TradingContext.Provider>
  );
}

export function useTrading() {
  const context = useContext(TradingContext);
  if (context === undefined) {
    throw new Error('useTrading must be used within a TradingProvider');
  }
  return context;
}

// Specialized hooks
export function useWatchlist() {
  const { watchlist, addToWatchlist, removeFromWatchlist, isInWatchlist } = useTrading();
  return { watchlist, addToWatchlist, removeFromWatchlist, isInWatchlist };
}

export function usePortfolio() {
  const { 
    portfolio, 
    getPortfolioItem, 
    getTotalPortfolioValue, 
    getPortfolioGainLoss 
  } = useTrading();
  return { 
    portfolio, 
    getPortfolioItem, 
    getTotalPortfolioValue, 
    getPortfolioGainLoss 
  };
}

export function usePriceAlerts() {
  const { alerts, addPriceAlert, removePriceAlert } = useTrading();
  return { alerts, addPriceAlert, removePriceAlert };
}

export function useRealtimeTrading() {
  const { 
    realtimeConnected, 
    realtimeSubscriptions, 
    subscribeToSymbol, 
    unsubscribeFromSymbol 
  } = useTrading();
  return { 
    realtimeConnected, 
    realtimeSubscriptions, 
    subscribeToSymbol, 
    unsubscribeFromSymbol 
  };
}