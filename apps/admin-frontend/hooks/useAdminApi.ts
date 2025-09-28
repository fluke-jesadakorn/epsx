import { useState, useCallback } from 'react';
import { adminApiClient } from '@/lib/api/simple-api-client';
import { ApiResponse, ApiError } from '@/lib/api/response-handler';

/**
 * Hook for making admin API calls with error handling
 * Provides loading state and error management for backend-only permission validation
 */
export function useAdminApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ApiError | null>(null);

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
        return response.data;
      } else {
        // Handle API error
        setError(response);
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
    return apiCall(() => adminApiClient.getUsers(params));
  }, [apiCall]);

  const getUser = useCallback((userId: string) => {
    return apiCall(() => adminApiClient.getUser(userId));
  }, [apiCall]);

  const createUser = useCallback((userData: {
    email: string;
    wallet_address?: string;
    role?: string;
  }) => {
    return apiCall(() => adminApiClient.createUser(userData));
  }, [apiCall]);

  const updateUser = useCallback((userId: string, userData: {
    email?: string;
    role?: string;
  }) => {
    return apiCall(() => adminApiClient.updateUser(userId, userData));
  }, [apiCall]);

  const deleteUser = useCallback((userId: string) => {
    return apiCall(() => adminApiClient.deleteUser(userId));
  }, [apiCall]);

  const grantPermission = useCallback((request: {
    user_id: string;
    permission: string;
    expires_at?: string;
    reason?: string;
  }) => {
    return apiCall(() => adminApiClient.grantPermission(request));
  }, [apiCall]);

  const revokePermission = useCallback((request: {
    user_id: string;
    permission: string;
    reason?: string;
  }) => {
    return apiCall(() => adminApiClient.revokePermission(request));
  }, [apiCall]);

  const getUserPermissions = useCallback((userId: string) => {
    return apiCall(() => adminApiClient.getUserPermissions(userId));
  }, [apiCall]);

  const getSystemHealth = useCallback(() => {
    return apiCall(() => adminApiClient.getSystemHealth());
  }, [apiCall]);

  const getSystemMetrics = useCallback(() => {
    return apiCall(() => adminApiClient.getSystemMetrics());
  }, [apiCall]);

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