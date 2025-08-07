export interface ServerActionError {
  message: string;
  status?: number;
  code?: string;
  details?: any;
}

export interface ServerActionResult<T = any> {
  data?: T;
  error?: ServerActionError;
  success: boolean;
}

export class ServerError extends Error {
  status: number;
  code?: string;
  details?: any;

  constructor(message: string, status = 500, code?: string, details?: any) {
    super(message);
    this.status = status;
    this.code = code;
    this.details = details;
    this.name = 'ServerError';
  }
}

/**
 * Standardized error handler for server actions
 */
export function handleServerError(error: any, context?: string): ServerActionError {
  console.error(`Server action error${context ? ` in ${context}` : ''}:`, error);

  if (error instanceof ServerError) {
    return {
      message: error.message,
      status: error.status,
      code: error.code,
      details: error.details
    };
  }

  if (error instanceof Error) {
    // Parse HTTP errors from fetch
    const httpMatch = error.message.match(/HTTP (\d+): (.*)/);
    if (httpMatch && httpMatch[1] && httpMatch[2]) {
      const status = parseInt(httpMatch[1]);
      const message = httpMatch[2];
      
      return {
        message: getStatusMessage(status, message),
        status,
        code: getStatusCode(status)
      };
    }

    return {
      message: error.message,
      status: 500,
      code: 'INTERNAL_ERROR'
    };
  }

  return {
    message: 'An unexpected error occurred',
    status: 500,
    code: 'UNKNOWN_ERROR',
    details: error
  };
}

/**
 * Wrapper for server actions that provides standardized error handling
 */
export async function withServerAction<T>(
  action: () => Promise<T>,
  context?: string
): Promise<ServerActionResult<T>> {
  try {
    const data = await action();
    return {
      success: true,
      data
    };
  } catch (error) {
    return {
      success: false,
      error: handleServerError(error, context)
    };
  }
}

function getStatusMessage(status: number, defaultMessage: string): string {
  const statusMessages: Record<number, string> = {
    400: 'Invalid request data',
    401: 'Authentication required',
    403: 'Access denied',
    404: 'Resource not found',
    409: 'Resource already exists',
    422: 'Validation error',
    429: 'Too many requests',
    500: 'Internal server error',
    502: 'Service unavailable',
    503: 'Service temporarily unavailable'
  };

  return statusMessages[status] || defaultMessage || 'Request failed';
}

function getStatusCode(status: number): string {
  const statusCodes: Record<number, string> = {
    400: 'BAD_REQUEST',
    401: 'UNAUTHORIZED',
    403: 'FORBIDDEN',
    404: 'NOT_FOUND',
    409: 'CONFLICT',
    422: 'VALIDATION_ERROR',
    429: 'RATE_LIMITED',
    500: 'INTERNAL_ERROR',
    502: 'SERVICE_ERROR',
    503: 'SERVICE_UNAVAILABLE'
  };

  return statusCodes[status] || 'REQUEST_FAILED';
}