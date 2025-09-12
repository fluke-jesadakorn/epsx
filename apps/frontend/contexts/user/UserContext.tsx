'use client';

import React, { createContext, useContext, useMemo } from 'react';
import { useContextStore, createAsyncState, loggingMiddleware } from '@/lib/state/core';
import type { 
  UserState, 
  StateAction, 
  UserProfile, 
  UserSubscription, 
  OptimisticUpdate 
} from '@/lib/state/types';
import { authLogger } from '@/lib/logger';

// Initial user state
const initialUserState: UserState = {
  ...createAsyncState(),
  data: {
    profile: null,
    preferences: {
      language: 'en',
      timezone: 'UTC',
      currency: 'USD',
      notifications: {
        email: true,
        push: true,
        tradingAlerts: true,
        priceAlerts: true,
      },
      trading: {
        defaultView: 'grid',
        riskTolerance: 'medium',
        autoRefresh: true,
        refreshInterval: 30000,
      },
    },
    subscription: null,
    permissions: [],
    packageTier: 'FREE',
  },
  optimisticUpdates: [],
};

// User Context interface
interface UserContextType {
  state: UserState;
  actions: {
    setProfile: (profile: UserProfile | null) => void;
    updatePreferences: (preferences: Partial<UserState['data']['preferences']>) => void;
    setSubscription: (subscription: UserSubscription | null) => void;
    updatePermissions: (permissions: string[]) => void;
    setPackageTier: (tier: string) => void;
    setLoading: (loading: boolean) => void;
    setError: (error: string | null) => void;
    addOptimisticUpdate: (update: OptimisticUpdate) => void;
    confirmOptimisticUpdate: (id: string) => void;
    rollbackOptimisticUpdate: (id: string) => void;
  };
}

const UserContext = createContext<UserContextType | undefined>(undefined);

// User Reducer
function userReducer(state: UserState, action: StateAction): UserState {
  switch (action.type) {
    case 'SET_USER_PROFILE':
      return {
        ...state,
        data: state.data
          ? {
              ...state.data,
              profile: action.payload,
            }
          : state.data,
      };

    case 'UPDATE_USER_PREFERENCES':
      return {
        ...state,
        data: state.data
          ? {
              ...state.data,
              preferences: {
                ...state.data.preferences,
                ...action.payload,
              },
            }
          : state.data,
      };

    case 'SET_USER_SUBSCRIPTION':
      return {
        ...state,
        data: state.data
          ? {
              ...state.data,
              subscription: action.payload,
            }
          : state.data,
      };

    case 'UPDATE_USER_PERMISSIONS':
      return {
        ...state,
        data: state.data
          ? {
              ...state.data,
              permissions: action.payload,
            }
          : state.data,
      };

    case 'SET_USER_PACKAGE_TIER':
      return {
        ...state,
        data: state.data
          ? {
              ...state.data,
              packageTier: action.payload,
            }
          : state.data,
      };

    case 'SET_USER_LOADING':
      return {
        ...state,
        loading: action.payload,
      };

    case 'SET_USER_ERROR':
      return {
        ...state,
        error: action.payload,
      };

    case 'ADD_OPTIMISTIC_UPDATE':
      return {
        ...state,
        optimisticUpdates: [...state.optimisticUpdates, action.payload],
      };

    case 'CONFIRM_OPTIMISTIC_UPDATE':
      return {
        ...state,
        optimisticUpdates: state.optimisticUpdates.filter(
          update => update.id !== action.payload
        ),
      };

    case 'ROLLBACK_OPTIMISTIC_UPDATE':
      const updateToRollback = state.optimisticUpdates.find(
        update => update.id === action.payload
      );
      if (updateToRollback) {
        updateToRollback.rollback();
      }
      return {
        ...state,
        optimisticUpdates: state.optimisticUpdates.filter(
          update => update.id !== action.payload
        ),
      };

    default:
      return state;
  }
}

