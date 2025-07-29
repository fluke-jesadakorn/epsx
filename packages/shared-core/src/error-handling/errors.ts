import type { BaseError, ApiError, ValidationError, ErrorSeverity } from './types';

export class AppError extends Error implements BaseError {
  public readonly code: string;
  public readonly status?: number;
  public readonly details?: any;
  public readonly timestamp: Date;
  public readonly context?: string;
  public readonly severity: ErrorSeverity;

  constructor(
    message: string,
    code: string,
    status?: number,
    details?: any,
    context?: string,
    severity: ErrorSeverity = 'medium'
  ) {
    super(message);
    this.name = 'AppError';
    this.code = code;
    this.status = status;
    this.details = details;
    this.timestamp = new Date();
    this.context = context;
    this.severity = severity;
  }
}

export class HttpError extends AppError implements ApiError {
  public readonly status: number;
  public readonly endpoint?: string;
  public readonly method?: string;

  constructor(
    message: string,
    status: number,
    endpoint?: string,
    method?: string,
    details?: any
  ) {
    super(message, getErrorCodeFromStatus(status), status, details, undefined, 'medium');
    this.name = 'HttpError';
    this.status = status; // Explicitly set status to ensure it's always a number
    this.endpoint = endpoint;
    this.method = method;
  }
}

export class SchemaValidationError extends AppError implements ValidationError {
  public readonly field?: string;
  public readonly violations?: string[];

  constructor(
    message: string,
    field?: string,
    violations?: string[],
    details?: any
  ) {
    super(message, 'VALIDATION_ERROR', 422, details, undefined, 'low');
    this.name = 'SchemaValidationError';
    this.field = field;
    this.violations = violations;
  }
}

function getErrorCodeFromStatus(status: number): string {
  const codes: Record<number, string> = {
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
  return codes[status] || 'REQUEST_FAILED';
}