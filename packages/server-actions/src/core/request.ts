export interface ServerRequestOptions extends RequestInit {
  endpoint: string;
  baseUrl?: string;
}

/**
 * Core server-side request handler that automatically handles authentication
 */
export async function makeServerRequest<T = any>(
  options: ServerRequestOptions
): Promise<T> {
  const { endpoint, baseUrl, ...fetchOptions } = options;
  
  try {
    let authToken = '';
    
    try {
      // Import cookies for session token access
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      const allCookies = cookieStore.getAll();
      
      // Look for NextAuth session token in cookies
      const sessionToken = allCookies.find(c => 
        c.name.includes('next-auth.session-token') || 
        c.name.includes('__Secure-next-auth.session-token')
      );
      
      if (sessionToken) {
        // Decode the NextAuth JWT token to get the access token
        try {
          const { decode } = await import('next-auth/jwt');
          const secret = process.env.NEXTAUTH_SECRET;
          
          if (secret) {
            const token = await decode({
              token: sessionToken.value,
              secret: secret,
            });
            
            if (token?.accessToken) {
              authToken = token.accessToken as string;
              console.log('🔑 Using NextAuth accessToken from JWT for backend request');
            } else if (token?.access_token) {
              authToken = token.access_token as string;
              console.log('🔑 Using NextAuth access_token from JWT for backend request');
            } else if (token?.session_id) {
              authToken = token.session_id as string;
              console.log('🔑 Using NextAuth session_id as token for backend request');
            } else if (token?.sub) {
              // Fallback to user ID if no access token
              authToken = token.sub;
              console.log('🔑 Using NextAuth user ID as fallback token for backend request');
            }
          }
        } catch (jwtError) {
          console.warn('Failed to decode NextAuth JWT token:', jwtError);
          // Use raw session token as fallback
          authToken = sessionToken.value;
          console.log('🔑 Using raw NextAuth session token for backend request');
        }
      }
    } catch (authError) {
      console.warn('NextAuth authentication failed, proceeding without authentication:', authError);
    }
    
    // Use environment variable for backend URL
    const backendUrl = baseUrl || process.env.BACKEND_URL || 'http://localhost:8080';
    const url = `${backendUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(fetchOptions.headers as Record<string, string> || {}),
    };
    
    // Add Authorization header with Bearer token if we have auth token
    if (authToken) {
      headers['Authorization'] = `Bearer ${authToken}`;
    }
    
    const response = await fetch(url, {
      ...fetchOptions,
      headers,
    });
    
    if (!response.ok) {
      const errorText = await response.text();
      const errorMessage = `HTTP ${response.status}: ${errorText || 'Request failed'}`;
      
      // Add more context for common errors
      if (response.status === 404) {
        console.error(`API endpoint not found: ${url}. Please check if the backend implements this endpoint.`);
        throw new Error(`${errorMessage}\nEndpoint: ${url}\nBackend URL: ${backendUrl}`);
      } else if (response.status === 401) {
        console.error(`Authentication failed for: ${url}. Please check if user is logged in.`);
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
      throw new Error(`Network connection failed: Unable to reach backend at ${baseUrl || process.env.BACKEND_URL || 'http://localhost:8080'}`);
    }
    throw error;
  }
}

/**
 * Helper for making GET requests
 */
export async function serverGet<T = any>(
  endpoint: string, 
  params?: Record<string, string | number | boolean>
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
  });
}

/**
 * Helper for making POST requests
 */
export async function serverPost<T = any>(
  endpoint: string, 
  data?: any
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'POST',
    body: data ? JSON.stringify(data) : undefined,
  });
}

/**
 * Helper for making PUT requests
 */
export async function serverPut<T = any>(
  endpoint: string, 
  data?: any
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'PUT',
    body: data ? JSON.stringify(data) : undefined,
  });
}

/**
 * Helper for making DELETE requests
 */
export async function serverDelete<T = any>(
  endpoint: string
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'DELETE',
  });
}