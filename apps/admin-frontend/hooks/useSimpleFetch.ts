/**
 * Simple fetch hook for admin frontend
 * Uses Shared AdminApiClient for API access
 */
import { useCallback } from 'react';
import { adminApiClient } from '../lib/api-client';

export function useSimpleFetch() {
  const fetchSimple = useCallback(async <T = any>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> => {
    const response = await adminApiClient.get<T>(endpoint, options as any);

    if (!response.success && response.status !== 404) {
      throw new Error(response.error || 'Request failed');
    }

    return response.data as T;
  }, []);

  return { fetchSimple };
}