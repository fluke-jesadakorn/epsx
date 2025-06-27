import { redirect } from 'next/navigation';

import { AuthError, ErrorCode } from '@/types/auth/errors';
import type {
  GoogleAuthParams,
  Permission,
  TokenFeature,
} from '@/types/auth/features';
import type { ChatRequest, ChatResponse } from '@/types/chat';

export interface ApiClient {
  get<T>(endpoint: string, headers?: Record<string, string>): Promise<T>;
  post<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers?: Record<string, string>,
  ): Promise<T>;
  put<T, TData = unknown>(
    endpoint: string,
    data: TData,
    headers?: Record<string, string>,
  ): Promise<T>;
  delete<T>(endpoint: string, headers?: Record<string, string>): Promise<T>;
  textQuery(data: ChatRequest): Promise<ChatResponse>;
  auth: {
    verifySession(): Promise<{
      email: string;
      user_id: string;
      role: string;
      token_balance: number;
      features: TokenFeature[];
      permissions: Permission[];
    }>;
    googleInit(params: GoogleAuthParams): Promise<never>;
    roles(): Promise<
      Array<{
        uid: string;
        email: string;
        role: string;
      }>
    >;
    logout(): Promise<void>;
  };
}

type RequestMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3002';

function isGoogleOAuthUrl(url: string): boolean {
  return url.includes('accounts.google.com');
}

// Helper function to handle responses
async function handleFetchResponse<T>(response: Response): Promise<T> {
  // Log request details
  console.debug('Request details:', {
    url: response.url,
    redirected: response.redirected,
    status: response.status,
    type: response.type,
  });

  // Log response headers
  console.debug(
    'Response headers:',
    [...response.headers.entries()].reduce(
      (acc, [key, value]) => ({ ...acc, [key]: value }),
      {},
    ),
  );

  // Special handling for redirects
  if (response.status === 302 || response.status === 307) {
    const location = response.headers.get('location');
    if (location) {
      if (isGoogleOAuthUrl(location)) {
        // For Google OAuth, use window.location to preserve cookies
        if (typeof window !== 'undefined') {
          window.location.href = location;
          return new Promise(() => {}); // Never resolves - page will reload
        }
      }
      // For internal redirects, use Next.js redirect
      redirect(location);
    }
  }

  // Handle errors and session expiration
  if (!response.ok) {
    let errorData: any;
    try {
      const contentType = response.headers.get('content-type');

      if (contentType?.includes('application/json')) {
        errorData = await response.json();
      } else {
        errorData = await response.text();
      }

      // Log full error details
      console.error('Request failed:', {
        url: response.url,
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries()),
        error: errorData,
        cookies: response.headers.getSetCookie(),
      });

      // Special handling for auth errors
      if (response.status === 401) {
        console.error('Authentication failed:', errorData);
        if (typeof window !== 'undefined') {
          // For client-side auth failures, clear cookies
          document.cookie.split(';').forEach((cookie) => {
            const [name] = cookie.split('=');
            document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
          });
        }
        throw new AuthError(
          ErrorCode.SESSION_EXPIRED,
          typeof errorData === 'object' && errorData.message
            ? errorData.message
            : 'Session expired',
        );
      }

      throw new AuthError(
        ErrorCode.NETWORK_ERROR,
        typeof errorData === 'object' && errorData.message
          ? errorData.message
          : errorData || response.statusText || 'Network response was not ok',
      );
    } catch (e) {
      if (e instanceof AuthError) {
        throw e;
      }
      // If reading response fails, throw original status
      throw new AuthError(
        ErrorCode.NETWORK_ERROR,
        response.statusText || 'Network response was not ok',
      );
    }
  }

  // For auth endpoints, ensure we forward Set-Cookie headers
  if (response.url.includes('/auth/')) {
    const cookies = response.headers.getSetCookie();
    if (cookies.length > 0) {
      console.log('Received cookies from auth endpoint:', cookies);
    }
  }

  const contentType = response.headers.get('content-type');
  if (contentType?.includes('application/json')) {
    return response.json() as Promise<T>;
  }

  // For non-JSON responses, return the final URL as part of an object
  return { redirect: response.url } as T;
}

