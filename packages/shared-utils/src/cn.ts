import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import type { ClassValue } from 'clsx';

/**
 * Combines class names with Tailwind CSS classes, handling conflicts and merging properly.
 * This is the single source of truth for the cn utility function used across all apps.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}