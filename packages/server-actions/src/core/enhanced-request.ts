import { logger, HttpError, getApiBaseUrl } from '@epsx/shared-core';

export interface ServerRequestOptions extends RequestInit {
  endpoint: string;
  baseUrl?: string;
  context?: {
    action?: string;
    userId?: string;
    requestId?: string;
  };
}

/**
 * Enhanced server-side request handler with improved error handling
 */
export async function makeServerRequest<T = any>(
  options: ServerRequestOptions
): Promise<T> {
  const { endpoint, baseUrl, context, ...fetchOptions } = options;
  
  try {
    logger.debug('Making server request', { 
      endpoint, 
      method: fetchOptions.method || 'GET' 
    }, {
      component: 'server-request',
      action: context?.action || 'makeServerRequest',
      userId: context?.userId,
      requestId: context?.requestId
    });

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
        if (errorMessage.includes('cookies" was called outside a request scope')) {
          logger.warn('Cookies not available - running outside request context', {}, {
            component: 'server-request',
            action: context?.action
          });
        } else {
          throw cookiesError;
        }
      } else {
        throw cookiesError;
      }
    }
    
    // Use environment variable for backend URL
    const backendUrl = baseUrl || getApiBaseUrl();
    const url = `${backendUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    // Safely merge headers
    if (fetchOptions.headers) {
      if (Array.isArray(fetchOptions.headers)) {
        // Handle Headers as array format
        fetchOptions.headers.forEach(([key, value]) => {
          headers[key] = value;
        });
      } else if (typeof fetchOptions.headers === 'object') {
        // Handle plain object headers
        Object.entries(fetchOptions.headers).forEach(([key, value]) => {
          if (typeof value === 'string') {
            headers[key] = value;
          }
        });
      }
    }
    
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
      let errorData: any = null;
      
      try {
        errorData = JSON.parse(errorText);
      } catch {
        // errorText is not JSON, use as is
      }
      
      const message = errorData?.error || errorData?.message || errorText || 'Request failed';
      const details = errorData?.details || `HTTP ${response.status}`;
      
      logger.error('Server request failed', {
        url,
        status: response.status,
        statusText: response.statusText,
        errorData
      }, {
        component: 'server-request',
        action: context?.action,
        userId: context?.userId,
        requestId: context?.requestId
      });
      
      throw new HttpError(message, response.status, endpoint, fetchOptions.method as string, details);
    }
    
    const result = await response.json();
    
    logger.debug('Server request successful', { 
      endpoint, 
      status: response.status 
    }, {
      component: 'server-request',
      action: context?.action,
      userId: context?.userId,
      requestId: context?.requestId
    });
    
    return result;
  } catch (error) {
    logger.error('Server request exception', {
      endpoint,
      error: error instanceof Error ? error.message : String(error)
    }, {
      component: 'server-request',
      action: context?.action,
      userId: context?.userId,
      requestId: context?.requestId
    });
    
    throw error;
  }
}

/**
 * Helper for making GET requests with enhanced error handling
 */
export async function serverGet<T = any>(
  endpoint: string, 
  params?: Record<string, string | number | boolean>,
  context?: ServerRequestOptions['context']
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
    context,
  });
}

/**
 * Helper for making POST requests with enhanced error handling
 */
export async function serverPost<T = any>(
  endpoint: string, 
  data?: any,
  context?: ServerRequestOptions['context']
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'POST',
    body: data ? JSON.stringify(data) : undefined,
    context,
  });
}

/**
 * Helper for making PUT requests with enhanced error handling
 */
export async function serverPut<T = any>(
  endpoint: string, 
  data?: any,
  context?: ServerRequestOptions['context']
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'PUT',
    body: data ? JSON.stringify(data) : undefined,
    context,
  });
}

/**
 * Helper for making DELETE requests with enhanced error handling
 */
export async function serverDelete<T = any>(
  endpoint: string,
  context?: ServerRequestOptions['context']
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'DELETE',
    context,
  });
}

/**
 * Helper for making PATCH requests with enhanced error handling
 */
export async function serverPatch<T = any>(
  endpoint: string, 
  data?: any,
  context?: ServerRequestOptions['context']
): Promise<T> {
  return makeServerRequest<T>({
    endpoint,
    method: 'PATCH',
    body: data ? JSON.stringify(data) : undefined,
    context,
  });
}