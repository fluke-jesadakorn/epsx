/**
 * Promotion utilities for calculating prices and status
 */
import { fmtAmt } from './formatting/currency'

export type PromoType = 'percentage' | 'fixed';
export type PromoStatus = 'active' | 'upcoming' | 'expired' | 'disabled';

/**
 * Options for calculating price
 */
export interface CalcPriceOptions {
  basePrice: number;
  type: PromoType;
  value: number;
  customPrice?: number;
}

/**
 * Calculate discounted price based on promotion type
 */
export function calcPrice({
  basePrice,
  type,
  value,
  customPrice
}: CalcPriceOptions): number {
  if (customPrice !== undefined && customPrice > 0) {
    return customPrice;
  }

  if (type === 'percentage') {
    return basePrice * (1 - value / 100);
  } else {
    return Math.max(0, basePrice - value);
  }
}

/**
 * Get promotion status based on dates
 */
export function getStatus(
  enabled: boolean,
  startDate: string,
  endDate: string
): PromoStatus {
  if (!enabled) {
    return 'disabled';
  }

  const now = new Date();
  const start = new Date(startDate);
  const end = new Date(endDate);

  if (now < start) {
    return 'upcoming';
  } else if (now > end) {
    return 'expired';
  } else {
    return 'active';
  }
}

/**
 * Check if promotion is currently active
 */
export function isActive(
  enabled: boolean,
  startDate: string,
  endDate: string
): boolean {
  return getStatus(enabled, startDate, endDate) === 'active';
}

/**
 * Format promotion badge text
 */
export function formatBadge(
  type: PromoType,
  value: number,
  status: PromoStatus
): string {
  if (status !== 'active') {
    return '';
  }

  if (type === 'percentage') {
    return `${Math.round(value)}% OFF`;
  } else {
    return `$${fmtAmt(value)} OFF`;
  }
}

/**
 * Get time remaining for active promotion (human-readable)
 */
export function getTimeRemaining(endDate: string): string {
  const now = new Date();
  const end = new Date(endDate);
  const diff = end.getTime() - now.getTime();

  if (diff <= 0) {
    return 'Expired';
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

  if (days > 0) {
    return hours > 0 ? `${days}d ${hours}h left` : `${days}d left`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m left`;
  } else {
    return `${minutes}m left`;
  }
}

/**
 * Get status badge color
 */
export function getStatusColor(status: PromoStatus): string {
  switch (status) {
    case 'active':
      return 'text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/20';
    case 'upcoming':
      return 'text-yellow-600 dark:text-yellow-400 bg-yellow-50 dark:bg-yellow-900/20';
    case 'expired':
      return 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20';
    case 'disabled':
      return 'text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-900/20';
  }
}

/**
 * Get status icon
 */
export function getStatusIcon(status: PromoStatus): string {
  switch (status) {
    case 'active':
      return '🟢';
    case 'upcoming':
      return '🟡';
    case 'expired':
      return '🔴';
    case 'disabled':
      return '⚪';
  }
}

/**
 * Format status text
 */
export function getStatusText(status: PromoStatus): string {
  switch (status) {
    case 'active':
      return 'Active';
    case 'upcoming':
      return 'Upcoming';
    case 'expired':
      return 'Expired';
    case 'disabled':
      return 'No Promotion';
  }
}

/**
 * Calculate time until promotion starts (for upcoming)
 */
export function getTimeUntilStart(startDate: string): string {
  const now = new Date();
  const start = new Date(startDate);
  const diff = start.getTime() - now.getTime();

  if (diff <= 0) {
    return 'Starting now';
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

  if (days > 0) {
    return `Starts in ${days}d ${hours}h`;
  } else if (hours > 0) {
    return `Starts in ${hours}h`;
  } else {
    return 'Starting soon';
  }
}

/**
 * Calculate time since promotion ended (for expired)
 */
export function getTimeSinceEnd(endDate: string): string {
  const now = new Date();
  const end = new Date(endDate);
  const diff = now.getTime() - end.getTime();

  if (diff <= 0) {
    return 'Just ended';
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days > 0) {
    return `Ended ${days}d ago`;
  } else {
    return 'Ended recently';
  }
}
