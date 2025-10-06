/**
 * ADMIN FRONTEND UTILS - MIGRATED TO SHARED
 * All utilities moved to shared/utils with compatibility layer
 * This file now re-exports shared utilities for backward compatibility
 */

// Re-export everything from shared utils
// Import specific utilities for local re-export  
import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export * from '../../../shared/utils'

/**
 * Local cn function for immediate compatibility (also available from shared utils)
 * @param {...any} inputs
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// ============================================================================
// LEGACY ALIASES FOR BACKWARDS COMPATIBILITY
// ============================================================================

// These are already available from shared utils but kept for immediate compatibility
export { formatDate as fmtDate } from '../../../shared/utils'
export { formatRelativeTime as fmtRelativeTime } from '../../../shared/utils'
export { formatFileSize as formatBytes } from '../../../shared/utils'
export { formatCurrency as fmtCurrency } from '../../../shared/utils'
export { formatPercentage as pct } from '../../../shared/utils'
export { formatPrice as prc } from '../../../shared/utils'
export { formatEPSGrowth as epsGr } from '../../../shared/utils'