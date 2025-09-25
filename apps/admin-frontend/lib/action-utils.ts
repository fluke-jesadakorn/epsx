/**
 * Shared utilities for actions (not server actions)
 */

export interface ActionResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

/**
 * Create successful action result
 */
export function createSuccessResult<T>(data: T, message?: string): ActionResult<T> {
  return {
    success: true,
    data,
    message
  };
}

/**
 * Create error action result
 */
export function createErrorResult(error: string): ActionResult {
  return {
    success: false,
    error
  };
}