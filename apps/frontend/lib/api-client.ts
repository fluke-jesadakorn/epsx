import type { ChatRequest, ChatResponse } from "@/types/chat";

type RequestMethod = "GET" | "POST" | "PUT" | "DELETE";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001/v1";
// Auth URL includes /v1 prefix for consistency with API routes
const AUTH_URL = process.env.NEXT_PUBLIC_AUTH_URL || "http://localhost:3001/v1";

// Original fetch function for API endpoints
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
    ...customHeaders,
  };

  const options: RequestInit = {
    method,
    headers: {
      ...headers,
      "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
      "X-Source": "Cloudflare-Workers",
    },
    credentials: "include",
  };

  if (data && (method === "POST" || method === "PUT")) {
    options.body = JSON.stringify(data);
  }

  const response = await fetch(url, options);

  if (!response.ok) {
    const errorText = await response.text();
    console.error('Auth request failed:', {
      url,
      status: response.status,
      statusText: response.statusText,
      body: errorText
    });
    
    try {
      const error = JSON.parse(errorText);
      throw new Error(error.message || "Network response was not ok");
    } catch (e) {
      if (e instanceof Error) {
        console.error('Error parsing error response:', e.message);
      }
      throw new Error(errorText || response.statusText || "Network response was not ok");
    }
  }

  return response.json();
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
    ...customHeaders,
  };

  const options: RequestInit = {
    method,
    headers: {
      ...headers,
      "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
      "X-Source": "Cloudflare-Workers",
    },
    credentials: "include",
  };

  if (data && (method === "POST" || method === "PUT")) {
    options.body = JSON.stringify(data);
  }

  const response = await fetch(url, options);

  if (!response.ok) {
    const errorText = await response.text();
    console.error('Auth request failed:', {
      url,
      status: response.status,
      statusText: response.statusText,
      body: errorText
    });
    
    try {
      const error = JSON.parse(errorText);
      throw new Error(error.message || "Network response was not ok");
    } catch (e) {
      if (e instanceof Error) {
        console.error('Error parsing error response:', e.message);
      }
      throw new Error(errorText || response.statusText || "Network response was not ok");
    }
  }

  return response.json();
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
      
    googleCallback: (code: string, state: string) =>
      fetchWithAuthBase(`/auth/google/callback?code=${encodeURIComponent(code)}&state=${encodeURIComponent(state)}`, "GET"),
      
    register: (data: { email: string; password: string }) =>
      fetchWithAuthBase("/auth/register", "POST", data),
      
    login: (data: { email: string; password: string }) =>
      fetchWithAuthBase("/auth/login", "POST", data),
      
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
