/**
 * FRONTEND UTILS - MIGRATED TO SHARED
 * All utilities moved to shared/utils with compatibility layer
 * This file now re-exports shared utilities for backward compatibility
 */

// Re-export everything from shared utils
export * from '../../../shared/utils';

// Explicitly export cn from shared to override local if needed, or just let strict export handle it
// If we remove local cn, the export * catches it if it's there.
// But to be safe and explicit:
export { cn } from '../../../shared/utils';
