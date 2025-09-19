// Main utils module - re-exports consolidated utility functionality
// Direct export of the cn function to fix immediate import issues
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Utility function to merge Tailwind CSS classes with clsx
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// Re-export everything else from utils directory
export * from './utils';