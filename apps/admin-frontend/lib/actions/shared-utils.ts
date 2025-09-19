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
 */
export async function makeAuthenticatedRequest(endpoint: string, options: RequestInit = {}) {
  const cookieStore = await cookies();
  const token = cookieStore.get('access_token')?.value;
  
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  };

  const response = await fetch(`${env.NEXT_PUBLIC_BACKEND_URL}${endpoint}`, {
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

/**
 * Common response types for consistency
 */
export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

/**
 * Create successful action result
 */
export function createSuccessResult<T>(data: T, message?: string): ActionResult<T> {
  return {
    success: true,
    data,
    message
  };
}

/**
 * Create error action result
 */
export function createErrorResult(error: string): ActionResult {
  return {
    success: false,
    error
  };
}