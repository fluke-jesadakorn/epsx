/**
 * Admin Frontend Permission Error Handler
 * Re-exports from shared with admin-specific defaults
 * 
 * @see shared/utils/permission-error-handler.ts for implementation
 */

export type {
  BackendPermissionError,
  PermissionErrorEvent,
  Platform
} from '@/shared/utils/permission-error-handler';

export {
  extractPermissionError, getErrorMessage, isPermissionError, onPermissionError, PermissionDeniedError
} from '@/shared/utils/permission-error-handler';

import {
  fetchWithPermissionHandling as sharedFetchWithPermissionHandling,
  handlePermissionError as sharedHandlePermissionError,
  type BackendPermissionError
} from '@/shared/utils/permission-error-handler';

/**
 * Handle permission error with admin context
 */
export function handlePermissionError(
  error: BackendPermissionError,
  context?: { feature?: string; action?: string }
): void {
  sharedHandlePermissionError(error, context, 'admin')
}

/**
 * Fetch with permission handling using admin context
 */
export async function fetchWithPermissionHandling(
  url: string,
  options?: RequestInit,
  context?: { feature?: string; action?: string }
): Promise<Response> {
  return sharedFetchWithPermissionHandling(url, options, context, 'admin')
}
