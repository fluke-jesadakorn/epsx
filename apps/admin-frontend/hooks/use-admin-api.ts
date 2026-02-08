import { useCallback, useMemo, useState } from 'react';

import { API_ROUTES } from '@/shared/config/route-constants';
import { createAdminApiClient, type ApiResponse } from '@/shared/utils/api-client';
import { type ApiError } from '@/shared/utils/response-handler';

/**
 * Hook for making admin API calls with error handling
 * Uses shared API client with loading state and error management
 */
export function useAdminApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ApiError | null>(null);

  // Create client instance (memoized)
  const client = useMemo(() => createAdminApiClient(), []);

  const clearError = useCallback(() => {
    setError(null);
  }, []);

  const apiCall = useCallback(async <T>(
    operation: () => Promise<ApiResponse<T>>
  ): Promise<T | null> => {
    setLoading(true);
    setError(null);

    try {
      const response = await operation();

      if (response.success) {
        return response.data ?? null;
      } else {
        // Handle API error
        setError(response as unknown as ApiError);
        return null;
      }
    } catch (err) {
      // Handle network or other errors
      const apiError: ApiError = {
        success: false,
        error: {
          type: 'NETWORK_ERROR',
          code: 'NETWORK_ERROR',
          message: err instanceof Error ? err.message : 'Network request failed',
          user_message: 'Unable to connect to the server. Please check your connection.',
          suggested_actions: ['Check your internet connection', 'Try again in a moment']
        }
      };
      setError(apiError);
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  // Specific API methods with error handling
  const getUsers = useCallback((params?: {
    page?: number;
    per_page?: number;
    search?: string;
    role?: string;
  }) => {
    return apiCall<unknown>(() => client.get(API_ROUTES.ADMIN.USERS, params));
  }, [apiCall, client]);

  const getUser = useCallback((userId: string) => {
    return apiCall<unknown>(() => client.get(API_ROUTES.ADMIN.USER_DETAILS.replace(':id', userId)));
  }, [apiCall, client]);

  const createUser = useCallback((userData: {
    email: string;
    wallet_address?: string;
    role?: string;
  }) => {
    return apiCall<unknown>(() => client.post(API_ROUTES.ADMIN.USERS, userData));
  }, [apiCall, client]);

  const updateUser = useCallback((userId: string, userData: {
    email?: string;
    role?: string;
  }) => {
    return apiCall<unknown>(() => client.put(API_ROUTES.ADMIN.USER_DETAILS.replace(':id', userId), userData));
  }, [apiCall, client]);

  const deleteUser = useCallback((userId: string) => {
    return apiCall<unknown>(() => client.delete(API_ROUTES.ADMIN.USER_DETAILS.replace(':id', userId)));
  }, [apiCall, client]);

  const grantPermission = useCallback((request: {
    user_id: string;
    permission: string;
    expires_at?: string;
    reason?: string;
  }) => {
    return apiCall<unknown>(() => client.post(API_ROUTES.PERMISSIONS.GRANT, request));
  }, [apiCall, client]);

  const revokePermission = useCallback((request: {
    user_id: string;
    permission: string;
    reason?: string;
  }) => {
    return apiCall<unknown>(() => client.post(API_ROUTES.PERMISSIONS.REVOKE, request));
  }, [apiCall, client]);

  const getUserPermissions = useCallback((userId: string) => {
    return apiCall<unknown>(() => client.get(API_ROUTES.ADMIN.USER_PERMISSIONS.replace(':id', userId)));
  }, [apiCall, client]);

  const getSystemHealth = useCallback(() => {
    return apiCall<unknown>(() => client.get(API_ROUTES.ADMIN.SYSTEM_STATUS));
  }, [apiCall, client]);

  const getSystemMetrics = useCallback(() => {
    return apiCall<unknown>(() => client.get(API_ROUTES.ADMIN.SYSTEM_METRICS));
  }, [apiCall, client]);

  return {
    // State
    loading,
    error,

    // Actions
    clearError,
    apiCall,

    // Specific API methods
    getUsers,
    getUser,
    createUser,
    updateUser,
    deleteUser,
    grantPermission,
    revokePermission,
    getUserPermissions,
    getSystemHealth,
    getSystemMetrics
  };
}

export default useAdminApi;