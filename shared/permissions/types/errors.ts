// ============================================================================
// SHARED PERMISSION ERROR TYPES
// ============================================================================
// Error types and classes for permission-related operations

// ============================================================================
// ERROR CODE TYPES
// ============================================================================

export type PermissionErrorCode = 
  | 'INSUFFICIENT_PERMISSION'
  | 'PERMISSION_EXPIRED'
  | 'PERMISSION_NOT_FOUND'
  | 'HASH_MISMATCH'
  | 'TOKEN_INVALID'
  | 'TOKEN_EXPIRED'
  | 'USER_NOT_FOUND'
  | 'USER_INACTIVE'
  | 'ADMIN_REQUIRED'
  | 'VALIDATION_ERROR'
  | 'CACHE_ERROR'
  | 'DATABASE_ERROR'
  | 'NETWORK_ERROR'
  | 'RATE_LIMITED'
  | 'FEATURE_NOT_AVAILABLE'
  | 'INVALID_PERMISSION_FORMAT'
  | 'BULK_OPERATION_FAILED'
  | 'TEMPLATE_NOT_FOUND'
  | 'AUDIT_DISABLED'
  | 'MIGRATION_IN_PROGRESS'
  | 'SYSTEM_MAINTENANCE'

// ============================================================================
// ERROR INTERFACES
// ============================================================================

export interface PermissionError {
  code: PermissionErrorCode
  message: string
  details?: string
  permission?: string
  user_id?: string
  timestamp?: number
  trace_id?: string
}

export interface ValidationError extends PermissionError {
  field?: string
  validation_type?: 'format' | 'required' | 'range' | 'pattern'
  expected_format?: string
}

export interface BulkOperationError extends PermissionError {
  batch_id?: string
  total_operations: number
  successful_operations: number
  failed_operations: number
  failed_items: Array<{
    user_id: string
    error: PermissionError
  }>
}

export interface RateLimitError extends PermissionError {
  limit: number
  window_seconds: number
  retry_after_seconds: number
  current_usage: number
}

export interface SystemError extends PermissionError {
  component: 'database' | 'cache' | 'auth' | 'api' | 'worker'
  recoverable: boolean
  retry_suggested: boolean
  health_impact: 'none' | 'low' | 'medium' | 'high' | 'critical'
}

// ============================================================================
// ERROR CLASSES
// ============================================================================

export class GranularPermissionError extends Error {
  public readonly code: PermissionErrorCode
  public readonly permission?: string
  public readonly details?: string
  public readonly user_id?: string
  public readonly timestamp: number
  public readonly trace_id?: string

  constructor(
    message: string,
    code: PermissionErrorCode,
    options?: {
      permission?: string
      details?: string
      user_id?: string
      trace_id?: string
      cause?: Error
    }
  ) {
    super(message)
    this.name = 'GranularPermissionError'
    this.code = code
    this.permission = options?.permission
    this.details = options?.details
    this.user_id = options?.user_id
    this.timestamp = Date.now()
    this.trace_id = options?.trace_id

    if (options?.cause) {
      this.cause = options.cause
    }

    // Maintain proper stack trace for where error was thrown (Node.js only)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, GranularPermissionError)
    }
  }

  toJSON(): PermissionError {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
      permission: this.permission,
      user_id: this.user_id,
      timestamp: this.timestamp,
      trace_id: this.trace_id
    }
  }

  static fromJSON(json: PermissionError): GranularPermissionError {
    return new GranularPermissionError(json.message, json.code, {
      permission: json.permission,
      details: json.details,
      user_id: json.user_id,
      trace_id: json.trace_id
    })
  }
}

export class PermissionValidationError extends GranularPermissionError {
  public readonly field?: string
  public readonly validation_type?: ValidationError['validation_type']
  public readonly expected_format?: string

  constructor(
    message: string,
    field?: string,
    validation_type?: ValidationError['validation_type'],
    expected_format?: string
  ) {
    super(message, 'VALIDATION_ERROR', { details: `Field: ${field}` })
    this.name = 'PermissionValidationError'
    this.field = field
    this.validation_type = validation_type
    this.expected_format = expected_format
  }
}

export class PermissionExpiredError extends GranularPermissionError {
  public readonly expires_at?: number
  public readonly expired_at: number

