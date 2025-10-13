// ============================================================================
// SHARED UNIFIED API CLIENT COMPONENT
// React hooks for making API requests with unified response handling
// ============================================================================

/**
 * CORE PRINCIPLES:
 * - Same API client standard for both apps
 * - Unified response format handling
 * - Bearer token authentication handled automatically
 * - Display helpers based on backend responses
 * - No permission logic in components
 */

'use client';

import React, { useState, useEffect, useCallback } from 'react';
import { useSharedAuth } from '../auth/Provider';
import { UnifiedApiResponse } from '../../auth/client';

// API request state
interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  success: boolean;
  meta?: any;
}

// Hook for making API requests
export function useUnifiedApiRequest<T>(
  endpoint: string,
  options?: RequestInit,
  dependencies: any[] = []
) {
  const { makeApiRequest } = useSharedAuth();
  const [state, setState] = useState<ApiState<T>>({
    data: null,
    loading: true,
    error: null,
    success: false,
  });

  const executeRequest = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const response = await makeApiRequest(endpoint, options);
      
      if (response.success) {
        setState({
          data: response.data || null,
          loading: false,
          error: null,
          success: true,
          meta: response.meta,
        });
      } else {
        setState({
          data: null,
          loading: false,
          error: response.error?.message || 'Request failed',
          success: false,
          meta: response.meta,
        });
      }
    } catch (error) {
      setState({
        data: null,
        loading: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        success: false,
      });
    }
  }, [endpoint, makeApiRequest, JSON.stringify(options)]);

  // Execute request on mount and when dependencies change
  useEffect(() => {
    executeRequest();
  }, [executeRequest, ...dependencies]);

  return {
    ...state,
    refetch: executeRequest,
  };
}

// Hook for making manual API requests (for mutations)
export function useUnifiedApiMutation<TRequest, TResponse>() {
  const { makeApiRequest } = useSharedAuth();
  const [state, setState] = useState<ApiState<TResponse>>({
    data: null,
    loading: false,
    error: null,
    success: false,
  });

  const execute = useCallback(async (
    endpoint: string,
    request?: TRequest,
    options?: RequestInit
  ) => {
    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const requestOptions = {
        ...options,
        method: options?.method || 'POST',
        body: request ? JSON.stringify(request) : options?.body,
      };

      const response = await makeApiRequest(endpoint, requestOptions);
      
      if (response.success) {
        setState({
          data: response.data || null,
          loading: false,
          error: null,
          success: true,
          meta: response.meta,
        });
        return { success: true, data: response.data };
      } else {
        const errorMessage = response.error?.message || 'Request failed';
        setState({
          data: null,
          loading: false,
          error: errorMessage,
          success: false,
          meta: response.meta,
        });
        return { success: false, error: errorMessage };
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      setState({
        data: null,
        loading: false,
        error: errorMessage,
        success: false,
      });
      return { success: false, error: errorMessage };
    }
  }, [makeApiRequest]);

  return {
    ...state,
    execute,
    reset: () => setState({
      data: null,
      loading: false,
      error: null,
      success: false,
    }),
  };
}

// Component for displaying API response data
export function ApiDataDisplay<T>({ 
  endpoint,
  options,
  dependencies = [],
  children,
  loadingComponent,
  errorComponent,
  emptyComponent
}: {
  endpoint: string;
  options?: RequestInit;
  dependencies?: any[];
  children: (data: T, meta?: any) => React.ReactNode;
  loadingComponent?: React.ReactNode;
  errorComponent?: (error: string, refetch: () => void) => React.ReactNode;
  emptyComponent?: React.ReactNode;
}) {
  const { data, loading, error, meta, refetch } = useUnifiedApiRequest<T>(
    endpoint,
    options,
    dependencies
  );

  if (loading) {
    return loadingComponent || (
      <div className="flex items-center justify-center p-4">
        <div className="flex items-center space-x-2">
          <svg className="animate-spin h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span className="text-sm text-gray-600">Loading...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return errorComponent ? errorComponent(error, refetch) : (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex">
          <div className="flex-shrink-0">
            <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
            </svg>
          </div>
          <div className="ml-3 flex-1">
            <h3 className="text-sm font-medium text-red-800">Error</h3>
            <p className="text-sm text-red-700 mt-1">{error}</p>
            <button
              onClick={refetch}
              className="mt-2 text-sm bg-red-100 hover:bg-red-200 text-red-800 px-3 py-1 rounded"
            >
              Try Again
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (!data) {
    return emptyComponent || (
      <div className="text-center p-4">
        <p className="text-sm text-gray-500">No data available</p>
      </div>
    );
  }

  return <>{children(data, meta)}</>;
}

// Component for rendering action buttons based on backend permissions
export function ApiActionButton({ 
  action,
  endpoint,
  request,
  method = 'POST',
  children,
  onSuccess,
  onError,
  className = "",
  disabled = false
}: {
  action: string;
  endpoint: string;
  request?: any;
  method?: string;
  children: React.ReactNode;
  onSuccess?: (data: any) => void;
  onError?: (error: string) => void;
  className?: string;
  disabled?: boolean;
}) {
  const { execute, loading, error } = useUnifiedApiMutation();
  const { user } = useSharedAuth();

  // Check if action is available based on backend meta (display helper only)
  const isActionAvailable = user?.permissions.some(p => 
    p.includes(action) || p.startsWith('admin:')
  ) || false;

  const handleClick = async () => {
    const result = await execute(endpoint, request, { method });
    
    if (result.success) {
      onSuccess?.(result.data);
    } else {
      onError?.(result.error || 'Action failed');
    }
  };

  // Don't render button if action not available (display helper only)
  if (!isActionAvailable) {
    return null;
  }

  return (
    <button
      onClick={handleClick}
      disabled={disabled || loading}
      className={`inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 ${className}`}
    >
      {loading ? (
        <>
          <svg className="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Loading...
        </>
      ) : (
        children
      )}
    </button>
  );
}

// Component for displaying permission-based content
export function PermissionBasedDisplay({ 
  requiredPermission,
  children,
  fallback,
  showReason = false
}: {
  requiredPermission: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
  showReason?: boolean;
}) {
  const { user, hasPermissionForDisplay } = useSharedAuth();

  // This is for display only - backend makes authorization decisions
  const hasPermission = hasPermissionForDisplay(requiredPermission);

  if (hasPermission) {
    return <>{children}</>;
  }

  if (fallback) {
    return <>{fallback}</>;
  }

  if (showReason) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
        <p className="text-sm text-yellow-800">
          Access restricted. Required permission: {requiredPermission}
        </p>
        <p className="text-xs text-yellow-600 mt-1">
          Current tier: {user?.tier_level || 'none'}
        </p>
      </div>
    );
  }

  return null;
}