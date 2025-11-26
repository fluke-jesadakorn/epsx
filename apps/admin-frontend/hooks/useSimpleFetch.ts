/**
 * Simple fetch hook for admin frontend
 * Uses Bearer token authentication for admin API access
 */
import { useCallback } from 'react';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

function getToken(): string | null {
  if (typeof document === 'undefined') return null;

  const cookies = document.cookie.split(';');

  // Try both development and production cookie names
  const tokenCookie = cookies.find(c => {
    const trimmed = c.trim();
    return trimmed.startsWith('epsx.access=') || trimmed.startsWith('__Host-epsx.access=');
  });

  if (!tokenCookie) return null;

  return tokenCookie.split('=')[1] || null;
}

export function useSimpleFetch() {
  const fetchSimple = useCallback(async (
    endpoint: string,
    options?: RequestInit
  ): Promise<Response> => {
    console.log(`🌐 Simple Request: ${endpoint}`);

    const token = getToken();

    const response = await fetch(`${BACKEND_URL}${endpoint}`, {
      ...options,
      headers: {
        ...(token && { 'Authorization': `Bearer ${token}` }),
        'Content-Type': 'application/json',
        ...options?.headers
      },
      credentials: 'include'
    });

    if (!response.ok) {
      const err = await response.text().catch(() => 'Request failed');
      throw new Error(`HTTP ${response.status}: ${err}`);
    }

    return response;
  }, []);

  return { fetchSimple };
}