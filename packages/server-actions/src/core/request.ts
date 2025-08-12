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
  const { endpoint, baseUrl, skipAuth = false, retryAuth = true, ...fetchOptions } = options;
  
  try {
    let authToken = '';
    
    // Skip authentication if requested
    if (!skipAuth) {
      try {
        // Try unified token manager first (client-side)
        if (typeof window !== 'undefined') {
          try {
            // Dynamic import to avoid circular dependencies
            // Import from frontend app when this package is used there
            const tokenManagerModule = await import('@/lib/auth/unified-token-manager' as any).catch(() => null);
            if (tokenManagerModule) {
              const { getUnifiedTokenManager } = tokenManagerModule;
              const tokenManager = getUnifiedTokenManager();
              const validToken = await tokenManager.getCurrentToken();
              
              if (validToken) {
                authToken = validToken;
                console.log('🔑 Using unified token from client-side token manager');
              }
            }
          } catch (importError) {
            console.debug('Token manager not available, falling back to cookie extraction:', importError);
          }
        } else {
          // Server-side: extract token from cookies
          const token = await extractServerSideToken();
          if (token) {
            authToken = token;
            console.log('🔑 Using unified token from server-side extraction');
          }
        }
      } catch (authError) {
        console.warn('Multi-provider authentication failed, proceeding without authentication:', authError);
      }
    }
    
    // Use environment variable for backend URL
    const backendUrl = baseUrl || process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
    const url = `${backendUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
      ...(fetchOptions.headers as Record<string, string> || {}),
    };
    
    // Add unified OpenID Bearer token authentication
    if (authToken) {
      headers['Authorization'] = `Bearer ${authToken}`;
      
      // Add provider hint for backend routing
      try {
        const providerDetectorModule = await import('@/lib/auth/provider-detector' as any).catch(() => null);
        if (providerDetectorModule) {
          const { default: ProviderDetector } = providerDetectorModule;
          const providerInfo = ProviderDetector.detectFromToken(authToken);
          if (providerInfo.provider !== 'unknown') {
            headers['X-Provider-Hint'] = providerInfo.provider;
          }
        }
      } catch (importError) {
        console.debug('Provider detector not available, skipping provider hint:', importError);
      }
    }
    
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
            try {
              // Import from frontend app when this package is used there
              const tokenManagerModule = await import('@/lib/auth/unified-token-manager' as any).catch(() => null);
              if (tokenManagerModule) {
                const { getUnifiedTokenManager } = tokenManagerModule;
                const tokenManager = getUnifiedTokenManager();
                refreshedToken = await tokenManager.refreshToken().then((t: any) => t?.access_token || null);
              }
            } catch (importError) {
              console.debug('Token manager not available for refresh:', importError);
            }
          } else {
            // Server-side refresh logic
            refreshedToken = await refreshServerSideToken();
          }
          
          if (refreshedToken && refreshedToken !== authToken) {
            console.log('🔄 Token refreshed, retrying request...');
            // Retry the request with refreshed token
            return makeServerRequest<T>({
              ...options,
              retryAuth: false // Prevent infinite retry loop
            });
          }
        } catch (refreshError) {
          console.error('Token refresh failed:', refreshError);
        }
      }
      
      // Add more context for common errors
      if (response.status === 404) {
        console.error(`API endpoint not found: ${url}. Please check if the backend implements this endpoint.`);
        throw new Error(`${errorMessage}\nEndpoint: ${url}\nBackend URL: ${backendUrl}`);
      } else if (response.status === 401) {
        console.error(`Authentication failed for: ${url}. Token may be invalid or expired.`);
        throw new Error(`${errorMessage}\nAuthentication required for: ${endpoint}`);
      } else if (response.status >= 500) {
        console.error(`Server error for: ${url}. Backend may be down or misconfigured.`);
        throw new Error(`${errorMessage}\nServer error at: ${endpoint}`);
      }
      
      throw new Error(errorMessage);
    }
    
    return response.json();
  } catch (error) {
    // Handle network failures more gracefully
    if (error instanceof TypeError && error.message.includes('fetch failed')) {
      const backendUrl = baseUrl || process.env.NEXT_PUBLIC_BACKEND_URL || process.env.BACKEND_URL || 'http://localhost:8080';
      throw new Error(`Network connection failed: Unable to reach backend at ${backendUrl}`);
    }
    throw error;
  }
}

/**
 * Extract unified JWT token from server-side cookies
 */
async function extractServerSideToken(): Promise<string | null> {
  try {
    const { cookies } = await import('next/headers');
    const cookieStore = await cookies();
    const allCookies = cookieStore.getAll();
    
    // Look for unified JWT tokens in cookies (priority order)
    const tokenCookieNames = [
      'unified_auth_token',
      'oidc-session-token',
      '__Secure-oidc-session-token',
      'jwt_token',
      'access_token'
    ];
    
    for (const cookieName of tokenCookieNames) {
      const cookie = allCookies.find(c => c.name === cookieName || c.name.includes(cookieName));
      if (cookie) {
        // Validate that it looks like a JWT
        if (cookie.value.includes('.')) {
          return cookie.value;
        }
      }
    }
    
    return null;
  } catch (error) {
    console.debug('Failed to extract server-side token:', error);
    return null;
  }
}

/**
 * Refresh token on server-side
 */
async function refreshServerSideToken(): Promise<string | null> {
  try {
    // TODO: Implement server-side token refresh
    // This would involve calling the backend refresh endpoint
    console.debug('Server-side token refresh not implemented yet');
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