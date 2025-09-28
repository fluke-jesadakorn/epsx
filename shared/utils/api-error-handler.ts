export interface ApiError {
  status: number;
  data?: any;
  message?: string;
}

export interface ErrorHandlerResult {
  type: 'permission_error' | 'auth_error' | 'generic_error';
  error: any;
}

export function handleApiError(error: ApiError): ErrorHandlerResult {
  // Handle permission errors (403 status with permission error type)
  if (error.status === 403 && error.data?.error_type?.includes('permission')) {
    return { 
      type: 'permission_error', 
      error: error.data 
    };
  }
  
  // Handle authentication errors (401 status)
  if (error.status === 401) {
    return { 
      type: 'auth_error', 
      error: error.data || { 
        message: 'Authentication required',
        user_message: 'Please sign in to access this feature'
      } 
    };
  }
  
  // Handle other HTTP errors
  return { 
    type: 'generic_error', 
    error: {
      message: error.message || 'An unexpected error occurred',
      status: error.status,
      data: error.data
    }
  };
}

export function isPermissionError(error: any): boolean {
  return error?.status === 403 && error?.data?.error_type?.includes('permission');
}

export function isAuthError(error: any): boolean {
  return error?.status === 401;
}

export async function makeApiCall<T>(
  url: string, 
  options?: RequestInit
): Promise<{ data: T | null; error: ErrorHandlerResult | null }> {
  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      let errorData;
      try {
        errorData = await response.json();
      } catch {
        errorData = { message: response.statusText };
      }

      const apiError: ApiError = {
        status: response.status,
        data: errorData,
        message: errorData?.message || response.statusText,
      };

      return {
        data: null,
        error: handleApiError(apiError),
      };
    }

    const data = await response.json();
    return { data, error: null };
  } catch (err) {
    return {
      data: null,
      error: {
        type: 'generic_error',
        error: {
          message: err instanceof Error ? err.message : 'Network error occurred',
        },
      },
    };
  }
}

export default handleApiError;