  constructor(
    permission: string,
    expires_at?: number,
    user_id?: string
  ) {
    const message = `Permission '${permission}' has expired`
    super(message, 'PERMISSION_EXPIRED', { permission, user_id })
    this.name = 'PermissionExpiredError'
    this.expires_at = expires_at
    this.expired_at = Date.now()
  }
}

export class InsufficientPermissionError extends GranularPermissionError {
  public readonly required_permission: string
  public readonly user_permissions?: string[]

  constructor(
    required_permission: string,
    user_id?: string,
    user_permissions?: string[]
  ) {
    const message = `Insufficient permissions: '${required_permission}' required`
    super(message, 'INSUFFICIENT_PERMISSION', { permission: required_permission, user_id })
    this.name = 'InsufficientPermissionError'
    this.required_permission = required_permission
    this.user_permissions = user_permissions
  }
}

export class BulkPermissionOperationError extends GranularPermissionError {
  public readonly batch_id?: string
  public readonly total_operations: number
  public readonly successful_operations: number
  public readonly failed_operations: number
  public readonly failed_items: Array<{ user_id: string; error: PermissionError }>

  constructor(
    operation: string,
    batch_id: string | undefined,
    total: number,
    successful: number,
    failed: number,
    failed_items: Array<{ user_id: string; error: PermissionError }>
  ) {
    const message = `Bulk ${operation} operation failed: ${failed}/${total} operations failed`
    super(message, 'BULK_OPERATION_FAILED', { details: `Batch: ${batch_id}` })
    this.name = 'BulkPermissionOperationError'
    this.batch_id = batch_id
    this.total_operations = total
    this.successful_operations = successful
    this.failed_operations = failed
    this.failed_items = failed_items
  }
}

// ============================================================================
// ERROR FACTORY FUNCTIONS
// ============================================================================

export const createPermissionError = (
  code: PermissionErrorCode,
  message: string,
  options?: {
    permission?: string
    details?: string
    user_id?: string
    trace_id?: string
    cause?: Error
  }
): GranularPermissionError => {
  return new GranularPermissionError(message, code, options)
}

export const createValidationError = (
  message: string,
  field?: string,
  validation_type?: ValidationError['validation_type'],
  expected_format?: string
): PermissionValidationError => {
  return new PermissionValidationError(message, field, validation_type, expected_format)
}

export const createExpiredError = (
  permission: string,
  expires_at?: number,
  user_id?: string
): PermissionExpiredError => {
  return new PermissionExpiredError(permission, expires_at, user_id)
}

export const createInsufficientPermissionError = (
  required_permission: string,
  user_id?: string,
  user_permissions?: string[]
): InsufficientPermissionError => {
  return new InsufficientPermissionError(required_permission, user_id, user_permissions)
}

// ============================================================================
// ERROR HANDLING UTILITIES
// ============================================================================

export const isPermissionError = (error: any): error is GranularPermissionError => {
  return error instanceof GranularPermissionError || (
    error &&
    typeof error === 'object' &&
    'code' in error &&
    'message' in error &&
    typeof error.code === 'string'
  )
}

export const getErrorMessage = (error: any): string => {
  if (isPermissionError(error)) {
    return error.message
  }
  
  if (error instanceof Error) {
    return error.message
  }
  
  if (typeof error === 'string') {
    return error
  }
  
  return 'An unknown error occurred'
}

export const getErrorCode = (error: any): PermissionErrorCode | 'UNKNOWN' => {
  if (isPermissionError(error)) {
    return error.code
  }
  
  return 'UNKNOWN'
}

export const shouldRetry = (error: any): boolean => {
  if (!isPermissionError(error)) {
    return false
  }
  
  const retryableCodes: PermissionErrorCode[] = [
    'NETWORK_ERROR',
    'DATABASE_ERROR',
    'CACHE_ERROR',
    'RATE_LIMITED'
  ]
  
  return retryableCodes.includes(error.code)
}

export const getRetryDelay = (error: any, attempt: number = 1): number => {
  if (!shouldRetry(error)) {
    return 0
  }
  
  if (isPermissionError(error) && error.code === 'RATE_LIMITED') {
    // For rate limiting, use the suggested retry delay if available
    const rateLimitError = error as any
    if (rateLimitError.retry_after_seconds) {
      return rateLimitError.retry_after_seconds * 1000
    }
  }
  
  // Exponential backoff: 1s, 2s, 4s, 8s, etc.
  return Math.min(1000 * Math.pow(2, attempt - 1), 30000)
}