// Simplified state management exports
import * as React from 'react';

export * from './store';
export * from './hooks';

// Missing exports that are needed by context files
export { useStateManager as useContextStore } from './store';

// Create a default async state helper
export function createAsyncState() {
  return {
    data: null,
    loading: false,
    error: null,
    lastUpdated: null,
    optimisticUpdates: []
  };
}

// Analytics middleware for tracking state changes
export const analyticsMiddleware = (action: any, prevState: any, nextState: any, store: string) => {
  // Only track specific analytics-related actions
  const trackableActions = ['UPDATE_ANALYTICS', 'SET_FILTERS', 'ADD_TO_ANALYTICS'];
  
  // External analytics tracking disabled for security
  // State changes are not sent to external services
};

// Logging middleware for debugging state changes
export const loggingMiddleware = (action: any, prevState: any, nextState: any, store: string) => {
  if (process.env.NODE_ENV === 'development') {
  }
};

// Client-only hook for components that need to run only on client side
export function useClientOnly() {
  const [hasMounted, setHasMounted] = React.useState(false);

  React.useEffect(() => {
    setHasMounted(true);
  }, []);

  return hasMounted;
}