/**
 * ADMIN FRONTEND UTILS - MIGRATED TO SHARED
 * All utilities moved to shared/utils with compatibility layer
 * This file now re-exports shared utilities for backward compatibility
 */

// Re-export everything from shared utils
export * from '@/shared/utils';

// Explicitly export cn from shared
export { cn } from '@/shared/utils';

// ============================================================================
// LEGACY ALIASES FOR BACKWARDS COMPATIBILITY
// ============================================================================

// These are already available from shared utils but kept for immediate compatibility
export { formatEPSGrowth as epsGr, formatCurrency as fmtCurrency, formatDate as fmtDate, formatRelativeTime as fmtRelativeTime, formatFileSize as formatBytes, formatPercentage as pct, formatPrice as prc } from '@/shared/utils';

