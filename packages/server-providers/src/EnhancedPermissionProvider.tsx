'use client';

import React, { createContext, useContext, useReducer, useEffect, ReactNode } from 'react';
import type { 
  PaymentPlan,
  ApiResponse 
} from '@epsx/types';
import type { 
  UserPermission,
  PaymentStatus as ServerPaymentStatus 
} from '@epsx/server-actions';

// Enhanced state structure
export interface ServerState {
  permissions: UserPermission[] | null;
  paymentStatus: ServerPaymentStatus | null;
  plans: PaymentPlan[] | null;
  featureAccess: Record<string, boolean>;
  rankingAccess: { allowed: boolean; tier: string; expiresAt?: string };
  loading: boolean;
  error: string | null;
  lastUpdated: Date | null;
}

// Action types for state management
export type ServerStateAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_PERMISSIONS'; payload: UserPermission[] }
  | { type: 'SET_PAYMENT_STATUS'; payload: ServerPaymentStatus }
  | { type: 'SET_PLANS'; payload: PaymentPlan[] }
  | { type: 'SET_FEATURE_ACCESS'; payload: Record<string, boolean> }
  | { type: 'SET_RANKING_ACCESS'; payload: { allowed: boolean; tier: string; expiresAt?: string } }
  | { type: 'RESET_STATE' }
  | { type: 'UPDATE_TIMESTAMP' };

// Initial state
const initialState: ServerState = {
  permissions: null,
  paymentStatus: null,
  plans: null,
  featureAccess: {},
  rankingAccess: { allowed: false, tier: 'BRONZE' },
  loading: true,
  error: null,
  lastUpdated: null,
};

// State reducer
function serverStateReducer(state: ServerState, action: ServerStateAction): ServerState {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    case 'SET_PERMISSIONS':
      return { 
        ...state, 
        permissions: action.payload, 
        error: null, 
        loading: false,
        lastUpdated: new Date()
      };
    case 'SET_PAYMENT_STATUS':
      return { 
        ...state, 
        paymentStatus: action.payload, 
        error: null,
        lastUpdated: new Date()
      };
    case 'SET_PLANS':
      return { 
        ...state, 
        plans: action.payload, 
        error: null,
        lastUpdated: new Date()
      };
    case 'SET_FEATURE_ACCESS':
      return { 
        ...state, 
        featureAccess: action.payload, 
        error: null,
        lastUpdated: new Date()
      };
    case 'SET_RANKING_ACCESS':
      return { 
        ...state, 
        rankingAccess: action.payload, 
        error: null,
        lastUpdated: new Date()
      };
    case 'RESET_STATE':
      return initialState;
    case 'UPDATE_TIMESTAMP':
      return { ...state, lastUpdated: new Date() };
    default:
      return state;
  }
}

// Context interface
interface ServerStateContextValue {
  state: ServerState;
  actions: {
    setLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
    setPermissions: (permissions: UserPermission[]) => void;
    setPaymentStatus: (status: ServerPaymentStatus) => void;
    setPlans: (plans: PaymentPlan[]) => void;
    setFeatureAccess: (access: Record<string, boolean>) => void;
    setRankingAccess: (access: { allowed: boolean; tier: string; expiresAt?: string }) => void;
    resetState: () => void;
    refresh: () => void;
  };
}

// Context
const ServerStateContext = createContext<ServerStateContextValue | null>(null);

// Provider props
interface EnhancedPermissionProviderProps {
  children: ReactNode;
  initialData?: Partial<ServerState>;
  enableRefresh?: boolean;
  refreshInterval?: number;
  onError?: (error: string) => void;
}

// Error boundary component
class ServerProviderErrorBoundary extends React.Component<
  { children: ReactNode; onError?: (error: string) => void },
  { hasError: boolean; error: Error | null }
