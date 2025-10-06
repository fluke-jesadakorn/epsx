'use client';

import React, { createContext, useContext, useMemo } from 'react';
import { useContextStore, createAsyncState, loggingMiddleware } from '@/lib/state';
import type { 
  UserState, 
  StateAction, 
  UserProfile, 
  UserSubscription, 
  OptimisticUpdate 
} from '@/lib/state/types';
import { authLogger } from '@/lib/utils/logging';

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
    permissionGroup: 'Basic Access Group',
  },
  optimisticUpdates: [],
};

// User Context interface
interface UserContextType {
  state: UserState;
  actions: {
    setProfile: (profile: UserProfile | null) => void;
    updatePreferences: (preferences: Record<string, any>) => void;
    setSubscription: (subscription: UserSubscription | null) => void;
    updatePermissions: (permissions: string[]) => void;
    setPermissionGroup: (group: string) => void;
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
  const userState = state as any; // Cast for AsyncState access
  switch (action.type) {
    case 'SET_USER_PROFILE':
      return {
        ...state,
        data: (state as any).data
          ? {
              ...(state as any).data,
              profile: action.payload,
            }
          : (state as any).data,
      } as UserState;

    case 'UPDATE_USER_PREFERENCES':
      return {
        ...state,
        data: (state as any).data
          ? {
              ...(state as any).data,
              preferences: {
                ...(state as any).data.preferences,
                ...(action.payload as any),
              },
            }
          : (state as any).data,
      } as UserState;

    case 'SET_USER_SUBSCRIPTION':
      return {
        ...state,
        data: (state as any).data
          ? {
              ...(state as any).data,
              subscription: action.payload,
            }
          : (state as any).data,
      } as UserState;

    case 'UPDATE_USER_PERMISSIONS':
      return {
        ...state,
        data: (state as any).data
          ? {
              ...(state as any).data,
              permissions: action.payload,
            }
          : (state as any).data,
      } as UserState;


    case 'SET_USER_PERMISSION_GROUP':
      return {
        ...state,
        data: (state as any).data
          ? {
              ...(state as any).data,
              permissionGroup: action.payload,
            }
          : (state as any).data,
      };

    case 'SET_USER_LOADING':
      return {
        ...state,
        loading: action.payload as boolean,
      };

    case 'SET_USER_ERROR':
      return {
        ...state,
        error: action.payload as string | null,
      };

    case 'ADD_OPTIMISTIC_UPDATE':
      return {
        ...state,
        optimisticUpdates: [...userState.optimisticUpdates, action.payload],
      };

    case 'CONFIRM_OPTIMISTIC_UPDATE':
      return {
        ...state,
        optimisticUpdates: userState.optimisticUpdates.filter(
          (update: any) => update.id !== action.payload
        ),
      } as UserState;

    case 'ROLLBACK_OPTIMISTIC_UPDATE': {
      const updateToRollback = userState.optimisticUpdates.find(
        (update: any) => update.id === action.payload
      );
      if (updateToRollback) {
        updateToRollback.rollback();
      }
      return {
        ...state,
        optimisticUpdates: userState.optimisticUpdates.filter(
          (update: any) => update.id !== action.payload
        ),
      } as UserState;
    }

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
        dispatch({
            type: 'SET_USER_PROFILE',
            payload: profile,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      updatePreferences: (preferences: Record<string, any>) =>
        dispatch({
            type: 'UPDATE_USER_PREFERENCES',
            payload: preferences,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      setSubscription: (subscription: UserSubscription | null) =>
        dispatch({
            type: 'SET_USER_SUBSCRIPTION',
            payload: subscription,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      updatePermissions: (permissions: string[]) =>
        dispatch({
            type: 'UPDATE_USER_PERMISSIONS',
            payload: permissions,
            meta: { timestamp: Date.now(), source: 'user' },
          }),


      setPermissionGroup: (group: string) =>
        dispatch({
            type: 'SET_USER_PERMISSION_GROUP',
            payload: group,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      setLoading: (loading: boolean) =>
        dispatch({
            type: 'SET_USER_LOADING',
            payload: loading,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      setError: (error: string | null) =>
        dispatch({
            type: 'SET_USER_ERROR',
            payload: error,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      addOptimisticUpdate: (update: OptimisticUpdate) =>
        dispatch({
            type: 'ADD_OPTIMISTIC_UPDATE',
            payload: update,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      confirmOptimisticUpdate: (id: string) =>
        dispatch({
            type: 'CONFIRM_OPTIMISTIC_UPDATE',
            payload: id,
            meta: { timestamp: Date.now(), source: 'user' },
          }),

      rollbackOptimisticUpdate: (id: string) =>
        dispatch({
            type: 'ROLLBACK_OPTIMISTIC_UPDATE',
            payload: id,
            meta: { timestamp: Date.now(), source: 'user' },
          }),
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
    profile: (state as any).data?.profile,
    setProfile: actions.setProfile,
    isLoading: (state as any).loading,
    error: (state as any).error,
  };
}

export function useUserPreferences() {
  const { state, actions } = useUserContext();
  return {
    preferences: (state as any).data?.preferences,
    updatePreferences: actions.updatePreferences,
  };
}

export function useUserSubscription() {
  const { state, actions } = useUserContext();
  return {
    subscription: (state as any).data?.subscription,
    setSubscription: actions.setSubscription,
    permissionGroup: (state as any).data?.permissionGroup,
    setPermissionGroup: actions.setPermissionGroup,
  };
}

export function useUserPermissions() {
  const { state, actions } = useUserContext();
  return {
    permissions: (state as any).data?.permissions || [],
    updatePermissions: actions.updatePermissions,
  };
}

export function useOptimisticUpdates() {
  const { state, actions } = useUserContext();
  return {
    optimisticUpdates: (state as any).optimisticUpdates,
    addOptimisticUpdate: actions.addOptimisticUpdate,
    confirmOptimisticUpdate: actions.confirmOptimisticUpdate,
    rollbackOptimisticUpdate: actions.rollbackOptimisticUpdate,
  };
}