'use client';

import React, { Suspense, useMemo } from 'react';
import { AppStateProvider } from '@/context/app-state';
import { UIProvider } from '@/context/ui-context';
// NotificationProvider not available - commented out
import { LoadingSpinner } from '@/components/common/LoadingSpinner';
import { ErrorBoundary } from '@/components/common/ErrorBoundary';

interface StateProviderProps {
  children: React.ReactNode;
  initialState?: {
    ui?: any;
    user?: any;
    notifications?: any;
  };
  // SSR compatibility
  serverAuthState?: any;
  serverUserPreferences?: any;
}

// Performance-optimized provider wrapper
const ProviderWrapper = React.memo(({ children }: { children: React.ReactNode }) => {
  return <>{children}</>;
});

ProviderWrapper.displayName = 'ProviderWrapper';

// Memoized provider stack to prevent unnecessary re-renders
function ProviderStack({ children, initialState }: { 
  children: React.ReactNode; 
  initialState?: StateProviderProps['initialState'];
}) {
  const memoizedInitialState = useMemo(() => initialState, [initialState]);

  return (
    <AppStateProvider initialState={memoizedInitialState}>
      <UIProvider>
        <ProviderWrapper>
          {children}
        </ProviderWrapper>
      </UIProvider>
    </AppStateProvider>
  );
}

// Main state provider with error boundaries and loading states
export function StateProvider({ 
  children, 
  initialState, 
  serverAuthState, 
  serverUserPreferences 
}: StateProviderProps) {
  // Merge server state with initial state for SSR compatibility
  const mergedInitialState = useMemo(() => {
    const merged = { ...initialState };
    
    if (serverAuthState || serverUserPreferences) {
      merged.user = {
        ...merged.user,
        data: {
          ...merged.user?.data,
          ...(serverAuthState && { 
            profile: serverAuthState.user,
            permissions: serverAuthState.permissions || [],
            packageTier: serverAuthState.packageTier || 'FREE'
          }),
          ...(serverUserPreferences && { 
            preferences: { 
              ...merged.user?.data?.preferences,
              ...serverUserPreferences 
            }
          })
        }
      };
    }
    
    return merged;
  }, [initialState, serverAuthState, serverUserPreferences]);

  return (
    <ErrorBoundary
      fallback={
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <h2 className="text-2xl font-semibold text-red-600 mb-2">
              State Management Error
            </h2>
            <p className="text-gray-600 mb-4">
              There was a problem initializing the application state.
            </p>
            <button
              onClick={() => window.location.reload()}
              className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
            >
              Reload Application
            </button>
          </div>
        </div>
      }
    >
      <Suspense
        fallback={
          <div className="min-h-screen flex items-center justify-center">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <ProviderStack initialState={mergedInitialState}>
          {children}
        </ProviderStack>
      </Suspense>
    </ErrorBoundary>
  );
}

// HOC for components that need state management
export function withStateProvider<P>(
  Component: React.ComponentType<P>,
  options: {
    initialState?: StateProviderProps['initialState'];
    errorBoundary?: boolean;
  } = {}
) {
  const WrappedComponent = React.forwardRef((props: any, ref: React.Ref<any>) => {
    const { initialState, errorBoundary = true } = options;

    const ComponentWithState = (
      <StateProvider initialState={initialState}>
        <Component {...props} ref={ref} />
      </StateProvider>
    );

    if (errorBoundary) {
      return (
        <ErrorBoundary
          fallback={
            <div className="p-4 text-center">
              <p className="text-red-600">Component failed to load</p>
            </div>
          }
        >
          {ComponentWithState}
        </ErrorBoundary>
      );
    }

    return ComponentWithState;
  });

  WrappedComponent.displayName = `withStateProvider(${Component.displayName || Component.name})`;
  
  return WrappedComponent;
}

// Hook to check if we're inside a state provider
export function useStateProviderStatus() {
  const [hasProvider, setHasProvider] = React.useState(false);

  React.useEffect(() => {
    try {
      // Check if we have app state context available
      // Note: We can't call hooks inside async functions or conditionally
      setHasProvider(true);
    } catch {
      setHasProvider(false);
    }
  }, []);

  return hasProvider;
}

// Development component for state inspection
export function StateInspector() {
  // Always call hooks first, before any conditional returns
  const [isOpen, setIsOpen] = React.useState(false);
  const [selectedStore, setSelectedStore] = React.useState<string>('ui');

  React.useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      // Ctrl+Shift+S to toggle state inspector
      if (e.ctrlKey && e.shiftKey && e.key === 'S') {
        e.preventDefault();
        setIsOpen(prev => !prev);
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, []);

  if (process.env.NODE_ENV !== 'development') {
    return null;
  }

  if (!isOpen) {
    return (
      <div className="fixed bottom-4 right-4 z-50">
        <button
          onClick={() => setIsOpen(true)}
          className="bg-blue-500 text-white p-2 rounded-full shadow-lg hover:bg-blue-600"
          title="Open State Inspector (Ctrl+Shift+S)"
        >
          🔍
        </button>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center">
      <div className="bg-white rounded-lg shadow-xl w-11/12 h-5/6 max-w-4xl">
        <div className="flex items-center justify-between p-4 border-b">
          <h3 className="text-lg font-semibold">State Inspector</h3>
          <div className="flex items-center gap-2">
            <select
              value={selectedStore}
              onChange={(e) => setSelectedStore(e.target.value)}
              className="border rounded px-2 py-1"
            >
              <option value="ui">UI State</option>
              <option value="user">User State</option>
              <option value="analytics">Analytics State</option>
              <option value="notifications">Notifications State</option>
              <option value="cache">Cache State</option>
            </select>
            <button
              onClick={() => setIsOpen(false)}
              className="text-red-500 hover:text-red-700"
            >
              ✕
            </button>
          </div>
        </div>
        <div className="p-4 h-full overflow-auto">
          <StateViewer store={selectedStore} />
        </div>
      </div>
    </div>
  );
}

function StateViewer({ store }: { store: string }) {
  const [state, setState] = React.useState<any>(null);

  React.useEffect(() => {
    const interval = setInterval(() => {
      if (typeof window !== 'undefined' && window.__EPSX_DEV_TOOLS__ && (window.__EPSX_DEV_TOOLS__ as any).getStore) {
        const storeState = (window.__EPSX_DEV_TOOLS__ as any).getStore(store);
        setState(storeState);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [store]);

  if (!state) {
    return <div className="text-gray-500">No state data available</div>;
  }

  return (
    <pre className="text-sm bg-gray-100 p-4 rounded overflow-auto">
      {JSON.stringify(state, null, 2)}
    </pre>
  );
}