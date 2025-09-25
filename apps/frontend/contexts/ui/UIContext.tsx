'use client';

import React, { createContext, useContext, useMemo } from 'react';
import { useContextStore, createAsyncState, loggingMiddleware } from '@/lib/state';
import type { UIState, StateAction } from '@/lib/state/types';
import { uiLogger } from '@/lib/utils/logging';

// Initial UI state
const initialUIState: UIState = {
  theme: 'system',
  sidebar: {
    open: true,
    collapsed: false,
  },
  modals: {},
  toasts: [],
  loading: {
    global: false,
    requests: {},
  },
  responsive: {
    isMobile: false,
    isTablet: false,
    breakpoint: 'lg',
  },
};

// UI Context interface
interface UIContextType {
  state: UIState;
  actions: {
    setTheme: (theme: UIState['theme']) => void;
    toggleSidebar: () => void;
    collapseSidebar: (collapsed: boolean) => void;
    openModal: (key: string, data?: unknown) => void;
    closeModal: (key: string) => void;
    addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) => void;
    removeToast: (id: string) => void;
    setLoading: (key: string | null, loading: boolean) => void;
    setResponsive: (responsive: Partial<UIState['responsive']>) => void;
  };
}

const UIContext = createContext<UIContextType | undefined>(undefined);

// UI Reducer
function uiReducer(state: UIState, action: StateAction): UIState {
  switch (action.type) {
    case 'SET_THEME':
      return { ...state, theme: action.payload as UIState['theme'] };

    case 'TOGGLE_SIDEBAR':
      return {
        ...state,
        sidebar: { ...state.sidebar, open: !state.sidebar.open },
      };

    case 'COLLAPSE_SIDEBAR':
      return {
        ...state,
        sidebar: { ...state.sidebar, collapsed: action.payload as boolean },
      };

    case 'OPEN_MODAL': {
      const payload = action.payload as { key: string; data?: any };
      return {
        ...state,
        modals: {
          ...state.modals,
          [payload.key]: {
            open: true,
            data: payload.data,
          },
        },
      };
    }

    case 'CLOSE_MODAL':
      return {
        ...state,
        modals: {
          ...state.modals,
          [action.payload as string]: { open: false },
        },
      };

    case 'ADD_TOAST':
      const newToast = {
        ...(action.payload as any),
        id: Math.random().toString(36).substr(2, 9),
        timestamp: Date.now(),
      };
      return {
        ...state,
        toasts: [...state.toasts, newToast],
      };

    case 'REMOVE_TOAST':
      return {
        ...state,
        toasts: state.toasts.filter(toast => toast.id !== action.payload),
      };

    case 'SET_LOADING': {
      const payload = action.payload as { key?: string; loading: boolean };
      return {
        ...state,
        loading: payload.key
          ? {
              ...state.loading,
              requests: {
                ...state.loading.requests,
                [payload.key]: payload.loading,
              },
            }
          : {
              ...state.loading,
              global: payload.loading,
            },
      };
    }

    case 'SET_RESPONSIVE':
      return {
        ...state,
        responsive: { ...state.responsive, ...(action.payload as any) },
      };

    default:
      return state;
  }
}

// UI Provider Props
interface UIProviderProps {
  children: React.ReactNode;
  initialState?: Partial<UIState>;
}

export function UIProvider({ children, initialState }: UIProviderProps) {
  const { state, dispatch } = useContextStore(
    { ...initialUIState, ...initialState },
    {
      persist: {
        key: 'epsx-ui-state',
        storage: 'localStorage',
        version: 1,
      },
      middleware: [loggingMiddleware],
      devtools: process.env.NODE_ENV === 'development',
    }
  );

  // UI Actions
  const actions = useMemo(
    () => ({
      setTheme: (theme: UIState['theme']) =>
        dispatch({
            type: 'SET_THEME',
            payload: theme,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      toggleSidebar: () =>
        dispatch({
            type: 'TOGGLE_SIDEBAR',
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      collapseSidebar: (collapsed: boolean) =>
        dispatch({
            type: 'COLLAPSE_SIDEBAR',
            payload: collapsed,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      openModal: (key: string, data?: unknown) =>
        dispatch({
            type: 'OPEN_MODAL',
            payload: { key, data },
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      closeModal: (key: string) =>
        dispatch({
            type: 'CLOSE_MODAL',
            payload: key,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      addToast: (toast: Omit<UIState['toasts'][0], 'id' | 'timestamp'>) =>
        dispatch({
            type: 'ADD_TOAST',
            payload: toast,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      removeToast: (id: string) =>
        dispatch({
            type: 'REMOVE_TOAST',
            payload: id,
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      setLoading: (key: string | null, loading: boolean) =>
        dispatch({
            type: 'SET_LOADING',
            payload: { key, loading },
            meta: { timestamp: Date.now(), source: 'ui' },
          }),

      setResponsive: (responsive: Partial<UIState['responsive']>) =>
        dispatch({
            type: 'SET_RESPONSIVE',
            payload: responsive,
            meta: { timestamp: Date.now(), source: 'ui' },
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
    <UIContext.Provider value={contextValue}>
      {children}
    </UIContext.Provider>
  );
}

// Custom hooks
export function useUIContext() {
  const context = useContext(UIContext);
  if (context === undefined) {
    throw new Error('useUIContext must be used within a UIProvider');
  }
  return context;
}

export function useTheme() {
  const { state, actions } = useUIContext();
  return {
    theme: state.theme,
    setTheme: actions.setTheme,
  };
}

export function useSidebar() {
  const { state, actions } = useUIContext();
  return {
    sidebar: state.sidebar,
    toggleSidebar: actions.toggleSidebar,
    collapseSidebar: actions.collapseSidebar,
  };
}

export function useModals() {
  const { state, actions } = useUIContext();
  return {
    modals: state.modals,
    openModal: actions.openModal,
    closeModal: actions.closeModal,
  };
}

export function useToasts() {
  const { state, actions } = useUIContext();
  return {
    toasts: state.toasts,
    addToast: actions.addToast,
    removeToast: actions.removeToast,
  };
}

export function useLoading() {
  const { state, actions } = useUIContext();
  return {
    loading: state.loading,
    setLoading: actions.setLoading,
  };
}

export function useResponsive() {
  const { state, actions } = useUIContext();
  return {
    responsive: state.responsive,
    setResponsive: actions.setResponsive,
  };
}