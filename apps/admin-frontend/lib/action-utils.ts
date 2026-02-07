/**
 * Shared utilities for actions (not server actions)
 */

export interface ActionResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

/**
 * Create successful action result
 * @param data
 * @param message
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
 * @param error
 */
export function createErrorResult(error: string): ActionResult {
  return {
    success: false,
    error
  };
}