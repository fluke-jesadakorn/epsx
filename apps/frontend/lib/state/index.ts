// State Management System - Main Export File
// Provides a comprehensive state management solution for the EPSX analytics platform

// Core types and utilities
export * from './types';
export * from './core';
export * from './middleware';
export * from './ssr';

// Context providers
export { AppStateProvider, useAppState } from '@/context/app-state';
export { UIProvider, useUI } from '@/context/ui-context';
export { NotificationProvider, useNotifications } from '@/context/notification-context';

// Enhanced state hooks
export * from '@/hooks/state/useAsyncAction';
export * from '@/hooks/state/useOptimisticState';
export * from '@/hooks/state/usePersistentState';
export * from '@/hooks/state/useStateSelector';

// State provider components
export { 
  StateProvider, 
  withStateProvider,
  StateInspector,
  useStateProviderStatus
} from '@/components/state/StateProvider';
export { 
  AppLayout, 
  withAppLayout 
} from '@/components/layout/AppLayout';

// Usage example and quick start guide
export const StateManagementGuide = {
  quickStart: `
    // 1. Wrap your app with StateProvider
    import { StateProvider } from '@/lib/state';
    
    function App() {
      return (
        <StateProvider>
          <YourAppContent />
        </StateProvider>
      );
    }
    
    // 2. Use specialized hooks in components
    import { useUI, useNotifications } from '@/lib/state';
    
    function AnalyticsDashboard() {
      const { addToast } = useUI();
      const { notifications, markRead } = useNotifications();
      
      // Your component logic here
    }
  `,
  
  optimisticUpdates: `
    // Using optimistic updates for better UX
    import { useOptimisticList } from '@/lib/state';
    
    function AnalyticsListComponent() {
      const { list, add, remove } = useOptimisticList(initialAnalytics);
      
      const handleAddAnalytics = async (item) => {
        await add(item, () => api.addToAnalytics(item.id));
      };
    }
  `,
  
  persistence: `
    // Persistent state with localStorage/sessionStorage
    import { usePersistentState, useUserPreferences } from '@/lib/state';
    
    function UserSettings() {
      const [settings, setSettings] = usePersistentState({
        key: 'user-settings',
        defaultValue: { theme: 'dark', currency: 'USD' }
      });
      
      const [preferences] = useUserPreferences();
    }
  `,
  
  selectors: `
    // Performance-optimized selectors
    import { useAnalyticsSelector, useMetricsSelector } from '@/lib/state';
    
    function AnalyticsSummary() {
      const analytics = useAnalyticsSelector();
      const metrics = useMetricsSelector();
      
      // Component only re-renders when these specific values change
    }
  `,
  
  middleware: `
    // Custom middleware for analytics/logging
    import { StateMiddleware, combineMiddleware } from '@/lib/state';
    
    const customMiddleware: StateMiddleware = (action, prev, next, store) => {
      // Your custom logic here
      // State action dispatched (debug logging removed)
    };
    
    // Apply middleware in StateProvider
    <StateProvider middleware={[customMiddleware]}>
      <App />
    </StateProvider>
  `,
  
  ssr: `
    // SSR-compatible state hydration
    import { getServerState } from '@/lib/state';
    
    // In your page component or layout
    export async function generateMetadata() {
      const serverState = await getServerState({
        includeAuth: true,
        includeUserPreferences: true
      });
      
      return (
        <StateProvider initialState={serverState}>
          <PageContent />
        </StateProvider>
      );
    }
  `
};

// Performance tips
export const PerformanceTips = {
  selectors: "Use specific selectors instead of accessing full state to minimize re-renders",
  memoization: "Components using state should be wrapped with React.memo() when appropriate",
  optimistic: "Use optimistic updates for actions that are likely to succeed (add/remove items)",
  persistence: "Only persist necessary data to avoid localStorage bloat",
  middleware: "Keep middleware lightweight to avoid impacting state update performance",
  ssr: "Pre-populate critical state on server to reduce loading states on client"
};

// Type-safe action creators
export const createActions = {
  ui: (dispatch: any) => ({
    setTheme: (theme: 'light' | 'dark' | 'system') => 
      dispatch({ type: 'SET_THEME', payload: theme }),
    addToast: (toast: any) => 
      dispatch({ type: 'ADD_TOAST', payload: toast }),
    openModal: (key: string, data?: any) => 
      dispatch({ type: 'OPEN_MODAL', payload: { key, data } })
  }),
  
  analytics: (dispatch: any) => ({
    addToAnalytics: (item: any) => 
      dispatch({ type: 'ADD_TO_ANALYTICS', payload: item }),
    updateMetrics: (id: string, data: any) => 
      dispatch({ type: 'UPDATE_METRICS', payload: { id, data } })
  }),
  
  notifications: (dispatch: any) => ({
    addNotification: (notification: any) => 
      dispatch({ type: 'ADD_NOTIFICATION', payload: notification }),
    markRead: (id: string) => 
      dispatch({ type: 'MARK_READ', payload: id })
  })
};

// State validation utilities
export const validateState = {
  ui: (state: any) => {
    return state && 
           typeof state.theme === 'string' &&
           typeof state.sidebar === 'object' &&
           Array.isArray(state.toasts);
  },
  
  user: (state: any) => {
    return state && 
           (state.data === null || typeof state.data === 'object') &&
           typeof state.loading === 'boolean' &&
           Array.isArray(state.optimisticUpdates);
  },
  
  analytics: (state: any) => {
    return state &&
           state.data &&
           Array.isArray(state.data.rankings) &&
           Array.isArray(state.data.metrics) &&
           typeof state.filters === 'object';
  }
};

// Default export for convenience - Fixed imports
export default {
  StateProvider: StateProvider,
  useAppState: useAppState,
  useUI: useUI,
  useNotifications: useNotifications,
  withStateProvider,
  AppLayout: AppLayout,
  StateManagementGuide,
  PerformanceTips
};