/**
 * Authenticated API Fetch Hook
 * Uses Shared AdminApiClient for API requests
 */
import { useCallback } from 'react';

import { adminApiClient } from '../lib/api-client';

/**
 *
 */
export function useAuthenticatedFetch() {
  const fetchWithAuth = useCallback(async <T = any>(
    endpoint: string,
    options?: RequestInit
  ): Promise<T> => {
    // Determine method from options or default to GET
    const method = options?.method?.toLowerCase() || 'get';

    let response;

    if (method === 'post') {
      response = await adminApiClient.post<T>(endpoint, options?.body ? JSON.parse(options.body as string) : undefined, options as any);
    } else if (method === 'put') {
      response = await adminApiClient.put<T>(endpoint, options?.body ? JSON.parse(options.body as string) : undefined, options as any);
    } else if (method === 'delete') {
      response = await adminApiClient.delete<T>(endpoint, options as any);
    } else {
      response = await adminApiClient.get<T>(endpoint, undefined, options as any);
    }

    if (!response.success) {
      throw new Error(response.error || `Request failed with status ${response.status}`);
    }

    return response.data as T;
  }, []);

  return { fetchWithAuth };
}