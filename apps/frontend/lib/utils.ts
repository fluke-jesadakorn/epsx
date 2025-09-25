/**
 * FRONTEND UTILS - MIGRATED TO SHARED
 * All utilities moved to shared/utils with compatibility layer
 * This file now re-exports shared utilities for backward compatibility
 */

// Re-export everything from shared utils
export * from '../../../shared/utils'

// Keep local cn function for immediate compatibility (also available in shared)
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Utility function to merge Tailwind CSS classes with clsx
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}