import type { ChatRequest, ChatResponse } from "@/types/chat";
import type { AuthResponse, GoogleAuthParams, TokenFeature, Permission } from "@/types/auth/features";
import { AuthError, ErrorCode } from "@/types/auth/errors";
import { redirect } from 'next/navigation';

export interface ApiClient {
  get<T>(endpoint: string, headers?: Record<string, string>): Promise<T>;
  post<T, TData = unknown>(endpoint: string, data: TData, headers?: Record<string, string>): Promise<T>;
  put<T, TData = unknown>(endpoint: string, data: TData, headers?: Record<string, string>): Promise<T>;
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
    roles(): Promise<Array<{
      uid: string;
      email: string;
      role: string;
    }>>;
    logout(): Promise<void>;
  };
}

type RequestMethod = "GET" | "POST" | "PUT" | "DELETE";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3002";

function isGoogleOAuthUrl(url: string): boolean {
  return url.includes('accounts.google.com');
}

function handleRedirect(location: string) {
  if (typeof window !== 'undefined' && isGoogleOAuthUrl(location)) {
    window.location.href = location;
    return new Promise(() => {}); // Never resolves - page will reload
  }
  // For internal redirects, use Next.js redirect
  redirect(location);
}

// Helper function to handle responses
async function handleFetchResponse(response: Response) {
  // Log request details
  console.debug('Request details:', {
    url: response.url,
    redirected: response.redirected,
    status: response.status,
    type: response.type,
  });

  // Log response headers
  console.debug('Response headers:', 
    [...response.headers.entries()].reduce((acc, [key, value]) => ({...acc, [key]: value}), {}));

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
      // For internal redirects, use the response as-is to preserve cookies
      return response;
    }
  }

  // Handle errors and session expiration
  if (!response.ok) {
    try {
      const contentType = response.headers.get('content-type');
      let errorData;
      
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
        cookies: response.headers.getSetCookie()
      });

      // Special handling for auth errors
      if (response.status === 401) {
        console.error('Authentication failed:', errorData);
        if (typeof window !== 'undefined') {
          // For client-side auth failures, clear cookies
          document.cookie.split(';').forEach(cookie => {
            const [name] = cookie.split('=');
            document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/`;
          });
        }
        throw new AuthError(
          ErrorCode.SESSION_EXPIRED,
          typeof errorData === 'object' ? errorData.message : 'Session expired'
        );
      }

      throw new AuthError(
        ErrorCode.NETWORK_ERROR,
        typeof errorData === 'object' ? errorData.message : errorData || response.statusText || "Network response was not ok"
      );
    } catch (e) {
      if (e instanceof AuthError) {
        throw e;
      }
      // If reading response fails, throw original status
      throw new AuthError(
        ErrorCode.NETWORK_ERROR,
        response.statusText || "Network response was not ok"
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
    return response.json();
  }

  // For non-JSON responses, return the final URL
  return { redirect: response.url };
}

// API endpoints fetch function
async function fetchWithAuth<TData = unknown>(
  endpoint: string,
  method: RequestMethod = "GET",
  data?: TData,
  customHeaders: Record<string, string> = {}
) {
  if (!API_URL) {
    throw new AuthError(
      ErrorCode.VALIDATION_ERROR,
      "API_URL is not defined in environment variables"
    );
  }

  const url = `${API_URL}${endpoint}`;
  
  // Get CSRF token from cookie if it exists
  let csrfToken = '';
  if (typeof window !== 'undefined') {
    csrfToken = document.cookie
      .split('; ')
      .find(row => row.startsWith('csrf_token='))
      ?.split('=')[1] || '';
  }

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Frontend-URL": typeof window !== 'undefined' ? window.location.origin : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    "Origin": typeof window !== 'undefined' ? window.location.origin : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    ...(csrfToken && { "X-CSRF-Token": csrfToken }),
    ...customHeaders,
  };

  const options: RequestInit = {
    method,
    headers,
    credentials: "include",
  };

  if (data && (method === "POST" || method === "PUT")) {
    options.body = JSON.stringify(data);
  }

  try {
    return handleFetchResponse(await fetch(url, {
      ...options,
      redirect: 'manual',
      credentials: 'include',
      mode: 'cors',
      headers: {
        ...headers,
        'Accept': 'application/json'
      }
    }));
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
    fetchWithAuth(endpoint, "GET", undefined, headers) as Promise<T>,

  post: <T, TData = unknown>(endpoint: string, data: TData, headers = {}) =>
    fetchWithAuth<TData>(endpoint, "POST", data, headers) as Promise<T>,

  put: <T, TData = unknown>(endpoint: string, data: TData, headers = {}) =>
    fetchWithAuth(endpoint, "PUT", data, headers) as Promise<T>,

  delete: <T>(endpoint: string, headers = {}) =>
    fetchWithAuth(endpoint, "DELETE", undefined, headers) as Promise<T>,
    
  // Chat query endpoint
  textQuery: (data: ChatRequest) =>
    fetchWithAuth("/text-query", "POST", data) as Promise<ChatResponse>,

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
          endpoint: '/v1/auth/session/validate'
        });
        
        const response = await fetchWithAuth('/v1/auth/session/validate', 'GET');
        // Log successful validation
        console.log('API Client - Session verified successfully');
        return response;
      } catch (error) {
        console.error('Session validation failed:', error);
        throw error;
      }
    },

    googleInit: async (params: { redirectUrl?: string }) => {
      const queryParams = new URLSearchParams();
      if (params.redirectUrl) {
        queryParams.set('redirect_url', params.redirectUrl);
      }
      const query = queryParams.toString() ? `?${queryParams.toString()}` : '';
      
      const response = await fetchWithAuth(`/v1/auth/google/init${query}`, "GET") as { oauth_url: string };
      if (response.oauth_url) {
        window.location.href = response.oauth_url;
        return new Promise(() => {}); // Never resolves - page will reload
      }
      throw new Error('Failed to get OAuth URL');
    },
      
    roles: () =>
      fetchWithAuth("/v1/auth/roles", "GET") as Promise<Array<{
        uid: string;
        email: string;
        role: string;
      }>>,
      
    logout: () =>
      fetchWithAuth("/v1/auth/logout", "POST")
  }
};
