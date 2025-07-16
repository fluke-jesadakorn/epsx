import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

import type { ClassValue } from 'clsx';

/**
 * Combines class names with Tailwind CSS classes, handling conflicts and merging properly
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
