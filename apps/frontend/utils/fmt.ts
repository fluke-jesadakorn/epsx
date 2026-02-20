// Re-export formatting utilities from shared
export { cur, formatCurrency, formatPrice, prc } from '@/shared/utils/formatting/currency';
export { dt, formatDate } from '@/shared/utils/formatting/date';

export const pct = (val: number, dec = 2): string => `${(val * 100).toFixed(dec)}%`

export const epsGr = (growth: number | null | undefined): string =>
  growth === null || growth === undefined ? 'N/A' : `${growth > 0 ? '+' : ''}${growth}%`

export const dtFmt = (date: string): string =>
  new Date(date).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })

// Backward compatibility
export const formatPercentage = pct
export const formatEpsGrowth = epsGr
