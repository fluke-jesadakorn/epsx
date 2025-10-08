import { useState, useCallback, useMemo } from 'react';
import { createAdminApiClient, ApiResponse } from '@/shared/utils/api-client';
import { ApiError } from '@/shared/utils/response-handler';

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
        setError(response as any);
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
    return apiCall<any>(() => client.get('/api/admin/users', params));
  }, [apiCall, client]);

  const getUser = useCallback((userId: string) => {
    return apiCall<any>(() => client.get(`/api/admin/users/${userId}`));
  }, [apiCall, client]);

  const createUser = useCallback((userData: {
    email: string;
    wallet_address?: string;
    role?: string;
  }) => {
    return apiCall<any>(() => client.post('/api/admin/users', userData));
  }, [apiCall, client]);

  const updateUser = useCallback((userId: string, userData: {
    email?: string;
    role?: string;
  }) => {
    return apiCall<any>(() => client.put(`/api/admin/users/${userId}`, userData));
  }, [apiCall, client]);

  const deleteUser = useCallback((userId: string) => {
    return apiCall<any>(() => client.delete(`/api/admin/users/${userId}`));
  }, [apiCall, client]);

  const grantPermission = useCallback((request: {
    user_id: string;
    permission: string;
    expires_at?: string;
    reason?: string;
  }) => {
    return apiCall<any>(() => client.post('/api/admin/permissions/grant', request));
  }, [apiCall, client]);

  const revokePermission = useCallback((request: {
    user_id: string;
    permission: string;
    reason?: string;
  }) => {
    return apiCall<any>(() => client.post('/api/admin/permissions/revoke', request));
  }, [apiCall, client]);

  const getUserPermissions = useCallback((userId: string) => {
    return apiCall<any>(() => client.get(`/api/admin/users/${userId}/permissions`));
  }, [apiCall, client]);

  const getSystemHealth = useCallback(() => {
    return apiCall<any>(() => client.get('/api/admin/system/health'));
  }, [apiCall, client]);

  const getSystemMetrics = useCallback(() => {
    return apiCall<any>(() => client.get('/api/admin/system/metrics'));
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