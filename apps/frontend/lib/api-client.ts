import { ChatRequest, ChatResponse } from "@/types/chat";

type RequestMethod = "GET" | "POST" | "PUT" | "DELETE";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

// TODO: Implement authentication system
// Consider options like:
// - JWT with HttpOnly cookies
// - NextAuth.js session
// - OAuth2 tokens

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
    ...customHeaders,
  };

  // TODO: Add authentication token to headers
  // Example: headers["Authorization"] = `Bearer ${getAuthToken()}`;

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
    const error = await response.json().catch(() => ({
      message: "An error occurred while fetching the data.",
    }));
    throw new Error(error.message || "Network response was not ok");
  }

  return response.json();
}

export const apiClient = {
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
};

// TODO: Add request/response interceptors for:
// - Automatic token refresh
// - Error handling
// - Request/response transformations
// - Logging
