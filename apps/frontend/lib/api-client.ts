import type { ChatRequest, ChatResponse } from "@/types/chat";
import type { AuthResponse } from "@/types/auth";

type RequestMethod = "GET" | "POST" | "PUT" | "DELETE";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3002/v1";
// Auth URL includes /v1 prefix for consistency with API routes
const AUTH_URL = process.env.NEXT_PUBLIC_AUTH_URL || "http://localhost:3002/v1";

// Helper function to handle responses
async function handleFetchResponse(response: Response) {
  // Log all response headers for debugging
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

    // Log specific headers we care about
    console.debug('Important headers:', {
      contentType: response.headers.get('content-type'),
      setCookie: response.headers.get('set-cookie'),
      location: response.headers.get('location'),
    });

  if (!response.ok) {
    const errorText = await response.text();
    console.error('Request failed:', {
      url: response.url,
      status: response.status,
      statusText: response.statusText,
      error: errorText
    });
    throw new Error(errorText || response.statusText || "Network response was not ok");
  }

  const contentType = response.headers.get('content-type');
  if (contentType?.includes('application/json')) {
    return response.json();
  }

  // For non-JSON responses (like after successful auth), return the final URL
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
    throw new Error("API_URL is not defined in environment variables");
  }

  const url = `${API_URL}${endpoint}`;
  
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Frontend-URL": typeof window !== 'undefined' ? window.location.origin : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    "X-Source": "Cloudflare-Workers",
    ...customHeaders, // Custom headers should be last to allow overriding defaults
  };

  const options: RequestInit = {
    method,
    headers,
    credentials: "include",
  };

  if (data && (method === "POST" || method === "PUT")) {
    options.body = JSON.stringify(data);
  }

  return handleFetchResponse(await fetch(url, {
    ...options,
    redirect: 'follow', // Follow redirects to get cookies set
    credentials: 'include', // Always send cookies
    mode: 'cors' // Enable CORS with credentials
  }));
}

// New fetch function specifically for auth endpoints
async function fetchWithAuthBase<TData = unknown>(
  endpoint: string,
  method: RequestMethod = "GET",
  data?: TData,
  customHeaders: Record<string, string> = {}
) {
  if (!AUTH_URL) {
    throw new Error("AUTH_URL is not defined");
  }

  const url = `${AUTH_URL}${endpoint}`;
  
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "X-Frontend-URL": typeof window !== 'undefined' ? window.location.origin : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000',
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    "X-Source": "Cloudflare-Workers",
    ...customHeaders, // Custom headers should be last to allow overriding defaults
  };

  const options: RequestInit = {
    method,
    headers,
    credentials: "include",
  };

  if (data && (method === "POST" || method === "PUT")) {
    options.body = JSON.stringify(data);
  }

  return handleFetchResponse(await fetch(url, {
    ...options,
    redirect: 'follow', // Follow redirects to get cookies set
    credentials: 'include', // Always send cookies
    mode: 'cors' // Enable CORS with credentials
  }));
}

export const apiClient = {
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

  // Auth endpoints using the base API URL
  auth: {
    googleInit: (params: { redirectUrl?: string; oauthRedirectUri?: string }) => {
      const queryParams = new URLSearchParams();
      if (params.redirectUrl) {
        queryParams.set('redirect_url', params.redirectUrl);
      }
      if (params.oauthRedirectUri) {
        queryParams.set('oauth_redirect_uri', params.oauthRedirectUri);
      }
      const query = queryParams.toString() ? `?${queryParams.toString()}` : '';
      return fetchWithAuthBase(`/auth/google/init${query}`, "GET") as Promise<{ url: string }>;
    },
      
    googleCallback: (code: string, state: string): Promise<{ redirect: string } | null> =>
      fetchWithAuthBase(`/auth/google/callback?code=${encodeURIComponent(code)}&state=${encodeURIComponent(state)}`, "GET", undefined, {
        "X-Frontend-URL": typeof window !== 'undefined' ? window.location.origin : process.env.NEXT_PUBLIC_FRONTEND_URL || 'http://localhost:3000'
      }),
      
    register: (data: { email: string; password: string }): Promise<AuthResponse> =>
      fetchWithAuthBase("/auth/register", "POST", data) as Promise<AuthResponse>,
      
    login: (data: { email: string; password: string }): Promise<{ redirect: string } | null> =>
      fetchWithAuthBase("/auth/login", "POST", data) as Promise<{ redirect: string } | null>,
      
    roles: () =>
      fetchWithAuthBase("/auth/roles", "GET") as Promise<Array<{
        uid: string;
        email: string;
        role: string;
      }>>,
      
    logout: () =>
      fetchWithAuthBase("/auth/logout", "POST")
  }
};

// TODO: Add request/response interceptors for:
// - Automatic token refresh
// - Error handling
// - Request/response transformations
// - Logging
