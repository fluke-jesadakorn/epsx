/**
 * Frontend Utility Exports
 */

import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export * from './logging';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
