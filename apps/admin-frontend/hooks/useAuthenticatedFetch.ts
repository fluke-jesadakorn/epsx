/**
 * Authenticated API Fetch Hook
 * Uses SharedWeb3AuthClient to get the access token for API requests
 */
import { env } from '@/config/env';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useCallback } from 'react';

const BACKEND_URL = env.BACKEND_URL;

export function useAuthenticatedFetch() {
  const { user } = useSharedAuth();

  const fetchWithAuth = useCallback(async (
    endpoint: string,
    options?: RequestInit
  ): Promise<Response> => {
    // Get the access token from the user object
    const token = user?.access;

    // Debug: Log token information for troubleshooting

    // Check JWT structure
    if (token && token.includes('.')) {
      const parts = token.split('.');
      if (parts.length === 3) {
        try {
          const header = JSON.parse(atob(parts[0] || ''));
          const payload = JSON.parse(atob(parts[1] || ''));
        } catch (e) {
        }
      }
    }

    const headers = {
      ...(token && { 'Authorization': `Bearer ${token}` }),
      'Content-Type': 'application/json',
      ...options?.headers
    };

    const response = await fetch(`${BACKEND_URL}${endpoint}`, {
      ...options,
      headers,
      credentials: 'include'
    });

    if (!response.ok) {
      const err = await response.text().catch(() => 'Request failed');
      throw new Error(`HTTP ${response.status}: ${err}`);
    }

    return response;
  }, [user?.access]);

  return { fetchWithAuth };
}