> {
  constructor(props: { children: ReactNode; onError?: (error: string) => void }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ServerProvider Error Boundary caught error', {
      error: error.message,
      stack: error.stack,
      errorInfo,
      component: 'ServerProviderErrorBoundary'
    });

    if (this.props.onError) {
      this.props.onError(error.message);
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-4 bg-red-50 border border-red-200 rounded-md">
          <h3 className="text-red-800 font-semibold">Server Provider Error</h3>
          <p className="text-red-600 text-sm mt-1">
            {this.state.error?.message || 'An unexpected error occurred'}
          </p>
          <button 
            onClick={() => this.setState({ hasError: false, error: null })}
            className="mt-2 px-3 py-1 bg-red-600 text-white text-sm rounded hover:bg-red-700"
          >
            Try Again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

// Main provider component
export function EnhancedPermissionProvider({
  children,
  initialData,
  enableRefresh = false,
  refreshInterval = 5 * 60 * 1000, // 5 minutes
  onError
}: EnhancedPermissionProviderProps) {
  const [state, dispatch] = useReducer(serverStateReducer, {
    ...initialState,
    ...initialData
  });

  // Action creators
  const actions = {
    setLoading: (loading: boolean) => dispatch({ type: 'SET_LOADING', payload: loading }),
    setError: (error: string | null) => dispatch({ type: 'SET_ERROR', payload: error }),
    setPermissions: (permissions: UserPermission[]) => 
      dispatch({ type: 'SET_PERMISSIONS', payload: permissions }),
    setPaymentStatus: (status: ServerPaymentStatus) => 
      dispatch({ type: 'SET_PAYMENT_STATUS', payload: status }),
    setPlans: (plans: PaymentPlan[]) => dispatch({ type: 'SET_PLANS', payload: plans }),
    setFeatureAccess: (access: Record<string, boolean>) => 
      dispatch({ type: 'SET_FEATURE_ACCESS', payload: access }),
    setRankingAccess: (access: { allowed: boolean; tier: string; expiresAt?: string }) => 
      dispatch({ type: 'SET_RANKING_ACCESS', payload: access }),
    resetState: () => dispatch({ type: 'RESET_STATE' }),
    refresh: () => dispatch({ type: 'UPDATE_TIMESTAMP' })
  };

  // Auto-refresh effect
  useEffect(() => {
    if (!enableRefresh || refreshInterval <= 0) return;

    const interval = setInterval(() => {
      console.debug('Auto-refreshing server state', { component: 'EnhancedPermissionProvider' });
      actions.refresh();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [enableRefresh, refreshInterval]);

  // Error handling effect
  useEffect(() => {
    if (state.error && onError) {
      onError(state.error);
    }
  }, [state.error, onError]);

  const contextValue: ServerStateContextValue = {
    state,
    actions
  };

  return (
    <ServerProviderErrorBoundary onError={onError}>
      <ServerStateContext.Provider value={contextValue}>
        {children}
      </ServerStateContext.Provider>
    </ServerProviderErrorBoundary>
  );
}

// Hook to use the enhanced permission context
export function useEnhancedPermissionContext() {
  const context = useContext(ServerStateContext);
  if (!context) {
    throw new Error('useEnhancedPermissionContext must be used within an EnhancedPermissionProvider');
  }
  return context;
}

// Specialized hooks for specific data
export function usePermissions() {
  const { state } = useEnhancedPermissionContext();
  return {
    permissions: state.permissions,
    loading: state.loading,
    error: state.error
  };
}

export function usePaymentStatus() {
  const { state } = useEnhancedPermissionContext();
  return {
    paymentStatus: state.paymentStatus,
    plans: state.plans,
    loading: state.loading,
    error: state.error
  };
}

export function useFeatureAccess() {
  const { state } = useEnhancedPermissionContext();
  return {
    featureAccess: state.featureAccess,
    rankingAccess: state.rankingAccess,
    loading: state.loading,
    error: state.error
  };
}

// Loading component
export function ServerStateLoading({ children }: { children?: ReactNode }) {
  const { state } = useEnhancedPermissionContext();
  
  if (state.loading) {
    return children || (
      <div className="animate-pulse">
        <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
        <div className="h-4 bg-gray-200 rounded w-1/2"></div>
      </div>
    );
  }
  
  return null;
}

// Error component  
export function ServerStateError({ children }: { children?: ReactNode }) {
  const { state, actions } = useEnhancedPermissionContext();
  
  if (state.error) {
    return children || (
      <div className="p-3 bg-red-50 border border-red-200 rounded text-red-700">
        <p className="text-sm">{state.error}</p>
        <button 
          onClick={() => actions.setError(null)}
          className="mt-1 text-xs underline hover:no-underline"
        >
          Dismiss
        </button>
      </div>
    );
  }
  
  return null;
}