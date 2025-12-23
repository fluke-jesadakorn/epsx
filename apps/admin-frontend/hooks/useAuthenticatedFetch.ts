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
    console.log(`🔐 Authenticated Request: ${endpoint}`);
    console.log('👤 User exists:', !!user);
    console.log('👤 User wallet:', user?.wallet_address?.slice(0, 8) + '...');
    console.log('🎫 Token exists:', !!token);
    console.log('🎫 Token length:', token?.length || 0);
    console.log('🎫 Token preview:', token ? `${token.substring(0, 20)}...` : 'null');
    console.log('🎫 Token structure:', token ? (token.includes('.') ? 'JWT format (3 parts)' : 'Not JWT format') : 'null');

    // Check JWT structure
    if (token && token.includes('.')) {
      const parts = token.split('.');
      console.log('🎫 JWT parts:', parts.length);
      if (parts.length === 3) {
        try {
          const header = JSON.parse(atob(parts[0] || ''));
          const payload = JSON.parse(atob(parts[1] || ''));
          console.log('🎫 JWT header:', header);
          console.log('🎫 JWT payload:', payload);
          console.log('🎫 JWT expires:', payload.exp ? new Date(payload.exp * 1000).toISOString() : 'no exp');
          console.log('🎫 JWT expired:', payload.exp ? (Date.now() / 1000 > payload.exp) : 'unknown');
        } catch (e) {
          console.log('❌ JWT parsing error:', e);
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