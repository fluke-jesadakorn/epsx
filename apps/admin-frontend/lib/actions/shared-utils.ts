/**
 * Shared utilities for server actions
 * Eliminates duplication across action files
 */

'use server';

import { cookies } from 'next/headers';

import { env } from '@/config/env';

/**
 * Make authenticated request to backend API
 * Shared utility to eliminate duplication across action files
 * @param endpoint
 * @param options
 */
export async function makeAuthenticatedRequest(endpoint: string, options: RequestInit = {}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  };

  const response = await fetch(`${env.BACKEND_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`HTTP ${response.status}: ${errorText || 'Request failed'}`);
  }

  return response.json();
}

/**
 * Alternative method using session-based auth (for compatibility)
 */
export async function getBearerTokenFromSession() {
  const cookieStore = await cookies();
  return cookieStore.get('access_token')?.value || null;
}

// Note: ActionResult types and utility functions are now in @/lib/action-utils
// Import them directly where needed since server actions can only export async functions