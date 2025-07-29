import type { BaseError, Result, ErrorContext } from './types';
import { AppError, HttpError } from './errors';

export class ErrorHandler {
  private static logError(error: BaseError, context?: ErrorContext): void {
    const errorInfo = {
      message: error.message,
      code: error.code,
      status: error.status,
      timestamp: error.timestamp,
      context: error.context,
      userContext: context,
      details: error.details
    };

    if (error.status && error.status >= 500) {
      console.error('🚨 Critical Error:', errorInfo);
    } else if (error.status && error.status >= 400) {
      console.warn('⚠️  Client Error:', errorInfo);
    } else {
      console.info('ℹ️  Error:', errorInfo);
    }
  }

  public static handle(error: any, context?: ErrorContext): BaseError {
    let processedError: BaseError;

    if (error instanceof AppError) {
      processedError = error;
    } else if (error instanceof Error) {
      // Parse HTTP errors from fetch responses
      const httpMatch = error.message.match(/HTTP (\d+): (.*)/);
      if (httpMatch) {
        const status = parseInt(httpMatch[1]);
        const message = httpMatch[2] || getStatusMessage(status);
        processedError = new HttpError(message, status);
      } else {
        processedError = new AppError(
          error.message,
          'INTERNAL_ERROR',
          500,
          { originalError: error.name },
          context?.component,
          'high'
        );
      }
    } else {
      processedError = new AppError(
        'An unexpected error occurred',
        'UNKNOWN_ERROR',
        500,
        { originalError: error },
        context?.component,
        'critical'
      );
    }

    this.logError(processedError, context);
    return processedError;
  }

  public static async withErrorHandling<T>(
    operation: () => Promise<T>,
    context?: ErrorContext
  ): Promise<Result<T>> {
    try {
      const data = await operation();
      return { success: true, data };
    } catch (error) {
      const processedError = this.handle(error, context);
      return { success: false, error: processedError };
    }
  }

  public static createResult<T>(data: T): Result<T> {
    return { success: true, data };
  }

  public static createErrorResult<T>(error: BaseError): Result<T> {
    return { success: false, error };
  }
}

function getStatusMessage(status: number): string {
  const messages: Record<number, string> = {
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
  return messages[status] || 'Request failed';
}