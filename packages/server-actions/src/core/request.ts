// Note: These imports will be available when used in the frontend app
// The actual imports are handled dynamically to avoid circular dependencies

export interface ServerRequestOptions extends RequestInit {
  endpoint: string;
  baseUrl?: string;
  skipAuth?: boolean;
  retryAuth?: boolean;
  // Legacy compatibility properties for server actions
  action?: string;
  userId?: string;
  requestId?: string;
}

/**
 * Core server-side request handler with multi-provider authentication
 */
export async function makeServerRequest<T = any>(
  options: ServerRequestOptions
): Promise<T> {
  const {
    endpoint,
    baseUrl,
    skipAuth = false,
    retryAuth = true,
    ...fetchOptions
  } = options;

  try {
    let authToken = '';

    // Skip authentication if requested
    if (!skipAuth) {
      try {
        // Simple, unified token extraction
        if (typeof window !== 'undefined') {
          // Client-side: get token from session storage or cookies
          authToken = await extractClientSideToken();
        } else {
          // Server-side: extract token from cookies
          authToken = await extractServerSideToken();
        }

        console.log(
          `🔐 Token extraction result - SkipAuth: ${skipAuth}, Token: ${authToken ? 'present' : 'missing'}`
        );

        if (authToken) {
          console.log('🔑 Using Auth.js session token');
        }
      } catch (authError) {
        console.warn('Authentication token extraction failed:', authError);
      }
    }

    // Use environment variable for backend URL
    const backendUrl =
      baseUrl ||
      process.env.NEXT_PUBLIC_BACKEND_URL ||
      process.env.BACKEND_URL ||
      'http://localhost:8080';
    const url = `${backendUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      ...((fetchOptions.headers as Record<string, string>) || {}),
    };

    // Add unified OpenID Bearer token authentication
    if (authToken) {
      headers['Authorization'] = `Bearer ${authToken}`;

      // Add provider hint for backend routing
      headers['X-Provider-Hint'] = 'auth.js';

      console.log(`📨 Adding Authorization header for request to ${url}`);
    } else {
      console.log(`📨 No auth token, making unauthenticated request to ${url}`);
    }

    console.log(
      `📤 Making request to ${url} with headers:`,
      Object.keys(headers)
    );

    const response = await fetch(url, {
      ...fetchOptions,
      headers,
    });

    if (!response.ok) {
      const errorText = await response.text();
      const errorMessage = `HTTP ${response.status}: ${errorText || 'Request failed'}`;

      // Handle authentication errors with token refresh
      if (response.status === 401 && retryAuth && !skipAuth && authToken) {
        console.log('🔄 Token expired, attempting refresh...');

        try {
          let refreshedToken: string | null = null;

          if (typeof window !== 'undefined') {
            // Client-side: trigger Auth.js session refresh
            refreshedToken = await refreshClientSideToken();
          } else {
            // Server-side refresh logic
            refreshedToken = await refreshServerSideToken();
          }

          if (refreshedToken && refreshedToken !== authToken) {
            console.log('🔄 Token refreshed, retrying request...');
            // Retry the request with refreshed token
            return makeServerRequest<T>({
              ...options,
              retryAuth: false, // Prevent infinite retry loop
            });
          }
        } catch (refreshError) {
          console.error('Token refresh failed:', refreshError);
        }
      }

      // Add more context for common errors
      if (response.status === 404) {
        console.error(
          `API endpoint not found: ${url}. Please check if the backend implements this endpoint.`
        );
        throw new Error(
          `${errorMessage}\nEndpoint: ${url}\nBackend URL: ${backendUrl}`
        );
      } else if (response.status === 401) {
        console.error(
          `Authentication failed for: ${url}. Token may be invalid or expired.`
        );
        throw new Error(
          `${errorMessage}\nAuthentication required for: ${endpoint}`
        );
      } else if (response.status >= 500) {
        console.error(
          `Server error for: ${url}. Backend may be down or misconfigured.`
        );
        throw new Error(`${errorMessage}\nServer error at: ${endpoint}`);
      }

      throw new Error(errorMessage);
    }

    return response.json();
  } catch (error) {
    // Handle network failures more gracefully
    if (error instanceof TypeError && error.message.includes('fetch failed')) {
      const backendUrl =
        baseUrl ||
        process.env.NEXT_PUBLIC_BACKEND_URL ||
        process.env.BACKEND_URL ||
        'http://localhost:8080';
      throw new Error(
        `Network connection failed: Unable to reach backend at ${backendUrl}`
      );
    }
    throw error;
  }
}

/**
 * Extract Auth.js session token from client-side
 */
async function extractClientSideToken(): Promise<string> {
  // Client-side token extraction will be handled by Auth.js session
  // For now, return empty string and let Auth.js handle it
  return '';
}

/**
 * Extract Auth.js session token from server-side cookies
 */
async function extractServerSideToken(): Promise<string> {
  try {
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();

    console.log(
      '🍪 Available cookies:',
      Array.from(
        cookieStore
          .getAll()
          .map(c => `${c.name}=${c.value?.substring(0, 10)}...`)
      ).join(', ')
    );

    // Look for Auth.js session token
    const sessionToken =
      cookieStore.get('next-auth.session-token') ||
      cookieStore.get('__Secure-next-auth.session-token');

    if (sessionToken?.value) {
      console.log('🔑 Found Auth.js session token');
      return sessionToken.value;
    } else {
      console.log('❌ Auth.js session token not found or empty');
    }

    // Fallback: look for other JWT tokens
    const allCookies = cookieStore.getAll();
    const jwtCookie = allCookies.find(
      c => c.value.includes('.') && c.value.split('.').length === 3
    );

    if (jwtCookie?.value) {
      console.log('🔑 Found JWT token in fallback');
      return jwtCookie.value;
    } else {
      console.log('❌ JWT token not found in fallback');
    }

    console.warn(
      '⚠️ No valid tokens found in cookies. Available cookies:',
      Array.from(cookieStore.getAll().map(c => c.name)).join(', ')
    );
    return '';
  } catch (error) {
    console.debug('Failed to extract server-side token:', error);
    return '';
  }
}

/**
 * Refresh Auth.js session token on client-side
 */
async function refreshClientSideToken(): Promise<string | null> {
  try {
    // Client-side refresh will be handled by Auth.js session hooks
    console.debug('Client-side token refresh handled by Auth.js');
    return null;
  } catch (error) {
    console.debug('Client-side token refresh failed:', error);
    return null;
  }
}

/**
 * Refresh token on server-side
 */
async function refreshServerSideToken(): Promise<string | null> {
  try {
    // Server-side refresh will be handled by Auth.js
    console.debug('Server-side token refresh handled by Auth.js');
    return null;
  } catch (error) {
    console.debug('Server-side token refresh failed:', error);
    return null;
  }
}

/**
 * Helper for making GET requests
 */
export async function serverGet<T = any>(
  endpoint: string,
  params?: Record<string, string | number | boolean>,
  options?: Omit<ServerRequestOptions, 'endpoint' | 'method'>
): Promise<T> {
  let url = endpoint;

  if (params) {
    const queryParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        queryParams.append(key, String(value));
      }
    });

    if (queryParams.toString()) {
      url += `?${queryParams.toString()}`;
    }
  }

  return makeServerRequest<T>({
    endpoint: url,
    method: 'GET',
    ...options,
  });
}

/**
 * Helper for making POST requests
 */
export async function serverPost<T = any>(
  endpoint: string,
  data?: any,
  options?: Omit<ServerRequestOptions, 'endpoint' | 'method'>
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'POST',
    body: data ? JSON.stringify(data) : undefined,
    ...options,
  });
}

/**
 * Helper for making PUT requests
 */
export async function serverPut<T = any>(
  endpoint: string,
  data?: any,
  options?: Omit<ServerRequestOptions, 'endpoint' | 'method'>
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'PUT',
    body: data ? JSON.stringify(data) : undefined,
    ...options,
  });
}

/**
 * Helper for making DELETE requests
 */
export async function serverDelete<T = any>(
  endpoint: string,
  options?: Omit<ServerRequestOptions, 'endpoint' | 'method'>
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'DELETE',
    ...options,
  });
}

/**
 * Helper for making PATCH requests
 */
export async function serverPatch<T = any>(
  endpoint: string,
  data?: any,
  options?: Omit<ServerRequestOptions, 'endpoint' | 'method'>
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'PATCH',
    body: data ? JSON.stringify(data) : undefined,
    ...options,
  });
}
