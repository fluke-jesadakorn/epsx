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
    let cookieHeader = '';
    
    try {
      // Import cookies dynamically to avoid client-side import issues
      const { cookies } = await import('next/headers');
      const cookieStore = await cookies();
      const allCookies = cookieStore.getAll();
      cookieHeader = allCookies.map(c => `${c.name}=${c.value}`).join('; ');
    } catch (cookiesError) {
      // Handle case where cookies() is called outside request context
      if (typeof cookiesError === 'object' && cookiesError && 'message' in cookiesError) {
        const errorMessage = String(cookiesError.message);
        if (errorMessage.includes('cookies" was called outside a request scope') || 
            errorMessage.includes('cookies() was called outside a request scope')) {
          console.warn('Cookies not available - running outside request context. Proceeding without authentication headers.');
          // Continue without cookies - this allows server actions to work in client context for development
        } else {
          throw cookiesError;
        }
      } else {
        throw cookiesError;
      }
    }
    
    // Use environment variable for backend URL
    const backendUrl = baseUrl || process.env.BACKEND_URL || 'http://localhost:8080';
    const url = `${backendUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(fetchOptions.headers as Record<string, string> || {}),
    };
    
    // Only add Cookie header if we have cookies
    if (cookieHeader) {
      headers['Cookie'] = cookieHeader;
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