// User Provider Props
interface UserProviderProps {
  children: React.ReactNode;
  initialState?: Partial<UserState>;
}

export function UserProvider({ children, initialState }: UserProviderProps) {
  const { state, dispatch } = useContextStore(
    { ...initialUserState, ...initialState },
    {
      persist: {
        key: 'epsx-user-state',
        storage: 'localStorage',
        version: 1,
      },
      middleware: [loggingMiddleware],
      devtools: process.env.NODE_ENV === 'development',
    }
  );

  // User Actions
  const actions = useMemo(
    () => ({
      setProfile: (profile: UserProfile | null) =>
        dispatch(
          {
            type: 'SET_USER_PROFILE',
            payload: profile,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      updatePreferences: (preferences: Partial<UserState['data']['preferences']>) =>
        dispatch(
          {
            type: 'UPDATE_USER_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      setSubscription: (subscription: UserSubscription | null) =>
        dispatch(
          {
            type: 'SET_USER_SUBSCRIPTION',
            payload: subscription,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      updatePermissions: (permissions: string[]) =>
        dispatch(
          {
            type: 'UPDATE_USER_PERMISSIONS',
            payload: permissions,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      setPackageTier: (tier: string) =>
        dispatch(
          {
            type: 'SET_USER_PACKAGE_TIER',
            payload: tier,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      setLoading: (loading: boolean) =>
        dispatch(
          {
            type: 'SET_USER_LOADING',
            payload: loading,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      setError: (error: string | null) =>
        dispatch(
          {
            type: 'SET_USER_ERROR',
            payload: error,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      addOptimisticUpdate: (update: OptimisticUpdate) =>
        dispatch(
          {
            type: 'ADD_OPTIMISTIC_UPDATE',
            payload: update,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      confirmOptimisticUpdate: (id: string) =>
        dispatch(
          {
            type: 'CONFIRM_OPTIMISTIC_UPDATE',
            payload: id,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),

      rollbackOptimisticUpdate: (id: string) =>
        dispatch(
          {
            type: 'ROLLBACK_OPTIMISTIC_UPDATE',
            payload: id,
            meta: { timestamp: Date.now(), source: 'user' },
          },
          userReducer
        ),
    }),
    [dispatch]
  );

  const contextValue = useMemo(
    () => ({
      state,
      actions,
    }),
    [state, actions]
  );

  return (
    <UserContext.Provider value={contextValue}>
      {children}
    </UserContext.Provider>
  );
}

// Custom hooks
export function useUserContext() {
  const context = useContext(UserContext);
  if (context === undefined) {
    throw new Error('useUserContext must be used within a UserProvider');
  }
  return context;
}

export function useUserProfile() {
  const { state, actions } = useUserContext();
  return {
    profile: state.data?.profile,
    setProfile: actions.setProfile,
    isLoading: state.loading,
    error: state.error,
  };
}

export function useUserPreferences() {
  const { state, actions } = useUserContext();
  return {
    preferences: state.data?.preferences,
    updatePreferences: actions.updatePreferences,
  };
}

export function useUserSubscription() {
  const { state, actions } = useUserContext();
  return {
    subscription: state.data?.subscription,
    setSubscription: actions.setSubscription,
    packageTier: state.data?.packageTier,
    setPackageTier: actions.setPackageTier,
  };
}

export function useUserPermissions() {
  const { state, actions } = useUserContext();
  return {
    permissions: state.data?.permissions || [],
    updatePermissions: actions.updatePermissions,
  };
}

export function useOptimisticUpdates() {
  const { state, actions } = useUserContext();
  return {
    optimisticUpdates: state.optimisticUpdates,
    addOptimisticUpdate: actions.addOptimisticUpdate,
    confirmOptimisticUpdate: actions.confirmOptimisticUpdate,
    rollbackOptimisticUpdate: actions.rollbackOptimisticUpdate,
  };
}