// API endpoints fetch function
async function fetchWithAuth<T, TData = unknown>(
  endpoint: string,
  method: RequestMethod = 'GET',
  data?: TData,
  customHeaders: Record<string, string> = {},
): Promise<T> {
  if (!API_URL) {
    throw new AuthError(
      ErrorCode.VALIDATION_ERROR,
      'API_URL is not defined in environment variables',
    );
  }

  const url = `${API_URL}${endpoint}`;

  // Get CSRF token from cookie if it exists
  let csrfToken = '';
  let authToken = '';
  if (typeof window !== 'undefined') {
    csrfToken =
      document.cookie
        .split('; ')
        .find((row) => row.startsWith('csrf_token='))
        ?.split('=')[1] || '';
    // Attempt to get auth token from local storage or context
    try {
      const storedToken = localStorage.getItem('auth_token');
      if (storedToken) {
        authToken = storedToken;
      }
    } catch (e) {
      console.error('Failed to retrieve auth token from storage:', e);
    }
  }

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'X-Frontend-URL':
      typeof window !== 'undefined'
        ? window.location.origin
        : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)',
    Origin:
      typeof window !== 'undefined'
        ? window.location.origin
        : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    ...(csrfToken && { 'X-CSRF-Token': csrfToken }),
    ...(authToken && { Authorization: `Bearer ${authToken}` }),
    ...customHeaders,
  };

  const options: RequestInit = {
    method,
    headers,
    credentials: 'include',
  };

  if (data && (method === 'POST' || method === 'PUT')) {
    options.body = JSON.stringify(data);
  }

  try {
    const response = await fetch(url, {
      ...options,
      redirect: 'manual',
      credentials: 'include',
      mode: 'cors',
      headers: {
        ...headers,
        Accept: 'application/json',
      },
    });
    return await handleFetchResponse<T>(response);
  } catch (error) {
    if (error instanceof Error && error.message.includes('NEXT_REDIRECT')) {
      throw error; // Let Next.js handle the redirect
    }
    throw error;
  }
}

export const apiClient: ApiClient = {
  // Regular API endpoints
  get: <T>(endpoint: string, headers = {}) =>
    fetchWithAuth<T>(endpoint, 'GET', undefined, headers),

  post: <T, TData = unknown>(endpoint: string, data: TData, headers = {}) =>
    fetchWithAuth<T, TData>(endpoint, 'POST', data, headers),

  put: <T, TData = unknown>(endpoint: string, data: TData, headers = {}) =>
    fetchWithAuth<T, TData>(endpoint, 'PUT', data, headers),

  delete: <T>(endpoint: string, headers = {}) =>
    fetchWithAuth<T>(endpoint, 'DELETE', undefined, headers),

  // Chat query endpoint
  textQuery: (data: ChatRequest) =>
    fetchWithAuth<ChatResponse>('/text-query', 'POST', data),

  // Auth endpoints
  auth: {
    verifySession: async (): Promise<{
      email: string;
      user_id: string;
      role: string;
      token_balance: number;
      features: TokenFeature[];
      permissions: Permission[];
    }> => {
      try {
        // Log API URL and endpoint
        console.log('API Client - Verify Session:', {
          url: API_URL,
          endpoint: '/v1/auth/session/validate',
        });

        const response = await fetchWithAuth<{
          email: string;
          user_id: string;
          role: string;
          token_balance: number;
          features: TokenFeature[];
          permissions: Permission[];
        }>('/v1/auth/session/validate', 'GET');
        // Log successful validation
        console.log('API Client - Session verified successfully');
        return response;
      } catch (error) {
        console.error('Session validation failed:', error);
        throw error;
      }
    },

    googleInit: async (params: { redirectUrl?: string }): Promise<never> => {
      const queryParams = new URLSearchParams();
      if (params.redirectUrl) {
        queryParams.set('redirect_url', params.redirectUrl);
      }
      const query = queryParams.toString() ? `?${queryParams.toString()}` : '';

      const response = await fetchWithAuth<{ oauth_url: string }>(
        `/v1/auth/google/init${query}`,
        'GET',
      );
      if (response.oauth_url) {
        if (typeof window !== 'undefined') {
          window.location.href = response.oauth_url;
        }
        return new Promise(() => {}); // Never resolves - page will reload
      }
      throw new Error('Failed to get OAuth URL');
    },

    roles: () =>
      fetchWithAuth<
        Array<{
          uid: string;
          email: string;
          role: string;
        }>
      >('/v1/auth/roles', 'GET'),

    logout: () => fetchWithAuth<void>('/v1/auth/logout', 'POST'),
  },
};
