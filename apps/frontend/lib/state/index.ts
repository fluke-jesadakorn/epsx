// Simplified state management exports
import * as React from 'react';

export * from './hooks';
export * from './store';

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
export const analyticsMiddleware = (action: { type: string }, _prevState: unknown, _nextState: unknown, _store: string) => {
  // Only track specific analytics-related actions
  const trackableActions = ['UPDATE_ANALYTICS', 'SET_FILTERS', 'ADD_TO_ANALYTICS'];
  if (trackableActions.includes(action.type)) {
    // Action is trackable, but we don't need to do anything here for now
  }

  // External analytics tracking disabled for security
  // State changes are not sent to external services
};

// Logging middleware for debugging state changes
export const loggingMiddleware = (_action: unknown, _prevState: unknown, _nextState: unknown, _store: string) => {
  if (process.env.NODE_ENV === 'development') {
    // Debug logging could be enabled here
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